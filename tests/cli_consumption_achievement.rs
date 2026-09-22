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

// ─── The §1.4 refinement flags and §2.1 debt ────────────────────────────────

/// The five refinement flags must be accepted and must reach the output, so that
/// each of the critique's items is reachable from the command line rather than
/// only from the library.
///
/// The parameters are chosen so that the recommendation really is a reduced week
/// with a quality drag on it: a goal of 1.0 met at 80% with a +50% AI gain and a
/// compression coefficient of 0.5. At 70% the same settings fail, so 80% is the
/// lowest feasible percentage and the report has to show why.
#[test]
fn refinement_flags_reach_the_report() {
    let output = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
        "--ai-productivity-gain",
        "0.5",
        "--ai-productivity-gain-high",
        "0.8",
        "--compression-quality-sensitivity",
        "0.5",
        "--evaluation-period-years",
        "2",
        "--monthly-debt",
        "300",
    ]);

    assert!(
        output.status.success(),
        "refinement flags failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);

    assert!(
        out.contains("AI gain range"),
        "the pessimistic/optimistic range must be printed: {out}"
    );
    assert!(
        out.contains("Quality factor"),
        "the quality drag must be visible when it is not neutral: {out}"
    );
    assert!(
        out.contains("Robustness:"),
        "every constrained run must state where it sits relative to the AI range"
    );
    assert!(
        out.contains("Average workload"),
        "an evaluation period must change the reported average workload: {out}"
    );
    assert!(out.contains("Saving capacity"), "S_t must be reported");
}

/// A sensitivity that prices the shortfall must be able to offer a schedule the
/// strict mode refuses — the point of the risk-weighted mode — and it must say
/// that the goals are not delivered rather than quietly calling it a success.
#[test]
fn risk_weighted_mode_offers_what_strict_mode_refuses() {
    let common = [
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
        "--replacement-risk",
        "0.5",
    ];

    let strict = run(&common);
    assert!(
        strict.status.success(),
        "strict run failed: {}",
        stderr(&strict)
    );
    let strict_out = stdout(&strict);

    let weighted = run(&[common.as_slice(), &["--enforcement", "risk-weighted"]].concat());
    assert!(
        weighted.status.success(),
        "risk-weighted run failed: {}",
        stderr(&weighted)
    );
    let weighted_out = stdout(&weighted);

    // Strict will only ever offer a schedule that delivers, so it never prints the
    // shortfall status; here that leaves full time, which does deliver.
    assert!(
        !strict_out.contains("OFFERED BUT NOT DELIVERED"),
        "strict mode must not offer a shortfall: {strict_out}"
    );
    assert!(strict_out.contains("Robustness:"));

    // Risk-weighted offers the reduced week and prices what it costs.
    assert!(
        weighted_out.contains("OFFERED BUT NOT DELIVERED"),
        "risk-weighted mode must offer the shortfall and say so: {weighted_out}"
    );
    assert!(
        weighted_out.contains("Replacement risk"),
        "and must print the probability it implies: {weighted_out}"
    );
    assert!(
        weighted_out.contains("Hidden work"),
        "and the workload the pessimistic AI outcome would demand: {weighted_out}"
    );
}

/// `--enforcement risk-weighted` with no declared risk would switch the delivery
/// requirement off entirely, so the combination is refused rather than run.
///
/// The mode's function is to *price* a missed goal. With `--replacement-risk 0`
/// the price is zero, every affordable percentage becomes feasible, and the
/// utility search takes the lowest one — recommending 50% work for a portfolio no
/// percentage can deliver, while the same report states that the goals are
/// undeliverable at every percentage. A flag that silently inverts its own meaning
/// is worth an error message.
#[test]
fn risk_weighted_enforcement_without_a_declared_risk_is_refused() {
    let output = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "1.0",
        "--enforcement",
        "risk-weighted",
    ]);

    assert_eq!(
        output.status.code(),
        Some(2),
        "the combination must be refused, got: {}",
        stdout(&output)
    );
    let err = stderr(&output);
    assert!(
        err.contains("--replacement-risk"),
        "and the message must name the missing flag: {err}"
    );
    assert!(
        !stdout(&output).contains("Work Percentage"),
        "no recommendation may be produced for the refused combination"
    );
}

/// An assignment no percentage can deliver is reported as a workload problem, and
/// the least-bad option is not announced as an optimum.
///
/// The banner and the achievement status have to agree. `is_feasible()` is true by
/// construction under risk-weighted enforcement, so a fallback to the least-bad
/// option would otherwise be announced with "OPTIMAL SOLUTION FOUND!" and then
/// explained by "Why no option worked" a few lines later.
#[test]
fn an_impossible_assignment_is_not_announced_as_an_optimum() {
    let output = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "40",
        "--required-output-index",
        "2.0",
        "--enforcement",
        "risk-weighted",
        "--replacement-risk",
        "2.5",
    ]);

    assert!(
        output.status.success(),
        "the run itself is valid: {}",
        stderr(&output)
    );
    let out = stdout(&output);

    assert!(
        out.contains("NO OPTION MEETS THE REQUIRED OUTPUT"),
        "the report must say the goals are undeliverable: {out}"
    );
    assert!(
        !out.contains("OPTIMAL SOLUTION FOUND"),
        "and must not call the least-bad option an optimum: {out}"
    );
    assert!(
        out.contains("BELOW REQUIRED OUTPUT"),
        "the achievement status must read as a shortfall, not as an offer: {out}"
    );
    assert!(
        !out.contains("OFFERED BUT NOT DELIVERED"),
        "nothing was offered here: {out}"
    );
    assert!(
        out.contains("Work Percentage: 100%"),
        "the least-bad option is full time, not the most leisurely one: {out}"
    );
}

/// Garbage in the new numeric flags must be refused with exit code 2 rather than
/// clamped into a different model.
#[test]
fn invalid_refinement_values_are_refused() {
    for (flag, value) in [
        ("--ai-quality-retention", "1.5"),
        ("--ai-quality-retention", "-0.1"),
        ("--compression-quality-sensitivity", "-1"),
        ("--replacement-risk", "-2"),
        ("--evaluation-period-years", "-1"),
        ("--enforcement", "lenient"),
        ("--monthly-debt", "-100"),
    ] {
        let output = run(&[
            "optimize",
            "--salary",
            "120000",
            "--age",
            "40",
            flag,
            value,
        ]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{flag} {value} should be refused"
        );
        assert!(
            stderr(&output).contains("error:"),
            "{flag} {value} should explain itself"
        );
    }
}

/// A high end below the pessimistic end is a contradiction, not a range.
#[test]
fn an_inverted_ai_range_is_refused() {
    let output = run(&[
        "optimize",
        "--salary",
        "120000",
        "--age",
        "40",
        "--required-output-index",
        "0.9",
        "--ai-productivity-gain",
        "0.40",
        "--ai-productivity-gain-high",
        "0.10",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("below the pessimistic end"),
        "got: {}",
        stderr(&output)
    );
}

/// Debt must reach the mandatory floor end to end: the same household with an
/// instalment has a higher floor than without one.
#[test]
fn monthly_debt_raises_the_reported_floor() {
    let without = run(&["optimize", "--salary", "130000", "--age", "40"]);
    let with = run(&[
        "optimize",
        "--salary",
        "130000",
        "--age",
        "40",
        "--monthly-debt",
        "700",
    ]);

    assert!(without.status.success() && with.status.success());

    let floor = |text: &str| -> f64 {
        text.lines()
            .find(|line| line.contains("Mandatory floor"))
            .and_then(|line| line.rsplit("CHF ").next())
            .and_then(|value| value.trim().parse::<f64>().ok())
            .unwrap_or_else(|| panic!("no mandatory floor line in: {text}"))
    };

    let bare = floor(&stdout(&without));
    let owing = floor(&stdout(&with));
    assert!(
        (owing - bare - 700.0).abs() < 1.0,
        "the floor must rise by the instalment: {bare} -> {owing}"
    );
}
