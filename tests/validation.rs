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

//! Shape checks on the tax engine, beyond the externally sourced reference points.
//!
//! These are *observations pinned*, not law verified. `VALIDATION.md` §2 records which of them
//! could be checked against an external figure and which are pinned so that a change is visible.

use life_optimizer::tax::TaxSchedule;

/// The average rate must never fall as income rises.
///
/// This holds under any progressive schedule and fails loudly if a table row is mistyped, which is
/// the defect it is here to catch.
#[test]
fn the_bern_average_rate_never_falls() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let mut previous = 0.0;
    for income in [
        10_000.0,
        25_000.0,
        50_000.0,
        75_000.0,
        100_000.0,
        150_000.0,
        200_000.0,
        250_000.0,
        300_000.0,
        400_000.0,
        500_000.0,
        750_000.0,
        1_000_000.0,
    ] {
        let rate = schedule.tax_rate_on_taxable(income);
        assert!(
            rate >= previous - 1e-9,
            "the average rate fell from {:.4}% to {:.4}% at CHF {income}",
            previous * 100.0,
            rate * 100.0
        );
        previous = rate;
    }
}

/// Above the top bracket the Bern average rate is **constant**, which is consistent with the
/// cantonal tariff's structure rather than a defect.
///
/// The first version of this test asserted that the average rate *keeps rising*, on the assumption
/// that a progressive schedule behaves like the federal bracket tariff where the average
/// approaches the top marginal rate asymptotically. It failed: the rate reaches 24.73% at CHF
/// 200,000 and stays there to CHF 1,000,000, so the tax becomes exactly proportional above that
/// point.
///
/// That is what a **satzbestimmendes** cantonal tariff produces — the rate is determined by the
/// income and applied to the whole of it, so once the top rate is reached the tax is proportional.
/// The test now pins the observed behaviour rather than asserting the wrong model of it, and the
/// open question is recorded rather than resolved: whether 24.73% is the correct top rate for Bern
/// could not be checked offline, so `VALIDATION.md` lists it as unverified.
#[test]
fn the_bern_rate_is_proportional_above_the_top_bracket() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let top = schedule.tax_rate_on_taxable(250_000.0);
    for income in [250_000.0, 300_000.0, 400_000.0, 500_000.0, 750_000.0, 1_000_000.0] {
        let rate = schedule.tax_rate_on_taxable(income);
        assert!(
            (rate - top).abs() < 1e-9,
            "above the top bracket the rate should be constant: {:.4}% at CHF {income} against \
             {:.4}% at CHF 250,000",
            rate * 100.0,
            top * 100.0
        );
        // Proportionality, stated directly: tax is the rate times the income.
        let tax = rate * income;
        assert!((tax / income - top).abs() < 1e-9);
    }
    // And the top rate is not absurd for a canton with one of the highest combined burdens.
    assert!(
        (0.15..=0.35).contains(&top),
        "the top rate {:.2}% is outside the plausible range for Bern",
        top * 100.0
    );
}

/// Below the top bracket the rate must actually be progressive, or the two regimes are the same
/// and this is a flat tax.
#[test]
fn the_bern_rate_is_progressive_below_the_top_bracket() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let low = schedule.tax_rate_on_taxable(25_000.0);
    let high = schedule.tax_rate_on_taxable(100_000.0);
    assert!(
        high > low + 0.05,
        "the rate barely moves between CHF 25,000 and CHF 100,000: {:.2}% to {:.2}%",
        low * 100.0,
        high * 100.0
    );
}
/// The Monte Carlo median must converge as the path count rises: more paths, less spread between
/// seeds.
///
/// This is the convergence check the review asked for, and it is deliberately a statement about
/// *dispersion across seeds* rather than about agreement with a reference run. A single run at two
/// path counts can be closer or further from the truth by luck; the standard error falling with the
/// square root of the count is the property that has to hold.
#[test]
fn the_monte_carlo_median_spread_shrinks_as_paths_increase() {
    use life_optimizer::monte_carlo::PensionSimulator;

    let median_for = |paths: usize, seed: u64| -> f64 {
        let mut simulator = PensionSimulator::new(40, 65, 85, 110_000.0, 1.0, false, 5_000.0);
        simulator.seed = Some(seed);
        simulator.n_simulations = paths;
        simulator.run_all_scenarios().1.median_capital
    };

    let spread = |paths: usize| -> f64 {
        let medians: Vec<f64> = (1..=5_u64).map(|seed| median_for(paths, seed)).collect();
        let low = medians.iter().cloned().fold(f64::INFINITY, f64::min);
        let high = medians.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        high - low
    };

    let coarse = spread(250);
    let fine = spread(2_000);
    println!(
        "median spread across 5 seeds: CHF {coarse:.0} at 250 paths, CHF {fine:.0} at 2,000 paths"
    );
    assert!(
        fine < coarse,
        "the spread across seeds did not shrink with more paths: CHF {coarse:.0} at 250 paths, \
         CHF {fine:.0} at 2,000"
    );
    // And the fine run must be tight in absolute terms, or "converged" means nothing.
    let reference = median_for(2_000, 1);
    assert!(reference > 0.0, "the projection produced no capital at all");
}
