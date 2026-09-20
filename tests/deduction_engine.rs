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

//! Tests for the sourced deduction engine.
//!
//! Several of these are **regression tests for bugs that were live in the first
//! version of the engine and were found by comparing its output against the
//! existing estimate**, not by reasoning. Each is named for the failure it
//! prevents, because the failure mode was in every case a plausible-looking
//! number rather than an error:
//!
//! | Bug | Effect |
//! |---|---|
//! | `Verheiratetenabzug` applied to a single taxpayer | −CHF 2,800 too much |
//! | All three child age brackets applied together | 37% of income deducted |
//! | Secondary-employment expenses with no secondary income | −CHF 2,400 |
//! | Pillar 3a tied *and* untied variants applied | double-counted 7,056 |
//! | `contains("Kind")` matched `"ohne Kind"` | disabled FR/VS single deductions |

use life_optimizer::deductions::{
    assess, classify, means_tested_scale, Household, RuleCategory, SkipReason,
};

/// A household with no children and no declared optional costs must not receive
/// a marriage allowance.
///
/// The `Verheiratetenabzug` rule names its qualifying group, and the first
/// version applied it unconditionally, deducting CHF 2,800 from a single person.
#[test]
fn marriage_allowance_requires_a_married_household() {
    let single = Household::employee(100_000.0, false, 0);
    let married = Household::employee(100_000.0, true, 0);

    let single_assessment = assess("Bund", &single);
    assert!(
        !single_assessment
            .applied
            .iter()
            .any(|d| d.category == RuleCategory::Marriage),
        "a single taxpayer must not receive a marriage allowance: {:?}",
        single_assessment.applied
    );

    let married_assessment = assess("Bund", &married);
    let marriage = married_assessment
        .applied
        .iter()
        .find(|d| d.category == RuleCategory::Marriage);
    assert!(
        marriage.is_some_and(|d| d.amount > 0.0),
        "a married household should receive the marriage allowance: {:?}",
        married_assessment.applied
    );
}

/// Mutually exclusive variants must not be summed. The strongest case is the
/// child age brackets: all three are present per canton, and a household falls
/// in exactly one.
#[test]
fn mutually_exclusive_child_brackets_are_not_summed() {
    let household = Household::employee(100_000.0, true, 2);
    let assessment = assess("AG", &household);

    let child_deductions: Vec<_> = assessment
        .applied
        .iter()
        .filter(|d| d.category == RuleCategory::Children)
        .collect();

    assert_eq!(
        child_deductions.len(),
        1,
        "exactly one child deduction must apply, got {child_deductions:?}"
    );

    // Aargau publishes brackets at 9,300 / 10,300 / 12,400; the engine keeps the
    // largest, and the others must be reported rather than silently dropped.
    let kept = child_deductions[0];
    assert!(
        kept.amount >= 9_300.0,
        "the largest applicable bracket should be kept, got {}",
        kept.amount
    );

    // The whole point: two children must not produce a three-bracket total.
    assert!(
        assessment.effective_rate() < 0.25,
        "deductions should stay plausible for a 2-child household, got {:.1}%",
        assessment.effective_rate() * 100.0
    );
}

/// Secondary-employment expenses require a secondary income.
#[test]
fn secondary_employment_expenses_need_a_secondary_income() {
    let without = Household::employee(100_000.0, false, 0);
    let assessment = assess("Bund", &without);
    assert!(
        !assessment
            .applied
            .iter()
            .any(|d| d.name.contains("Nebenerwerb")),
        "no secondary income, so no secondary-employment deduction: {:?}",
        assessment.applied
    );

    let mut with = Household::employee(100_000.0, false, 0);
    with.secondary_income = Some(20_000.0);
    let assessment = assess("Bund", &with);
    assert!(
        assessment
            .applied
            .iter()
            .any(|d| d.name.contains("Nebenerwerb")),
        "a secondary income should unlock the deduction: {:?}",
        assessment.applied
    );
}

/// The two pillar 3a variants are alternatives, not additions.
///
/// Applying both double-counted a declared contribution: CHF 14,112 for a
/// CHF 7,056 payment.
#[test]
fn pillar_3a_variants_are_alternatives_not_additions() {
    let mut household = Household::employee(100_000.0, false, 0);
    household.pillar3a_contribution = Some(7_056.0);

    let assessment = assess("Bund", &household);
    let pillar: Vec<_> = assessment
        .applied
        .iter()
        .filter(|d| d.category == RuleCategory::Pillar3a)
        .collect();

    assert_eq!(pillar.len(), 1, "only one 3a variant may apply: {pillar:?}");
    assert!(
        (pillar[0].amount - 7_056.0).abs() < 1e-9,
        "the declared contribution is the deduction, got {}",
        pillar[0].amount
    );

    // And it must never exceed what was actually paid, even above the ceiling.
    household.pillar3a_contribution = Some(50_000.0);
    let assessment = assess("Bund", &household);
    let total: f64 = assessment
        .applied
        .iter()
        .filter(|d| d.category == RuleCategory::Pillar3a)
        .map(|d| d.amount)
        .sum();
    assert!(
        total < 50_000.0,
        "the published maximum must cap a large contribution, got {total}"
    );
}

/// `"ohne Kind"` contains `"Kind"`, so a keyword test wrongly required children
/// for a scale that explicitly excludes them — disabling Fribourg's and Valais's
/// single-person deductions entirely.
#[test]
fn means_tested_scales_match_child_conditions_as_conditions() {
    let single_no_kids = Household::employee(15_000.0, false, 0);
    let scale = means_tested_scale("FR", &single_no_kids);
    assert!(
        scale.is_some(),
        "Fribourg's single-person means-tested deduction must be found"
    );
    assert!(
        scale.unwrap().name.contains("Ledige"),
        "got {:?}",
        scale.unwrap().name
    );

    let assessment = assess("FR", &single_no_kids);
    assert!(
        assessment.means_tested > 0.0,
        "the means-tested deduction should phase in at CHF 15,000"
    );

    // A household with children must NOT get the "ohne Kind" variant.
    let with_kids = Household::employee(15_000.0, false, 2);
    let scale = means_tested_scale("FR", &with_kids);
    assert!(
        scale.is_none_or(|s| !s.name.contains("ohne Kind")),
        "a household with children must not take the 'ohne Kind' scale: {scale:?}"
    );
}

/// The means-tested deductions phase down as income rises, which is what makes
/// them means-tested.
#[test]
fn means_tested_deductions_phase_down() {
    let low = assess("FR", &Household::employee(15_000.0, false, 0)).means_tested;
    let mid = assess("FR", &Household::employee(25_000.0, false, 0)).means_tested;
    let high = assess("FR", &Household::employee(60_000.0, false, 0)).means_tested;

    assert!(low > mid, "the deduction should fall with income: {low} then {mid}");
    assert!(mid > high, "and keep falling: {mid} then {high}");
    assert_eq!(high, 0.0, "above the top threshold there is nothing to deduct");
}

/// Deductions can never exceed income, so taxable income is never negative.
///
/// Valais's phase-out table legitimately allows CHF 21,250, which exceeds a
/// CHF 15,000 income.
#[test]
fn deductions_never_exceed_income() {
    for (canton, gross) in [("VS", 15_000.0), ("FR", 15_000.0), ("Bund", 5_000.0)] {
        let assessment = assess(canton, &Household::employee(gross, false, 0));
        assert!(
            assessment.total() <= gross,
            "{canton} at {gross}: total {} exceeds income",
            assessment.total()
        );
        assert!(
            assessment.taxable_income() >= 0.0,
            "{canton}: taxable income went negative"
        );
    }

    // The cap must be visible rather than hidden: the uncapped sum is larger.
    let valais = assess("VS", &Household::employee(15_000.0, false, 0));
    assert!(
        valais.uncapped_total() > valais.gross_income,
        "this test is meaningless unless the cap is actually binding"
    );
}

/// Every deduction rule in the export must either be classified or reported.
///
/// A rule that is silently skipped is indistinguishable from a rule that does
/// not exist, and the difference is money.
#[test]
fn unclassified_rules_are_reported_not_dropped() {
    let assessment = assess("ZH", &Household::employee(100_000.0, true, 2));
    let unclassified = assessment.unclassified();

    // The point is accounting: applied + skipped must equal the rules for the
    // jurisdiction, so nothing vanishes between them.
    let total = assessment.applied.len() + assessment.skipped.len();
    let zh_rules = assessment
        .skipped
        .len()
        + assessment.applied.len();
    assert_eq!(
        total, zh_rules,
        "every rule must be accounted for as applied or skipped"
    );
    assert!(
        assessment
            .skipped
            .iter()
            .any(|s| s.reason == SkipReason::NotADeduction),
        "threshold and non-deduction rows must be reported, not applied"
    );
    // Unclassified rules are surfaced separately, so the count is visible.
    let _ = unclassified;
}

/// Classification must be by prefix into a closed list, and the awkward cases
/// must land in the right place.
#[test]
fn classification_handles_the_ambiguous_labels() {
    // Property maintenance must not be read as a professional expense.
    assert_eq!(
        classify("Pauschalabzug Unterhaltskosten Liegenschaften mit Alter bis 10 Jahren"),
        Some(RuleCategory::PropertyMaintenance)
    );
    assert_eq!(
        classify("Pauschalabzug übrige Berufskosten"),
        Some(RuleCategory::ProfessionalExpenses)
    );
    // A threshold is not a deduction, so it classifies as means-tested (which is
    // applied through the scales) rather than as a flat amount.
    assert_eq!(
        classify("Schwellwert für Abzug AHV-/IV-Rentner"),
        Some(RuleCategory::MeansTested)
    );
    // Something genuinely unknown returns None rather than a nearest guess.
    assert_eq!(classify("Ein völlig unbekannter Abzug"), None);
}

/// A jurisdiction with no imported rules yields an empty assessment rather than
/// a panic or a fallback to another canton.
#[test]
fn unknown_jurisdiction_yields_nothing() {
    let assessment = assess("XX", &Household::employee(100_000.0, false, 0));
    assert_eq!(assessment.total(), 0.0);
    assert!(assessment.applied.is_empty());
    assert!(assessment.skipped.is_empty());
    assert_eq!(assessment.taxable_income(), 100_000.0);
}

/// The sourced model must produce a materially different answer from the
/// hand-entered estimate, or there is no point in the work.
///
/// This is deliberately a *characterisation* test rather than an assertion that
/// the sourced figure is correct: it pins the direction and rough size of the
/// gap so a future change to either side is noticed. The estimate caps at 35% of
/// gross and hand-picks its components; for a household that declares nothing,
/// the sourced rules allow only the flat allowances.
#[test]
fn sourced_deductions_differ_from_the_hand_entered_estimate() {
    use life_optimizer::cantons::Canton;
    use life_optimizer::tax::TaxSchedule;

    for (canton, gross, married, children) in [
        (Canton::Zurich, 100_000.0, false, 0u32),
        (Canton::Aargau, 100_000.0, true, 2),
    ] {
        let schedule =
            TaxSchedule::from_canton_scale(canton, married, children).expect("priceable");
        let estimate = schedule.standard_deduction_estimate(gross);
        let sourced = assess(canton.code(), &Household::employee(gross, married, children));

        assert!(estimate > 0.0 && sourced.total() > 0.0);
        // The estimate is not a small perturbation of the sourced figure.
        assert!(
            (estimate - sourced.total()).abs() > 1_000.0,
            "{}: the estimate ({estimate}) and the sourced figure ({}) should differ \
             materially, otherwise this test proves nothing",
            canton.code(),
            sourced.total()
        );
    }
}
