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

//! End-to-end tests that drive the compiled `life-optimizer` binary.
//!
//! These are the only tests that cover the CLI surface itself — flag parsing,
//! exit codes, and the exact text a user sees. Behaviour that unit tests cannot
//! reach (process exit, panic-vs-clean-error) belongs here.

use std::process::{Command, Output};

/// Path to the binary under test, provided by Cargo.
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

// ─── Age validation: used to panic with a u32 underflow ─────────────────────

/// Regression: a retirement age at or before the current age used to reach
/// `u32` subtraction in the optimizer and panic. It must now fail cleanly.
#[test]
fn retirement_age_at_or_before_current_age_exits_with_error_not_panic() {
    for (age, retirement_age) in [("66", "65"), ("65", "65"), ("70", "65")] {
        let output = run(&[
            "optimize",
            "--salary",
            "100000",
            "--age",
            age,
            "--retirement-age",
            retirement_age,
        ]);

        assert_eq!(
            output.status.code(),
            Some(2),
            "age {age} / retirement {retirement_age} should exit 2, got {:?}",
            output.status.code()
        );
        let err = stderr(&output);
        assert!(
            err.contains("must be greater than"),
            "expected a clear validation message, got: {err}"
        );
        assert!(!err.contains("panicked"), "must not panic, got: {err}");
    }
}

/// Life expectancy at or before retirement is equally invalid.
#[test]
fn life_expectancy_at_or_before_retirement_exits_with_error() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--retirement-age",
        "65",
        "--life-expectancy",
        "60",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("must be greater than"));
}

// ─── Conversion rate: FutureWork.md §10 ─────────────────────────────────────

/// `--conversion-rate` must drive the headline figure and be echoed back.
#[test]
fn pension_with_custom_conversion_rate_shows_the_custom_row() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
        "--conversion-rate",
        "0.055",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);

    assert!(
        out.contains("MONTHLY PENSION BY CONVERSION RATE"),
        "missing section header"
    );
    assert!(out.contains("Your pension fund"), "missing custom-rate row");
    assert!(out.contains("Realistic range"), "missing range line");
    // The disclaimer telling the user the figure depends on their fund.
    assert!(
        out.contains("--conversion-rate"),
        "output should point the user at the flag"
    );
}

/// Without the flag, all three reference scenarios must still be shown, so the
/// uncertainty is visible even when the user has not supplied their own rate.
#[test]
fn pension_without_custom_rate_shows_all_three_scenarios() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);

    assert!(out.contains("Statutory minimum"), "missing statutory row");
    assert!(out.contains("Typical Swiss fund"), "missing typical row");
    assert!(out.contains("Projected for"), "missing projection row");
    assert!(
        !out.contains("Your pension fund"),
        "custom row must not appear when unset"
    );
}

/// A rate above the statutory minimum must be reported as better, not worse.
#[test]
fn custom_rate_above_statutory_is_reported_as_better() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
        "--conversion-rate",
        "0.075",
    ]);

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(
        out.contains("above the statutory minimum"),
        "a 7.5% rate should read as above the statutory 6.8%: {out}"
    );
}

// ─── Feasibility reporting ──────────────────────────────────────────────────

/// Regression: the tool used to print "OPTIMAL SOLUTION FOUND!" immediately
/// above "BELOW REQUIREMENTS". It must not claim an optimum it does not have.
#[test]
fn infeasible_household_does_not_claim_an_optimal_solution() {
    // 60k with two children cannot cover the default requirement basket.
    let output = run(&[
        "optimize",
        "--salary",
        "60000",
        "--age",
        "35",
        "--married",
        "true",
        "--children",
        "2",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);

    assert!(
        !out.contains("OPTIMAL SOLUTION FOUND"),
        "must not announce an optimum when nothing is affordable"
    );
    assert!(
        out.contains("NO AFFORDABLE OPTION"),
        "should state plainly that no option is affordable: {out}"
    );
    assert!(
        out.contains("closest option"),
        "should describe the fallback honestly"
    );
}

/// A feasible household must still report an optimum.
#[test]
fn feasible_household_reports_an_optimal_solution() {
    let output = run(&[
        "optimize",
        "--salary",
        "150000",
        "--age",
        "50",
        "--married",
        "true",
        "--children",
        "0",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        out.contains("OPTIMAL SOLUTION FOUND"),
        "an affordable household should get an optimum: {out}"
    );
    assert!(
        out.contains("MEETS ALL REQUIREMENTS"),
        "should report feasibility"
    );
}

// ─── General CLI health ─────────────────────────────────────────────────────

/// Every subcommand must at least parse and exit successfully.
#[test]
fn all_subcommands_run_successfully() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["optimize", "--salary", "120000", "--age", "38"],
        vec!["compare", "--salary", "100000", "--age", "40"],
        vec!["lifetime", "--salary", "100000", "--age", "30"],
        vec![
            "pension",
            "--salary",
            "100000",
            "--age",
            "40",
            "--work-pct",
            "0.8",
        ],
    ];

    for args in cases {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{:?} failed with {:?}: {}",
            args,
            output.status.code(),
            stderr(&output)
        );
        assert!(!stdout(&output).is_empty(), "{args:?} produced no output");
    }
}

/// Help must list every subcommand, and the new flag must be discoverable.
#[test]
fn help_lists_subcommands_and_conversion_rate_flag() {
    let top = stdout(&run(&["--help"]));
    for cmd in ["optimize", "compare", "lifetime", "interactive", "pension"] {
        assert!(top.contains(cmd), "top-level help should list `{cmd}`");
    }

    let pension = stdout(&run(&["pension", "--help"]));
    assert!(
        pension.contains("--conversion-rate"),
        "pension help should document --conversion-rate"
    );

    let optimize = stdout(&run(&["optimize", "--help"]));
    assert!(
        optimize.contains("--conversion-rate"),
        "optimize help should document --conversion-rate"
    );
}

/// An unknown flag must fail rather than be silently ignored.
#[test]
fn unknown_flag_is_rejected() {
    let output = run(&[
        "optimize",
        "--salary",
        "100000",
        "--age",
        "40",
        "--not-a-flag",
    ]);
    assert!(
        !output.status.success(),
        "an unknown flag must not exit successfully"
    );
}

// ─── Pension fund profiles and stochastic rate uncertainty (§10.5) ──────────

/// A named fund profile must be accepted, echoed with its provenance, and carry
/// a verification reminder — a stale rate is a plausible-looking wrong input.
#[test]
fn pension_fund_profile_is_used_and_flagged_for_verification() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
        "--pension-fund",
        "publica",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        out.contains("Using pension fund profile"),
        "should echo the profile: {out}"
    );
    assert!(out.contains("Publica"), "should name the fund");
    assert!(
        out.contains("Verify"),
        "must remind the user to verify the rate"
    );
    assert!(
        out.contains("Your pension fund"),
        "should appear as the custom row"
    );
}

/// An unknown fund id must fail with the valid options listed.
#[test]
fn unknown_pension_fund_is_rejected() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--pension-fund",
        "not-a-fund",
    ]);

    assert_eq!(output.status.code(), Some(2), "should exit 2");
    let err = stderr(&output);
    assert!(err.contains("unknown pension fund"), "got: {err}");
    assert!(err.contains("publica"), "should list valid ids: {err}");
}

/// An explicit rate must take precedence over a named profile, because the
/// user's own figure is always better than a reference value.
#[test]
fn explicit_conversion_rate_overrides_fund_profile() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
        "--pension-fund",
        "publica",
        "--conversion-rate",
        "0.045",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        !out.contains("Using pension fund profile"),
        "an explicit rate should bypass the profile: {out}"
    );
    assert!(
        out.contains("4.50%"),
        "the explicit rate should be used: {out}"
    );
}

/// The stochastic section must appear on a normal run, with CVaR reported
/// alongside the percentiles.
#[test]
fn stochastic_rate_uncertainty_is_reported() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
    ]);

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        out.contains("CONVERSION-RATE UNCERTAINTY"),
        "missing stochastic section: {out}"
    );
    assert!(
        out.contains("CVaR"),
        "downside risk must be reported explicitly"
    );
    assert!(
        out.contains("Probability of falling below"),
        "missing shortfall probability"
    );
}

/// The BVG-only pension table must be labelled as such, so a reader cannot
/// mistake it for total retirement income.
#[test]
fn bvg_only_pension_table_is_labelled() {
    let output = run(&[
        "pension",
        "--salary",
        "100000",
        "--age",
        "40",
        "--work-pct",
        "0.8",
    ]);
    let out = stdout(&output);
    assert!(
        out.contains("BVG (occupational) annuity only"),
        "the BVG-only table must say so: {out}"
    );
}
