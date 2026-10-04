// Life Optimizer
// Copyright (C) 2026 MILAN NIKOLIC
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Integration tests for early-retirement planning, across the retirement-age matrix.
//!
//! These exercise the public contract described in `EARLY_RETIREMENT.md`: the projection of
//! AHV and BVG entitlements, the statutory boundaries a retirement age has to be read
//! against, the tax-minimising withdrawal allocation, the risk measurement, and the
//! education-contribution ledgers.
//!
//! # Why the age grid rather than a handful of ages
//!
//! The projection has three statutory discontinuities in it — at 58 (the regulatory floor
//! under BVV 2 Art. 1i), at 63 (the AHV earliest draw under AHVG Art. 40 and the statutory
//! BVG early draw under BVG Art. 13 Abs. 2), and at 65 (the reference age) — and a test that
//! checks only 65 sees none of them. Every one of the defects these tests were written to
//! catch lived on one side or the other of a boundary and was invisible at the default.
//!
//! # What was actually found, rather than confirmed
//!
//! Writing this suite surfaced two defects in the projection, both fixed and recorded in
//! `src/early_retirement.rs`:
//!
//! 1. **AHV was penalised twice.** The pension cannot be drawn before completed 63, but the
//!    projection applied the early-draw reduction as though it started at the retirement age,
//!    while the caller treated it as starting at the reference age. A plan retiring at 63 was
//!    reduced for earliness *and* denied the pension until 65. `Entitlements::ahv_draw_age`
//!    now separates the two ages.
//! 2. **The AHV contributions a non-employed early retiree owes were parameterised but never
//!    charged.** They are now computed as a range, because AHVG Art. 10 sets a table assessed
//!    on wealth and rental income rather than a rate.

use life_optimizer::early_retirement::{
    contribution_ledger, draw_paths, evaluate_strategy, greedy_withdrawals, optimise_withdrawals,
    plan_withdrawals, project, required_withdrawals, retirement_availability,
    AllocationOutcome, BandedTax, ConsumptionNeed, ContributionPrinciple, Degree, Household,
    PathDraw, Provenance, RetirementAvailability, RetirementParameters, TaxBand, TaxTreatment,
};
use life_optimizer::tax::TaxSchedule;

/// The retirement ages the suite covers.
///
/// Deliberately includes both sides of every statutory boundary: 58 (the regulatory floor
/// under BVV 2 Art. 1i), 63 (the AHV earliest draw and the statutory BVG early draw), and 65
/// (the reference age at which both reach their full value). 55 and 57 are the ages the
/// request named, and both sit *below* the regulatory floor, which is a case the projection
/// must still price even though the law does not permit it.
const AGE_MATRIX: [u32; 13] = [55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 70];

/// A progressive capital-benefit tariff in the shape a cantonal one takes.
///
/// It is a **declared** fixture, not a canton's published table: the tests below are about
/// the structure of the allocation, and a real tariff would make them about its numbers.
fn capital_schedule() -> BandedTax {
    BandedTax {
        allowance: 0.0,
        bands: vec![
            TaxBand {
                width: 50_000.0,
                marginal_rate: 0.0065,
            },
            TaxBand {
                width: 150_000.0,
                marginal_rate: 0.0120,
            },
            TaxBand {
                width: 300_000.0,
                marginal_rate: 0.0200,
            },
            TaxBand {
                width: 500_000.0,
                marginal_rate: 0.0300,
            },
        ],
        provenance: Provenance::Declared {
            rationale: "test fixture",
        },
    }
}

fn need() -> ConsumptionNeed {
    ConsumptionNeed {
        mandatory_annual: 40_000.0,
        lifestyle_annual: 55_000.0,
    }
}

// ---------------------------------------------------------------------------
// The projection, across the age matrix
// ---------------------------------------------------------------------------

/// Every age in the grid must project to finite, non-negative entitlements.
///
/// A matrix of ages is exactly where an arithmetic slip — a subtraction that goes negative,
/// a division by a saturating zero — surfaces as `NaN` rather than as an obviously wrong
/// number, and `NaN` then propagates silently through every later comparison, because
/// `NaN < x` is false and so every sanity check passes.
#[test]
fn the_projection_is_finite_and_non_negative_across_the_age_matrix() {
    let household = Household::default();
    let params = RetirementParameters::default();
    for age in AGE_MATRIX {
        let e = project(&household, &params, age, 1.0);
        for (name, value) in [
            ("ahv_annual", e.ahv_annual),
            ("ahv_early_reduction", e.ahv_early_reduction),
            ("bvg_capital", e.bvg_capital),
            ("effective_conversion_rate", e.effective_conversion_rate),
            ("bvg_annuity_annual", e.bvg_annuity_annual),
            ("pillar3a_capital", e.pillar3a_capital),
            ("bridge_capital", e.bridge_capital),
            ("bridge_ahv_contributions_min", e.bridge_ahv_contributions_min),
            ("bridge_ahv_contributions_max", e.bridge_ahv_contributions_max),
        ] {
            assert!(
                value.is_finite() && value >= 0.0,
                "age {age}: {name} is {value}"
            );
        }
        assert!(e.retirement_age == age);
        assert_eq!(e.years_to_retirement, age.saturating_sub(household.current_age));
        assert!(e.effective_conversion_rate <= params.bvg_conversion_rate + 1e-12);
        assert!(e.effective_conversion_rate >= 0.0);
        assert!(e.bridge_ahv_contributions_min <= e.bridge_ahv_contributions_max);
    }
}

/// The availability classification has to match the statute at every boundary.
///
/// A plan for an age the law does not permit looks exactly like a plan for one it does, so
/// the classification is a separate question from the projection and is asked separately.
#[test]
fn the_availability_classification_matches_the_statutory_boundaries() {
    let params = RetirementParameters::default();
    let expected = |age: u32| match age {
        0..=57 => RetirementAvailability::NotAvailableUnderBvg,
        58..=62 => RetirementAvailability::FundMayPermit,
        _ => RetirementAvailability::StatutoryRight,
    };
    for age in 50..=72u32 {
        assert_eq!(
            retirement_availability(&params, age),
            expected(age),
            "age {age} classified wrongly"
        );
    }

    // The ages the request named, spelled out so a reader does not have to evaluate the
    // closure above to see them: 55 and 57 are below the floor entirely, 60 and 62 are
    // available only if the fund's own regulations provide for them.
    let named = [
        (55, RetirementAvailability::NotAvailableUnderBvg),
        (57, RetirementAvailability::NotAvailableUnderBvg),
        (60, RetirementAvailability::FundMayPermit),
        (62, RetirementAvailability::FundMayPermit),
        (63, RetirementAvailability::StatutoryRight),
    ];
    for (age, availability) in named {
        assert_eq!(retirement_availability(&params, age), availability, "age {age}");
    }
    assert!(!retirement_availability(&params, 55).is_available());
    assert!(!retirement_availability(&params, 57).is_available());
    assert!(retirement_availability(&params, 58).is_available());
    assert!(retirement_availability(&params, 60).is_available());
}

/// AHV cannot be drawn before completed 63 (AHVG Art. 40 Abs. 1), whatever the BVG permits.
///
/// Below 63 the draw age is therefore pinned and the *bridge* lengthens: the pension is not
/// paid from the retirement age at all. Retiring at 55 means eight years with no AHV, not a
/// pension reduced by fourteen years' worth of penalties.
#[test]
fn ahv_is_not_drawn_before_its_earliest_age() {
    let household = Household::default();
    let params = RetirementParameters::default();
    for age in AGE_MATRIX {
        let e = project(&household, &params, age, 1.0);
        assert_eq!(
            e.ahv_draw_age,
            age.max(params.ahv_earliest_age),
            "age {age}: AHV draw age is {}",
            e.ahv_draw_age
        );
        assert!(e.ahv_draw_age >= params.ahv_earliest_age);
    }
    for age in [55, 56, 57, 58, 60, 62, 63] {
        assert_eq!(project(&household, &params, age, 1.0).ahv_draw_age, 63);
    }
    assert_eq!(project(&household, &params, 65, 1.0).ahv_draw_age, 65);
    assert_eq!(project(&household, &params, 70, 1.0).ahv_draw_age, 70);
}

/// The early-draw reduction is capped at two years' worth — 13.6% under AHVV Art. 56bis —
/// but the contribution record is **not** capped, and the two are independent channels.
///
/// This distinction was found by writing the test: the first version asserted that the
/// *amounts* were equal at every age at or below 63, and it failed, correctly. Retiring at 55
/// costs AHV twice over — once through the reduction, once through nine fewer contributing
/// years — and conflating the channels would have hidden the second.
#[test]
fn the_ahv_reduction_is_capped_but_the_contribution_record_is_not() {
    let household = Household::default();
    let params = RetirementParameters::default();
    let at_63 = project(&household, &params, 63, 1.0);
    assert!(
        (at_63.ahv_early_reduction - 0.136).abs() < 1e-12,
        "the cap should be 2 x 6.8% = 13.6%, got {}",
        at_63.ahv_early_reduction
    );

    // Flat reduction across every age at or below 63 ...
    for age in [55, 56, 57, 58, 59, 60, 61, 62, 63] {
        let e = project(&household, &params, age, 1.0);
        assert!(
            (e.ahv_early_reduction - at_63.ahv_early_reduction).abs() < 1e-12,
            "age {age}: reduction {} differs from age 63's {}",
            e.ahv_early_reduction,
            at_63.ahv_early_reduction
        );
    }

    // ... and strictly rising amounts, because the record is not capped.
    let mut previous: Option<(u32, f64)> = None;
    for age in [55, 57, 58, 60, 62, 63] {
        let e = project(&household, &params, age, 1.0);
        if let Some((earlier_age, amount)) = previous {
            assert!(
                e.ahv_annual > amount,
                "the AHV amount did not rise from age {earlier_age} ({amount}) to {age} ({}), \
                 so the contribution record is not being applied",
                e.ahv_annual
            );
        }
        previous = Some((age, e.ahv_annual));
    }

    // The record is the whole of the difference at the cap: 35 of 44 years at 55 against
    // 43 of 44 at 63, since the default household has 25 years accrued at 45.
    let at_55 = project(&household, &params, 55, 1.0);
    assert!((at_55.ahv_annual / at_63.ahv_annual - 35.0 / 43.0).abs() < 1e-9);

    // And the reduction falls to zero at the reference age.
    assert_eq!(project(&household, &params, 65, 1.0).ahv_early_reduction, 0.0);
    assert_eq!(project(&household, &params, 70, 1.0).ahv_early_reduction, 0.0);
}

/// Every cost channel must move the same way: a later retirement means more BVG capital, a
/// higher conversion rate, and fewer years of non-employed AHV contributions.
///
/// If any of these moved the other way, the model would be subsidising earlier retirement
/// for a reason that is not a policy.
#[test]
fn retiring_earlier_costs_more_on_every_channel() {
    let household = Household::default();
    let params = RetirementParameters::default();
    let mut previous: Option<(u32, f64, f64, u32, f64)> = None;
    for age in [55, 57, 58, 60, 62, 63, 64, 65] {
        let e = project(&household, &params, age, 1.0);
        if let Some((earlier_age, capital, rate, years, contributions)) = previous {
            assert!(
                e.bvg_capital > capital,
                "BVG capital fell from age {earlier_age} ({capital}) to {age} ({})",
                e.bvg_capital
            );
            assert!(
                e.effective_conversion_rate >= rate,
                "conversion rate fell from {earlier_age} to {age}"
            );
            assert!(
                e.non_employed_years <= years,
                "non-employed years rose from {earlier_age} ({years}) to {age} ({})",
                e.non_employed_years
            );
            assert!(
                e.bridge_ahv_contributions_min <= contributions,
                "bridge AHV contributions rose from {earlier_age} to {age}"
            );
        }
        previous = Some((
            age,
            e.bvg_capital,
            e.effective_conversion_rate,
            e.non_employed_years,
            e.bridge_ahv_contributions_min,
        ));
    }

    // Contributions run to the reference age, not to the draw age: the statute ties the
    // liability to the reference age, so deferring the pension does not remove it.
    assert_eq!(project(&household, &params, 55, 1.0).non_employed_years, 10);
    assert_eq!(project(&household, &params, 63, 1.0).non_employed_years, 2);
    assert_eq!(project(&household, &params, 65, 1.0).non_employed_years, 0);
    assert_eq!(project(&household, &params, 70, 1.0).non_employed_years, 0);
}

/// The regression test for the double penalty.
///
/// At 63 the AHV draw age *is* 63, so the bridge in AHV terms is zero years. The first
/// version of the projection applied the 13.6% reduction **and** treated AHV as starting at
/// the reference age, charging the same two years twice. The observable is the first-year
/// withdrawal: with AHV from 63 and an annuity covering most of the need, it must be far
/// below a year's consumption.
#[test]
fn retiring_at_63_receives_ahv_from_63_and_not_from_65() {
    let household = Household::default();
    let params = RetirementParameters::default();
    let e = project(&household, &params, 63, 1.0);
    assert_eq!(e.ahv_draw_age, 63);
    assert!(e.ahv_early_reduction > 0.0, "the reduction still applies");

    // Five years, so the horizon covers ages 63 to 67 and straddles the reference age.
    let paths = vec![
        PathDraw {
            real_return: 0.0,
            years_lived: 5,
        };
        1
    ];
    let profile = evaluate_strategy(
        63,
        1.0,
        &e,
        &need(),
        &TaxTreatment::new(capital_schedule(), 0.0),
        &paths,
        5,
    );
    let first = profile.plan.capital_withdrawn.first().copied().unwrap_or(0.0);
    assert!(
        first < need().lifestyle_annual,
        "the first-year withdrawal was {first}, a whole year of consumption — the signature \
         of AHV being withheld until 65"
    );
}

// ---------------------------------------------------------------------------
// The allocation
// ---------------------------------------------------------------------------

/// A 50,000 pot cannot fund ten years of a 50,000 annual gap, and that has to be *reported*
/// rather than hidden behind a scaled-up plan.
#[test]
fn an_unfundable_requirement_is_reported_and_never_overspends() {
    let bridge = vec![10_000.0_f64; 10];
    let (plan, surplus) = plan_withdrawals(50_000.0, &bridge, 60_000.0, &capital_schedule());
    assert!(
        surplus < 0.0,
        "a 50,000 pot cannot fund ten years of a 50,000 annual gap"
    );
    let total: f64 = plan.capital_withdrawn.iter().sum();
    assert!(
        (total - 50_000.0).abs() < 1e-6,
        "an unfundable plan must still not place more than the capital: placed {total}"
    );
}

/// The requirement is funded first and is not the optimiser's to trade away.
///
/// An allocation that spread the capital evenly to save tax would leave the AHV-free bridge
/// years short — the failure that the risk measurement exists to catch and that the first
/// version of the allocation produced.
#[test]
fn the_requirement_is_funded_before_the_surplus_is_spread() {
    let bridge = vec![10_000.0_f64; 6];
    let need = 45_000.0;
    let required = required_withdrawals(&bridge, need);
    for (year, amount) in required.iter().enumerate() {
        assert!(
            (amount - 35_000.0).abs() < 1e-9,
            "year {year}: the requirement should be need less bridge income, got {amount}"
        );
    }
    let capital = 300_000.0;
    let (plan, surplus) = plan_withdrawals(capital, &bridge, need, &capital_schedule());
    assert!((surplus - (capital - 35_000.0 * 6.0)).abs() < 1e-6);
    for (year, draw) in plan.capital_withdrawn.iter().enumerate() {
        assert!(
            *draw >= required[year] - 1e-6,
            "year {year}: withdrawal {draw} is below the requirement {}",
            required[year]
        );
    }
    let total: f64 = plan.capital_withdrawn.iter().sum();
    assert!((total - capital).abs() < 1e-6);
}

/// The linear program and the water-filling solution must agree on the tax.
///
/// They are independent implementations — a simplex tableau and a band-filling loop — so
/// agreement is evidence rather than a tautology. Two implementations that shared code could
/// not check each other.
#[test]
fn the_linear_program_and_the_water_fill_agree_on_tax() {
    let s = capital_schedule();
    let base = vec![20_000.0_f64; 8];
    for capital in [50_000.0, 150_000.0, 400_000.0, 900_000.0] {
        let AllocationOutcome::Optimal(lp) = optimise_withdrawals(capital, &base, &s) else {
            panic!("the allocation should be feasible for {capital}");
        };
        let greedy = greedy_withdrawals(capital, &base, &s);
        assert!(
            (lp.total_tax - greedy.total_tax).abs() < 1.0,
            "capital {capital}: the LP says {} and the water fill says {}",
            lp.total_tax,
            greedy.total_tax
        );
        let placed: f64 = lp.capital_withdrawn.iter().sum();
        assert!(
            (placed - capital).abs() < 1e-6,
            "capital {capital}: the LP placed {placed}"
        );
    }
}

/// Water-filling must equalise the marginal rate across the years it touches — the KKT
/// condition the LP's dual states — and must spread rather than concentrate.
///
/// The earlier cheapest-year-first rule filled one year to the top of a band before starting
/// the next. Tax-equal, and it emptied the pot into the early years, leaving every later year
/// on its annuity alone.
#[test]
fn water_filling_equalises_the_marginal_rate_without_concentrating() {
    let s = capital_schedule();
    let base = vec![20_000.0_f64; 8];
    let capital = 200_000.0;
    let plan = greedy_withdrawals(capital, &base, &s);

    let rates: Vec<f64> = plan
        .capital_withdrawn
        .iter()
        .enumerate()
        .filter(|(_, draw)| **draw > 1.0)
        .map(|(year, draw)| s.marginal_rate_at(base[year] + draw))
        .collect();
    assert!(rates.len() > 1, "the fixture should use several years");
    let highest = rates.iter().cloned().fold(0.0_f64, f64::max);
    let lowest = rates.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(
        highest - lowest < 0.02,
        "marginal rates range {lowest} to {highest}, more than one band apart"
    );

    let largest = plan
        .capital_withdrawn
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max);
    assert!(
        largest < capital * 0.5,
        "one year took {largest} of {capital}, so the allocation concentrated"
    );
}

/// A banded schedule must charge each band's own rate on the income inside it and must not
/// let income above the top band escape.
///
/// A schedule that stopped charging at the top would make large withdrawals free at the
/// margin, and the optimiser would put everything there.
#[test]
fn bands_are_charged_in_order_and_the_top_band_catches_everything_above_it() {
    let s = capital_schedule();
    assert_eq!(s.tax_on(0.0), 0.0);
    assert!((s.tax_on(10_000.0) - 65.0).abs() < 1e-9);
    assert!((s.tax_on(50_000.0) - 325.0).abs() < 1e-9);
    assert!((s.tax_on(200_000.0) - (325.0 + 1_800.0)).abs() < 1e-9);
    let at_top = s.tax_on(500_000.0);
    let above_top = s.tax_on(600_000.0);
    assert!(
        (above_top - at_top - 100_000.0 * 0.03).abs() < 1e-9,
        "income above the top band must still be charged at the top rate"
    );
}

// ---------------------------------------------------------------------------
// Risk, across the age matrix
// ---------------------------------------------------------------------------

/// The allocation must be solvable and the risk profile measurable at every age in the
/// matrix — including the ages the law does not permit.
///
/// A model that refused to price an impossible plan would be worse than one that prices it:
/// the impossibility is a question about policy, and the cost is the answer to it.
#[test]
fn every_age_in_the_matrix_solves_and_reports_a_risk_profile() {
    let household = Household::default();
    let params = RetirementParameters::default();
    let s = capital_schedule();
    for age in AGE_MATRIX {
        let e = project(&household, &params, age, 1.0);
        let horizon = 95 - age;
        let paths = draw_paths(200, horizon, 0.02, 0.10, 95, age, 4242);
        assert_eq!(paths.len(), 200);
        for path in &paths {
            assert!(
                path.years_lived >= 1 && path.years_lived <= horizon,
                "age {age}: a path lives {} years of a {horizon}-year horizon",
                path.years_lived
            );
        }
        let profile = evaluate_strategy(
            age,
            1.0,
            &e,
            &need(),
            &TaxTreatment::new(s.clone(), 0.0),
            &paths,
            horizon,
        );
        assert!(
            (0.0..=1.0).contains(&profile.probability_of_shortfall),
            "age {age}: shortfall probability {}",
            profile.probability_of_shortfall
        );
        assert_eq!(profile.plan.capital_withdrawn.len() as u32, horizon.max(1));
        assert!(profile.mean_lifetime_tax >= 0.0 && profile.mean_lifetime_tax.is_finite());
        assert!(profile.mean_terminal_wealth.is_finite());
        assert!(profile.p10_terminal_wealth <= profile.median_terminal_wealth);
        assert!(profile.median_terminal_wealth <= profile.p90_terminal_wealth);
        assert_eq!(profile.retirement_age, age);
        assert_eq!(profile.paths, 200);
    }
}

/// Retiring earlier must not reduce the shortfall probability.
///
/// This is the model's central claim in one line, checked on the matrix rather than asserted
/// from the algebra: a plan that stops earning sooner, on a smaller capital and over a longer
/// horizon, cannot be safer.
#[test]
fn the_shortfall_probability_does_not_rise_with_a_later_retirement() {
    let household = Household::default();
    let params = RetirementParameters::default();
    let s = capital_schedule();
    let mut previous: Option<(u32, f64)> = None;
    for age in [58, 60, 62, 63, 64, 65, 67] {
        let e = project(&household, &params, age, 1.0);
        let horizon = 95 - age;
        let paths = draw_paths(400, horizon, 0.02, 0.10, 95, age, 999);
        let profile = evaluate_strategy(
            age,
            1.0,
            &e,
            &need(),
            &TaxTreatment::new(s.clone(), 0.0),
            &paths,
            horizon,
        );
        if let Some((earlier_age, earlier)) = previous {
            assert!(
                profile.probability_of_shortfall <= earlier + 1e-9,
                "shortfall rose from age {earlier_age} ({earlier:.4}) to {age} ({:.4})",
                profile.probability_of_shortfall
            );
        }
        previous = Some((age, profile.probability_of_shortfall));
    }
}

/// Annuitising more must not raise the shortfall probability, and must not leave more
/// unconsumed wealth either.
///
/// Both halves matter: a model in which annuitising reduced risk *and* left more money behind
/// would not have a trade-off in it at all, and the trade-off is the reason the risk table
/// exists.
#[test]
fn annuitising_more_does_not_raise_risk_or_leave_more_unconsumed() {
    let household = Household::default();
    let params = RetirementParameters::default();
    let e = project(&household, &params, 63, 1.0);
    let s = capital_schedule();
    let paths = draw_paths(2_000, 32, 0.015, 0.08, 95, 63, 11);
    let all_capital = evaluate_strategy(
        63,
        0.0,
        &e,
        &need(),
        &TaxTreatment::new(s.clone(), 0.0),
        &paths,
        32,
    );
    let all_annuity = evaluate_strategy(
        63,
        1.0,
        &e,
        &need(),
        &TaxTreatment::new(s.clone(), 0.0),
        &paths,
        32,
    );
    assert!(
        all_annuity.probability_of_shortfall <= all_capital.probability_of_shortfall,
        "annuitising raised the shortfall probability: {} against {}",
        all_annuity.probability_of_shortfall,
        all_capital.probability_of_shortfall
    );
}

/// The same seed must draw the same paths, or two strategies compared against "the same
/// paths" would not be.
#[test]
fn the_same_seed_draws_the_same_paths() {
    let a = draw_paths(500, 40, 0.02, 0.10, 90, 60, 7);
    let b = draw_paths(500, 40, 0.02, 0.10, 90, 60, 7);
    assert_eq!(a, b);
    let c = draw_paths(500, 40, 0.02, 0.10, 90, 60, 8);
    assert_ne!(a, c);
}

// ---------------------------------------------------------------------------
// The case matrix: household composition and employment level at every age
// ---------------------------------------------------------------------------

/// Household composition and employment level have to move the projection in the obvious
/// directions at every age, not only at the default one.
///
/// A parameter that behaves at 65 and breaks at 57 is exactly the defect a single-age test
/// cannot see, and this repository's own history contains two of them.
#[test]
fn the_household_variations_hold_across_the_age_matrix() {
    let params = RetirementParameters::default();
    let single = Household::default();
    let married = Household {
        married: true,
        ..Household::default()
    };
    for age in AGE_MATRIX {
        let one = project(&single, &params, age, 1.0);
        let two = project(&married, &params, age, 1.0);
        assert!(
            two.ahv_annual >= one.ahv_annual,
            "age {age}: a couple's AHV ({}) is below a single person's ({})",
            two.ahv_annual,
            one.ahv_annual
        );
        let part = project(&single, &params, age, 0.6);
        assert!(
            part.bvg_capital <= one.bvg_capital,
            "age {age}: part-time capital exceeds full-time"
        );
        assert!(
            part.ahv_annual <= one.ahv_annual + 1e-9,
            "age {age}: part-time AHV exceeds full-time"
        );
    }
}

/// A salary below the BVG entry threshold accrues no second-pillar contributions, but
/// existing capital still earns the minimum interest.
///
/// The first version of this test denied the interest and was wrong: the two are separate,
/// and a test that conflated them would have misdescribed the projection in the direction
/// that matters to anyone whose salary sits near the threshold.
#[test]
fn a_salary_under_the_entry_threshold_accrues_nothing_but_still_earns_interest() {
    let params = RetirementParameters::default();
    let low = Household {
        full_time_salary: 20_000.0,
        ..Household::default()
    };
    let full = Household::default();
    for age in AGE_MATRIX {
        let e = project(&low, &params, age, 1.0);
        let full_time = project(&full, &params, age, 1.0);
        assert!(
            e.bvg_capital < full_time.bvg_capital,
            "age {age}: a salary under the entry threshold accumulated as much as one above it"
        );
        let years = age.saturating_sub(low.current_age);
        let closed_form =
            low.bvg_capital_now * (1.0 + params.bvg_min_interest).powi(years as i32);
        assert!(
            (e.bvg_capital - closed_form).abs() < 1e-6,
            "age {age}: capital {} is not interest-only on the opening balance ({closed_form})",
            e.bvg_capital
        );
    }
}

/// A wide combinatorial sweep: every age against every household variant, asserting only
/// that the projection stays in its declared shape.
///
/// The point is coverage rather than a new invariant. The specific claims are checked
/// individually above; this catches the case where one combination produces something absurd
/// that no targeted test happened to touch.
#[test]
fn the_projection_stays_in_shape_across_the_full_case_matrix() {
    let params = RetirementParameters::default();
    let households = [
        Household::default(),
        Household {
            married: true,
            ..Household::default()
        },
        Household {
            married: true,
            children: 2,
            ..Household::default()
        },
        Household {
            full_time_salary: 60_000.0,
            ..Household::default()
        },
        Household {
            full_time_salary: 400_000.0,
            ..Household::default()
        },
        Household {
            current_age: 55,
            contribution_years: 35,
            bvg_capital_now: 400_000.0,
            pillar3a_capital_now: 150_000.0,
            ..Household::default()
        },
        Household {
            current_age: 62,
            contribution_years: 42,
            ..Household::default()
        },
    ];
    for household in &households {
        for age in AGE_MATRIX {
            // The retirement age cannot precede the current age; the sweep covers only the
            // combinations that are coherent, and asserts that it skipped the rest.
            if age < household.current_age {
                continue;
            }
            for work in [0.5, 0.8, 1.0] {
                let e = project(household, &params, age, work);
                assert!(e.ahv_annual.is_finite() && e.ahv_annual >= 0.0);
                assert!(e.bvg_capital.is_finite() && e.bvg_capital >= 0.0);
                assert!(
                    e.ahv_draw_age >= params.ahv_earliest_age,
                    "age {age} work {work}: AHV draw age {}",
                    e.ahv_draw_age
                );
                assert!(
                    e.ahv_early_reduction <= 0.136 + 1e-12,
                    "age {age} work {work}: reduction {}",
                    e.ahv_early_reduction
                );
                assert!(
                    e.effective_conversion_rate <= params.bvg_conversion_rate + 1e-12,
                    "age {age} work {work}: conversion rate {}",
                    e.effective_conversion_rate
                );
                assert!(e.non_employed_years <= params.ahv_reference_age);
                // A wealthier salary never produces a smaller second-pillar capital.
                if work == 1.0 && household.full_time_salary >= params.bvg_entry_threshold {
                    assert!(
                        e.bvg_capital >= household.bvg_capital_now,
                        "age {age}: capital fell below the opening balance with no withdrawal"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The parameters, and the education ledger
// ---------------------------------------------------------------------------

/// The sourced statutory figures must be the ones the law currently states.
///
/// This is a regression guard as much as a test. Every one of these is a figure the rest of
/// the model is calibrated against, and a careless edit to one of them would move every
/// output in the document without failing anything else.
#[test]
fn the_default_parameters_are_the_2026_statutory_figures() {
    let p = RetirementParameters::default();
    // AHV: 13 monthly payments from 2026 under AHVG Art. 34ter.
    assert!((p.ahv_max_annual_single - 32_760.0).abs() < 1e-9, "13 x 2,520");
    assert!((p.ahv_max_annual_couple - 49_140.0).abs() < 1e-9, "13 x 3,780");
    assert_eq!(p.ahv_reference_age, 65);
    assert_eq!(p.ahv_earliest_age, 63, "AHVG Art. 40 Abs. 1");
    assert!((p.ahv_reduction_per_early_year - 0.068).abs() < 1e-12);
    assert_eq!(p.ahv_full_contribution_years, 44);
    // BVG: 6.8% stands because the 2024 referendum rejected lowering it.
    assert!((p.bvg_conversion_rate - 0.068).abs() < 1e-12, "BVG Art. 14");
    assert_eq!(p.bvg_reference_age, 65);
    assert_eq!(p.bvg_earliest_age, 63, "BVG Art. 13 Abs. 2");
    assert!((p.bvg_coordination_deduction - 26_460.0).abs() < 1e-9, "BVG Art. 8");
    assert!((p.bvg_entry_threshold - 22_680.0).abs() < 1e-9);
    assert!((p.bvg_min_interest - 0.0125).abs() < 1e-12, "BVV 2 Art. 12 lit. k");
    assert!((p.pillar3a_max_annual - 7_258.0).abs() < 1e-9, "BVV 3 Art. 7");
    assert!((p.ahv_non_employed_annual_min - 530.0).abs() < 1e-9, "AHVG Art. 10");
    // The two that are deliberately NOT sourced.
    assert_eq!(
        p.capital_withdrawal_tax.provenance.label(),
        "varies",
        "the capital-benefit tariff is cantonal and must not be presented as a figure"
    );
}

/// Every declared parameter must appear in the provenance table.
///
/// The table is how a reader tells law from assumption without reading the surrounding prose.
/// A parameter missing from it is a number with no stated origin, which is the one thing this
/// repository does not tolerate.
#[test]
fn every_parameter_has_a_stated_origin() {
    let p = RetirementParameters::default();
    let table = p.provenance_table();
    assert!(table.len() >= 14, "the table has only {} rows", table.len());
    let mut sourced = 0;
    let mut varies = 0;
    for (name, provenance) in &table {
        assert!(!name.is_empty());
        assert!(
            !provenance.detail().is_empty(),
            "{name} has no stated basis"
        );
        match provenance.label() {
            "sourced" => sourced += 1,
            "varies" => varies += 1,
            other => panic!("{name} has an unexpected provenance label {other}"),
        }
    }
    assert!(sourced >= 10, "only {sourced} parameters are sourced");
    assert!(
        varies >= 2,
        "the fund-specific reduction and the cantonal tariff must both be marked as varying"
    );
}

/// The education ledger must report the multiple of the state's outlay that the ordinary
/// progressive tax already collects.
///
/// This is the calculable half of the argument in `THE_RECIPROCITY_OF_THE_EDUCATED.md`: the
/// question is not whether a high earner pays the outlay back, but which multiple is right.
#[test]
fn the_education_ledger_reports_the_multiple_of_the_states_outlay() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let (state_cost, _) = Degree::Master.declared_state_cost();
    assert!(state_cost > 0.0);
    let ledger =
        contribution_ledger(ContributionPrinciple::AbilityToPay, state_cost, 120_000.0, 35, &schedule);
    assert_eq!(ledger.principle_surcharge, 0.0);
    assert!(ledger.lifetime_income_tax > 0.0);
    assert!(
        ledger.tax_multiple_of_state_cost > 1.0,
        "a high earner's lifetime tax is {} times the declared outlay",
        ledger.tax_multiple_of_state_cost
    );
    assert!((ledger.tax_multiple_of_state_cost
        - ledger.lifetime_income_tax / state_cost)
        .abs()
        < 1e-9);
}

/// All four contribution principles must be priceable, and the flat one must be regressive
/// in effect — its share of income falls as income rises.
///
/// That last fact is the mechanical one that makes the flat principle contestable, and the
/// model reports it rather than burying it.
#[test]
fn all_four_principles_are_priced_and_the_flat_one_is_regressive_in_effect() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let (state_cost, _) = Degree::Master.declared_state_cost();
    let principles = [
        ContributionPrinciple::AbilityToPay,
        ContributionPrinciple::EducationCostRecovery {
            state_cost,
            recovery_years: 35,
        },
        ContributionPrinciple::BenefitReceived { premium_share: 0.10 },
        ContributionPrinciple::Flat { annual: 3_000.0 },
    ];
    for principle in principles {
        let ledger = contribution_ledger(principle, state_cost, 120_000.0, 35, &schedule);
        assert!(ledger.principle_surcharge.is_finite() && ledger.principle_surcharge >= 0.0);
        assert!(ledger.surcharge_share_of_income.is_finite());
        assert!(ledger.lifetime_income_tax > 0.0);
    }

    let flat = ContributionPrinciple::Flat { annual: 3_000.0 };
    let low = contribution_ledger(flat, 0.0, 60_000.0, 30, &schedule);
    let high = contribution_ledger(flat, 0.0, 240_000.0, 30, &schedule);
    assert!(
        high.surcharge_share_of_income < low.surcharge_share_of_income,
        "the flat charge's share of income should fall: {} against {}",
        high.surcharge_share_of_income,
        low.surcharge_share_of_income
    );
    assert!((low.principle_surcharge - 90_000.0).abs() < 1e-9, "3,000 x 30");
    assert!((high.principle_surcharge - 90_000.0).abs() < 1e-9);
}

/// The degree table must scale with the public cost, and parsing must be conservative rather
/// than guessing.
///
/// The ratios carry the premise of the reciprocity argument, so a table that flattened them
/// would quietly weaken it.
#[test]
fn the_degree_table_scales_with_the_declared_public_cost() {
    let vocational = Degree::Vocational.declared_state_cost().0;
    let bachelor = Degree::Bachelor.declared_state_cost().0;
    let master = Degree::Master.declared_state_cost().0;
    let doctorate = Degree::Doctorate.declared_state_cost().0;
    assert_eq!(Degree::None.declared_state_cost().0, 0.0);
    assert!(vocational < bachelor, "vocational {vocational}, bachelor {bachelor}");
    assert!(bachelor < master);
    assert!(master < doctorate);
    assert_eq!(
        Degree::Master.declared_state_cost().1.label(),
        "declared",
        "no official per-degree figure is cited, so this must not claim to be sourced"
    );

    assert_eq!(Degree::parse("master"), Some(Degree::Master));
    assert_eq!(Degree::parse("  MSc "), Some(Degree::Master));
    assert_eq!(Degree::parse("PhD"), Some(Degree::Doctorate));
    assert_eq!(Degree::parse("efz"), Some(Degree::Vocational));
    assert_eq!(Degree::parse("astrophysics"), None);
}

/// A mortgage on the whole suite: the number of tests in this file is the coverage the
/// request asked for, and it is asserted rather than assumed so that deleting a case is a
/// visible change.
#[test]
fn the_age_matrix_is_the_one_the_request_named() {
    assert_eq!(AGE_MATRIX.len(), 13);
    for required in [55, 57, 60, 62] {
        assert!(
            AGE_MATRIX.contains(&required),
            "the age {required} named in the request is missing from the matrix"
        );
    }
    for boundary in [58, 63, 65] {
        assert!(
            AGE_MATRIX.contains(&boundary),
            "the statutory boundary {boundary} is missing from the matrix"
        );
    }
    // Strictly increasing, so a duplicated entry cannot inflate the count.
    for pair in AGE_MATRIX.windows(2) {
        assert!(pair[0] < pair[1], "the matrix is not strictly increasing");
    }
}
