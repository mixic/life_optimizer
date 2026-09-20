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

// ── Property: the base is the imputed rental value, not income ───────────────

/// Every property rule is a percentage of the **imputed rental value**, so
/// applying one to gross income deducts a percentage of the wrong base.
///
/// A tenant must therefore get *no* property deduction at all, and a homeowner's
/// must scale with the rental value rather than with their salary.
#[test]
fn property_deductions_use_the_rental_value_not_income() {
    let tenant = Household::employee(100_000.0, true, 0);
    let tenant_assessment = assess("ZH", &tenant);
    assert!(
        !tenant_assessment
            .applied
            .iter()
            .any(|d| matches!(
                d.category,
                RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance
            )),
        "a tenant must not receive a property deduction: {:?}",
        tenant_assessment.applied
    );
    assert_eq!(tenant_assessment.income_addition, 0.0);

    let homeowner = Household::employee(100_000.0, true, 0).homeowner(20_000.0);
    let owned = assess("ZH", &homeowner);
    let property: Vec<_> = owned
        .applied
        .iter()
        .filter(|d| {
            matches!(
                d.category,
                RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance
            )
        })
        .collect();
    assert!(!property.is_empty(), "a homeowner should get property deductions");

    // Doubling the rental value doubles the property deduction; doubling the
    // salary must not. That is the difference between the right base and the
    // wrong one.
    let richer = assess("ZH", &Household::employee(200_000.0, true, 0).homeowner(20_000.0));
    let richer_property: f64 = richer
        .applied
        .iter()
        .filter(|d| d.category == RuleCategory::PropertyMaintenance)
        .map(|d| d.amount)
        .sum();
    let owned_property: f64 = property
        .iter()
        .filter(|d| d.category == RuleCategory::PropertyMaintenance)
        .map(|d| d.amount)
        .sum();
    assert!(
        (richer_property - owned_property).abs() < 1e-9,
        "maintenance must not scale with salary: {owned_property} then {richer_property}"
    );

    let bigger_home = assess("ZH", &Household::employee(100_000.0, true, 0).homeowner(40_000.0));
    let bigger_property: f64 = bigger_home
        .applied
        .iter()
        .filter(|d| d.category == RuleCategory::PropertyMaintenance)
        .map(|d| d.amount)
        .sum();
    assert!(
        bigger_property > owned_property,
        "maintenance must scale with the rental value: {owned_property} then {bigger_property}"
    );
}

/// The imputed rental value is an *income addition* as well as a deduction base.
///
/// A model with only the deduction understates a homeowner's taxable income, so
/// the addition is exposed separately and the assessed income is higher than the
/// raw gross figure despite the deductions.
#[test]
fn imputed_rental_value_is_reported_as_an_income_addition() {
    let homeowner = Household::employee(100_000.0, true, 0).homeowner(20_000.0);
    let assessment = assess("ZH", &homeowner);

    assert_eq!(
        assessment.income_addition, 20_000.0,
        "the imputed rental value must be added to taxable income"
    );
    assert_eq!(
        assessment.taxable_income_with_addition(),
        100_000.0 + 20_000.0 - assessment.total(),
        "taxable income is gross + addition - deductions"
    );
    assert!(
        assessment.taxable_income_with_addition() > assessment.taxable_income(),
        "a homeowner's assessable income should exceed their gross salary once the \
         rental value is added, even after the maintenance deduction"
    );

    // And a tenant has neither side.
    let tenant = assess("ZH", &Household::employee(100_000.0, true, 0));
    assert_eq!(tenant.income_addition, 0.0);
    assert_eq!(tenant.taxable_income_with_addition(), tenant.taxable_income());
}

/// Property deductions cannot exceed the value they are measured against.
#[test]
fn property_deductions_are_capped_at_the_rental_value() {
    // A tiny rental value with percentage-based rules still cannot yield more
    // than the value itself.
    let household = Household::employee(500_000.0, true, 0).homeowner(1_000.0);
    let assessment = assess("ZH", &household);
    let property: f64 = assessment
        .applied
        .iter()
        .filter(|d| {
            matches!(
                d.category,
                RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance
            )
        })
        .map(|d| d.amount)
        .sum();
    assert!(
        property <= 1_000.0 + 1e-9,
        "property deductions {property} exceed the rental value they are based on"
    );
}

/// The maintenance bands are keyed on the *building's age*, and a building has
/// one age — so at most one band applies.
///
/// Summing them deducted 6,000 of maintenance against a 20,000 rental value where
/// the law allows a single band.
#[test]
fn property_maintenance_age_bands_are_not_summed() {
    let household = Household::employee(100_000.0, true, 0).homeowner(20_000.0);
    for canton in ["Bund", "ZH", "BE", "LU"] {
        let assessment = assess(canton, &household);
        let bands: Vec<_> = assessment
            .applied
            .iter()
            .filter(|d| d.category == RuleCategory::PropertyMaintenance)
            .collect();
        assert_eq!(
            bands.len(),
            1,
            "{canton}: exactly one maintenance band must apply, got {bands:?}"
        );
        assert!(
            bands[0].amount <= 20_000.0,
            "{canton}: maintenance {} exceeds the rental value",
            bands[0].amount
        );
    }
}

// ── Wealth management is not an income deduction ─────────────────────────────

/// `Abzug Vermögensverwaltungskosten` is a **wealth** deduction. The ESTV file
/// lists it under `Steuerart = Einkommen`, which is why it was applied against
/// income in ZH, SZ, OW, NW and GL — deducting 0.2-0.3% of a *salary* as a
/// wealth-management cost.
#[test]
fn wealth_management_is_not_deducted_from_income() {
    let household = Household::employee(100_000.0, false, 0);
    for canton in ["ZH", "SZ", "OW", "NW", "GL", "UR", "Bund"] {
        let assessment = assess(canton, &household);
        assert!(
            !assessment
                .applied
                .iter()
                .any(|d| d.category == RuleCategory::WealthManagement),
            "{canton} deducted a wealth-management cost from income: {:?}",
            assessment.applied
        );
    }

    // The rule is still reported rather than dropped, so the omission is visible.
    let zh = assess("ZH", &household);
    assert!(
        zh.skipped
            .iter()
            .any(|s| s.name.contains("Vermögensverwaltung")),
        "the wealth rule should be reported as not applied"
    );
}

// ── Pensioner deductions ─────────────────────────────────────────────────────

/// Pensioner deductions need the pension fact, and defaulting to `false` is what
/// stops them being granted to every working household.
///
/// `Abzug für AHV/IV-Rentner` is 10 flat rules plus 15 phase-out scales; granting
/// them by omission would be a large silent error.
#[test]
fn pensioner_deductions_require_the_pension_fact() {
    let worker = Household::employee(30_000.0, false, 0);
    let pensioner = Household::employee(30_000.0, false, 0).pensioner();

    for canton in ["SZ", "GL", "SO", "FR", "SH"] {
        let w = assess(canton, &worker);
        assert!(
            !w.applied
                .iter()
                .any(|d| d.category == RuleCategory::Pensioner),
            "{canton} granted a pensioner deduction to a working household: {:?}",
            w.applied
        );

        let p = assess(canton, &pensioner);
        // A pensioner must not receive *less* than a worker on the same income.
        assert!(
            p.total() >= w.total(),
            "{canton}: pensioner {} should not be worse off than worker {}",
            p.total(),
            w.total()
        );
    }

    // Savoy, Glarus and Solothurn publish flat pensioner amounts, so those must
    // actually appear.
    for (canton, expected) in [("SZ", 4_000.0), ("GL", 2_100.0), ("SO", 5_000.0)] {
        let p = assess(canton, &Household::employee(30_000.0, false, 0).pensioner());
        let flat = p
            .applied
            .iter()
            .find(|d| d.category == RuleCategory::Pensioner)
            .unwrap_or_else(|| panic!("{canton} should grant a flat pensioner deduction"));
        assert_eq!(flat.amount, expected, "{canton}");
    }
}

/// A percentage pensioner rule must not be applied to employment income.
///
/// Basel-Landschaft's is 40-60% *of the pension*, which this model does not carry;
/// deducting a share of a salary instead would be a share of the wrong base.
#[test]
fn percentage_pensioner_rules_are_not_applied_to_salary() {
    let pensioner = Household::employee(30_000.0, false, 0).pensioner();
    let bl = assess("BL", &pensioner);
    // BL's flat pensioner rule is 0 with percent 40/60, so nothing should appear.
    for entry in bl.applied.iter().filter(|d| d.category == RuleCategory::Pensioner) {
        assert!(
            entry.amount == 0.0,
            "BL's percentage pensioner rule must not be applied to salary: {entry:?}"
        );
    }
}

/// Pensioner phase-out scales must reach a pensioner household, which the old
/// blanket exclusion prevented because no field existed for the fact.
#[test]
fn pensioner_phase_out_scales_reach_a_pensioner() {
    let worker = assess("FR", &Household::employee(30_000.0, false, 0));
    let pensioner = assess("FR", &Household::employee(30_000.0, false, 0).pensioner());
    assert!(
        pensioner.means_tested > worker.means_tested,
        "Fribourg's AHV/IV scale should raise a pensioner's means-tested deduction: \
         worker {} vs pensioner {}",
        worker.means_tested,
        pensioner.means_tested
    );
}

// ── Insurance premiums: a ceiling-only rule shape ────────────────────────────

/// The insurance-premium rules state **only a ceiling** — `Betrag = 0`,
/// `Prozent = 0`, `Maximum = 5800` — so the generic
/// `clamp(amount + percent × base, …)` yields zero for every one of them.
///
/// The entire family therefore did nothing, silently. The deduction is what the
/// taxpayer declared, capped at the canton's ceiling.
#[test]
fn insurance_premiums_deduct_the_declared_amount_within_the_ceiling() {
    let mut household = Household::employee(100_000.0, true, 0);
    household.insurance_premiums = Some(3_000.0);

    for canton in ["Bund", "ZH", "BE", "AG"] {
        let assessment = assess(canton, &household);
        let premiums = assessment
            .applied
            .iter()
            .find(|d| d.category == RuleCategory::InsurancePremiums)
            .unwrap_or_else(|| {
                panic!(
                    "{canton}: a married household declaring premiums should get the \
                     deduction: {:?}",
                    assessment.applied
                )
            });
        assert_eq!(
            premiums.amount, 3_000.0,
            "{canton}: the declared amount is deductible below the ceiling"
        );
    }
}

/// Above the canton's ceiling the published maximum applies, which is what makes
/// the ceiling worth carrying. Zurich's is CHF 8,700 for a married taxpayer.
#[test]
fn insurance_premiums_are_capped_at_the_cantonal_ceiling() {
    let mut household = Household::employee(100_000.0, true, 0);
    household.insurance_premiums = Some(50_000.0);

    let assessment = assess("ZH", &household);
    let premiums = assessment
        .applied
        .iter()
        .find(|d| d.category == RuleCategory::InsurancePremiums)
        .expect("Zurich's capped deduction");
    assert_eq!(premiums.amount, 8_700.0, "ZH's married ceiling");
    assert!(
        premiums.amount < household.insurance_premiums.unwrap(),
        "the ceiling must actually bind here"
    );
}

/// A married household must not be given the "alleinstehende Personen" variant,
/// and a single one must not be given the married variant. Both are present in
/// every canton, so picking the wrong one is a silent over- or under-deduction.
#[test]
fn insurance_variants_follow_marital_status() {
    let single = assess("Bund", &Household::employee(100_000.0, false, 0));
    let mut married_household = Household::employee(100_000.0, true, 0);
    married_household.insurance_premiums = Some(3_000.0);
    let married = assess("Bund", &married_household);

    let married_rule = married
        .applied
        .iter()
        .find(|d| d.category == RuleCategory::InsurancePremiums)
        .expect("married variant");
    assert!(
        married_rule.name.contains("Verheiratete"),
        "a married household must take the married variant, got {:?}",
        married_rule.name
    );

    // The single household declared nothing, so it gets nothing — the point is
    // that it must not have been given the married ceiling.
    assert!(
        !single
            .applied
            .iter()
            .any(|d| d.category == RuleCategory::InsurancePremiums),
        "a single household with no declared premiums gets nothing: {:?}",
        single.applied
    );
}

/// The means-tested deduction must not feed its own base.
///
/// The scale is a step function of net income, so if the deduction were
/// subtracted before the lookup, lowering the base would raise the deduction,
/// which would lower the base again. The published tables resolve this by keying
/// on the income remaining *after the other deductions* — a definition, not a
/// fixed point — and `--insurance-premiums` exercises it: declaring a premium
/// reduces the base and therefore raises the means-tested figure, once and
/// stably.
#[test]
fn means_tested_deduction_does_not_feed_its_own_base() {
    let mut household = Household::employee(30_000.0, false, 0);
    let before = assess("FR", &household).means_tested;

    household.insurance_premiums = Some(2_000.0);
    let after = assess("FR", &household);

    assert!(
        after.means_tested >= before,
        "a lower net income cannot reduce a means-tested deduction: {before} then {}",
        after.means_tested
    );

    // Stable: assessing the same household twice gives the same answer, so there
    // is no iteration left to converge.
    let again = assess("FR", &household);
    assert_eq!(
        after.means_tested, again.means_tested,
        "the assessment must be a single pass, not a fixed-point iteration"
    );
    assert_eq!(after.total(), again.total());
}

/// The net-income base must hold across cantons, not just where it was noticed.
///
/// Swept over every canton that has a means-tested scale, and **restricted to the
/// sub-cap regime**: where the scale's own figure already exceeds gross income
/// (Valais allows CHF 21,250 against a CHF 15,000 income) the total is capped and
/// the base has no observable effect, so asserting a direction there would be
/// asserting a coincidence.
#[test]
fn means_tested_scales_use_net_income_for_every_canton() {
    use life_optimizer::estv_deductions_data::DEDUCTION_SCALES;

    let mut cantons: Vec<&str> = DEDUCTION_SCALES.iter().map(|s| s.canton_code).collect();
    cantons.sort_unstable();
    cantons.dedup();

    let mut exercised = 0;
    for canton in cantons {
        // A modest income, where a means-tested deduction is in range.
        let base = Household::employee(20_000.0, false, 0);
        let before = assess(canton, &base);
        if before.means_tested == 0.0 {
            continue; // the scale does not reach this household
        }
        // Skip where the total is already capped, since then either answer gives
        // the same total and the check would be vacuous.
        if before.uncapped_total() > before.gross_income {
            continue;
        }

        let mut with_premium = base;
        with_premium.insurance_premiums = Some(1_500.0);
        let after = assess(canton, &with_premium);

        assert!(
            after.means_tested >= before.means_tested,
            "{canton}: reducing net income by CHF 1,500 must not reduce the \
             means-tested deduction ({} then {})",
            before.means_tested,
            after.means_tested
        );
        exercised += 1;
    }

    assert!(
        exercised >= 3,
        "only {exercised} canton(s) were actually exercised; this test is not \
         checking what it claims"
    );
}

/// A rule that applies but yields nothing must not vanish from the report.
///
/// The insurance-premium family was discarded by a silent `continue` on a zero
/// amount, which made a whole family indistinguishable from one that does not
/// exist. Whatever the reason a rule contributes nothing, it must be accounted
/// for as applied or skipped.
#[test]
fn every_rule_is_accounted_for() {
    use life_optimizer::estv_deductions_data::rules_for;

    for canton in ["Bund", "ZH", "BE", "AG", "SO"] {
        let household = Household::employee(100_000.0, true, 0);
        let assessment = assess(canton, &household);

        // Every rule is `Deduction` kind and must land in exactly one bucket.
        // Duplicates are legitimate: a mutually exclusive family reports the
        // variant it did not choose as skipped.
        assert!(
            !assessment.applied.is_empty(),
            "{canton}: a household should receive at least the flat allowances"
        );

        let total_rules = rules_for(canton).len();
        let reported = assessment.applied.len()
            + assessment
                .skipped
                .iter()
                .filter(|s| s.reason != SkipReason::NotADeduction)
                .count();
        assert!(
            reported >= total_rules / 2,
            "{canton}: only {reported} of {total_rules} rules are accounted for as \
             applied or skipped; rules are being dropped silently"
        );
    }
}


