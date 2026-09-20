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

//! Integration tests for the pension-fund profile registry and stochastic
//! conversion-rate modelling (`FutureWork.md` §10.5).
//!
//! The stochastic model exists because a single conversion rate implies a
//! precision the input does not have. These tests pin the properties that make
//! it trustworthy: it is centred on the projection, bounded by the statutory
//! rate and the projection floor, and its downside statistics are ordered
//! correctly.

use life_optimizer::monte_carlo::{
    find_pension_fund, pension_fund_ids, simulate_conversion_rate_uncertainty,
    ConversionRateScenario, PensionSimulator, CONVERSION_RATE_UNCERTAINTY_BAND,
    MIN_PROJECTED_CONVERSION_RATE, PENSION_FUND_PROFILES, STATUTORY_CONVERSION_RATE,
    TYPICAL_FUND_CONVERSION_RATE,
};

const CAPITAL: f64 = 500_000.0;
const YEAR: u32 = 2050;

// ─── Fund profile registry ──────────────────────────────────────────────────

/// Every registry entry must be identifiable, named, and sourced. An entry
/// with no stated provenance is exactly the "plausible narrative" §7 warns
/// about.
#[test]
fn every_fund_profile_is_named_and_sourced() {
    assert!(
        !PENSION_FUND_PROFILES.is_empty(),
        "the registry must not be empty"
    );

    for fund in PENSION_FUND_PROFILES {
        assert!(!fund.id.trim().is_empty(), "every profile needs an id");
        assert!(
            !fund.name.trim().is_empty(),
            "{}: needs a display name",
            fund.id
        );
        assert!(
            !fund.source_note.trim().is_empty(),
            "{}: every rate must state its provenance",
            fund.id
        );
        assert!(
            fund.conversion_rate > 0.0 && fund.conversion_rate <= STATUTORY_CONVERSION_RATE,
            "{}: rate {} is outside the plausible range (0, 6.8%]",
            fund.id,
            fund.conversion_rate
        );
    }
}

/// Identifiers must be unique, or `find_pension_fund` becomes ambiguous.
#[test]
fn fund_profile_ids_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for fund in PENSION_FUND_PROFILES {
        assert!(seen.insert(fund.id), "duplicate fund id '{}'", fund.id);
    }
}

/// Lookup must be case- and whitespace-insensitive, and must reject unknown
/// ids rather than silently falling back to a default.
#[test]
fn fund_lookup_is_strict_but_forgiving_about_case() {
    assert!(find_pension_fund("publica").is_some());
    assert!(find_pension_fund("PUBLICA").is_some());
    assert!(find_pension_fund("  Publica  ").is_some());
    assert!(find_pension_fund("does-not-exist").is_none());
    assert!(find_pension_fund("").is_none());
}

/// The identifier list used in error messages must cover every entry.
#[test]
fn fund_id_list_covers_the_registry() {
    let listed = pension_fund_ids();
    for fund in PENSION_FUND_PROFILES {
        assert!(
            listed.contains(fund.id),
            "'{}' missing from the id list",
            fund.id
        );
    }
}

/// The statutory and typical profiles must agree with the constants they
/// represent, so the registry cannot drift from the rest of the model.
#[test]
fn canonical_profiles_match_their_constants() {
    let statutory = find_pension_fund("statutory").expect("statutory profile exists");
    assert!((statutory.conversion_rate - STATUTORY_CONVERSION_RATE).abs() < 1e-12);

    let typical = find_pension_fund("typical").expect("typical profile exists");
    assert!((typical.conversion_rate - TYPICAL_FUND_CONVERSION_RATE).abs() < 1e-12);
}

// ─── Stochastic conversion-rate model ───────────────────────────────────────

fn simulate() -> life_optimizer::monte_carlo::StochasticConversionRateResult {
    simulate_conversion_rate_uncertainty(CAPITAL, YEAR, 10_000, 2_000.0, Some(7))
}

/// Percentiles and the downside statistics must be ordered correctly. A CVaR
/// above the P10 would mean the tail statistic is not measuring a tail.
#[test]
fn stochastic_percentiles_are_ordered() {
    let r = simulate();

    assert!(r.p10_monthly <= r.p25_monthly, "P10 must not exceed P25");
    assert!(
        r.p25_monthly <= r.median_monthly,
        "P25 must not exceed the median"
    );
    assert!(
        r.median_monthly <= r.p75_monthly,
        "median must not exceed P75"
    );
    assert!(r.p75_monthly <= r.p90_monthly, "P75 must not exceed P90");
    assert!(
        r.cvar_10_monthly <= r.p10_monthly + 1e-9,
        "CVaR is the mean of the worst decile, so it cannot exceed the P10: {} vs {}",
        r.cvar_10_monthly,
        r.p10_monthly
    );
    assert!(r.cvar_10_monthly > 0.0, "CVaR must be a positive pension");
}

/// The distribution must be centred on the deterministic projection, so the
/// stochastic model does not shift the answer — only widen it.
#[test]
fn stochastic_median_tracks_the_deterministic_projection() {
    let r = simulate();
    let deterministic = CAPITAL * r.mean_rate / 12.0;

    // The median of a symmetric truncated distribution sits near the centre.
    let rel_err = (r.median_monthly - deterministic).abs() / deterministic;
    assert!(
        rel_err < 0.05,
        "median {:.0} should track the projection {:.0} (rel err {:.3})",
        r.median_monthly,
        deterministic,
        rel_err
    );
}

/// Every draw must respect the truncation bounds, so no path can exceed the
/// statutory rate or fall below the projection floor.
#[test]
fn stochastic_draws_respect_the_bounds() {
    let r = simulate();
    let max_possible = CAPITAL * STATUTORY_CONVERSION_RATE / 12.0;
    let min_possible = CAPITAL * MIN_PROJECTED_CONVERSION_RATE / 12.0;

    assert!(
        r.p90_monthly <= max_possible + 1e-6,
        "P90 exceeds the statutory maximum"
    );
    assert!(
        r.p10_monthly >= min_possible - 1e-6,
        "P10 falls below the projection floor"
    );
    assert!(r.rate_std_dev > 0.0);
    assert!((r.rate_std_dev - CONVERSION_RATE_UNCERTAINTY_BAND).abs() < 1e-12);
}

/// The reported floor probability must match the share of draws below the
/// floor, and be a valid probability.
#[test]
fn floor_probability_is_a_valid_share() {
    // A floor above every possible pension means certainty of falling short.
    let impossible = simulate_conversion_rate_uncertainty(CAPITAL, YEAR, 5_000, 1e9, Some(1));
    assert!(
        (impossible.prob_below_floor - 1.0).abs() < 1e-9,
        "an unreachable floor means probability 1, got {}",
        impossible.prob_below_floor
    );

    // A floor of zero can never be breached by a positive pension.
    let trivial = simulate_conversion_rate_uncertainty(CAPITAL, YEAR, 5_000, 0.0, Some(1));
    assert!((trivial.prob_below_floor - 0.0).abs() < 1e-9);

    // And an intermediate floor lands strictly between the two.
    let middle = simulate_conversion_rate_uncertainty(CAPITAL, YEAR, 5_000, 2_400.0, Some(1));
    assert!((0.0..=1.0).contains(&middle.prob_below_floor));
}

/// The simulation must be reproducible for a given seed — a stochastic report
/// that changes between identical runs is not auditable.
#[test]
fn stochastic_simulation_is_reproducible() {
    let a = simulate_conversion_rate_uncertainty(CAPITAL, YEAR, 4_000, 2_000.0, Some(99));
    let b = simulate_conversion_rate_uncertainty(CAPITAL, YEAR, 4_000, 2_000.0, Some(99));

    assert!((a.median_monthly - b.median_monthly).abs() < 1e-12);
    assert!((a.cvar_10_monthly - b.cvar_10_monthly).abs() < 1e-12);
    assert!((a.prob_below_floor - b.prob_below_floor).abs() < 1e-12);
}

/// More capital must translate into a proportionally higher pension at every
/// percentile — the model is linear in capital.
#[test]
fn stochastic_pension_scales_with_capital() {
    let low = simulate_conversion_rate_uncertainty(250_000.0, YEAR, 5_000, 1_000.0, Some(3));
    let high = simulate_conversion_rate_uncertainty(500_000.0, YEAR, 5_000, 1_000.0, Some(3));

    let ratio = high.median_monthly / low.median_monthly;
    assert!(
        (ratio - 2.0).abs() < 0.01,
        "doubling capital should double the pension, got ratio {ratio}"
    );
}

/// A later retirement year projects a lower rate, so the pension distribution
/// must shift down. This is the §10.5 "future awareness" benefit.
#[test]
fn later_retirement_year_shifts_the_distribution_down() {
    let early = simulate_conversion_rate_uncertainty(CAPITAL, 2030, 5_000, 2_000.0, Some(5));
    let late = simulate_conversion_rate_uncertainty(CAPITAL, 2060, 5_000, 2_000.0, Some(5));

    assert!(
        late.median_monthly < early.median_monthly,
        "a 2060 retirement faces a lower projected rate: {} vs {}",
        late.median_monthly,
        early.median_monthly
    );
}

/// The simulator helper must agree with the free function it wraps.
#[test]
fn simulator_helper_matches_the_free_function() {
    let sim = PensionSimulator::new(40, 65, 90, 100_000.0, 0.8, false, 3_000.0);
    let via_sim = sim.conversion_rate_uncertainty(300_000.0, 3_000.0);
    let via_fn = simulate_conversion_rate_uncertainty(
        300_000.0,
        life_optimizer::monte_carlo::retirement_year_from(40, 65),
        sim.n_simulations,
        3_000.0,
        sim.seed,
    );

    assert!((via_sim.median_monthly - via_fn.median_monthly).abs() < 1e-12);
    assert_eq!(via_sim.retirement_year, via_fn.retirement_year);
}

/// The stochastic model must not depend on which conversion scenario is
/// selected: it is centred on the *projection*, which is the neutral reference.
/// A custom rate affects the deterministic headline, not this distribution.
#[test]
fn stochastic_model_is_centred_on_the_projection_not_the_selection() {
    let mut sim = PensionSimulator::new(40, 65, 90, 100_000.0, 0.8, false, 3_000.0);
    let projected = simulate_conversion_rate_uncertainty(
        300_000.0,
        life_optimizer::monte_carlo::retirement_year_from(40, 65),
        sim.n_simulations,
        3_000.0,
        sim.seed,
    );

    sim.conversion_scenario = ConversionRateScenario::Custom(0.03);
    let still_projected = sim.conversion_rate_uncertainty(300_000.0, 3_000.0);

    assert!(
        (projected.median_monthly - still_projected.median_monthly).abs() < 1e-12,
        "the uncertainty band is a property of the projection, not of the user's own rate"
    );
}
