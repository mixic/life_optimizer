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

//! The three market presets, compared against published Swiss pension-fund figures.
//!
//! `VALIDATION.md` §5 used to say the market assumptions were "not calibrated to any
//! fund", and stopped there — a statement that is true and tells a reader nothing about
//! whether the numbers are plausible. This file puts the presets next to the published
//! distribution they claim to represent.
//!
//! # The published figures
//!
//! From the **26th Swiss Pension Fund Study (Schweizer Pensionskassenstudie) by
//! Swisscanto**, presented June 2026, covering 528 institutions with CHF 939bn and
//! about four million insured members:
//!
//! | Figure | Value |
//! |---|---:|
//! | Average interest credited to retirement assets, 2025 | 4.7% |
//! | Real interest credited, 2025 (highest in 25 years) | 4.6% |
//! | Range of credited rates across funds, 2025 | 1.25% – 17.0% |
//! | 10th percentile of credited rate | 2.0% |
//! | 90th percentile of credited rate | 9.25% |
//! | Average net investment return, 2025 | 6.0% |
//! | Average funded ratio (Deckungsgrad), end 2025 | 119.6% |
//!
//! Sources: [investrends.ch](https://investrends.ch/themen/finanzplaetze/pensionskassen-hochste-realverzinsung-seit-25-jahren-doch-die-zinsunterschiede-wachsen/62509/)
//! and [vorsorgeforum.ch](https://www.vorsorgeforum.ch/bvg-aktuell/2026/9/1/rekordverzinsung-bei-pensionskassen-aber-enorme-unterschiede.html),
//! both reporting the Swisscanto study. The study itself is published as a PDF, which
//! this repository's tooling cannot read offline; the two secondary reports agree on
//! every figure used here.
//!
//! # What this comparison is, and is not
//!
//! It is a **bound check against one exceptional year**, not a calibration. 2025 was
//! the highest real credit in twenty-five years, so it is an upper-tail observation:
//! a long-run preset below it is not thereby validated, and one above it would not be
//! absurd. The presets are long-run real returns; the published figures are one year's
//! nominal outcomes.
//!
//! What the comparison does establish is the direction of the error that matters in a
//! retirement calculator — optimism. If the presets had put the most favourable case
//! above the best observed percentile, every projection would inherit that optimism.
//! They do not: the optimistic preset's nominal 6.0% sits at the 2025 *average* gross
//! return and well below the top decile's 9.25%, so even the rosy column is not rosy
//! relative to the best year in a quarter century.
//!
//! # Defect this file was written alongside
//!
//! Two doc comments claimed the opposite of this: the module header said the
//! distribution was "calibrated to Swiss BVG pension fund data", and `MarketAssumptions`
//! said "Historical Swiss BVG fund parameters (1985-2024)". Neither was true — no
//! fitting code exists in the repository — while `mc_display.rs` and `main.rs` printed
//! that the presets were *not* calibrated. Four statements, two of them false, in a
//! tool whose purpose is to say what it does and does not know. Both are corrected.

use life_optimizer::monte_carlo::MarketAssumptions;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The three presets, as the code actually holds them.
///
/// The printed scenario legend in `mc_display.rs` states "2.0% real return, 6.0%
/// volatility", "3.2% / 8.5%" and "4.5% / 11.0%". Those strings are hardcoded, so
/// nothing in the type system ties them to the parameters; this test does.
#[test]
fn the_presets_are_the_ones_the_legend_prints() {
    let conservative = MarketAssumptions::conservative();
    assert!(close(conservative.expected_real_return, 0.020));
    assert!(close(conservative.volatility, 0.060));

    let base = MarketAssumptions::base_case();
    assert!(close(base.expected_real_return, 0.032));
    assert!(close(base.volatility, 0.085));

    let optimistic = MarketAssumptions::optimistic();
    assert!(close(optimistic.expected_real_return, 0.045));
    assert!(close(optimistic.volatility, 0.110));

    // The legend also prints "All include 1.5% annual inflation".
    for preset in [&conservative, &base, &optimistic] {
        assert!(close(preset.inflation, 0.015));
    }

    // Ordering is what makes the three comparable at all.
    assert!(conservative.expected_real_return < base.expected_real_return);
    assert!(base.expected_real_return < optimistic.expected_real_return);
    assert!(conservative.volatility < base.volatility);
    assert!(base.volatility < optimistic.volatility);
}

/// The presets' real returns plus their inflation assumption, against the published
/// 2025 figures.
#[test]
fn the_presets_do_not_exceed_the_best_observed_year() {
    // Swisscanto Pensionskassenstudie, 26th edition: 2025 averages and percentiles.
    const PUBLISHED_AVERAGE_CREDITED: f64 = 0.047; // nominal interest credited
    const PUBLISHED_AVERAGE_RETURN: f64 = 0.060; // gross investment return
    const PUBLISHED_TOP_DECILE_CREDITED: f64 = 0.0925;
    const PUBLISHED_BOTTOM_DECILE_CREDITED: f64 = 0.020;

    let conservative = MarketAssumptions::conservative();
    let base = MarketAssumptions::base_case();
    let optimistic = MarketAssumptions::optimistic();

    // The presets hold real returns, so an implied nominal return is real + inflation.
    let nominal = |preset: &MarketAssumptions| preset.expected_real_return + preset.inflation;

    let (low, mid, high) = (nominal(&conservative), nominal(&base), nominal(&optimistic));
    assert!(close(low, 0.035), "conservative nominal {low}");
    assert!(close(mid, 0.047), "base nominal {mid}");
    assert!(close(high, 0.060), "optimistic nominal {high}");

    // The base case's implied nominal return coincides with the average credited rate
    // to the published precision. Report it as the coincidence it is: the two are
    // different quantities (a credited rate is the fund's decision after reserves, not
    // its return) in different periods (a long-run preset against one year).
    assert!(
        (mid - PUBLISHED_AVERAGE_CREDITED).abs() < 0.0005,
        "base nominal {mid} against the published 4.7%"
    );

    // The optimistic preset must not exceed the 2025 *average* gross return, which was
    // itself the best year in twenty-five. If this fails, the tool has become optimistic
    // relative to the most favourable recent evidence and every projection is inflated.
    assert!(
        high <= PUBLISHED_AVERAGE_RETURN + 1e-9,
        "the optimistic nominal return {high} exceeds the 2025 average gross return \
         {PUBLISHED_AVERAGE_RETURN}"
    );

    // And it sits well below the top decile of funds, so it does not claim best-in-class.
    assert!(
        high < PUBLISHED_TOP_DECILE_CREDITED,
        "the optimistic nominal return {high} claims more than the top decile's \
         {PUBLISHED_TOP_DECILE_CREDITED}"
    );

    // The conservative preset is above the bottom decile but not by much: a bond-heavy
    // fund that only matched the weakest tenth would be a poor long-run assumption, and
    // one that assumed the weakest tenth would be needlessly alarming.
    assert!(
        low > PUBLISHED_BOTTOM_DECILE_CREDITED,
        "the conservative nominal return {low} is at or below the bottom decile's \
         {PUBLISHED_BOTTOM_DECILE_CREDITED}"
    );
    assert!(
        low < 2.0 * PUBLISHED_BOTTOM_DECILE_CREDITED,
        "the conservative nominal return {low} is far above the bottom decile"
    );
}

/// The three presets do not span the observed dispersion, and the test says so rather
/// than pretending otherwise.
///
/// The published range of credited rates in 2025 ran from 1.25% to 17%, and the
/// 10th-to-90th percentile band from 2.0% to 9.25%. The presets' nominal 3.5%-6.0% is a
/// much narrower band. That is a deliberate simplification — a three-scenario model
/// cannot represent a seventeen-fold spread — but it means the pessimistic column is
/// *not* the worst fund, and a reader should not take it for one.
#[test]
fn the_preset_range_is_narrower_than_the_published_dispersion() {
    let low = MarketAssumptions::conservative().expected_real_return
        + MarketAssumptions::conservative().inflation;
    let high = MarketAssumptions::optimistic().expected_real_return
        + MarketAssumptions::optimistic().inflation;

    // Published extremes, 2025.
    const PUBLISHED_MIN: f64 = 0.0125;
    const PUBLISHED_MAX: f64 = 0.170;

    assert!(
        high - low < PUBLISHED_MAX - PUBLISHED_MIN,
        "the preset band {low}-{high} is as wide as the published {PUBLISHED_MIN}-{PUBLISHED_MAX}"
    );
    assert!(
        low > PUBLISHED_MIN,
        "the conservative preset {low} is below the weakest published fund {PUBLISHED_MIN}, \
         which would make the pessimistic scenario more alarming than the evidence"
    );
    assert!(
        high < PUBLISHED_MAX,
        "the optimistic preset {high} reaches the strongest published fund {PUBLISHED_MAX}"
    );

    // The published percentile band is also wider than the presets, by a factor of
    // about three on the upside.
    const PUBLISHED_P90: f64 = 0.0925;
    const PUBLISHED_P10: f64 = 0.020;
    assert!((PUBLISHED_P90 - PUBLISHED_P10) > (high - low));
}
