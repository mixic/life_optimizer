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

//! A BVG contribution computed by hand from the statute, asserted against the engine.
//!
//! `VALIDATION.md` §3 records the defect this file exists to prevent: contributions were levied
//! on gross salary, so a CHF 120,000 earner aged 40 contributed CHF 12,000 where the law requires
//! CHF 6,426. That overstated every projected pension by roughly a fifth and survived 458 passing
//! tests, because no test asked *what salary a contribution is levied on*.
//!
//! The tests in `bvg_contribution_base.rs` pin the base, but they express their expectations in
//! the library's own constants (`BVG_UPPER_LIMIT - BVG_COORDINATION_DEDUCTION`). A test written
//! that way cannot catch a wrong constant: change 90,720 to 91,720 and it still passes. This file
//! is the complement — every figure below is a literal from the statute or the arithmetic done on
//! paper, and the constants are checked *against* those literals rather than used to derive them.
//!
//! # The statute
//!
//! - **BVG Art. 2 Abs. 1** with **BVV 2 Art. 5**: the entry threshold, below which there is no
//!   occupational-pension obligation at all — CHF 22,680 for 2025.
//! - **BVG Art. 8 Abs. 1**: the coordination deduction, subtracted from insured salary — CHF
//!   26,460.
//! - **BVG Art. 8 Abs. 1** with **BVG Art. 2 Abs. 1**: the upper limit of insured salary — CHF
//!   90,720.
//! - **BVG Art. 16 Abs. 1**: the savings credits (Altersgutschriften) by age band — 7% from 25,
//!   10% from 35, 15% from 45, 18% from 55, and nothing below 25.
//!
//! # What this does not establish
//!
//! That a real pension fund's statement agrees. These are the statutory minimums for the
//! obligatory part of the scheme; a fund may credit more, and the conversion rate at retirement is
//! a separate assumption. The projection is checked against the law, not against an institution.

use life_optimizer::monte_carlo::{
    bvg_savings_rate_at_age, coordinated_salary, BVG_COORDINATION_DEDUCTION, BVG_ENTRY_THRESHOLD,
    BVG_UPPER_LIMIT,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The three statutory parameters, against the literal figures rather than against each other.
#[test]
fn the_statutory_parameters_are_the_published_figures() {
    assert_eq!(BVG_UPPER_LIMIT, 90_720.0);
    assert_eq!(BVG_COORDINATION_DEDUCTION, 26_460.0);
    assert_eq!(BVG_ENTRY_THRESHOLD, 22_680.0);
}

/// BVG Art. 16 Abs. 1: the savings credit by age band.
#[test]
fn the_savings_credits_follow_the_statutory_bands() {
    // Nothing before 25.
    assert_eq!(bvg_savings_rate_at_age(0), 0.0);
    assert_eq!(bvg_savings_rate_at_age(24), 0.0);
    // 7% from 25 to 34.
    assert_eq!(bvg_savings_rate_at_age(25), 0.07);
    assert_eq!(bvg_savings_rate_at_age(34), 0.07);
    // 10% from 35 to 44.
    assert_eq!(bvg_savings_rate_at_age(35), 0.10);
    assert_eq!(bvg_savings_rate_at_age(44), 0.10);
    // 15% from 45 to 54.
    assert_eq!(bvg_savings_rate_at_age(45), 0.15);
    assert_eq!(bvg_savings_rate_at_age(54), 0.15);
    // 18% from 55.
    assert_eq!(bvg_savings_rate_at_age(55), 0.18);
    assert_eq!(bvg_savings_rate_at_age(65), 0.18);
}

/// The worked example from `VALIDATION.md` §3, computed on paper.
///
/// A CHF 120,000 salary at age 40. Insured salary is capped at CHF 90,720, the coordination
/// deduction of CHF 26,460 comes off, and the age 35-44 credit of 10% applies:
///
/// ```text
///   insured salary    min(120,000; 90,720) =  90,720
///   coordination      -                     26,460
///   coordinated salary                      64,260
///   savings credit    x 10%                  6,426
/// ```
///
/// The old code applied 10% to the gross salary and contributed CHF 12,000, which is 87% more.
#[test]
fn a_salary_above_the_upper_limit_contributes_from_the_capped_base() {
    let coordinated = coordinated_salary(120_000.0);
    assert!(close(coordinated, 64_260.0), "base {coordinated}");

    let contribution = coordinated * bvg_savings_rate_at_age(40);
    assert!(close(contribution, 6_426.0), "contribution {contribution}");

    // The figure the defect produced, and the ratio §3 reports.
    let on_gross = 120_000.0 * 0.10;
    assert!(close(on_gross, 12_000.0));
    let overstatement = on_gross / contribution - 1.0;
    assert!(
        (overstatement - 0.8673).abs() < 0.001,
        "the gross-salary defect overstated the contribution by {:.2}%",
        overstatement * 100.0
    );
}

/// Three salaries and three ages, each computed from the statute rather than from the engine.
#[test]
fn contributions_match_hand_arithmetic_across_the_schedule() {
    // (salary, age, coordinated salary, contribution)
    let cases = [
        // 50,000 at 30: 50,000 - 26,460 = 23,540, at 7% = 1,647.80
        (50_000.0, 30, 23_540.0, 1_647.80),
        // 75,000 at 47: 75,000 - 26,460 = 48,540, at 15% = 7,281.00
        (75_000.0, 47, 48_540.0, 7_281.00),
        // 300,000 at 58: capped to 90,720 - 26,460 = 64,260, at 18% = 11,566.80
        (300_000.0, 58, 64_260.0, 11_566.80),
        // 22,680 at 40: below the entry threshold, so nothing at all.
        (22_680.0, 40, 0.0, 0.0),
        // 24,000 at 40: above the threshold but the coordination deduction swallows it.
        (24_000.0, 40, 0.0, 0.0),
        // 26,460 at 40: exactly the coordination deduction, so still nothing.
        (26_460.0, 40, 0.0, 0.0),
        // 30,000 at 40: 30,000 - 26,460 = 3,540, at 10% = 354.00
        (30_000.0, 40, 3_540.0, 354.00),
    ];

    for (salary, age, expected_base, expected_contribution) in cases {
        let base = coordinated_salary(salary);
        assert!(
            close(base, expected_base),
            "salary {salary} at age {age}: base {base}, expected {expected_base}"
        );
        let contribution = base * bvg_savings_rate_at_age(age);
        assert!(
            close(contribution, expected_contribution),
            "salary {salary} at age {age}: contribution {contribution}, expected \
             {expected_contribution}"
        );
    }
}

/// Two salaries that both exceed the upper limit insure the same salary, so they must contribute
/// the same amount at the same age. This is the invariant the gross-salary defect broke, stated
/// as arithmetic rather than as a simulation.
#[test]
fn salaries_above_the_upper_limit_contribute_identically() {
    let modest = coordinated_salary(120_000.0) * bvg_savings_rate_at_age(40);
    let large = coordinated_salary(400_000.0) * bvg_savings_rate_at_age(40);
    assert!(close(modest, large), "{modest} against {large}");
    assert!(close(modest, 6_426.0));

    // And a salary just under the cap contributes less, so the cap does not flatten the schedule.
    let under = coordinated_salary(90_000.0) * bvg_savings_rate_at_age(40);
    assert!(under < modest, "{under} should be below {modest}");
    assert!(close(under, (90_000.0 - 26_460.0) * 0.10));
}
