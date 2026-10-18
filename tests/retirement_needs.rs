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

//! Retirement needs as an input, a range, and a break-even.

use life_optimizer::retirement_needs::{
    projected_monthly, Adequacy, Basis, NeedsReport, NEEDS_RANGE,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The default stays the conventional rule of thumb, and must describe itself as one. Replacing it
/// with a different guess would be the same error with a different number.
#[test]
fn the_default_is_the_rule_of_thumb_and_says_so() {
    let basis = Basis::default();
    assert_eq!(basis, Basis::PercentOfNet(0.75));
    assert!(!basis.is_explicit());
    assert!(close(basis.monthly(10_000.0), 7_500.0));
    let description = basis.describe(10_000.0);
    assert!(
        description.contains("rule of thumb"),
        "the default must name itself as a rule of thumb, got {description:?}"
    );
    assert!(description.contains("75%"));
}

/// An explicit monthly figure is the only version that is a measurement of the user's life, so it
/// takes precedence and reports itself as supplied.
#[test]
fn an_explicit_figure_takes_precedence() {
    let basis = Basis::ExplicitMonthly(6_400.0);
    assert!(basis.is_explicit());
    assert!(close(basis.monthly(12_000.0), 6_400.0));
    assert!(basis.describe(12_000.0).contains("supplied by you"));
}

/// Coverage is the ratio of projected income to need, and the verdict must bracket one exactly.
#[test]
fn the_verdict_brackets_the_coverage_ratio() {
    assert_eq!(Adequacy::of(0.99), Adequacy::Short);
    assert_eq!(Adequacy::of(1.0), Adequacy::Marginal);
    assert_eq!(Adequacy::of(1.09), Adequacy::Marginal);
    assert_eq!(Adequacy::of(1.10), Adequacy::Adequate);
    assert_eq!(Adequacy::of(2.5), Adequacy::Adequate);
    // A need of zero cannot be short, whatever the pension.
    assert_eq!(Adequacy::of(f64::INFINITY), Adequacy::Adequate);
}

/// The break-even is the projected pension, because coverage is linear in needs. Stated as a test
/// because the number is easy to get wrong when the two sides of the ratio are swapped.
#[test]
fn the_break_even_is_the_projected_pension() {
    let report = NeedsReport::new(6_000.0, 8_000.0, Basis::PercentOfNet(0.75));
    assert!(close(report.break_even_monthly, 6_000.0));
    // At exactly the break-even need, coverage is one.
    let at_break_even = NeedsReport::new(6_000.0, 8_000.0, Basis::ExplicitMonthly(6_000.0));
    assert!(close(at_break_even.chosen.coverage, 1.0));
    assert_eq!(at_break_even.verdict(), Adequacy::Marginal);
}

/// The reported range is 60/75/90 and its needs are increasing, whatever basis the user chose.
#[test]
fn the_range_is_sixty_seventy_five_ninety() {
    let report = NeedsReport::new(6_000.0, 8_000.0, Basis::PercentOfNet(0.75));
    assert_eq!(report.range.len(), NEEDS_RANGE.len());
    let needs: Vec<f64> = report.range.iter().map(|point| point.monthly).collect();
    assert!(close(needs[0], 4_800.0));
    assert!(close(needs[1], 6_000.0));
    assert!(close(needs[2], 7_200.0));
    assert!(needs.windows(2).all(|pair| pair[1] > pair[0]));
    // And the middle of the range is the chosen basis when that basis is a percentage.
    assert!(close(report.chosen.monthly, needs[1]));
}

/// The point of the range: a plan that is adequate at 60% and short at 90% has not been shown to be
/// adequate, and the report must say so rather than quoting the middle figure.
#[test]
fn a_verdict_that_depends_on_spending_is_reported_as_depending() {
    // 6,000 projected against 8,000 net: adequate at 60%, marginal at 75%, short at 90%.
    let report = NeedsReport::new(6_000.0, 8_000.0, Basis::PercentOfNet(0.75));
    assert_eq!(report.range[0].verdict, Adequacy::Adequate);
    assert_eq!(report.range[1].verdict, Adequacy::Marginal);
    assert_eq!(report.range[2].verdict, Adequacy::Short);
    assert!(
        report.is_sensitive_to_the_range(),
        "the verdict changes across the range and the report must notice"
    );
    let headline = report.headline();
    assert!(
        headline.contains("depends"),
        "the headline must say the verdict depends on spending, got {headline:?}"
    );
    assert!(headline.contains("6000"), "the headline must quote the break-even: {headline:?}");
}

/// A plan that clears every point of the range, or none of them, is not range-sensitive: reporting
/// it as though it were would train the reader to ignore the flag.
#[test]
fn a_verdict_that_holds_across_the_range_is_not_flagged() {
    let comfortable = NeedsReport::new(20_000.0, 8_000.0, Basis::PercentOfNet(0.75));
    assert!(!comfortable.is_sensitive_to_the_range());
    assert_eq!(comfortable.verdict(), Adequacy::Adequate);

    let hopeless = NeedsReport::new(3_000.0, 8_000.0, Basis::PercentOfNet(0.75));
    assert!(!hopeless.is_sensitive_to_the_range());
    assert_eq!(hopeless.verdict(), Adequacy::Short);
    assert!(!hopeless.headline().contains("depends"));
}

/// When the user supplies a figure, the range is built around *their* figure rather than reverting
/// to percentages of income. The supplied number is the only one they have actually said.
#[test]
fn an_explicit_basis_still_gets_a_range_around_it() {
    let report = NeedsReport::new(6_000.0, 8_000.0, Basis::ExplicitMonthly(6_000.0));
    assert!(close(report.range[1].monthly, 6_000.0), "the middle of the range is the user's figure");
    assert!(close(report.range[0].monthly, 4_800.0), "60% of the rule of thumb, applied to their figure");
    assert!(close(report.range[2].monthly, 7_200.0));
}

/// The two halves of retirement income are joined in one place, so the adequacy figure and the
/// needs report cannot disagree about what the plan pays.
#[test]
fn the_projection_is_bvg_plus_ahv() {
    assert!(close(projected_monthly(4_200.0, 2_300.0), 6_500.0));
    // Negative inputs are clamped rather than allowed to reduce the other half.
    assert!(close(projected_monthly(-100.0, 2_300.0), 2_300.0));
    assert!(close(projected_monthly(4_200.0, -100.0), 4_200.0));
}

/// Every input must be described, including the ones that are declared rather than measured.
#[test]
fn every_input_is_described() {
    let table = life_optimizer::retirement_needs::provenance_table();
    assert_eq!(table.len(), 3);
    for (name, detail) in &table {
        assert!(!name.is_empty());
        assert!(detail.len() > 40, "the description for {name} is not a description");
    }
    // The rule of thumb must be labelled as one and the break-even as derived.
    let needs = table
        .iter()
        .find(|(name, _)| *name == "retirement needs")
        .map(|(_, detail)| *detail)
        .expect("the needs row exists");
    assert!(needs.contains("rule of thumb"));
    let break_even = table
        .iter()
        .find(|(name, _)| *name == "break-even")
        .map(|(_, detail)| *detail)
        .expect("the break-even row exists");
    assert!(break_even.contains("derived"));
}
/// The range rows must be labelled by the share they were built from, not by a ratio of the
/// monthly figures. The first version of the report recomputed the ratio and printed the 60% row
/// as 80%, because the range is built by scaling relative to the 75% reference.
#[test]
fn the_range_is_labelled_by_the_share_it_was_built_from() {
    let report = NeedsReport::new(6_000.0, 8_000.0, Basis::Modelled(6_000.0));
    for (point, expected) in report.range.iter().zip(NEEDS_RANGE.iter()) {
        assert!(
            (point.share - expected).abs() < 1e-12,
            "a range point carries share {} instead of {expected}",
            point.share
        );
    }
    // The bug this guards: the ratio of monthly figures is NOT the share.
    let ratio = report.range[0].monthly / report.chosen.monthly;
    assert!(
        (ratio - report.range[0].share).abs() > 0.01,
        "the fixture no longer distinguishes the share from the ratio, so the test proves nothing"
    );
    // A percentage basis labels itself with its own share.
    let percentage = NeedsReport::new(6_000.0, 8_000.0, Basis::PercentOfNet(0.6));
    assert!((percentage.chosen.share - 0.6).abs() < 1e-12);
}
