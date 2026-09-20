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

//! End-to-end tests for `--canton` resolution.
//!
//! Before this was wired, `--canton` was accepted, defaulted to `ZH`, and
//! silently ignored: `--canton ZH` produced Bern numbers. A flag that appears to
//! change the answer but does not is worse than an absent flag, so the contract
//! these tests pin is:
//!
//! * omitting `--canton` uses Bern and says so;
//! * naming a canton whose tax scale is not loaded **fails** with instructions,
//!   rather than substituting another canton's rates;
//! * an unrecognised code is rejected as a code, not as missing data;
//! * `--custom-tax-rate` bypasses cantonal data entirely, so it must not be
//!   blocked by an unresolvable canton.

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

/// With no `--canton`, the run must work and state which canton it assumed.
#[test]
fn omitted_canton_uses_bern_and_says_so() {
    let output = run(&["optimize", "--salary", "120000", "--age", "40"]);

    assert!(output.status.success(), "command failed: {}", stderr(&output));
    let out = stdout(&output);
    assert!(
        out.contains("using Bern"),
        "the assumed canton must be disclosed, not implicit: {out}"
    );
}

/// Naming a canton without a loaded scale must fail loudly. This is the
/// objective's central requirement.
///
/// Ticino is used deliberately rather than Zürich: Zürich has since gained an
/// imported scale and now prices, so it can no longer serve as the failing case.
#[test]
fn unpriced_canton_fails_loudly_and_suggests_a_way_forward() {
    let output = run(&["optimize", "--salary", "120000", "--age", "40", "--canton", "TI"]);

    assert_eq!(
        output.status.code(),
        Some(2),
        "an unpriced canton must exit non-zero rather than return numbers"
    );
    let err = stderr(&output);
    assert!(err.contains("cannot price canton TI"), "got: {err}");
    assert!(
        err.contains("base tax scale"),
        "the message must name what is actually missing: {err}"
    );
    assert!(
        err.contains("--custom-tax-rate"),
        "the message must offer a usable workaround: {err}"
    );
    assert!(
        err.contains("SWISS_TAX_DATA.md"),
        "the message must point at the format documentation: {err}"
    );

    // And it must not have produced a projection alongside the error.
    assert!(
        !stdout(&output).contains("OPTIMAL SOLUTION"),
        "no result may be printed when the canton cannot be priced"
    );
}

/// Cantons whose scales have been imported must now produce a real result, and
/// disclose which basis was used.
///
/// This is the counterweight to the failing case above: an import that silently
/// did nothing would leave these cantons refusing, so the two tests together
/// pin that the data actually reached the calculation.
#[test]
fn imported_cantons_produce_a_result_and_disclose_the_basis() {
    for code in ["ZH", "BS", "LU", "SH", "SO", "AG"] {
        let output = run(&["optimize", "--salary", "120000", "--age", "40", "--canton", code]);

        assert!(
            output.status.success(),
            "{code} should be priceable from the imported scale: {}",
            stderr(&output)
        );
        let out = stdout(&output);
        assert!(
            out.contains("ESTV imported scale x Steuerfuss"),
            "{code} should disclose its tax basis: {out}"
        );
        assert!(
            out.contains("Verify against your own tax assessment"),
            "{code} should carry a verification reminder"
        );
        assert!(
            out.contains("OPTIMAL SOLUTION") || out.contains("NO AFFORDABLE"),
            "{code} should produce a projection"
        );
    }
}

/// The Bern path must remain fully functional — fixing the flag must not break
/// the one canton that works.
#[test]
fn bern_remains_fully_priceable() {
    for canton_args in [vec![], vec!["--canton", "BE"]] {
        let mut args = vec!["optimize", "--salary", "120000", "--age", "40"];
        args.extend(canton_args.iter().copied());

        let output = run(&args);
        assert!(
            output.status.success(),
            "{args:?} failed: {}",
            stderr(&output)
        );
        assert!(
            stdout(&output).contains("OPTIMAL SOLUTION"),
            "{args:?} should produce a result"
        );
    }
}

/// An unrecognised code is a user error about the code itself, and the message
/// should list the valid ones rather than complaining about missing data.
#[test]
fn unknown_canton_code_is_rejected_as_a_code() {
    let output = run(&["optimize", "--salary", "120000", "--age", "40", "--canton", "XX"]);

    assert_eq!(output.status.code(), Some(2));
    let err = stderr(&output);
    assert!(err.contains("not a Swiss canton code"), "got: {err}");
    assert!(err.contains("ZH"), "the valid codes should be listed: {err}");
    assert!(err.contains("BE"), "the valid codes should be listed: {err}");
}

/// `--custom-tax-rate` does not need cantonal data, so an unresolvable canton
/// must not block it. Rejecting this run would be a false obstruction.
#[test]
fn custom_tax_rate_bypasses_canton_resolution() {
    let output = run(&[
        "optimize",
        "--salary", "120000",
        "--age", "40",
        "--canton", "ZH",
        "--custom-tax-rate", "0.14",
    ]);

    assert!(
        output.status.success(),
        "an observed personal rate needs no canton table: {}",
        stderr(&output)
    );
    assert!(stdout(&output).contains("Using custom tax rate"));
}

/// Codes are case-insensitive, like the other code-valued flags.
#[test]
fn canton_codes_are_case_insensitive() {
    let lower = run(&["optimize", "--salary", "120000", "--age", "40", "--canton", "be"]);
    assert!(
        lower.status.success(),
        "lowercase 'be' should be accepted: {}",
        stderr(&lower)
    );
}

/// The help text must not promise a default canton that the tool does not use.
#[test]
fn help_describes_the_real_canton_behaviour() {
    let help = stdout(&run(&["optimize", "--help"]));
    assert!(
        help.contains("--canton"),
        "the flag should still be documented"
    );
    assert!(
        help.contains("SWISS_TAX_DATA.md"),
        "help should point at the data-status document: {help}"
    );
}
