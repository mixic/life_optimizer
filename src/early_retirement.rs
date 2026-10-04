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

//! Early retirement as an **optimisation problem**, not a projection.
//!
//! # What this replaces
//!
//! `monte_carlo.rs` answers "given this retirement age and this work percentage, what
//! pension results, and how often does it run out?". That is a projection, and it
//! leaves the actual decision to the reader. This module answers the decision:
//! **what is the least a person must accumulate to fund the consumption they have
//! declared, and how should it be taken out?** The second half matters as much as the
//! first, because in a progressive system the *timing* of a withdrawal can move the tax
//! on it by more than a year of contributions does.
//!
//! # The three claims this module makes, and how each is checked
//!
//! 1. **The withdrawal problem is a linear program.** Given a banded tax schedule — an
//!    allowance, then bands of constant marginal rate — allocating a fixed capital
//!    across years to minimise total tax is an LP. It is solved by
//!    [`crate::simplex`], and the dual of the "place the whole capital" constraint is
//!    the **marginal tax rate of the last franc withdrawn**, which is the number the
//!    whole exercise is about.
//! 2. **Its optimum is the greedy one, and the greedy one is the equalised one.** Three
//!    independent routes — simplex, cheapest-band-first filling, and the KKT
//!    characterisation that marginal rates are equalised across the years that receive
//!    anything — must agree. They are computed separately and cross-checked, because a
//!    solver that agrees only with itself proves nothing.
//! 3. **Over-saving is a cost, not a safety margin.** The objective counts the terminal
//!    pot that was never consumed, at a declared weight, so that "save enough to be safe
//!    forever" is not silently optimal. That weight is a preference and is exposed as
//!    one; see [`Preferences::terminal_wealth_weight`].
//!
//! # The normative question, which this module prices and does not answer
//!
//! A highly educated worker was educated at public expense and earns more, so their
//! lifetime tax already exceeds a low earner's by a large multiple. Whether it *should*
//! exceed it by more, and by how much, is a question about distributive justice with at
//! least four defensible answers — ability to pay, benefit received, recovery of the
//! state's investment, and flat. **This module computes what each principle implies and
//! refuses to rank them.** [`ContributionPrinciple`] is a declared input; the report
//! prints the ledger for every principle side by side. A model that baked in one of them
//! would be presenting an argument as a calculation, which is the failure mode the rest
//! of this repository is organised against.
//!
//! # Provenance
//!
//! Every parameter carries a [`Provenance`], and [`RetirementParameters::provenance_table`]
//! prints them together. Statutory Swiss figures reuse the citations already in
//! `monte_carlo.rs` rather than restating them; anything fund-specific or cantonal is
//! marked **declared**, because it genuinely varies and a single number for it would be
//! an invention with a citation-shaped hole where the caveat should be.

use crate::consumption::ConsumptionTiers;
use crate::simplex;
use crate::tax::TaxSchedule;

/// Where a number came from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provenance {
    /// Published and citable, at the named source and vintage.
    Sourced {
        source: &'static str,
        vintage: &'static str,
    },
    /// A statutory figure that varies by fund or canton. The *rule* is sourced; the
    /// number is not, because there is no single number.
    VariesByFundOrCanton { basis: &'static str },
    /// Invented: plausible in sign and rough magnitude, not calibrated.
    Declared { rationale: &'static str },
}

impl Provenance {
    pub fn label(self) -> &'static str {
        match self {
            Provenance::Sourced { .. } => "sourced",
            Provenance::VariesByFundOrCanton { .. } => "varies",
            Provenance::Declared { .. } => "declared",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Provenance::Sourced { source, .. } => source,
            Provenance::VariesByFundOrCanton { basis } => basis,
            Provenance::Declared { rationale } => rationale,
        }
    }

    pub fn is_sourced(self) -> bool {
        matches!(self, Provenance::Sourced { .. })
    }
}

/// A band of constant marginal tax rate, applied to a year's taxable retirement income.
///
/// Bands rather than a smooth curve, because a banded schedule is what makes the
/// allocation an LP, and because the capital-withdrawal taxes this module exists to
/// optimise are in fact published as banded or stepped tables.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaxBand {
    /// Width of the band, in francs of taxable income.
    pub width: f64,
    /// Marginal rate applied within it.
    pub marginal_rate: f64,
}

/// A banded tax schedule: an exemption, then the bands above it.
#[derive(Debug, Clone, PartialEq)]
pub struct BandedTax {
    /// Income taxed at zero before the first band begins.
    pub allowance: f64,
    pub bands: Vec<TaxBand>,
    pub provenance: Provenance,
}

impl BandedTax {
    /// Total tax on a taxable income, by filling the bands in order.
    pub fn tax_on(&self, taxable: f64) -> f64 {
        let mut remaining = (taxable - self.allowance).max(0.0);
        let mut tax = 0.0;
        for band in &self.bands {
            if remaining <= 0.0 {
                break;
            }
            let filled = remaining.min(band.width.max(0.0));
            tax += filled * band.marginal_rate;
            remaining -= filled;
        }
        // Income above the last band is charged at the last band's rate rather than
        // escaping the schedule. A schedule that ran out would make large withdrawals
        // free at the margin, which is both wrong and would make the LP unbounded in the
        // direction the optimiser wants to go.
        if remaining > 0.0 {
            if let Some(last) = self.bands.last() {
                tax += remaining * last.marginal_rate;
            }
        }
        tax
    }

    /// The marginal rate applying to the next franc above `taxable`.
    pub fn marginal_rate_at(&self, taxable: f64) -> f64 {
        let mut remaining = (taxable - self.allowance).max(0.0);
        for band in &self.bands {
            if remaining < band.width {
                return band.marginal_rate;
            }
            remaining -= band.width;
        }
        self.bands.last().map(|band| band.marginal_rate).unwrap_or(0.0)
    }

    /// Total width of the bands, for reporting how many years a capital must be spread
    /// over to stay inside the cheapest ones.
    pub fn total_width(&self) -> f64 {
        self.bands.iter().map(|band| band.width).sum()
    }
}

/// The Swiss statutory and declared parameters the projection needs.
#[derive(Debug, Clone, PartialEq)]
pub struct RetirementParameters {
    /// Maximum annual AHV old-age pension, single person.
    pub ahv_max_annual_single: f64,
    /// Maximum annual AHV old-age pension, married couple.
    pub ahv_max_annual_couple: f64,
    /// AHV reference age for a full pension.
    pub ahv_reference_age: u32,
    /// Earliest age at which AHV may be drawn.
    pub ahv_earliest_age: u32,
    /// Actuarial reduction per year of early AHV draw.
    pub ahv_reduction_per_early_year: f64,
    /// Contribution years needed for a full AHV pension.
    pub ahv_full_contribution_years: u32,
    /// Statutory BVG minimum conversion rate (the Umwandlungssatz).
    pub bvg_conversion_rate: f64,
    /// Age at which the full BVG conversion rate applies.
    pub bvg_reference_age: u32,
    /// Reduction in the conversion rate per year of early BVG withdrawal.
    pub bvg_early_reduction_per_year: f64,
    /// Earliest age at which BVG capital may be drawn.
    pub bvg_earliest_age: u32,
    /// BVG coordination deduction: the part of salary above which contributions begin.
    pub bvg_coordination_deduction: f64,
    /// BVG entry threshold: earnings below this are not insured.
    pub bvg_entry_threshold: f64,
    /// BVG minimum interest rate credited to the retirement capital.
    pub bvg_min_interest: f64,
    /// Maximum annual Pillar 3a contribution for an employee with a pension fund.
    pub pillar3a_max_annual: f64,
    /// Minimum annual AHV contribution for a non-employed person (AHV alone).
    pub ahv_non_employed_annual_min: f64,
    /// Maximum annual AHV/IV/EO contribution for a non-employed person.
    pub ahv_non_employed_annual_max: f64,
    /// Tax on a lump-sum capital withdrawal, as a banded schedule.
    pub capital_withdrawal_tax: BandedTax,
}

impl Default for RetirementParameters {
    /// Defaults are the **2026** statutory figures where the law fixes them, and are
    /// marked **declared** where it does not. The distinction is load-bearing here: the
    /// conversion rate's *early-withdrawal reduction* has no statutory schedule at all —
    /// each fund writes its own — so a single number for it would be an invention wearing
    /// a citation.
    fn default() -> Self {
        RetirementParameters {
            // 13 monthly payments from 2026 under AHVG Art. 34ter: 13 x 2,520 single and
            // 13 x 3,780 for a couple. The 13th pension is why these are not 12 x the
            // monthly figure, and using the pre-2026 annual amounts would understate a
            // funded plan by a full month of pension.
            ahv_max_annual_single: 32_760.0,
            ahv_max_annual_couple: 49_140.0,
            ahv_reference_age: 65,
            ahv_earliest_age: 63,
            ahv_reduction_per_early_year: 0.068,
            ahv_full_contribution_years: 44,
            bvg_conversion_rate: crate::monte_carlo::STATUTORY_CONVERSION_RATE,
            bvg_reference_age: 65,
            // Declared. No statutory or ordinance schedule exists for the reduction on
            // early withdrawal: it is a reglementary benefit. Two published funds give
            // about 0.27 and 0.13 **percentage points** a year — 0.0027 and 0.0013 as
            // decimals — so the value here sits between them. The first version of this
            // default was 0.02, which is two percentage points a year and about ten times
            // too steep: it turned a 6.8% rate into 2.8% at age 63 and made early
            // retirement look far more expensive than any published fund makes it.
            bvg_early_reduction_per_year: 0.002,
            // BVG Art. 13 Abs. 2: statutory early draw from completed age 63. Funds may
            // permit earlier, from 58, under BVV 2 Art. 1i — which is a plan feature and
            // not a right, so the statutory floor is what defaults here.
            bvg_earliest_age: 63,
            bvg_coordination_deduction: 26_460.0,
            bvg_entry_threshold: 22_680.0,
            bvg_min_interest: 0.0125,
            pillar3a_max_annual: 7_258.0,
            // Non-employed AHV contributions are a fixed table amount, not a rate:
            // AHVG Art. 10 Abs. 1 gives a minimum of CHF 435 and a maximum of fifty times
            // it. A retiree below the reference age who is not working owes these, and
            // leaving them out would make the bridge years look cheaper than they are.
            ahv_non_employed_annual_min: 530.0,
            ahv_non_employed_annual_max: 26_500.0,
            capital_withdrawal_tax: BandedTax {
                allowance: 0.0,
                bands: vec![
                    TaxBand {
                        width: 50_000.0,
                        marginal_rate: 0.0065,
                    },
                    TaxBand {
                        width: 150_000.0,
                        marginal_rate: 0.012,
                    },
                    TaxBand {
                        width: 300_000.0,
                        marginal_rate: 0.02,
                    },
                    TaxBand {
                        width: 500_000.0,
                        marginal_rate: 0.03,
                    },
                ],
                provenance: Provenance::VariesByFundOrCanton {
                    basis: "the capital-benefit rate is set by the canton of domicile: the \
                            federal share is one fifth of the ordinary tariff (DBG Art. 38), \
                            while cantonal tariffs run from a flat 2% (Zurich, Thurgau) to a \
                            progressive scale reaching about 6% (Geneva). The bands here are \
                            a declared stand-in in that shape",
                },
            },
        }
    }
}

impl RetirementParameters {
    /// Every parameter with its provenance, so a reader can tell law from assumption
    /// without reading the surrounding prose.
    pub fn provenance_table(&self) -> Vec<(&'static str, Provenance)> {
        vec![
            (
                "AHV maximum, single",
                Provenance::Sourced {
                    source: "BSV, Beträge gültig ab dem 1. Januar 2026: maximum single old-age \
                             pension CHF 2,520 a month, and 13 monthly payments from 2026 under \
                             AHVG Art. 34ter",
                    vintage: "2026",
                },
            ),
            (
                "AHV maximum, couple",
                Provenance::Sourced {
                    source: "BSV, Beträge gültig ab dem 1. Januar 2026: couple maximum CHF 3,780 \
                             a month, capped at 150% of the single maximum by AHVG Art. 35; \
                             13 payments from 2026",
                    vintage: "2026",
                },
            ),
            (
                "AHV reference age",
                Provenance::Sourced {
                    source: "AHVG Art. 21 Abs. 1, as amended by AHV 21 in force 1 January 2024: \
                             65 for women and men, with a transition for the cohorts of 1961 to \
                             1963",
                    vintage: "2024",
                },
            ),
            (
                "AHV earliest draw, and reduction",
                Provenance::Sourced {
                    source: "AHVG Art. 40 Abs. 1 (earliest draw at completed age 63) and AHVV \
                             Art. 56bis (6.8% a year, maximum 13.6%). AHVG Art. 40a Abs. 3 cuts \
                             the reduction by 40% for low incomes",
                    vintage: "2025",
                },
            ),
            (
                "AHV full contribution years",
                Provenance::Sourced {
                    source: "AHVG Art. 29: contribution years equal to the cohort's; 44 for men \
                             and for women from the 1964 cohort",
                    vintage: "2026",
                },
            ),
            (
                "BVG conversion rate (Umwandlungssatz)",
                Provenance::Sourced {
                    source: "BVG Art. 14 Abs. 2: a minimum conversion rate of 6.8% at reference \
                             age 65. Introduced by the 1st BVG revision (AS 2004 1677) and \
                             phased in, reaching 6.8% for both sexes in 2014. The attempt to \
                             lower it to 6.0% was REJECTED in the referendum of 22 September \
                             2024 by 1,655,513 to 810,569 (BBl 2025 1534), so the rate stands",
                    vintage: "6.8% since the phase-in completed in 2014; reform rejected 2024",
                },
            ),
            (
                "BVG early-withdrawal reduction",
                Provenance::VariesByFundOrCanton {
                    basis: "there is NO statutory or ordinance schedule. The reduction is a \
                            reglementary benefit each fund writes into its own Vorsorgereglement. \
                            Two published funds differ by about a factor of two per year of early \
                            draw, so this must come from the fund's own statement",
                },
            ),
            (
                "BVG reference age and earliest draw",
                Provenance::Sourced {
                    source: "BVG Art. 13 Abs. 1 (reference age 65, via AHVG Art. 21), BVG Art. 13 \
                             Abs. 2 (statutory early draw from completed age 63, deferral to 70), \
                             BVV 2 Art. 1i (funds may permit retirement from 58)",
                    vintage: "2024",
                },
            ),
            (
                "BVG coordination deduction and entry threshold",
                Provenance::Sourced {
                    source: "BVG Art. 8 Abs. 1 and BVV 2 Art. 5: coordination deduction CHF 26,460 \
                             and entry threshold CHF 22,680, upper limit CHF 90,720, unchanged \
                             for 2026",
                    vintage: "2026",
                },
            ),
            (
                "BVG minimum interest",
                Provenance::Sourced {
                    source: "BVV 2 Art. 12 lit. k: 1.25% from 1 January 2024, confirmed unchanged \
                             for 2026 by the Federal Council on 5 November 2025",
                    vintage: "2026",
                },
            ),
            (
                "BVG savings credit rates",
                Provenance::Sourced {
                    source: "BVG Art. 16: 7% at 25-34, 10% at 35-44, 15% at 45-54, 18% from 55; \
                             the employee share may not exceed half (BVG Art. 16 Abs. 1)",
                    vintage: "2026",
                },
            ),
            (
                "Pillar 3a maximum",
                Provenance::Sourced {
                    source: "BVV 3 Art. 7 Abs. 1 lit. a: 8% of the BVG upper limit, giving \
                             CHF 7,258 for an employee with a pension fund; 20% of earned income \
                             capped at CHF 36,288 for someone without one",
                    vintage: "2026",
                },
            ),
            (
                "AHV contributions of a non-employed early retiree",
                Provenance::Sourced {
                    source: "AHVG Art. 3 Abs. 1bis (liability continues to the reference age) and \
                             AHVG Art. 10 Abs. 1 (a table amount, minimum CHF 435 and maximum \
                             fifty times it, assessed on wealth and rental income under AHVV \
                             Art. 28). Note this is a fixed amount, not a percentage",
                    vintage: "2026",
                },
            ),
            (
                "Capital-withdrawal tax bands",
                Provenance::VariesByFundOrCanton {
                    basis: "the federal share is one fifth of the ordinary tariff (DBG Art. 38); \
                            the cantonal tariff is set by the canton of domicile and runs from a \
                            flat 2% (Zurich, Thurgau) to a progressive scale reaching about 6% \
                            (Geneva). The bands are a declared stand-in in that shape",
                },
            ),
        ]
    }
}

/// Who the plan is for.
#[derive(Debug, Clone, PartialEq)]
pub struct Household {
    pub current_age: u32,
    /// Salary at 100% employment.
    pub full_time_salary: f64,
    pub married: bool,
    pub children: u32,
    /// AHV contribution years already accrued at `current_age`.
    pub contribution_years: u32,
    pub bvg_capital_now: f64,
    pub pillar3a_capital_now: f64,
    /// Taxable savings outside the pension wrappers.
    pub taxable_savings: f64,
    pub life_expectancy: u32,
}

impl Default for Household {
    fn default() -> Self {
        Household {
            current_age: 45,
            full_time_salary: 120_000.0,
            married: true,
            children: 1,
            contribution_years: 25,
            bvg_capital_now: 180_000.0,
            pillar3a_capital_now: 60_000.0,
            taxable_savings: 50_000.0,
            life_expectancy: 90,
        }
    }
}

/// What the household has decided it needs to spend, from the consumption model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumptionNeed {
    /// The floor: the inelastic and committed part, which must be funded.
    pub mandatory_annual: f64,
    /// The lifestyle target, which is what the optimisation tries to fund.
    pub lifestyle_annual: f64,
}

impl ConsumptionNeed {
    /// Take the two levels from a [`ConsumptionTiers`], which is where this repository
    /// already decides what is mandatory and what is discretionary. Converting from the
    /// existing model rather than re-declaring the basket is the point: a second
    /// definition of "needed" would be free to disagree with the first.
    pub fn from_tiers(tiers: &ConsumptionTiers) -> Self {
        ConsumptionNeed {
            mandatory_annual: tiers.mandatory_monthly() * 12.0,
            lifestyle_annual: tiers.total_monthly() * 12.0,
        }
    }
}

/// What the household is entitled to, at a given retirement age.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entitlements {
    pub retirement_age: u32,
    pub years_to_retirement: u32,
    /// AHV annual pension, after any early-draw reduction.
    pub ahv_annual: f64,
    /// The reduction actually applied, as a fraction.
    pub ahv_early_reduction: f64,
    pub bvg_capital: f64,
    /// The conversion rate applied to that capital, after early-withdrawal reduction.
    pub effective_conversion_rate: f64,
    /// Annual BVG pension if the whole capital is annuitised.
    pub bvg_annuity_annual: f64,
    pub pillar3a_capital: f64,
    /// Capital available to fund the years between retirement and the AHV/annuity start.
    pub bridge_capital: f64,
}

/// Project entitlements for a retirement age and employment level.
///
/// # What is modelled and what is not
///
/// Contributions accrue at a flat rate on insured salary, credited at the minimum
/// interest rate. That is deliberately the *statutory floor* and not a forecast of any
/// fund's crediting policy: a projection that assumed above-minimum returns would be
/// presenting an investment view as a pension entitlement.
///
/// `work_fraction` scales salary, and therefore both contributions and the AHV
/// contribution record.
pub fn project(
    household: &Household,
    params: &RetirementParameters,
    retirement_age: u32,
    work_fraction: f64,
) -> Entitlements {
    let work_fraction = work_fraction.clamp(0.0, 1.0);
    let years_to_retirement = retirement_age.saturating_sub(household.current_age);

    // ---- AHV ---------------------------------------------------------------
    //
    // The pension scales with the contribution record up to the full number of years,
    // then with the early-draw reduction. Scaling by the record is what makes an early
    // retirement cost something in Pillar 1 as well as Pillar 2, and leaving it out
    // would understate the price of retiring early by a wide margin.
    let accrued_years = household.contribution_years
        + (f64::from(years_to_retirement) * work_fraction).round() as u32;
    let record_fraction =
        (f64::from(accrued_years) / f64::from(params.ahv_full_contribution_years.max(1))).min(1.0);
    let full_pension = if household.married {
        params.ahv_max_annual_couple
    } else {
        params.ahv_max_annual_single
    };
    let early_years = params
        .ahv_reference_age
        .saturating_sub(retirement_age)
        .min(params.ahv_reference_age.saturating_sub(params.ahv_earliest_age));
    let ahv_early_reduction = f64::from(early_years) * params.ahv_reduction_per_early_year;
    let ahv_annual = full_pension * record_fraction * (1.0 - ahv_early_reduction).max(0.0);

    // ---- BVG ---------------------------------------------------------------
    let insured_salary =
        (household.full_time_salary * work_fraction - params.bvg_coordination_deduction).max(0.0);
    let insured = if household.full_time_salary * work_fraction >= params.bvg_entry_threshold {
        insured_salary
    } else {
        0.0
    };
    // The statutory age-banded savings rates, applied by the age at each year of service.
    // Flat-rate would misstate the accumulation of anyone retiring in their fifties,
    // because the bands step up sharply at 45 and again at 55.
    let mut capital = household.bvg_capital_now;
    for year in 0..years_to_retirement {
        let age = household.current_age + year;
        let rate = bvg_savings_rate(age);
        let contribution = insured * rate * work_fraction;
        capital = capital * (1.0 + params.bvg_min_interest) + contribution;
    }
    let bvg_capital = capital;

    // Every year of early withdrawal cuts the conversion rate, so the annuity bought with
    // a given capital is smaller as well as the capital itself being smaller. The first
    // version of this formula compared the retirement age against the *earliest permitted*
    // age, which is always zero years of reduction and silently made early retirement
    // free in Pillar 2. The test below caught it.
    let bvg_early_years = params.bvg_reference_age.saturating_sub(retirement_age);
    let effective_conversion_rate = (params.bvg_conversion_rate
        - f64::from(bvg_early_years) * params.bvg_early_reduction_per_year)
        .max(0.0);

    // ---- Pillar 3a ---------------------------------------------------------
    let mut pillar3a = household.pillar3a_capital_now;
    for _ in 0..years_to_retirement {
        pillar3a = pillar3a * (1.0 + params.bvg_min_interest)
            + params.pillar3a_max_annual * work_fraction;
    }

    Entitlements {
        retirement_age,
        years_to_retirement,
        ahv_annual,
        ahv_early_reduction,
        bvg_capital,
        effective_conversion_rate,
        bvg_annuity_annual: bvg_capital * effective_conversion_rate,
        pillar3a_capital: pillar3a,
        bridge_capital: household.taxable_savings,
    }
}

/// The statutory BVG savings contribution rate for an age, in the mandatory scheme.
///
/// The bands are the statutory ones: a low rate under 35, then 10%, then 15%, then 18%
/// from 55. Written as a function rather than a table lookup so the age used is the age
/// *in that service year*, which is the detail a flat average gets wrong.
pub fn bvg_savings_rate(age: u32) -> f64 {
    match age {
        0..=24 => 0.0,
        25..=34 => 0.07,
        35..=44 => 0.10,
        45..=54 => 0.15,
        _ => 0.18,
    }
}

/// Where the retirement income comes from, year by year.
///
/// One row per retirement year, because the whole point of the model is that the
/// allocation *across* years is a decision. A single "annual withdrawal" figure would
/// have thrown away the optimisation.
#[derive(Debug, Clone, PartialEq)]
pub struct WithdrawalPlan {
    /// Capital withdrawn in each year of retirement, in order.
    pub capital_withdrawn: Vec<f64>,
    /// Total ordinary tax paid across the plan, including tax on the pension income
    /// that is already committed.
    pub total_tax: f64,
    /// Total capital placed.
    pub total_capital: f64,
    /// The marginal tax rate of the last franc withdrawn, from the LP dual. This is the
    /// number that says whether the plan is spread thinly enough.
    pub marginal_rate_at_optimum: f64,
}

/// The bands actually applied, with the allowance promoted to a real zero-rate band.
///
/// The allowance has to be a band rather than a subtraction for the allocation to be an
/// LP over *income*: without it, the variables would only span income above the
/// allowance, and a year whose committed income sits below the allowance would have
/// untaxed room the optimiser could not see. That was the bug in the first version — the
/// placement constraint then summed income above the allowance against a target that
/// counted all of it, and neither the solver nor the greedy fill could place the capital.
fn effective_bands(schedule: &BandedTax) -> Vec<TaxBand> {
    let mut bands = Vec::new();
    if schedule.allowance > 0.0 {
        bands.push(TaxBand {
            width: schedule.allowance,
            marginal_rate: 0.0,
        });
    }
    bands.extend(schedule.bands.iter().copied());
    bands
}

/// The outcome of allocating a capital across years.
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationOutcome {
    Optimal(WithdrawalPlan),
    /// The capital cannot be placed: a band's width limit, or the schedule, makes the
    /// problem infeasible. Reported rather than returned as a partial plan, because a
    /// partial plan would look like a strategy.
    Infeasible(String),
}

/// Allocate `capital` across `years` of retirement to minimise tax, by linear
/// programming.
///
/// # The formulation
///
/// One variable per (year, band) pair: how much of that year's taxable income sits in
/// that band. Each variable is capped at the band's width, the year's income is the sum
/// of its variables, and the only coupling between years is that the total capital has
/// to be placed. The objective sums each band's marginal rate over its variables.
///
/// # Why this is the right formulation rather than a convenient one
///
/// The dual of the "place the whole capital" constraint is the **marginal rate of the
/// last franc placed**. At the optimum every year that receives anything sits at that
/// same marginal rate — which is the KKT condition, and which is exactly the rule of
/// thumb the whole exercise is trying to make precise: *spread the withdrawal until each
/// year's top band is the same, and no further*.
///
/// `base_income` is the retirement income already committed in each year — the AHV and
/// BVG annuities — since the bands are filled from the bottom by that income before any
/// withdrawal is placed on top of it.
pub fn optimise_withdrawals(
    capital: f64,
    base_income: &[f64],
    schedule: &BandedTax,
) -> AllocationOutcome {
    if capital <= 0.0 {
        return AllocationOutcome::Optimal(WithdrawalPlan {
            capital_withdrawn: vec![0.0; base_income.len()],
            total_tax: base_income.iter().map(|b| schedule.tax_on(*b)).sum(),
            total_capital: 0.0,
            marginal_rate_at_optimum: 0.0,
        });
    }
    if base_income.is_empty() {
        return AllocationOutcome::Infeasible("no retirement years to allocate across".to_string());
    }

    // Headroom in each year before the last band is exhausted. Beyond this the marginal
    // rate is the last band's, so there is no reason to cap further.
    let room: Vec<f64> = base_income
        .iter()
        .map(|base| (schedule.total_width() - (base - schedule.allowance).max(0.0)).max(0.0))
        .collect();
    let total_room: f64 = room.iter().sum();
    if capital > total_room + 1e-9 {
        // Still feasible — everything above the last band is charged at the last band's
        // rate — but the caller should know the capital does not fit inside the cheap
        // bands, because that is the interesting economic fact about the plan.
    }

    let all_bands = effective_bands(schedule);
    let bands = all_bands.len();
    let years = base_income.len();
    let columns = years * bands;

    // Objective: the marginal rate of each (year, band) variable.
    let mut objective = Vec::with_capacity(columns);
    for _ in 0..years {
        for band in &all_bands {
            objective.push(band.marginal_rate);
        }
    }

    // Residual capacity of each (year, band) pair, given that the year already carries its
    // committed base income.
    //
    // Working in residuals removes the need for a per-year "at least the base"
    // constraint. That constraint would have had a negative right-hand side, which the
    // solver refuses — and rightly, because negating such a row to make it acceptable
    // silently flips the meaning of its dual.
    // Bound on the top band's variable. It has to be large enough never to bind and small
    // enough not to wreck the tableau's scaling: `1e12` was the first choice and made the
    // solver fail on ordinary inputs, because a column bounded at 1e12 next to rows whose
    // right-hand sides are ~1e5 is a badly conditioned problem, not a big one. The total
    // to place plus one cannot bind by construction.
    let top_bound = capital + base_income.iter().sum::<f64>() + 1.0;

    let mut available = vec![vec![0.0_f64; bands]; years];
    let mut start = 0.0;
    for band_index in 0..bands {
        let end = start + all_bands[band_index].width;
        for year in 0..years {
            available[year][band_index] = if band_index + 1 == bands {
                // The top band is uncapped upwards, so a capital larger than every cheap
                // band together remains placeable.
                top_bound
            } else {
                (end - base_income[year].max(start)).max(0.0)
            };
        }
        start = end;
    }

    let mut a_ub: Vec<Vec<f64>> = Vec::with_capacity(columns);
    let mut b_ub: Vec<f64> = Vec::with_capacity(columns);
    for year in 0..years {
        for band_index in 0..bands {
            let mut row = vec![0.0; columns];
            row[year * bands + band_index] = 1.0;
            a_ub.push(row);
            b_ub.push(available[year][band_index]);
        }
    }

    // No explicit bottom-up ordering constraints. They look necessary and are not: the
    // bands' rates increase, so a solution that used an expensive band while a cheaper
    // one still had room would be strictly improvable and cannot be optimal. Adding them
    // anyway was a real bug — once the committed base income fills the cheap bands, their
    // residual capacity is zero, and an ordering constraint then forces every higher band
    // to zero as well, making the whole problem infeasible for any capital at all.

    // The one coupling constraint: the whole capital is placed on top of the base.
    let mut placement = vec![0.0; columns];
    for year in 0..years {
        for band_index in 0..bands {
            placement[year * bands + band_index] = 1.0;
        }
    }
    let target = capital;

    let solution = match simplex::solve(&objective, &a_ub, &b_ub, &[placement], &[target]) {
        Ok(solution) => solution,
        Err(error) => return AllocationOutcome::Infeasible(error.to_string()),
    };
    if solution.status != simplex::LpStatus::Optimal {
        return AllocationOutcome::Infeasible(format!(
            "the allocation problem was {}",
            solution.status.label()
        ));
    }

    // Each variable is now the income placed *on top of* the committed base, so the
    // withdrawal is simply its row sum.
    let mut capital_withdrawn = Vec::with_capacity(years);
    for year in 0..years {
        let extra: f64 = (0..bands)
            .map(|band_index| solution.x[year * bands + band_index])
            .sum();
        capital_withdrawn.push(extra.max(0.0));
    }
    // The objective prices only the extra income, because the base is fixed. The tax on
    // the base is a constant of the problem, so it is added here for reporting — and it
    // has to be, or the LP figure and the greedy figure would measure different things.
    let base_tax: f64 = base_income.iter().map(|value| schedule.tax_on(*value)).sum();
    let extra_tax: f64 = (0..columns).map(|c| objective[c] * solution.x[c]).sum();

    AllocationOutcome::Optimal(WithdrawalPlan {
        capital_withdrawn,
        total_tax: base_tax + extra_tax,
        total_capital: capital,
        // The dual of the placement constraint is the marginal rate of the last franc
        // placed. Positive, because placing another franc costs tax.
        marginal_rate_at_optimum: solution.dual_eq[0],
    })
}

/// The allocation that equalises the marginal rate across the years it uses — the
/// water-filling solution, and the closed form the LP must reproduce.
///
/// # Why water-filling rather than cheapest-year-first
///
/// Both fill the cheapest band before touching the next, so both are tax-optimal. They
/// differ in *which* year gets the money within a band, and there the tax is indifferent —
/// so the choice has to be made on something else, and the something else decides whether
/// the plan works at all.
///
/// The first version of this function took the emptiest year at each step. That is
/// tax-equal and it **concentrates**: with a large surplus and generous cheap bands it puts
/// the whole surplus into the first year or two that have room, empties the pot, and leaves
/// every later year drawing only its annuity. The plan then fails not from bad luck but
/// from its own allocation — a 100% shortfall probability from a tax-optimal plan.
///
/// Water-filling instead raises every year in the current band **together**, which is
/// exactly the condition the LP's dual states: equalise the marginal rate. It spreads, it
/// is still tax-optimal, and it is the canonical solution to a separable convex allocation
/// with one budget constraint.
pub fn greedy_withdrawals(
    capital: f64,
    base_income: &[f64],
    schedule: &BandedTax,
) -> WithdrawalPlan {
    let years = base_income.len();
    let all_bands = effective_bands(schedule);
    let mut income: Vec<f64> = base_income.to_vec();
    let mut remaining = capital;

    for band in &all_bands {
        if remaining <= 1e-9 {
            break;
        }
        // Raise every year that is currently in this band together, in equal steps, until
        // the band is full everywhere or the money runs out.
        for _ in 0..10_000 {
            if remaining <= 1e-9 {
                break;
            }
            let candidates: Vec<usize> = (0..years)
                .filter(|year| {
                    (schedule.marginal_rate_at(income[*year]) - band.marginal_rate).abs() < 1e-12
                        && band_end_income(&all_bands, band.marginal_rate) - income[*year] > 1e-9
                })
                .collect();
            if candidates.is_empty() {
                break;
            }
            let share = remaining / candidates.len() as f64;
            let mut added = 0.0;
            for year in candidates {
                let headroom =
                    (band_end_income(&all_bands, band.marginal_rate) - income[year]).max(0.0);
                let take = share.min(headroom);
                income[year] += take;
                added += take;
            }
            if added <= 1e-9 {
                break;
            }
            remaining -= added;
        }
    }
    // Anything left goes into the emptiest year at the top rate. Only reachable when the
    // capital exceeds every band's room, in which case the top band takes the excess.
    if remaining > 1e-9 {
        if let Some(year) = (0..years).min_by(|a, b| {
            income[*a]
                .partial_cmp(&income[*b])
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            income[year] += remaining;
        }
    }

    let total_tax = income.iter().map(|value| schedule.tax_on(*value)).sum();
    WithdrawalPlan {
        capital_withdrawn: (0..years)
            .map(|year| (income[year] - base_income[year]).max(0.0))
            .collect(),
        total_tax,
        total_capital: capital,
        marginal_rate_at_optimum: income
            .iter()
            .map(|value| schedule.marginal_rate_at(*value))
            .fold(0.0_f64, f64::max),
    }
}

/// The income at which the band carrying `rate` ends.
///
/// Takes the effective band list, which includes the allowance as a zero-rate band, so
/// that the end of the zero band is the allowance itself.
fn band_end_income(bands: &[TaxBand], rate: f64) -> f64 {
    let mut cumulative = 0.0;
    for band in bands {
        cumulative += band.width;
        if (band.marginal_rate - rate).abs() < 1e-12 {
            return cumulative;
        }
    }
    f64::INFINITY
}

/// The withdrawal each year **must** make for the consumption floor to be met, given the
/// committed retirement income in that year.
///
/// This is what stops the allocation from being a pure tax exercise. Spreading a capital
/// evenly across a retirement minimises nothing if the early years need more of it than
/// the later ones — and the bridge years, with no AHV in them, need a great deal more.
pub fn required_withdrawals(bridge_income: &[f64], annual_need: f64) -> Vec<f64> {
    bridge_income
        .iter()
        .map(|income| (annual_need - income).max(0.0))
        .collect()
}

/// The withdrawal plan for a capital, funding the required amount each year and spreading
/// whatever is left.
///
/// Returns the plan and the **surplus** — the capital left once every year's requirement is
/// covered. A negative surplus is the answer "this pot cannot fund this consumption at this
/// retirement age", which is a result and not a failure, and the caller is expected to
/// report it rather than to scale the plan silently.
pub fn plan_withdrawals(
    capital: f64,
    bridge_income: &[f64],
    annual_need: f64,
    schedule: &BandedTax,
) -> (WithdrawalPlan, f64) {
    let required = required_withdrawals(bridge_income, annual_need);
    let total_required: f64 = required.iter().sum();
    let surplus = capital - total_required;

    if surplus < 0.0 {
        // The requirement exceeds the pot. The plan is the requirement scaled to what is
        // actually available, which is a declared rule for an unfundable plan: it spreads
        // the failure proportionally instead of pretending the early years can be funded
        // and the later ones cannot. The shortfall measurement then reports it.
        let scale = if total_required > 0.0 {
            capital / total_required
        } else {
            0.0
        };
        let scaled: Vec<f64> = required.iter().map(|value| value * scale).collect();
        return (
            WithdrawalPlan {
                total_tax: scaled.iter().map(|value| schedule.tax_on(*value)).sum(),
                capital_withdrawn: scaled,
                total_capital: capital,
                marginal_rate_at_optimum: 0.0,
            },
            surplus,
        );
    }

    // The requirement is the base and the surplus is spread on top by the greedy fill,
    // which is tax-optimal and takes from the emptiest year at every step.
    let plan = greedy_withdrawals(surplus, &required, schedule);
    let mut combined = plan.capital_withdrawn.clone();
    for (year, need) in required.iter().enumerate() {
        if let Some(value) = combined.get_mut(year) {
            *value += need;
        }
    }
    (
        WithdrawalPlan {
            capital_withdrawn: combined,
            total_tax: required
                .iter()
                .zip(plan.capital_withdrawn.iter())
                .map(|(need, extra)| schedule.tax_on(need + extra))
                .sum(),
            total_capital: capital,
            marginal_rate_at_optimum: plan.marginal_rate_at_optimum,
        },
        surplus,
    )
}

/// One simulated path's market and longevity outcome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathDraw {
    /// Real return net of inflation, applied to invested capital.
    pub real_return: f64,
    /// Years lived past retirement.
    pub years_lived: u32,
}

/// The risk profile of one strategy, measured rather than asserted.
#[derive(Debug, Clone, PartialEq)]
pub struct RiskProfile {
    pub retirement_age: u32,
    /// Fraction of BVG capital taken as an annuity rather than as capital.
    pub annuity_share: f64,
    /// Tax-minimising plan for the capital portion.
    pub plan: WithdrawalPlan,
    pub paths: usize,
    /// Share of paths in which the money ran out before the end of life.
    pub probability_of_shortfall: f64,
    /// Mean shortfall in the paths that had one, in present-value francs.
    pub expected_shortfall: f64,
    /// Mean terminal pot across all paths: the *over-saving* measure. A large number
    /// here is a cost, not a safety margin.
    pub mean_terminal_wealth: f64,
    pub p10_terminal_wealth: f64,
    pub median_terminal_wealth: f64,
    pub p90_terminal_wealth: f64,
    /// Mean lifetime tax across paths.
    pub mean_lifetime_tax: f64,
    /// Whether the pot covers every year's *required* withdrawal, before luck is drawn.
    ///
    /// False means the plan is unfundable by construction — the pot cannot meet the
    /// consumption floor at this retirement age however the markets behave — and a
    /// shortfall probability is then a statement about arithmetic rather than about risk.
    /// Reported separately because those two situations call for different responses.
    pub capital_funds_the_planned_consumption: bool,
}

impl RiskProfile {
    /// Whether the plan funds the lifestyle target with the required confidence.
    pub fn meets(self_confidence: f64, profile: &RiskProfile) -> bool {
        profile.probability_of_shortfall <= 1.0 - self_confidence
    }
}

/// Draw market paths for the Monte Carlo, using a fixed seed so a comparison between two
/// strategies is paired.
///
/// The distribution is a declared lognormal in real terms with the volatility supplied,
/// not a fitted market model. `economic_regimes.rs` holds the regime-switching version,
/// and it is deliberately not used here: this module's question is about the *allocation
/// of a known capital*, and mixing in a second, richer return model would make the
/// strategy comparison depend on a market view rather than on the withdrawal decision.
pub fn draw_paths(
    n_paths: usize,
    horizon: u32,
    real_return_mean: f64,
    real_return_std: f64,
    life_expectancy: u32,
    retirement_age: u32,
    seed: u64,
) -> Vec<PathDraw> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use rand_distr::{Distribution, StandardNormal};
    let mut rng = StdRng::seed_from_u64(seed);
    let mut paths = Vec::with_capacity(n_paths);
    for _ in 0..n_paths {
        // A single return draw stands in for the whole retirement period. Averaging
        // per-year draws would shrink the variance toward zero as the horizon grew,
        // which would make a long retirement look safer than a short one.
        //
        // The draw is normal and truncated at three standard deviations. The first version
        // drew uniformly over plus or minus three sigma, which has no tails but puts as
        // much mass at plus 30% real as at zero — and compounded over 27 years that turns
        // one lucky path into a terminal pot of sixteen million, which is not a scenario
        // about retirement but an artefact of the distribution.
        let standard: f64 = StandardNormal.sample(&mut rng);
        let standard = standard.clamp(-3.0, 3.0);
        let real_return = real_return_mean + real_return_std * standard;
        // Longevity: a simple triangular spread around life expectancy, clipped to the
        // plan horizon. Declared, and the report says so.
        let spread: f64 = rng.gen_range(-1.0_f64..1.0);
        let drawn = f64::from(life_expectancy) + spread * 6.0;
        let years_lived = (drawn - f64::from(retirement_age))
            .round()
            .clamp(1.0, f64::from(horizon)) as u32;
        paths.push(PathDraw {
            real_return,
            years_lived,
        });
    }
    paths
}

/// How retirement income is taxed, kept as one object because the two halves must not be
/// confused: a capital benefit is taxed **separately** from the pension income.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxTreatment {
    /// The separate capital-benefit tariff (DBG Art. 38, StHG Art. 11 Abs. 3).
    pub capital: BandedTax,
    /// The ordinary income tax on the AHV and annuity income, as an annual amount.
    pub annual_ordinary_tax_on_annuity: f64,
}

impl TaxTreatment {
    pub fn new(capital: BandedTax, annual_ordinary_tax_on_annuity: f64) -> Self {
        TaxTreatment {
            capital,
            annual_ordinary_tax_on_annuity,
        }
    }
}
/// Evaluate one strategy over a set of simulated paths.
///
/// `capital_to_place` is the capital not annuitised. `tax` carries the **separate**
/// capital-benefit tariff and the ordinary income tax
/// on the AHV and annuity income, which the caller computes with the ordinary progressive
/// schedule.
///
/// # Why the two taxes are kept apart
///
/// A capital benefit is taxed **separately** from ordinary income, as a full annual tax on
/// the capital amount alone (DBG Art. 38; StHG Art. 11 Abs. 3). So the withdrawal does not
/// sit on top of the annuity — it starts from the bottom of the capital tariff each year.
/// An earlier version of this function ran the withdrawal through the same bands as the
/// annuity, which is the ordinary-income treatment and is the wrong tax: it overstated the
/// tax on the withdrawal whenever the capital tariff was the lower of the two, and it made
/// the allocation depend on the annuity, which under the separate tax it does not.
/// `horizon` is the number of **retirement years** to plan over, not a calendar year and
/// not an age. An earlier version subtracted the retirement age from it, so a caller that
/// had already expressed the horizon in retirement years had the age taken off a second
/// time: a 27-year plan became a 1-year plan, which withdrew the entire capital in year one
/// and reported a 100% shortfall probability for every plan at every age. The parameter is
/// documented here because the mistake is invisible at the call site and the symptom looks
/// like a finding about the world.
pub fn evaluate_strategy(
    retirement_age: u32,
    annuity_share: f64,
    entitlements: &Entitlements,
    need: &ConsumptionNeed,
    tax: &TaxTreatment,
    paths: &[PathDraw],
    horizon: u32,
) -> RiskProfile {
    let annuity_share = annuity_share.clamp(0.0, 1.0);
    let capital = entitlements.bvg_capital * (1.0 - annuity_share) + entitlements.pillar3a_capital;
    let annuity = entitlements.ahv_annual
        + entitlements.bvg_capital * annuity_share * entitlements.effective_conversion_rate;

    let years = horizon.max(1) as usize;
    // The bridge income each year: before the reference age there is no AHV, so only a
    // bought BVG annuity is available. This is what makes early retirement expensive and
    // it is used for the *shortfall* test, not for the capital tax.
    let bridge_income: Vec<f64> = (0..years)
        .map(|offset| {
            let age = retirement_age + offset as u32;
            if age < 65 {
                annuity - entitlements.ahv_annual
            } else {
                annuity
            }
        })
        .collect();

    // The plan funds the requirement first and spreads the surplus. The requirement is not
    // optional and not the LP's to trade away: an allocation that minimises tax by
    // spreading the capital evenly leaves the AHV-free bridge years short, which is the
    // failure the risk measure exists to catch. See `plan_withdrawals`.
    let (plan, surplus) = plan_withdrawals(capital, &bridge_income, need.lifestyle_annual, &tax.capital);
    let funded = surplus >= 0.0;

    let mut shortfalls = 0usize;
    let mut shortfall_total = 0.0;
    let mut taxes = Vec::with_capacity(paths.len());
    let mut terminals = Vec::with_capacity(paths.len());
    let annual_need = need.lifestyle_annual;

    for path in paths {
        // The pot grows in real terms, and the plan's withdrawal is taken each year it
        // lasts. A path that outlives the plan draws only the annuity, and any gap
        // between that and the need is the shortfall.
        let mut pot = capital;
        let mut shortfall = 0.0;
        let mut tax_paid = 0.0;
        for offset in 0..path.years_lived {
            let year = (offset as usize).min(plan.capital_withdrawn.len().saturating_sub(1));
            let gross = pot * (1.0 + path.real_return);
            // The withdrawal the plan asks for and the withdrawal the pot can actually
            // fund are different things once the pot is gone. An earlier version counted
            // the *planned* draw as available even when nothing was left, so a plan that
            // exhausted its pot in five years reported a zero shortfall probability — the
            // single most dangerous kind of error this model could make, because the whole
            // point is to detect that case.
            let desired = plan.capital_withdrawn.get(year).copied().unwrap_or(0.0);
            let draw = desired.min(gross.max(0.0));
            let available = bridge_income.get(year).copied().unwrap_or(annuity);
            if available + draw < annual_need {
                shortfall += annual_need - (available + draw);
            }
            pot = (gross - draw).max(0.0);
            // Ordinary tax on the pension income, plus the separate tax on this year's
            // capital benefit.
            tax_paid += tax.annual_ordinary_tax_on_annuity + tax.capital.tax_on(draw);
        }
        if shortfall > 0.0 {
            shortfalls += 1;
            shortfall_total += shortfall;
        }
        taxes.push(tax_paid);
        terminals.push(pot);
    }

    terminals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = terminals.len().max(1);
    let mean = |values: &[f64]| values.iter().sum::<f64>() / values.len().max(1) as f64;

    RiskProfile {
        retirement_age,
        annuity_share,
        plan,
        paths: paths.len(),
        probability_of_shortfall: shortfalls as f64 / n as f64,
        expected_shortfall: if shortfalls > 0 {
            shortfall_total / shortfalls as f64
        } else {
            0.0
        },
        mean_terminal_wealth: mean(&terminals),
        // The tenth percentile is the value at index len/10. The first version wrote
        // `len / 10.min(len - 1)`, which because of precedence is a *division* by
        // `min(10, len-1)` rather than an index — clippy flagged the redundant `.max(0)`
        // and the arithmetic error underneath it. It would have reported a plausible
        // number that was not a percentile at all.
        p10_terminal_wealth: terminals[(terminals.len() / 10).min(terminals.len() - 1)],
        median_terminal_wealth: terminals[terminals.len() / 2],
        p90_terminal_wealth: terminals[(9 * terminals.len() / 10).min(terminals.len() - 1)],
        mean_lifetime_tax: mean(&taxes),
        capital_funds_the_planned_consumption: funded,
    }
}

/// Preferences the optimisation is allowed to weigh. Declared, and exposed, because a
/// model that hid them would be making the value judgement silently.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preferences {
    /// Weight on a franc left unconsumed at death, relative to a franc consumed.
    ///
    /// One means the planner is indifferent between consuming a franc and leaving it.
    /// Below one means over-saving is penalised, which is the setting that asks the
    /// question "what is the *least* I need", and zero means leaving money behind is
    /// worth nothing at all. This is the single dial that separates "fund the plan" from
    /// "fund the plan and no more".
    pub terminal_wealth_weight: f64,
    /// Weight on the lifestyle target rather than the mandatory floor.
    pub lifestyle_weight: f64,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            terminal_wealth_weight: 0.25,
            lifestyle_weight: 1.0,
        }
    }
}

/// Which principle decides what a person with a given education should contribute.
///
/// **This enum is the model's answer to the philosophical question, and the answer is
/// that the model does not have one.** Each variant is a defensible distributive
/// principle; the module prices all four and ranks none. See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContributionPrinciple {
    /// Pay in proportion to income, which is what a progressive income tax already does.
    /// Needs no new instrument, and under it a high earner already pays far more than
    /// their education cost.
    AbilityToPay,
    /// Recover the state's outlay on the education, as a surcharge on top of income tax.
    EducationCostRecovery {
        /// The state's cost of the degree, in francs.
        state_cost: f64,
        /// Years over which the surcharge is levied.
        recovery_years: u32,
    },
    /// Pay in proportion to the benefit received from holding the qualification.
    BenefitReceived {
        /// Share of the qualification's earnings premium that is contributed.
        premium_share: f64,
    },
    /// Everyone contributes the same amount, whatever they earn.
    Flat {
        /// The annual amount.
        annual: f64,
    },
}

/// The education and contribution ledger for one career.
#[derive(Debug, Clone, PartialEq)]
pub struct ContributionLedger {
    pub principle: ContributionPrinciple,
    /// What the state is taken to have spent on the qualification. Declared.
    pub state_cost_of_education: f64,
    /// Lifetime income tax paid under the ordinary progressive schedule.
    pub lifetime_income_tax: f64,
    /// Additional contribution required by the principle, over the whole career.
    pub principle_surcharge: f64,
    /// `lifetime_income_tax / state_cost_of_education`: how many times over the state's
    /// outlay is repaid by the existing tax alone.
    pub tax_multiple_of_state_cost: f64,
    /// The surcharge as a share of lifetime gross income.
    pub surcharge_share_of_income: f64,
}

/// Compute the ledger for one principle.
///
/// The point of returning a *ledger* rather than a single number is that the comparison
/// between principles is the deliverable. `AbilityToPay` returns a surcharge of zero and
/// a `tax_multiple_of_state_cost` far above one, which is the fact that makes the
/// "the educated should pay more" argument non-obvious rather than self-evident: under
/// the existing schedule they already do, and the question is whether *more* is wanted
/// and on what principle.
pub fn contribution_ledger(
    principle: ContributionPrinciple,
    state_cost_of_education: f64,
    annual_salary: f64,
    career_years: u32,
    schedule: &TaxSchedule,
) -> ContributionLedger {
    let mut lifetime_income_tax = 0.0;
    let mut lifetime_gross = 0.0;
    for _ in 0..career_years {
        let taxable = schedule.taxable_income_after_estimated_deductions(annual_salary);
        lifetime_income_tax += taxable * schedule.tax_rate_on_taxable(taxable);
        lifetime_gross += annual_salary;
    }

    let principle_surcharge = match principle {
        ContributionPrinciple::AbilityToPay => 0.0,
        ContributionPrinciple::EducationCostRecovery {
            state_cost,
            recovery_years,
        } => {
            let years = recovery_years.min(career_years).max(1);
            state_cost / f64::from(years) * f64::from(career_years.min(years))
        }
        ContributionPrinciple::BenefitReceived { premium_share } => {
            // The premium over the median earner is the declared base; without a median
            // supplied, the whole salary is used and the report says so.
            lifetime_gross * premium_share
        }
        ContributionPrinciple::Flat { annual } => annual * f64::from(career_years),
    };

    ContributionLedger {
        principle,
        state_cost_of_education,
        lifetime_income_tax,
        principle_surcharge,
        tax_multiple_of_state_cost: if state_cost_of_education > 0.0 {
            lifetime_income_tax / state_cost_of_education
        } else {
            f64::NAN
        },
        surcharge_share_of_income: if lifetime_gross > 0.0 {
            principle_surcharge / lifetime_gross
        } else {
            f64::NAN
        },
    }
}

/// The qualification a person holds, for the education ledger.
///
/// # These costs are declared, not sourced
///
/// What the state spends on a qualification varies by institution, canton, subject and
/// cohort, and no single official per-degree figure is cited here. The values are declared
/// order-of-magnitude stand-ins whose *ratios* are the part that matters: a doctorate is
/// taken to cost the state several times a vocational qualification, which is the premise
/// the "the educated should pay more" argument rests on. Replacing them with a real figure
/// is a matter of entering one, and the ledger's conclusion — how many times over the
/// existing tax repays the outlay — scales with it directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Degree {
    /// No post-compulsory qualification.
    None,
    /// Vocational education and training, the Swiss default.
    Vocational,
    Bachelor,
    Master,
    Doctorate,
}

impl Degree {
    pub fn label(self) -> &'static str {
        match self {
            Degree::None => "none",
            Degree::Vocational => "vocational",
            Degree::Bachelor => "bachelor",
            Degree::Master => "master",
            Degree::Doctorate => "doctorate",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" => Some(Degree::None),
            "vocational" | "apprenticeship" | "efz" => Some(Degree::Vocational),
            "bachelor" | "bsc" | "ba" => Some(Degree::Bachelor),
            "master" | "msc" | "ma" => Some(Degree::Master),
            "phd" | "doctorate" | "dr" => Some(Degree::Doctorate),
            _ => None,
        }
    }

    /// The state's declared cost of the qualification, and where the number comes from.
    pub fn declared_state_cost(self) -> (f64, Provenance) {
        let provenance = Provenance::Declared {
            rationale: "no single official per-degree figure is cited; the amounts are \
                        order-of-magnitude stand-ins whose ratios carry the premise, and \
                        the ledger scales with them directly",
        };
        let cost = match self {
            Degree::None => 0.0,
            Degree::Vocational => 25_000.0,
            Degree::Bachelor => 60_000.0,
            Degree::Master => 100_000.0,
            Degree::Doctorate => 200_000.0,
        };
        (cost, provenance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schedule() -> BandedTax {
        BandedTax {
            allowance: 10_000.0,
            bands: vec![
                TaxBand {
                    width: 20_000.0,
                    marginal_rate: 0.05,
                },
                TaxBand {
                    width: 30_000.0,
                    marginal_rate: 0.15,
                },
                TaxBand {
                    width: 60_000.0,
                    marginal_rate: 0.30,
                },
            ],
            provenance: Provenance::Declared {
                rationale: "test fixture",
            },
        }
    }

    /// Water-filling has to equalise the marginal rate across the years it touches, which
    /// is the KKT condition the LP's dual states. The earlier cheapest-year-first rule did
    /// not: it filled one year to the top of a band before starting the next, so years sat
    /// at different marginal rates. Tax-equal, and it concentrated the money into the early
    /// years, which emptied the pot and left the later ones unfunded.
    #[test]
    fn water_filling_equalises_the_marginal_rate() {
        let s = schedule();
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
            highest - lowest < 0.16,
            "marginal rates range {lowest} to {highest}, more than one band apart"
        );
        let total: f64 = plan.capital_withdrawn.iter().sum();
        assert!((total - capital).abs() < 1e-6);
        // And the spread has to be even rather than concentrated: no year may take half
        // the capital, which is what the cheapest-year-first rule did.
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

    /// A funded plan on a single deterministic path with a positive real return must not
    /// report a shortfall. If it does, the plan or the simulation is wrong rather than the
    /// world being unlucky — and this is the check that separates those two, because a
    /// shortfall probability alone cannot.
    #[test]
    fn a_funded_plan_with_a_positive_return_has_no_shortfall() {
        let household = Household::default();
        let params = RetirementParameters::default();
        let entitlements = project(&household, &params, 63, 1.0);
        let need = ConsumptionNeed {
            mandatory_annual: 45_000.0,
            lifestyle_annual: 60_000.0,
        };
        let s = schedule();
        let years = 90 - 63;
        let paths = vec![
            PathDraw {
                real_return: 0.02,
                years_lived: years,
            };
            1
        ];
        let profile = evaluate_strategy(63, 1.0, &entitlements, &need, &TaxTreatment::new(s.clone(), 0.0), &paths, years);
        assert!(
            profile.capital_funds_the_planned_consumption,
            "the fixture is meant to be funded, but the requirement exceeds the capital"
        );
        assert!(
            profile.probability_of_shortfall < 1e-9,
            "a funded plan on a positive-return path reported a shortfall probability of {}",
            profile.probability_of_shortfall
        );
    }

    /// The other half, and the one that matters for honesty: when the requirement exceeds
    /// the capital, that must be *reported* rather than hidden behind a scaled plan.
    #[test]
    fn an_unfundable_requirement_is_reported_and_never_overspends() {
        let bridge = vec![10_000.0_f64; 10];
        let (plan, surplus) = plan_withdrawals(50_000.0, &bridge, 60_000.0, &schedule());
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

    /// A banded schedule has to charge each band's own rate on the income inside it, and
    /// must not let income above the top band escape. A schedule that stopped charging
    /// at the top would make large withdrawals free at the margin and the optimiser would
    /// put everything there.
    #[test]
    fn bands_are_charged_in_order_and_the_top_band_catches_everything_above_it() {
        let s = schedule();
        // Below the allowance: nothing.
        assert_eq!(s.tax_on(8_000.0), 0.0);
        // Inside the first band only: 10,000 above the allowance at 5%.
        assert!((s.tax_on(20_000.0) - 500.0).abs() < 1e-9);
        // Through the second band: 20,000 at 5% plus 30,000 at 15%.
        assert!((s.tax_on(60_000.0) - (1_000.0 + 4_500.0)).abs() < 1e-9);
        // Above the last band, the top rate continues rather than falling to zero.
        let at_top = s.tax_on(120_000.0);
        let above_top = s.tax_on(220_000.0);
        assert!(
            (above_top - at_top - 100_000.0 * 0.30).abs() < 1e-9,
            "income above the top band must still be charged at the top rate"
        );
    }

    /// The optimal allocation has to come out of the LP, and the greedy allocation has to
    /// agree with it. These are two independent implementations — a tableau solver and a
    /// band-filling loop — so agreement is evidence and not a tautology.
    #[test]
    fn the_linear_program_and_the_greedy_fill_agree() {
        let s = schedule();
        let base = vec![30_000.0_f64; 6];
        let capital = 150_000.0;

        let AllocationOutcome::Optimal(lp) = optimise_withdrawals(capital, &base, &s) else {
            panic!("the allocation should be feasible");
        };
        let greedy = greedy_withdrawals(capital, &base, &s);

        assert!(
            (lp.total_tax - greedy.total_tax).abs() < 1e-6,
            "LP tax {} against greedy tax {}",
            lp.total_tax,
            greedy.total_tax
        );
        let lp_total: f64 = lp.capital_withdrawn.iter().sum();
        assert!(
            (lp_total - capital).abs() < 1e-6,
            "the LP must place the whole capital, placed {lp_total} of {capital}"
        );
    }

    /// The KKT condition, checked directly: at the optimum, every year that receives a
    /// withdrawal sits at the same marginal rate. This is the rule the whole model exists
    /// to make precise, so it is asserted rather than described.
    #[test]
    fn the_optimum_equalises_the_marginal_rate_across_the_years_it_uses() {
        let s = schedule();
        let base = vec![20_000.0_f64; 8];
        let capital = 200_000.0;
        let AllocationOutcome::Optimal(plan) = optimise_withdrawals(capital, &base, &s) else {
            panic!("feasible");
        };

        let active: Vec<f64> = plan
            .capital_withdrawn
            .iter()
            .enumerate()
            .filter(|(_, draw)| **draw > 1.0)
            .map(|(year, _)| s.marginal_rate_at(base[year] + plan.capital_withdrawn[year]))
            .collect();
        assert!(active.len() > 1, "the fixture should use several years");
        let highest = active.iter().cloned().fold(0.0_f64, f64::max);
        let lowest = active.iter().cloned().fold(f64::INFINITY, f64::min);
        // The equalisation holds up to one band's width, because a year stops exactly at a
        // band boundary and the next year starts in the band below it.
        assert!(
            highest - lowest < 0.16,
            "active years' marginal rates range {lowest} to {highest}, which is more than \
             one band apart"
        );
    }

    /// The dual of the placement constraint is the marginal rate of the *next* franc, and
    /// it has to be the derivative it claims to be. Checked by perturbing the capital and
    /// re-solving, because the obvious reading — "the highest rate any year reaches" — is
    /// wrong: a year that ends exactly on a band boundary reports the next band's rate
    /// even though its last franc was charged the one below it.
    #[test]
    fn the_placement_dual_is_the_marginal_rate_of_the_next_franc() {
        let s = schedule();
        let base = vec![20_000.0_f64; 8];
        let capital = 200_000.0;
        let AllocationOutcome::Optimal(plan) = optimise_withdrawals(capital, &base, &s) else {
            panic!("feasible");
        };
        let delta = 1.0;
        let AllocationOutcome::Optimal(raised) = optimise_withdrawals(capital + delta, &base, &s)
        else {
            panic!("feasible");
        };
        let observed = (raised.total_tax - plan.total_tax) / delta;
        assert!(
            (plan.marginal_rate_at_optimum - observed).abs() < 1e-3,
            "dual {} against the observed derivative {observed}",
            plan.marginal_rate_at_optimum
        );
    }

    /// The KKT conditions, stated as complementary slackness rather than as equality.
    ///
    /// At the optimum no year may offer a *cheaper* next franc than the multiplier, and
    /// at least one year must offer exactly that rate — otherwise the plan could be
    /// improved. Asserting instead that all active years sit at the same rate is the
    /// version that looks right and is not, because a year can stop exactly on a band
    /// boundary and report the band above.
    #[test]
    fn no_year_offers_a_cheaper_next_franc_than_the_multiplier() {
        let s = schedule();
        let base = vec![20_000.0_f64; 8];
        let capital = 200_000.0;
        let AllocationOutcome::Optimal(plan) = optimise_withdrawals(capital, &base, &s) else {
            panic!("feasible");
        };
        let multiplier = plan.marginal_rate_at_optimum;
        let next_franc: Vec<f64> = (0..base.len())
            .map(|year| s.marginal_rate_at(base[year] + plan.capital_withdrawn[year]))
            .collect();

        for (year, rate) in next_franc.iter().enumerate() {
            assert!(
                *rate >= multiplier - 1e-9,
                "year {year} offers a next franc at {rate}, below the multiplier \
                 {multiplier}, so the plan is not optimal"
            );
        }
        assert!(
            next_franc
                .iter()
                .any(|rate| (rate - multiplier).abs() < 1e-9),
            "no year offers the multiplier {multiplier}, so it is not the marginal rate: \
             {next_franc:?}"
        );
    }

    /// Spreading a capital over more years must not increase the tax.
    ///
    /// The comparison holds **total** committed income constant and re-divides it across
    /// the years, which is the comparison the claim is about. The first version held the
    /// *annual* base income constant, so adding years added committed annuity income as
    /// well as spreading room, and the tax rose — correctly, because those are different
    /// worlds and not the same income spread differently.
    #[test]
    fn spreading_the_same_income_over_more_years_never_costs_more_tax() {
        let s = schedule();
        let capital = 150_000.0;
        let total_base = 150_000.0;
        let mut previous = f64::INFINITY;
        for years in 1..=12u32 {
            let per_year = total_base / f64::from(years);
            let base = vec![per_year; years as usize];
            let AllocationOutcome::Optimal(plan) = optimise_withdrawals(capital, &base, &s) else {
                panic!("feasible");
            };
            assert!(
                plan.total_tax <= previous + 1e-6,
                "{years} years cost {} against {previous} for {} years",
                plan.total_tax,
                years - 1
            );
            previous = plan.total_tax;
        }
    }

    /// The projection has to price early retirement in both pillars. Retiring earlier
    /// must lower the AHV pension (fewer contribution years, plus an actuarial
    /// reduction) *and* the BVG capital, not just one of them.
    #[test]
    fn retiring_earlier_costs_in_both_pillars() {
        let household = Household::default();
        let params = RetirementParameters::default();
        let at_65 = project(&household, &params, 65, 1.0);
        let at_60 = project(&household, &params, 60, 1.0);

        assert!(
            at_60.ahv_annual < at_65.ahv_annual,
            "AHV {} at 60 against {} at 65",
            at_60.ahv_annual,
            at_65.ahv_annual
        );
        assert!(at_60.ahv_early_reduction > 0.0);
        assert!(
            at_60.bvg_capital < at_65.bvg_capital,
            "BVG capital {} at 60 against {} at 65",
            at_60.bvg_capital,
            at_65.bvg_capital
        );
        assert!(
            at_60.effective_conversion_rate < at_65.effective_conversion_rate,
            "the conversion rate must fall with early withdrawal"
        );
    }

    /// Working part-time has to reduce the pension, and the AHV record has to reflect it.
    /// A model in which a 60% career produced a full AHV record would make part-time work
    /// free in Pillar 1, which is the opposite of the truth.
    #[test]
    fn a_part_time_career_earns_a_smaller_pension_and_a_shorter_record() {
        let household = Household::default();
        let params = RetirementParameters::default();
        let full = project(&household, &params, 65, 1.0);
        let part = project(&household, &params, 65, 0.6);
        assert!(part.ahv_annual < full.ahv_annual);
        assert!(part.bvg_capital < full.bvg_capital);
    }

    /// The education ledger has to show that the existing schedule already repays the
    /// state many times over for a high earner. This is the fact the normative debate
    /// turns on, so it is computed rather than asserted in prose.
    #[test]
    fn a_high_earner_repays_the_states_education_outlay_many_times_over() {
        let schedule = TaxSchedule::bern_city_default(true, 1);
        let ledger = contribution_ledger(
            ContributionPrinciple::AbilityToPay,
            150_000.0,
            180_000.0,
            35,
            &schedule,
        );
        assert_eq!(ledger.principle_surcharge, 0.0);
        assert!(
            ledger.tax_multiple_of_state_cost > 3.0,
            "a high earner's lifetime tax is {} times the state's outlay, which is not the \
             'many times over' the discussion assumes",
            ledger.tax_multiple_of_state_cost
        );
    }

    /// Charging the same amount to everyone is regressive in effect, and the ledger must
    /// say so in the only place it can: as a share of income. A flat charge that is a
    /// smaller share of a high income is the mechanical fact that makes the principle
    /// contestable, and the model reports it rather than hiding it in a total.
    #[test]
    fn a_flat_charge_falls_as_a_share_of_income() {
        let schedule = TaxSchedule::bern_city_default(false, 0);
        let low = contribution_ledger(
            ContributionPrinciple::Flat { annual: 3_000.0 },
            0.0,
            60_000.0,
            30,
            &schedule,
        );
        let high = contribution_ledger(
            ContributionPrinciple::Flat { annual: 3_000.0 },
            0.0,
            240_000.0,
            30,
            &schedule,
        );
        assert!(
            high.surcharge_share_of_income < low.surcharge_share_of_income,
            "flat charge shares: {} against {}",
            high.surcharge_share_of_income,
            low.surcharge_share_of_income
        );
    }

    /// The Monte Carlo has to be reproducible from its seed, or two strategies compared
    /// against "the same paths" would not be.
    #[test]
    fn the_same_seed_draws_the_same_paths() {
        let a = draw_paths(500, 40, 0.02, 0.10, 90, 60, 7);
        let b = draw_paths(500, 40, 0.02, 0.10, 90, 60, 7);
        assert_eq!(a, b);
        let c = draw_paths(500, 40, 0.02, 0.10, 90, 60, 8);
        assert_ne!(a, c);
    }

    /// And the risk measurement has to move the way it claims: annuitising more of the
    /// capital must lower the probability of running out, because it removes the
    /// sequence and longevity risk from that part of the pot. If it did not, the trade-off
    /// the whole model is built around would not exist.
    #[test]
    fn annuitising_more_reduces_the_probability_of_running_out() {
        let household = Household::default();
        let params = RetirementParameters::default();
        let entitlements = project(&household, &params, 63, 1.0);
        let need = ConsumptionNeed {
            mandatory_annual: 45_000.0,
            lifestyle_annual: 60_000.0,
        };
        let s = schedule();
        let paths = draw_paths(2_000, 45, 0.015, 0.08, 90, 63, 11);

        // A declared ordinary tax on the pension income, held constant across both arms so
        // that what separates them is the capital strategy and not the annuity tax.
        let annuity_tax = 4_000.0;
        let all_capital =
            evaluate_strategy(63, 0.0, &entitlements, &need, &TaxTreatment::new(s.clone(), annuity_tax), &paths, 45);
        let all_annuity =
            evaluate_strategy(63, 1.0, &entitlements, &need, &TaxTreatment::new(s.clone(), annuity_tax), &paths, 45);
        assert!(
            all_annuity.probability_of_shortfall <= all_capital.probability_of_shortfall,
            "annuitising raised the shortfall probability: {} against {}",
            all_annuity.probability_of_shortfall,
            all_capital.probability_of_shortfall
        );
        assert!(
            all_annuity.mean_terminal_wealth <= all_capital.mean_terminal_wealth + 1e-6,
            "annuitising left more unconsumed wealth, which would mean it is not doing the \
             job the trade-off claims"
        );
    }
}





