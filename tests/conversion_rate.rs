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

//! Integration tests for conversion-rate (Umwandlungssatz) modelling.
//!
//! These cover the contract described in `FutureWork.md` §10 and documented in
//! `MATHEMATICS.md` §6.2.1: the statutory/typical/projected scenarios, the
//! linear projection and its floor, deferral scaling, and the invariant that the
//! optimizer and the Monte Carlo projection agree on which rate they assume.

use life_optimizer::monte_carlo::{
    calculate_pension_range, effective_conversion_rate, project_conversion_rate,
    retirement_year_from, ConversionRateScenario, PensionSimulator, MIN_PROJECTED_CONVERSION_RATE,
    STATUTORY_CONVERSION_RATE, TYPICAL_FUND_CONVERSION_RATE,
};

const CAPITAL: f64 = 500_000.0;

/// The projection must reproduce the anchor table in `FutureWork.md` §10.3.
#[test]
fn projection_matches_documented_table() {
    let table = [
        (2024, 0.0680),
        (2030, 0.0658),
        (2040, 0.0622),
        (2050, 0.0586),
    ];
    for (year, expected) in table {
        let actual = project_conversion_rate(year);
        assert!(
            (actual - expected).abs() < 0.00005,
            "year {year}: expected {expected}, got {actual}"
        );
    }
}

/// The projection is bounded below rather than declining without limit.
#[test]
fn projection_is_floored() {
    // By 2074 the unfloored line would reach 5.0%; beyond that it must hold.
    assert!((project_conversion_rate(2074) - MIN_PROJECTED_CONVERSION_RATE).abs() < 1e-9);
    assert!((project_conversion_rate(2200) - MIN_PROJECTED_CONVERSION_RATE).abs() < 1e-9);
    // It never projects below the floor, nor above the statutory rate.
    assert!(project_conversion_rate(2100) >= MIN_PROJECTED_CONVERSION_RATE);
    assert!(project_conversion_rate(2024) <= STATUTORY_CONVERSION_RATE);
}

/// Deferral raises the effective rate; early retirement lowers it.
#[test]
fn deferral_scales_the_base_rate() {
    let at_65 = effective_conversion_rate(ConversionRateScenario::Statutory, 65, 2024);
    let at_70 = effective_conversion_rate(ConversionRateScenario::Statutory, 70, 2024);
    let at_62 = effective_conversion_rate(ConversionRateScenario::Statutory, 62, 2024);

    assert!(
        (at_65 - STATUTORY_CONVERSION_RATE).abs() < 1e-9,
        "age 65 is the anchor"
    );
    assert!(at_70 > at_65, "deferring to 70 must raise the rate");
    assert!(at_62 < at_65, "retiring at 62 must lower the rate");

    // A deferred retirement scales whatever base rate applies, rather than
    // snapping to the statutory table.
    let deferred_low_base =
        effective_conversion_rate(ConversionRateScenario::Custom(0.050), 70, 2024);
    assert!(
        (deferred_low_base - 0.050 * (0.078 / 0.068)).abs() < 1e-9,
        "deferral must scale the caller's base rate, got {deferred_low_base}"
    );
}

/// The statutory scenario must remain the default so that existing behaviour
/// and previously computed results are unchanged.
#[test]
fn statutory_scenario_preserves_legacy_default() {
    let sim = PensionSimulator::new(40, 65, 90, 100_000.0, 1.0, false, 5_000.0);
    assert_eq!(sim.conversion_scenario, ConversionRateScenario::Statutory);
    assert!((sim.effective_conversion_rate() - 0.068).abs() < 1e-9);
}

/// Monthly pension is capital x rate / 12, and the range must be ordered so the
/// low end is the worst case a person should actually plan against.
#[test]
fn range_is_consistent_and_ordered() {
    let range = calculate_pension_range(CAPITAL, 2024, ConversionRateScenario::Statutory);

    assert!((range.statutory - CAPITAL * STATUTORY_CONVERSION_RATE / 12.0).abs() < 1e-9);
    assert!((range.typical - CAPITAL * TYPICAL_FUND_CONVERSION_RATE / 12.0).abs() < 1e-9);

    assert!(range.custom.is_none(), "no custom rate was supplied");
    assert!(range.range.0 <= range.range.1, "range must be ordered");
    assert!(
        (range.range.1 - range.statutory).abs() < 1e-9,
        "statutory is the high case"
    );

    // The statutory rate does flatter the outcome; this is the gap the display
    // exists to make visible.
    assert!(range.statutory_minus_typical > 0.0);
    assert!(
        (range.statutory_minus_typical
            - CAPITAL * (STATUTORY_CONVERSION_RATE - TYPICAL_FUND_CONVERSION_RATE) / 12.0)
            .abs()
            < 1e-9
    );
}

/// A user-supplied rate becomes the headline figure and extends the range.
#[test]
fn custom_rate_drives_headline_and_extends_range() {
    let range = calculate_pension_range(CAPITAL, 2024, ConversionRateScenario::Custom(0.042));
    let custom_monthly = CAPITAL * 0.042 / 12.0;

    assert!((range.selected_monthly - custom_monthly).abs() < 1e-9);
    assert!((range.custom.unwrap() - custom_monthly).abs() < 1e-9);
    assert!((range.selected_rate - 0.042).abs() < 1e-9);

    // 4.2% is below every preset, so it sets the lower end of the range.
    assert!((range.range.0 - custom_monthly).abs() < 1e-9);
    assert!(range.difference_from_statutory(custom_monthly) > 0.0);
}

/// A custom rate *above* the statutory rate must extend the range upward.
#[test]
fn custom_rate_above_statutory_extends_range_upward() {
    let range = calculate_pension_range(CAPITAL, 2024, ConversionRateScenario::Custom(0.075));
    let custom_monthly = CAPITAL * 0.075 / 12.0;

    assert!(
        (range.range.1 - custom_monthly).abs() < 1e-9,
        "7.5% is the new high case"
    );
    // And the statutory comparison goes the other way: the person does better.
    assert!(range.difference_from_statutory(custom_monthly) < 0.0);
}

/// The optimizer's estimate and the Monte Carlo projection must agree on the
/// conversion rate for the same person.
#[test]
fn optimizer_and_simulator_agree_on_conversion_rate() {
    let sim = PensionSimulator::new(40, 65, 90, 100_000.0, 1.0, false, 5_000.0);
    let from_simulator = sim.effective_conversion_rate();
    let from_shared_fn = effective_conversion_rate(
        ConversionRateScenario::Statutory,
        65,
        retirement_year_from(40, 65),
    );
    assert!((from_simulator - from_shared_fn).abs() < 1e-12);
}

/// Retirement year is anchored to today, so a person already at retirement age
/// gets no spurious future projection.
#[test]
fn retirement_year_anchors_to_today() {
    assert_eq!(retirement_year_from(65, 65), retirement_year_from(40, 40));
    assert_eq!(
        retirement_year_from(40, 65) - retirement_year_from(40, 40),
        25
    );
}

/// The scenario labels are user-facing; an unset label would be a display bug.
#[test]
fn scenario_labels_are_non_empty_and_show_the_rate() {
    let labels = [
        ConversionRateScenario::Statutory.label(),
        ConversionRateScenario::FundTypical.label(),
        ConversionRateScenario::FutureProjection.label(),
        ConversionRateScenario::Custom(0.042).label(),
    ];
    for label in &labels {
        assert!(!label.trim().is_empty(), "label must not be empty");
    }
    assert!(
        labels[0].contains("6.8"),
        "statutory label should name its rate: {}",
        labels[0]
    );
    assert!(
        labels[1].contains("5.5"),
        "typical label should name its rate: {}",
        labels[1]
    );
    assert!(
        labels[3].contains("4.2"),
        "custom label should name its rate: {}",
        labels[3]
    );
}

/// A pension range must be derivable from a simulated capital, which is what
/// the CLI does for the regime-switching median.
#[test]
fn pension_range_derives_from_simulated_capital() {
    let sim = PensionSimulator::new(40, 65, 90, 100_000.0, 0.8, false, 5_000.0);
    let range = sim.pension_range_for_capital(300_000.0);

    assert!((range.capital - 300_000.0).abs() < 1e-9);
    assert!((range.statutory - 300_000.0 * STATUTORY_CONVERSION_RATE / 12.0).abs() < 1e-9);
    assert!(range.range.0 > 0.0, "range must contain a positive pension");
    // The retirement year must reflect this person's horizon, not a fixed one.
    assert_eq!(range.retirement_year, retirement_year_from(40, 65));
}
