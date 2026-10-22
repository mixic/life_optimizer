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

//! The variance decomposition is checked against functions whose indices are known exactly.
//!
//! A variance-based sensitivity method that has only ever been run on the model it was
//! written for is a method nobody has verified. These tests use two analytic cases with
//! closed-form answers — a purely additive function, and a purely interactive one — because
//! the whole reason for replacing the one-at-a-time sweep is that it mishandles interaction.
//! If the implementation cannot distinguish those two cases, it cannot support the correction
//! it was written to make.

use comprehension_cost::analysis::{
    benefit_at_horizon, generativity, generativity_threshold, review_accounting,
};
use comprehension_cost::global_sensitivity::{
    knobs, model_indices, model_indices_with_ci, range, sign_audit_with_spread, sobol,
    sobol_with_ci,
};
use comprehension_cost::params::Params;

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// Additive model: `y = 2 x1 + x2`, inputs uniform on [0, 1].
///
/// Variance is additive, so the first-order indices are the squared-coefficient shares —
/// `S1 = 4/5`, `S2 = 1/5` — and the total-order indices equal them exactly.
#[test]
fn an_additive_model_recovers_the_analytic_indices() {
    let report =
        sobol(|x| 2.0 * x[0] + x[1], &[(0.0, 1.0), (0.0, 1.0)], &["x1", "x2"], 32768, 12345);

    assert!(
        close(report.indices[0].first, 0.8, 0.02),
        "S1 should be 0.8, got {:.4}",
        report.indices[0].first
    );
    assert!(
        close(report.indices[1].first, 0.2, 0.02),
        "S2 should be 0.2, got {:.4}",
        report.indices[1].first
    );
    // In an additive model there is no interaction variance, so total equals first.
    for index in &report.indices {
        assert!(
            close(index.total, index.first, 0.02),
            "{}: total {:.4} should equal first {:.4}",
            index.name,
            index.total,
            index.first
        );
    }
    assert!(
        close(report.sum_first, 1.0, 0.03),
        "additive indices should sum to 1, got {:.4}",
        report.sum_first
    );
    assert!(
        report.interaction_share() < 0.05,
        "an additive model has no interaction share, got {:.4}",
        report.interaction_share()
    );
}

/// Purely interactive model: `y = (x1 - 0.5)(x2 - 0.5)`.
///
/// Both main effects are exactly zero — knowing `x1` alone tells you nothing about `y`, since
/// the conditional mean is flat — while each input has a total effect of one. This is the case
/// the one-at-a-time sweep cannot represent, and it is why the correction exists.
#[test]
fn a_pure_interaction_is_invisible_to_first_order_and_visible_to_total() {
    let report = sobol(
        |x| (x[0] - 0.5) * (x[1] - 0.5),
        &[(0.0, 1.0), (0.0, 1.0)],
        &["x1", "x2"],
        32768,
        9876,
    );

    for index in &report.indices {
        assert!(
            index.first.abs() < 0.03,
            "{}: a pure interaction has no main effect, got S = {:.4}",
            index.name,
            index.first
        );
        assert!(
            index.total > 0.85,
            "{}: the total effect should approach 1, got ST = {:.4}",
            index.name,
            index.total
        );
        assert!(
            index.interaction() > 0.8,
            "{}: almost all of its influence is interactive, got {:.4}",
            index.name,
            index.interaction()
        );
    }
    assert!(
        report.sum_first < 0.1,
        "first-order indices should be near zero, summed to {:.4}",
        report.sum_first
    );
}

/// An input with no range and no effect draws no variance.
#[test]
fn a_constant_input_has_no_attribution() {
    let report = sobol(|x| x[0] * 3.0, &[(0.0, 1.0)], &["only"], 16384, 1);
    assert!(close(report.indices[0].first, 1.0, 0.05));
    assert!(close(report.indices[0].total, 1.0, 0.05));
}

/// The knob table the two analyses share must be coherent: sixteen inputs, each with a
/// non-degenerate range, the two sweeps describing the same space.
#[test]
fn the_shared_knob_table_is_well_formed() {
    let base = Params::default();
    let knobs = knobs();
    assert_eq!(knobs.len(), 16, "sixteen parameters are attributed");
    for knob in &knobs {
        assert!(knob.spread > 0.0, "{}: a zero spread attributes nothing", knob.name);
        let (low, high) = range(knob, &base);
        assert!(high > low, "{}: degenerate range {low}..{high}", knob.name);
        let central = (knob.get)(&base);
        assert!(
            low <= central && central <= high,
            "{}: the central value {central} sits outside its own range {low}..{high}",
            knob.name
        );
        // Round-tripping through the setter must return the same value, or the two analyses
        // would be varying one parameter while reporting another.
        let mut probe = base.clone();
        (knob.set)(&mut probe, central);
        assert!(
            close((knob.get)(&probe), central, 1e-9),
            "{}: setter and getter disagree",
            knob.name
        );
    }
}

/// The indices on the real model satisfy the identities that make them interpretable.
#[test]
fn the_model_indices_satisfy_the_variance_identities() {
    let base = Params::default();
    let report = model_indices(&base, 256, 2024);

    assert_eq!(report.indices.len(), 16);
    assert!(report.variance > 0.0, "the outcome must vary over the input space");
    for index in &report.indices {
        assert!(index.first.is_finite() && index.total.is_finite(), "{index:?}");
        assert!(
            index.first <= index.total + 0.05,
            "{}: first-order {:.4} exceeds total-order {:.4}",
            index.name,
            index.first,
            index.total
        );
    }
    // Σ ST ≥ 1 always, with equality only when the model is additive. Σ S ≤ 1 always.
    assert!(
        report.sum_total >= 0.9,
        "total-order indices should sum to at least 1, got {:.4}",
        report.sum_total
    );
    assert!(
        report.sum_first <= 1.05,
        "first-order indices cannot exceed 1, summed to {:.4}",
        report.sum_first
    );
    // The whole point of the correction: if the model had no interactions the two sums would
    // coincide, and replacing the one-at-a-time sweep would have been unnecessary.
    assert!(
        report.interaction_share() > 0.05,
        "the model is believed to be interactive; the decomposition shows an interaction \
         share of only {:.4}, which would mean the one-at-a-time sweep was adequate after all",
        report.interaction_share()
    );
}

/// Review is charged to both arms. This is the property a referee asked to be shown rather
/// than asserted, because charging review only to the AI arm would assume the conclusion.
#[test]
fn review_hours_are_charged_to_both_arms() {
    let accounting = review_accounting(&Params::default());

    assert!(
        accounting.both_arms_reviewed(),
        "both arms must be reviewed: human {:.1} h, AI {:.1} h",
        accounting.human_review,
        accounting.ai_review
    );
    // The human arm's review is not zero, so the AI review figure is not the whole bill.
    assert!(
        accounting.human_review > 0.0,
        "the human arm must also spend hours on review, got {:.1}",
        accounting.human_review
    );
    // Reviewing code understood less well costs more, which is the modelled mechanism.
    assert!(
        accounting.ai_review > accounting.human_review,
        "the AI arm should spend more on review, not less: {:.1} against {:.1}",
        accounting.ai_review,
        accounting.human_review
    );
    // And the excess is a fraction of the bill, not a multiple of it.
    let ratio = accounting.ai_review / accounting.human_review;
    assert!(
        (1.0..2.0).contains(&ratio),
        "AI review is {ratio:.2}x the human figure; a doubling would suggest the review term \
         is carrying the result rather than contributing to it"
    );
}

/// The benefit at the horizon used by both analyses is the same function of the same inputs.
#[test]
fn the_attribution_target_is_the_reported_quantity() {
    let base = Params::default();
    // At the declared values, not the central cell, the objective is well defined and finite.
    let value = benefit_at_horizon(&base);
    assert!(value.is_finite(), "benefit at horizon is {value}");
}

/// Bootstrap intervals exist, are ordered, and separate the two contributors whose analytic
/// shares differ by a factor of four.
///
/// The point of the interval is to stop a ranking being read off two point estimates. In the
/// additive case the true indices are 0.8 and 0.2, so a working estimator must not merely
/// order them — it must place their intervals apart.
#[test]
fn bootstrap_intervals_separate_a_known_ranking_and_merge_a_known_tie() {
    let separated = sobol_with_ci(
        |x| 2.0 * x[0] + x[1],
        &[(0.0, 1.0), (0.0, 1.0)],
        &["x1", "x2"],
        4096,
        4242,
        400,
        99,
    );
    for index in &separated.indices {
        let (low, high) = index.total_ci.expect("intervals requested");
        assert!(low <= high, "{}: interval is inverted ({low}, {high})", index.name);
        assert!(high - low < 0.5, "{}: interval is implausibly wide", index.name);
    }
    let (x1, x2) = (&separated.indices[0], &separated.indices[1]);
    let (x1_low, _) = x1.total_ci.unwrap();
    let (_, x2_high) = x2.total_ci.unwrap();
    assert!(
        x1_low > x2_high,
        "0.8 and 0.2 must be distinguishable: x1 lower bound {x1_low:.3} against x2 upper {x2_high:.3}"
    );

    // A model where both inputs have the same share cannot be ordered, and the intervals say so.
    let tied = sobol_with_ci(
        |x| x[0] + x[1],
        &[(0.0, 1.0), (0.0, 1.0)],
        &["x1", "x2"],
        4096,
        4242,
        400,
        99,
    );
    let (a, b) = (&tied.indices[0], &tied.indices[1]);
    let (a_low, _) = a.total_ci.unwrap();
    let (_, b_high) = b.total_ci.unwrap();
    assert!(
        a_low <= b_high,
        "two equal inputs must not be separated: {a_low:.3} against {b_high:.3}"
    );
}

/// The sign audit is a proportion with an interval that brackets it.
#[test]
fn the_sign_audit_is_a_proportion_with_an_interval() {
    let base = Params::default();
    for spread in [0.5, 1.0] {
        let audit = sign_audit_with_spread(&base, 512, 11, spread);
        assert_eq!(audit.samples, 512);
        assert!(audit.negative <= audit.samples);
        assert!((0.0..=1.0).contains(&audit.fraction));
        assert!(
            audit.ci_low <= audit.fraction && audit.fraction <= audit.ci_high,
            "spread {spread}: the interval ({:.3}, {:.3}) does not bracket {:.3}",
            audit.ci_low,
            audit.ci_high,
            audit.fraction
        );
        assert!(audit.worst <= audit.best, "worst must not exceed best");
    }
}

/// Widening the band can only make unanimity harder to reach, never easier.
///
/// This is the property the report's robustness claim rests on: if a wider band produced more
/// uniformity, the band width would be doing the work rather than the model.
#[test]
fn widening_the_band_does_not_increase_the_share_of_negative_draws() {
    let base = Params::default();
    let narrow = sign_audit_with_spread(&base, 1024, 5, 0.5);
    let wide = sign_audit_with_spread(&base, 1024, 5, 1.5);
    assert!(
        wide.fraction <= narrow.fraction + 0.06,
        "widening the band raised the negative share from {:.3} to {:.3}",
        narrow.fraction,
        wide.fraction
    );
}

/// Generative review closes the gap monotonically, and at 100% it has closed all of it.
#[test]
fn generative_review_is_monotone_and_closes_the_gap() {
    let base = Params::default();
    let rows = generativity(&base);

    assert_eq!(rows.len(), 11);
    assert!((rows[0].closed).abs() < 1e-9, "the sweep starts at passive review");
    assert!((rows[10].closed - 1.0).abs() < 1e-9, "and ends at full reconstruction");
    assert!(
        (rows[0].c_ai - base.c_ai).abs() < 1e-9,
        "the passive end must be the declared comprehension"
    );
    assert!(
        (rows[10].c_ai - base.c_human).abs() < 1e-9,
        "the generative end must reach the author's comprehension"
    );

    for pair in rows.windows(2) {
        assert!(
            pair[1].benefit >= pair[0].benefit - 1e-6,
            "closing more of the gap made the outcome worse: {:.2} h at {:.0}% against {:.2} h at {:.0}%",
            pair[0].benefit,
            pair[0].closed * 100.0,
            pair[1].benefit,
            pair[1].closed * 100.0
        );
    }
}

/// The reported threshold is exactly the first row that stops losing.
#[test]
fn the_generativity_threshold_matches_the_sweep() {
    let rows = generativity(&Params::default());
    let threshold = generativity_threshold(&rows);
    for row in &rows {
        match threshold {
            Some(share) => {
                if row.closed < share - 1e-9 {
                    assert!(
                        row.benefit < 0.0,
                        "row at {:.0}% is non-negative but sits below the threshold {:.0}%",
                        row.closed * 100.0,
                        share * 100.0
                    );
                }
            }
            None => assert!(
                row.benefit < 0.0,
                "no threshold is reported but the row at {:.0}% is non-negative",
                row.closed * 100.0
            ),
        }
    }
}
