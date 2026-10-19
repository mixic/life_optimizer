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

//! The BVG contribution base.
//!
//! This file exists because the defect it guards survived a suite of four hundred tests. Nothing
//! asserted *what salary* the savings rate is applied to, so a projection that computed
//! contributions on gross salary passed every check the repository had — and overstated the
//! pension by roughly a factor of two for an ordinary Swiss salary. The projection is what the
//! adequacy verdict rests on, so this was the most expensive kind of error: not a missing feature
//! but a wrong number, presented with the tool's usual confidence.

use life_optimizer::monte_carlo::{
    coordinated_salary, PensionSimulator, BVG_COORDINATION_DEDUCTION, BVG_ENTRY_THRESHOLD,
    BVG_UPPER_LIMIT,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The contribution base is insured salary less the coordination deduction, with insured salary
/// capped at the upper limit. Checked against arithmetic done by hand.
#[test]
fn contributions_are_levied_on_the_coordinated_salary() {
    // CHF 120,000 is above the upper limit, so insured salary is the cap, not the salary.
    let expected = BVG_UPPER_LIMIT - BVG_COORDINATION_DEDUCTION;
    assert!(close(coordinated_salary(120_000.0), expected));
    // The old behaviour: 120,000 itself, about twice as much.
    assert!(coordinated_salary(120_000.0) < 120_000.0 * 0.6);

    // A salary between the entry threshold and the upper limit is insured in full.
    let mid = 50_000.0;
    assert!(close(coordinated_salary(mid), mid - BVG_COORDINATION_DEDUCTION));
}

/// Below the entry threshold there is no BVG obligation, so the contribution base is zero rather
/// than negative or a small number.
#[test]
fn below_the_entry_threshold_there_is_no_contribution_base() {
    assert_eq!(coordinated_salary(0.0), 0.0);
    assert_eq!(coordinated_salary(BVG_ENTRY_THRESHOLD - 1.0), 0.0);
    // Exactly at the threshold the coordination deduction swallows the salary, so still zero.
    assert_eq!(coordinated_salary(BVG_ENTRY_THRESHOLD), 0.0);
    // And never negative anywhere.
    for salary in [0.0, 10_000.0, 22_679.0, 26_459.0, 26_460.0] {
        assert!(coordinated_salary(salary) >= 0.0);
    }
}

/// The upper limit means contributions stop rising with pay. Before the fix they rose without
/// bound, which made a high salary look like a generous pension.
#[test]
fn the_upper_limit_caps_the_insured_salary() {
    let at_cap = coordinated_salary(BVG_UPPER_LIMIT);
    assert!(close(coordinated_salary(200_000.0), at_cap));
    assert!(close(coordinated_salary(1_000_000.0), at_cap));
    assert!(coordinated_salary(BVG_UPPER_LIMIT - 1.0) < at_cap);
}

/// The cap must reach the projection, not just the helper. Two salaries far above the upper limit
/// insure the same salary, so with the same seed they must accumulate the same BVG capital.
///
/// This is the test that would have caught the original defect: under gross-salary contributions
/// the two capitals differ by a factor of nearly two.
#[test]
fn two_salaries_above_the_cap_accumulate_the_same_capital() {
    let build = |salary: f64| {
        let mut simulator = PensionSimulator::new(40, 65, 85, salary, 1.0, false, 5_000.0);
        simulator.seed = Some(7);
        simulator.n_simulations = 50;
        simulator.run_all_scenarios().1
    };
    let modest = build(120_000.0);
    let large = build(400_000.0);
    assert!(
        close(modest.median_capital, large.median_capital),
        "salaries above the cap accumulated {} and {}, which means the cap is not reaching the \
         projection",
        modest.median_capital,
        large.median_capital
    );
    assert!(modest.median_capital > 0.0);
}

/// And the base must matter below the cap: a higher salary between the threshold and the cap must
/// accumulate more, or the fix has gone too far and flattened the whole schedule.
#[test]
fn below_the_cap_a_higher_salary_accumulates_more() {
    let build = |salary: f64| {
        let mut simulator = PensionSimulator::new(40, 65, 85, salary, 1.0, false, 5_000.0);
        simulator.seed = Some(7);
        simulator.n_simulations = 50;
        simulator.run_all_scenarios().1
    };
    let low = build(45_000.0);
    let high = build(80_000.0);
    assert!(
        high.median_capital > low.median_capital,
        "below the cap a higher salary did not accumulate more: {} against {}",
        high.median_capital,
        low.median_capital
    );
}
