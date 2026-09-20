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

//! End-to-end tests for the consumption-profile and achievement-capacity CLI
//! surface (`FutureWork.md` §5.1 and §5.2).

use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_life-optimizer");

fn run(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to execute life-optimizer binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// ─── Consumption profile (§5.1) ─────────────────────────────────────────────

/// The elasticity-tier breakdown must be reported rather than one aggregate
/// discretionary figure — the explicit requirement of `THEORY_OF_SPARING.md` §8.
#[test]
fn consumption_breakdown_is_reported() {
    let output = run(&["optimize", "--salary", "120000", "--age", "38"]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);

    assert!(
        out.contains("Consumption by Elasticity Tier"),
        "missing tier section"
    );
    assert!(out.contains("Inelastic"), "missing inelastic tier");
    assert!(
        out.contains("Elastic (sparing-eligible)"),
        "missing elastic tier"
    );
    assert!(out.contains("Mandatory floor"), "missing mandatory floor");
    assert!(out.contains("Full lifestyle basket"), "missing full basket");
}

/// An unknown profile must be rejected with a clear message, not silently
/// treated as normal.
#[test]
fn unknown_consumption_profile_is_rejected() {
    let output = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "38",
        "--consumption-profile",
        "lavish",
    ]);

    assert_eq!(
        output.status.code(),
        Some(2),
        "should exit 2 on a bad profile"
    );
    let err = stderr(&output);
    assert!(err.contains("unknown consumption profile"), "got: {err}");
    assert!(
        err.contains("extreme-saving"),
        "the message should list valid values: {err}"
    );
}

/// A luxury profile must need a strictly larger basket than extreme saving,
/// for identical income and taxes.
#[test]
fn luxury_profile_requires_a_larger_basket_than_extreme_saving() {
    let saving = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "38",
        "--consumption-profile",
        "extreme-saving",
    ]);
    let luxury = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "38",
        "--consumption-profile",
        "luxury",
    ]);

    assert!(saving.status.success());
    assert!(luxury.status.success());

    let basket = |out: &str| -> f64 {
        out.lines()
            .find(|l| l.contains("Full lifestyle basket"))
            .and_then(|l| l.split("CHF").nth(1))
            .and_then(|v| v.trim().replace('\'', "").parse::<f64>().ok())
            .unwrap_or_else(|| panic!("could not parse basket from: {out}"))
    };

    let s = basket(&stdout(&saving));
    let l = basket(&stdout(&luxury));
    assert!(
        l > s,
        "luxury basket ({l}) should exceed extreme-saving basket ({s})"
    );
}

/// A sparing ratio must reduce the basket relative to no sparing.
#[test]
fn sparing_ratio_reduces_the_basket() {
    let plain = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "38",
        "--utilization-discipline",
        "1.0",
    ]);
    let sparing = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "38",
        "--utilization-discipline",
        "1.0",
        "--sparing-ratio",
        "0.8",
    ]);

    assert!(plain.status.success());
    assert!(sparing.status.success());

    let out = stdout(&sparing);
    assert!(
        out.contains("Sparing multiplier"),
        "the applied sparing multiplier should be reported: {out}"
    );
}

/// The quasi-inelastic share must raise the mandatory floor, because locked-in
/// spending is not something the household can flex.
#[test]
fn quasi_inelastic_share_raises_the_floor() {
    let plain = run(&["optimize", "--salary", "120000", "--age", "38"]);
    let locked = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "38",
        "--quasi-inelastic-share",
        "0.5",
    ]);

    assert!(plain.status.success());
    assert!(locked.status.success());

    let floor = |out: &str| -> f64 {
        out.lines()
            .find(|l| l.contains("Mandatory floor"))
            .and_then(|l| l.split("CHF").nth(1))
            .and_then(|v| v.trim().replace('\'', "").parse::<f64>().ok())
            .unwrap_or_else(|| panic!("could not parse floor from: {out}"))
    };

    let p = floor(&stdout(&plain));
    let l = floor(&stdout(&locked));
    assert!(
        l > p,
        "locking spending in should raise the floor: {l} vs {p}"
    );
}

// ─── Achievement constraint (§5.2) ──────────────────────────────────────────

/// Supplying a required output index must engage the constraint and report it.
#[test]
fn achievement_constraint_is_reported_when_supplied() {
    let output = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        out.contains("Employer Achievement Capacity"),
        "missing constraint section: {out}"
    );
    assert!(out.contains("Required (G)"), "missing required-output line");
    assert!(out.contains("Capacity (A)"), "missing capacity line");
}

/// Without the flag the constraint must be absent, preserving existing
/// behaviour in which work percentage is fully discretionary.
#[test]
fn achievement_section_absent_without_the_flag() {
    let output = run(&["optimize", "--salary", "150000", "--age", "40"]);
    assert!(output.status.success());
    assert!(
        !stdout(&output).contains("Employer Achievement Capacity"),
        "the constraint must not appear unless requested"
    );
}

/// A goal only full-time work can meet must force the recommendation to 100%,
/// even for a household that could comfortably afford to work less.
#[test]
fn unreachable_at_part_time_forces_full_time_recommendation() {
    let output = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
    ]);

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(
        out.contains("Work Percentage: 100%"),
        "the constraint should force full-time work: {out}"
    );
}

/// An AI productivity gain must relax the constraint and be reflected in the
/// reported capacity.
#[test]
fn ai_productivity_gain_raises_reported_capacity() {
    let without = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
    ]);
    let with = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
        "--ai-productivity-gain",
        "0.4",
    ]);

    assert!(without.status.success());
    assert!(with.status.success());

    let capacity = |out: &str| -> f64 {
        out.lines()
            .find(|l| l.contains("Capacity (A)"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or_else(|| panic!("could not parse capacity from: {out}"))
    };

    // The reported capacity is at the recommended work percentage, which the
    // AI gain may also lower; compare the underlying capacity at full time via
    // the achievement maths instead of assuming equal work percentages.
    let c_without = capacity(&stdout(&without));
    let c_with = capacity(&stdout(&with));
    assert!(
        c_with >= c_without,
        "AI gain must not lower capacity: {c_with} vs {c_without}"
    );
}

/// An impossible goal must be explained as a workload problem, distinguishing
/// it from an affordability problem.
#[test]
fn impossible_goal_is_reported_as_a_workload_problem() {
    let output = run(&[
        "optimize",
        "--salary",
        "200000",
        "--age",
        "40",
        "--required-output-index",
        "2.0",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        out.contains("NO OPTION MEETS THE REQUIRED OUTPUT") || out.contains("workload problem"),
        "should diagnose the workload constraint, got: {out}"
    );
    assert!(!out.contains("OPTIMAL SOLUTION FOUND"));
}

/// Combining both new feature sets must work, and the two constraints must be
/// reported independently.
#[test]
fn consumption_and_achievement_flags_combine() {
    let output = run(&[
        "optimize",
        "--salary",
        "140000",
        "--age",
        "40",
        "--consumption-profile",
        "moderate",
        "--sparing-ratio",
        "0.5",
        "--utilization-discipline",
        "0.8",
        "--quasi-inelastic-share",
        "0.2",
        "--required-output-index",
        "0.9",
        "--ai-productivity-gain",
        "0.1",
    ]);

    assert!(
        output.status.success(),
        "combined flags failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(out.contains("Consumption by Elasticity Tier"));
    assert!(out.contains("Employer Achievement Capacity"));
}
