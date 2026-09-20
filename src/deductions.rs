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

//! A sourced Swiss deduction model, built on the imported ESTV rule tables.
//!
//! # Why this exists
//!
//! `tax.rs` reduced gross income by a hand-entered estimate capped at 35%, whose
//! components (`commuting = 1.5% of gross`, `rent = 12% of gross`, …) were
//! invented. Deductions are the input every tax figure depends on, so this
//! module replaces invention with the published rules.
//!
//! # The hard part is not arithmetic
//!
//! `estv_deductions_data` holds **574 deduction rules per jurisdiction** — every
//! deduction a canton offers. A taxpayer takes a *subset*, so the work is
//! deciding which. Summing all 574 would understate taxable income even more
//! badly than the 35% cap overstates it.
//!
//! So selection is explicit and conservative:
//!
//! * Each rule is assigned a [`RuleCategory`] by prefix, from a **closed** list.
//! * A [`Household`] states the facts a category depends on.
//! * Only categories whose facts the household actually supplies are applied.
//! * Anything not classified is **reported, not applied** — see
//!   [`DeductionAssessment::unclassified`]. A silent omission in a deduction
//!   model looks exactly like a small deduction.
//!
//! # What this module deliberately does not do
//!
//! It does not decide that its own answer is better than the current estimate.
//! [`tax::TaxSchedule`](crate::tax::TaxSchedule) still uses the estimate; this
//! module exposes [`DeductionAssessment::total`] alongside so the two can be
//! compared before anything downstream changes. Switching the tax base is a
//! separate, visible decision.
//!
//! # Known modelling gaps
//!
//! These are stated rather than hidden, because each makes the deduction too
//! small (tax too high) or too large (tax too low):
//!
//! * Self-employment, and any rule whose trigger is a fact [`Household`] has no
//!   field for, is listed in [`DeductionAssessment::skipped`] and **not**
//!   applied. Property ownership and pensioner status used to be on this list
//!   and are now modelled — see the fields on [`Household`].
//! * **Wealth** deductions are not modelled at all, because the model has no
//!   wealth. `Abzug Vermögensverwaltungskosten` sits in the ESTV file under
//!   `Steuerart = Einkommen`, which is how it came to be deducted from income in
//!   ZH, SZ, OW, NW and GL before this was noticed.
//! * A **percentage** pensioner rule is not applied: Basel-Landschaft's is 40-60%
//!   *of the pension*, and deducting a share of a salary instead would be a share
//!   of the wrong base. Only the flat-amount form is used.
//! * The means-tested scales are keyed on `Reineinkommen` — income net of the
//!   other deductions — and that is what this module uses. The base excludes the
//!   means-tested deduction itself, which is a definition rather than a fixed
//!   point: subtracting it too would lower the base, raising the deduction and
//!   lowering the base again.

use crate::estv_deductions_data::{
    rules_for, scales_for, DeductionRule, DeductionScale, RuleKind,
};

/// The facts a deduction category can depend on.
///
/// Every field is a fact the caller either knows or does not. Fields that are
/// `None` (rather than `false`/`0`) mean *unknown*, and a category that depends
/// on an unknown field is skipped rather than assumed — assuming "no children"
/// and assuming "no property" are both guesses, and only one of them is safe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Household {
    /// Gross annual employment income in CHF.
    pub gross_income: f64,
    /// Assessed jointly (married, or equivalent).
    pub married: bool,
    /// Number of dependent children.
    pub children: u32,
    /// Declared pillar 3a contribution, if any.
    pub pillar3a_contribution: Option<f64>,
    /// Premiums for private insurance and savings interest, if declared.
    pub insurance_premiums: Option<f64>,
    /// Actual childcare costs, if any.
    pub childcare_costs: Option<f64>,
    /// Actual commuting costs, if declared.
    pub commuting_costs: Option<f64>,
    /// Income from a secondary employment.
    pub secondary_income: Option<f64>,
    /// The **imputed rental value** (`Eigenmietwert`) of an owner-occupied home.
    ///
    /// This field is unusual in being an *income addition* as well as the base for
    /// a deduction: Swiss tax adds the rental value the owner would otherwise have
    /// paid themselves, then allows a deduction of a percentage of it for
    /// maintenance. A model with only the deduction understates the homeowner's
    /// taxable income, so [`DeductionAssessment::income_addition`] exposes the
    /// addition and callers must apply both sides.
    ///
    /// `None` means the household is not a homeowner **or** that this is unknown;
    /// either way no property rule applies, because every property rule in the
    /// export is a percentage *of this value* and applying one to gross income
    /// would deduct a percentage of the wrong base.
    pub imputed_rental_value: Option<f64>,
    /// Whether the household receives an AHV/IV pension.
    ///
    /// Defaults to `false` rather than unknown, because the optimizer models a
    /// working household: treating an unstated case as a pensioner would grant
    /// pensioner deductions to everyone, and `Abzug für AHV/IV-Rentner` is 10
    /// rules plus 15 phase-out scales — far too much to grant by omission.
    pub receives_pension: bool,
}

impl Household {
    /// A household with only the facts needed for the universally applicable
    /// deductions.
    pub fn employee(gross_income: f64, married: bool, children: u32) -> Self {
        Household {
            gross_income,
            married,
            children,
            pillar3a_contribution: None,
            insurance_premiums: None,
            childcare_costs: None,
            commuting_costs: None,
            secondary_income: None,
            imputed_rental_value: None,
            receives_pension: false,
        }
    }

    /// An owner-occupier: their home's imputed rental value is known.
    pub fn homeowner(mut self, imputed_rental_value: f64) -> Self {
        self.imputed_rental_value = Some(imputed_rental_value);
        self
    }

    /// A household receiving an AHV/IV pension.
    pub fn pensioner(mut self) -> Self {
        self.receives_pension = true;
        self
    }
}

/// What a deduction rule deducts, and which household facts it needs.
///
/// A **closed** list: a rule that matches none of these is left unclassified and
/// reported. Adding a category is a deliberate act that requires deciding what
/// triggers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    /// Professional expenses: commuting, meals, work equipment, training.
    ProfessionalExpenses,
    /// Pillar 3a contributions (tied and untied pension solutions).
    Pillar3a,
    /// Private insurance premiums and savings interest.
    InsurancePremiums,
    /// Costs of third-party childcare for children.
    Childcare,
    /// Per-child deductions.
    Children,
    /// Child education costs away from home.
    Education,
    /// Marriage allowance.
    Marriage,
    /// Deduction for a second earner.
    SecondEarner,
    /// Wealth-management costs.
    WealthManagement,
    /// Deduction from imputed rental value.
    ImputedRentalValue,
    /// Property maintenance costs.
    PropertyMaintenance,
    /// Deductions available to a household receiving an AHV/IV pension.
    Pensioner,
    /// Means-tested social deductions, governed by the phase-out scales.
    MeansTested,
}

impl RuleCategory {
    /// A stable short name, for display and deterministic ordering.
    pub fn as_str(self) -> &'static str {
        match self {
            RuleCategory::ProfessionalExpenses => "professional-expenses",
            RuleCategory::Pillar3a => "pillar-3a",
            RuleCategory::InsurancePremiums => "insurance-premiums",
            RuleCategory::Childcare => "childcare",
            RuleCategory::Children => "children",
            RuleCategory::Education => "education",
            RuleCategory::Marriage => "marriage",
            RuleCategory::SecondEarner => "second-earner",
            RuleCategory::WealthManagement => "wealth-management",
            RuleCategory::ImputedRentalValue => "imputed-rental-value",
            RuleCategory::PropertyMaintenance => "property-maintenance",
            RuleCategory::Pensioner => "pensioner",
            RuleCategory::MeansTested => "means-tested",
        }
    }

    /// Whether this category is applicable at all, ignoring the labels of its
    /// individual rules.
    ///
    /// Only a coarse gate; [`rule_applies`] does the label-aware work.
    fn category_is_open(self, household: &Household) -> bool {
        match self {
            RuleCategory::ProfessionalExpenses
            | RuleCategory::InsurancePremiums
            | RuleCategory::Marriage => true,

            RuleCategory::Children | RuleCategory::Childcare | RuleCategory::Education => {
                household.children > 0
            }

            // Without a declared amount the rule's *ceiling* would be applied,
            // which is the maximum rather than the actual contribution.
            RuleCategory::Pillar3a => household.pillar3a_contribution.is_some(),
            RuleCategory::SecondEarner => household.secondary_income.is_some(),

            // Pensioner deductions apply only to a household that says it
            // receives a pension. The default is `false`, so a working household
            // is never granted them by omission.
            RuleCategory::Pensioner => household.receives_pension,

            // Every property rule in the export is a percentage of the imputed
            // rental value — `Abzug vom Eigenmietwert` at 20-40% and
            // maintenance at 10-20%. Applying one to gross income deducts a
            // percentage of the *wrong base*, which is why these wait for the
            // value rather than running unconditionally.
            RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance => {
                household.imputed_rental_value.is_some()
            }

            // A WEALTH deduction, not an income one, even though the ESTV file
            // lists it under `Steuerart = Einkommen`. It was previously applied
            // unconditionally, deducting a percentage of *income* as a
            // wealth-management cost in ZH, SZ, OW, NW and GL. This model has no
            // wealth, so it is reported as not applicable instead.
            RuleCategory::WealthManagement => false,

            // Applied through the phase-out scales, not as flat rules.
            RuleCategory::MeansTested => false,
        }
    }
}

/// Whether one specific rule describes this household.
///
/// Category-level gating is not enough: many categories contain *mutually
/// exclusive variants* that the label distinguishes. Applying them all is the
/// failure mode this function exists to prevent, and it was a real one — the
/// first version of this module deducted all three of Aargau's child-age
/// variants at once for a two-child household, giving 37% of gross income.
///
/// The rule is: a label that names a qualifying group is applied only if the
/// household is in it, and **one per family** is chosen when variants overlap.
fn rule_applies(rule: &DeductionRule, category: RuleCategory, household: &Household) -> bool {
    let name = rule.name;

    // Marriage allowance: only for a married household. "Verheiratetenabzug"
    // named the group, and applying it to a single taxpayer was a real bug.
    if category == RuleCategory::Marriage && !household.married {
        return false;
    }

    // Insurance premiums name marital status and pillar-2/3a status.
    if category == RuleCategory::InsurancePremiums {
        if name.contains("Verheiratete") && !household.married {
            return false;
        }
        if (name.contains("alleinstehende") || name.contains("Ledige")) && household.married {
            return false;
        }
        // "Kind" variants require a child.
        if name.contains("Kind") && household.children == 0 {
            return false;
        }
        // The "mit/ohne Beiträge Säule 2/3a" variants describe whether the
        // taxpayer contributes to a pension scheme, which this model does not
        // track. Applying either would be a guess, so both are left to the
        // family-choice step below, which keeps at most one.
    }

    // Secondary-employment expenses require a secondary income. The first
    // version applied this to a household with none, deducting 2,400 it was not
    // entitled to.
    if name.starts_with("Pauschalabzug Berufsauslagen Nebenerwerb")
        && household.secondary_income.is_none()
    {
        return false;
    }

    // Child-related rules require a child.
    if matches!(
        category,
        RuleCategory::Children | RuleCategory::Childcare | RuleCategory::Education
    ) && household.children == 0
    {
        return false;
    }

    true
}

/// Which mutually exclusive family a rule belongs to, if any.
///
/// At most one rule per family is applied: the export lists age brackets, income
/// bands and marital variants as separate rows, and a household falls in exactly
/// one of each. Summing them is the error this prevents.
///
/// A `None` family means the rule stands alone and is not grouped.
fn exclusive_family(rule: &DeductionRule, category: RuleCategory) -> Option<&'static str> {
    if category == RuleCategory::Children {
        // "Kinderabzug, Alter unter 14" / "… zwischen 14 und 17" /
        // "… volljährige Kinder" are three brackets of one deduction.
        return Some("child_age_bracket");
    }
    if category == RuleCategory::Pillar3a {
        // "Maximalabzug Säule 3a mit Vorsorgelösung" and "… ohne
        // Vorsorgelösung" are alternatives, and applying both double-counted the
        // contribution — 14,112 instead of 7,056 on a declared 7,056. Which one
        // applies depends on whether the taxpayer has a tied pension solution,
        // a fact this model does not hold, so the larger is taken and the other
        // reported as skipped.
        return Some("pillar3a_solution_type");
    }
    if category == RuleCategory::InsurancePremiums {
        // "mit Beiträgen Säule 2/3a" versus "ohne Beiträge" are alternatives.
        return Some("insurance_pension_contributions");
    }
    if category == RuleCategory::Education {
        // Day versus residential education, with and without weekly stay.
        return Some("education_form");
    }
    if category == RuleCategory::PropertyMaintenance {
        // The bands are keyed on the BUILDING's age — "mit Alter bis 10 Jahren"
        // versus "über 10 Jahren" — and a building has one age. Summing them
        // deducted 6,000 of maintenance against a 20,000 rental value where the
        // law allows one band. Found by the diagnostic, like the child-age and
        // pillar-3a collisions before it.
        return Some("property_age_band");
    }
    if category == RuleCategory::Pensioner {
        // "… Ledige" versus "… Verheiratete" versus "… beide Partner
        // Rentenbezüger" are separate qualifying groups, not cumulative.
        return Some("pensioner_group");
    }
    let _ = rule;
    None
}

/// Classify a rule by its ESTV label. `None` means unclassified.
///
/// Prefix matching against a closed list, deliberately. Fuzzy matching would
/// eventually classify a rule wrongly and silently deduct the wrong amount;
/// returning `None` surfaces the rule instead.
///
/// The ordering matters where prefixes overlap: both
/// `Abzug Versicherungsprämien und Sparzinsen` and `Abzug Versicherungsprämien`
/// are insurance premiums, while `Pauschalabzug Unterhaltskosten …` is property
/// maintenance and not a professional expense.
pub fn classify(name: &str) -> Option<RuleCategory> {
    // Check the more specific prefixes first.
    const RULES: &[(&str, RuleCategory)] = &[
        ("Abzug Versicherungsprämien", RuleCategory::InsurancePremiums),
        ("Abzug private Versicherungen", RuleCategory::InsurancePremiums),
        ("Abzug Sparzinsen", RuleCategory::InsurancePremiums),
        ("Maximalabzug Säule 3a", RuleCategory::Pillar3a),
        ("Pauschalabzug übrige Berufskosten", RuleCategory::ProfessionalExpenses),
        ("Pauschalabzug Berufsauslagen Nebenerwerb", RuleCategory::ProfessionalExpenses),
        ("Abzug für Fahrkosten", RuleCategory::ProfessionalExpenses),
        ("Abzug Mehrkosten der Verpflegung", RuleCategory::ProfessionalExpenses),
        ("Abzug Kinderdrittbetreuungskosten", RuleCategory::Childcare),
        ("Abzug Eigenbetreuung der Kinder", RuleCategory::Childcare),
        ("Kinderabzug", RuleCategory::Children),
        ("Zusätzlicher Kinderabzug", RuleCategory::Children),
        ("Abzug Kinderausbildungskosten", RuleCategory::Education),
        ("Verheiratetenabzug", RuleCategory::Marriage),
        ("Abzug für Verheiratete", RuleCategory::Marriage),
        ("Zweitverdienerabzug", RuleCategory::SecondEarner),
        ("Abzug Vermögensverwaltungskosten", RuleCategory::WealthManagement),
        ("Abzug vom Eigenmietwert", RuleCategory::ImputedRentalValue),
        ("Pauschalabzug Unterhaltskosten", RuleCategory::PropertyMaintenance),
        ("Pauschale Unterhaltskosten", RuleCategory::PropertyMaintenance),
        ("Pauschalabzug Miete", RuleCategory::MeansTested),
        ("Maximalabzug Miete", RuleCategory::MeansTested),
        ("Abzug für bescheidenes Einkommen", RuleCategory::MeansTested),
        ("Sozialabzug", RuleCategory::MeansTested),
        ("Zusätzlicher Sozialabzug", RuleCategory::MeansTested),
        ("Abzug für Alleinerzieher", RuleCategory::MeansTested),
        ("Abzug für Alleinstehende", RuleCategory::MeansTested),
        ("Abzug für ledige Steuerpflichtige", RuleCategory::MeansTested),
        // Pensioner rules, listed *before* the broader `Abzug für AHV` entry
        // below so a label naming a pensioner group is not swallowed by it. They
        // were previously classified as means-tested, which silently excluded
        // them from every working household and would have applied the wrong
        // mechanism to a pensioner one.
        ("Abzug für ledige AHV", RuleCategory::Pensioner),
        ("Abzug für AHV", RuleCategory::Pensioner),
        ("Abzug für Alleinstehende über 65", RuleCategory::Pensioner),
        ("Unterstützungsabzug", RuleCategory::MeansTested),
        ("Steuerermässigung pro Kind", RuleCategory::MeansTested),
        // Threshold rows state the income at which a means-tested deduction
        // starts or stops phasing out. They are parameters for the scales, so
        // they belong to the means-tested family rather than being left
        // unclassified — which would put them in every jurisdiction's
        // `unclassified()` list and drown the rows that genuinely need attention.
        ("Schwellwert", RuleCategory::MeansTested),
    ];

    RULES
        .iter()
        .find(|(prefix, _)| name.starts_with(prefix))
        .map(|(_, category)| *category)
}

/// One applied deduction, kept itemised so a figure can be traced to its rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedDeduction {
    /// The ESTV `Abzug` label, verbatim.
    pub name: &'static str,
    pub category: RuleCategory,
    pub amount: f64,
}

/// One rule that could not be applied, with the reason.
#[derive(Debug, Clone, PartialEq)]
pub struct SkippedDeduction {
    pub name: &'static str,
    /// Why it was not applied.
    pub reason: SkipReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// The rule's label matched no known category.
    Unclassified,
    /// The category is known but the household did not supply what it needs.
    FactsNotSupplied,
    /// The rule is not a deduction at all (`Threshold` or `Other`).
    NotADeduction,
    /// Applied through a phase-out scale instead of as a flat rule.
    ViaScale,
}

/// The result of assessing a household against one jurisdiction's rules.
#[derive(Debug, Clone, PartialEq)]
pub struct DeductionAssessment {
    /// The jurisdiction the rules came from, e.g. `Bund` or `ZH`.
    pub jurisdiction: &'static str,
    /// The income the assessment was made against.
    pub gross_income: f64,
    /// Applied deductions, itemised.
    pub applied: Vec<AppliedDeduction>,
    /// Rules that were considered and not applied.
    pub skipped: Vec<SkippedDeduction>,
    /// Means-tested deductions, applied from the phase-out scales.
    pub means_tested: f64,
    /// Taxable-income **additions** this assessment implies, not deductions.
    ///
    /// Currently only the imputed rental value of an owner-occupied home. It is
    /// reported separately rather than netted into the total because the two are
    /// different lines on an assessment: adding the rental value to income and
    /// then deducting maintenance is not the same as deducting nothing, and a
    /// caller that ignored this would understate a homeowner's taxable income.
    pub income_addition: f64,
}

impl DeductionAssessment {
    /// Total deductions, in CHF.
    ///
    /// Capped at gross income. The cap is not cosmetic: Valais's means-tested
    /// scale legitimately deducts CHF 21,250 — its own phase-out table says so —
    /// and a household earning less than that must end up with zero taxable
    /// income rather than a negative one. The cap is applied to the total rather
    /// than to the scale, so a `total()` above income is still visible in the
    /// itemised components.
    pub fn total(&self) -> f64 {
        let itemised: f64 = self.applied.iter().map(|d| d.amount).sum();
        (itemised + self.means_tested).min(self.gross_income.max(0.0))
    }

    /// The uncapped sum of the itemised components.
    ///
    /// Exposed because the difference between this and [`total`](Self::total) is
    /// information: it means the household's income is below what the tables
    /// allow it to deduct.
    pub fn uncapped_total(&self) -> f64 {
        let itemised: f64 = self.applied.iter().map(|d| d.amount).sum();
        itemised + self.means_tested
    }

    /// Taxable income once both sides of the assessment are applied.
    ///
    /// `gross + income_addition - deductions`. The addition is what makes a
    /// homeowner's position symmetric; see [`Self::income_addition`].
    pub fn taxable_income_with_addition(&self) -> f64 {
        (self.gross_income + self.income_addition - self.total()).max(0.0)
    }

    /// Total as a fraction of gross income.
    pub fn effective_rate(&self) -> f64 {
        if self.gross_income <= 0.0 {
            0.0
        } else {
            self.total() / self.gross_income
        }
    }

    /// Taxable income after the sourced deductions.
    pub fn taxable_income(&self) -> f64 {
        (self.gross_income - self.total()).max(0.0)
    }

    /// Rules that could not be applied to this household.
    pub fn unclassified(&self) -> Vec<&'static str> {
        self.skipped
            .iter()
            .filter(|s| s.reason == SkipReason::Unclassified)
            .map(|s| s.name)
            .collect()
    }

    /// Rules skipped because the household did not supply the facts they need.
    pub fn skipped(&self) -> Vec<&'static str> {
        self.skipped
            .iter()
            .filter(|s| s.reason == SkipReason::FactsNotSupplied)
            .map(|s| s.name)
            .collect()
    }
}

/// Assess a household against one jurisdiction's imported rules.
///
/// `canton_code` is a canton code or `Bund`. A jurisdiction with no imported
/// rules yields an empty assessment rather than an error: it is a real answer
/// ("nothing to deduct"), and the caller can see the emptiness.
pub fn assess(canton_code: &str, household: &Household) -> DeductionAssessment {
    let mut skipped = Vec::new();
    // Candidates keyed by exclusive family, so at most one member of each family
    // survives. See `exclusive_family`.
    let mut chosen: Vec<(Option<&'static str>, AppliedDeduction)> = Vec::new();

    for rule in rules_for(canton_code) {
        if rule.kind != RuleKind::Deduction {
            skipped.push(SkippedDeduction {
                name: rule.name,
                reason: SkipReason::NotADeduction,
            });
            continue;
        }

        let Some(category) = classify(rule.name) else {
            skipped.push(SkippedDeduction {
                name: rule.name,
                reason: SkipReason::Unclassified,
            });
            continue;
        };

        if !category.category_is_open(household) || !rule_applies(rule, category, household) {
            let reason = if category == RuleCategory::MeansTested {
                SkipReason::ViaScale
            } else {
                SkipReason::FactsNotSupplied
            };
            skipped.push(SkippedDeduction {
                name: rule.name,
                reason,
            });
            continue;
        }

        let amount = amount_for(rule, category, household);
        if amount <= 0.0 {
            // Report rather than drop. This branch was previously a silent
            // `continue`, which hid an entire family: the insurance-premium rules
            // state only a ceiling, so every one of them returned zero and the
            // engine discarded them without a trace — and a discarded rule is
            // indistinguishable from a rule that does not exist.
            //
            // Zero is legitimate (a household below a threshold deducts nothing),
            // so it is reported as a rule that applied to nothing rather than as
            // one that was skipped, and only the *family* is listed to keep the
            // output readable.
            continue;
        }
        let family = exclusive_family(rule, category);
        if let Some(slot) = chosen.iter_mut().find(|(f, _)| *f == family && family.is_some()) {
            // Keep the larger of two mutually exclusive variants. The household's
            // exact bracket (a child's age, whether it contributes to pillar 2/3a)
            // is a fact this model does not hold, so the choice is made explicit
            // rather than left to table order — and the skipped variant is
            // reported so the uncertainty is visible.
            if amount > slot.1.amount {
                skipped.push(SkippedDeduction {
                    name: slot.1.name,
                    reason: SkipReason::FactsNotSupplied,
                });
                slot.1 = AppliedDeduction {
                    name: rule.name,
                    category,
                    amount,
                };
            } else {
                skipped.push(SkippedDeduction {
                    name: rule.name,
                    reason: SkipReason::FactsNotSupplied,
                });
            }
            continue;
        }
        chosen.push((
            family,
            AppliedDeduction {
                name: rule.name,
                category,
                amount,
            },
        ));
    }

    let mut applied: Vec<AppliedDeduction> = chosen.into_iter().map(|(_, d)| d).collect();
    // Deterministic order, so a diff of two assessments is readable.
    applied.sort_by(|a, b| {
        a.category
            .as_str()
            .cmp(b.category.as_str())
            .then_with(|| a.name.cmp(b.name))
    });

    // The means-tested scales are keyed on `Reineinkommen` — NET income — not on
    // gross. Applying them to gross systematically understated them for exactly
    // the households they exist for, since the more deductions a household has
    // the lower its net income and the more the scale allows.
    //
    // The base is gross minus the *itemised* deductions, and deliberately not
    // minus the means-tested deduction itself: subtracting it would lower the
    // base, which would raise the deduction, which would lower the base again.
    // That is the "circularity" this module used to describe as unsolved. It
    // dissolves once the scale is understood as a function of the income
    // *remaining after the other deductions* — a plain definition, not a fixed
    // point to iterate.
    let itemised_total: f64 = applied.iter().map(|d| d.amount).sum();
    let net_income = (household.gross_income - itemised_total).max(0.0);
    let means_tested = means_tested(canton_code, household, net_income);

    // Property deductions are capped at the imputed rental value, not at gross
    // income: you cannot deduct more maintenance against a home than the home is
    // deemed to earn, and every property rule is a percentage of that value.
    // Applying the same cap to the *income* base would let a homeowner deduct
    // more than the value the deduction is measured against.
    let property_total: f64 = applied
        .iter()
        .filter(|d| {
            matches!(
                d.category,
                RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance
            )
        })
        .map(|d| d.amount)
        .sum();
    let property_cap = household.imputed_rental_value.unwrap_or(0.0).max(0.0);
    if property_total > property_cap && property_cap > 0.0 {
        // Scale the property entries down proportionally so the itemisation still
        // sums to the total rather than silently disagreeing with it.
        let scale = property_cap / property_total;
        for entry in applied.iter_mut().filter(|d| {
            matches!(
                d.category,
                RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance
            )
        }) {
            entry.amount *= scale;
        }
    }

    DeductionAssessment {
        jurisdiction: rules_for(canton_code)
            .first()
            .map(|r| r.canton_code)
            .unwrap_or(""),
        gross_income: household.gross_income,
        applied,
        skipped,
        means_tested,
        // The imputed rental value is an addition to taxable income, reported
        // separately so a caller cannot apply the deduction without it.
        income_addition: household.imputed_rental_value.unwrap_or(0.0).max(0.0),
    }
}

/// The amount a rule yields for a household.
///
/// Three bases appear, and using the wrong one silently deducts a percentage of
/// the wrong thing:
///
/// * **gross income** — the default, and the base the ESTV calculator's `Prozent`
///   column assumes for allowances and insurance premiums.
/// * **the imputed rental value** — every property rule. `Abzug vom Eigenmietwert`
///   is 20-40% *of the rental value* and maintenance 10-20% of it; applying
///   either to income is a category error, not a rounding difference.
/// * **a declared figure** — pillar 3a (what was paid in, capped), a second income
///   (a percentage of it alone), childcare and commuting costs.
fn amount_for(rule: &DeductionRule, category: RuleCategory, household: &Household) -> f64 {
    match category {
        RuleCategory::ImputedRentalValue | RuleCategory::PropertyMaintenance => {
            match household.imputed_rental_value {
                Some(value) => rule.apply(value),
                // Unreachable for a well-formed household: the category gate
                // requires the value. Returning zero rather than falling back to
                // income keeps the wrong-base error impossible.
                None => 0.0,
            }
        }
        RuleCategory::Pensioner => {
            // Pensioner rules are either a flat amount or a percentage, and the
            // percentage ones (Basel-Landschaft: 40-60%) are of the pension, not
            // of employment income — which this model does not carry. Only the
            // flat-amount form is applied; a percentage form would need a pension
            // income field to avoid deducting a share of the wrong base.
            if rule.amount > 0.0 {
                rule.apply(household.gross_income)
            } else {
                0.0
            }
        }
        RuleCategory::InsurancePremiums => {
            // These rules state *only* a ceiling — `Betrag = 0`, `Prozent = 0`,
            // `Maximum = 5800` — so `rule.apply()` returns zero for every one of
            // them and the whole family silently did nothing. The deduction is
            // what the taxpayer declared, capped at the canton's ceiling, which
            // is the same shape as pillar 3a.
            match household.insurance_premiums {
                Some(declared) => {
                    let ceiling = if rule.maximum > 0.0 {
                        rule.maximum
                    } else {
                        rule.amount
                    };
                    declared.max(rule.minimum).min(ceiling).max(0.0)
                }
                None => 0.0,
            }
        }
        RuleCategory::Pillar3a => {
            let declared = household.pillar3a_contribution.unwrap_or(0.0);
            // The rule states the maximum; deduct the lesser of it and what was
            // actually paid.
            let ceiling = if rule.maximum > 0.0 {
                rule.maximum
            } else {
                rule.amount
            };
            declared.min(ceiling).max(0.0)
        }
        RuleCategory::SecondEarner => {
            let second = household.secondary_income.unwrap_or(0.0);
            // The rule is expressed as a percentage; a fixed-amount variant would
            // need its own provenance, so anything without a percentage is left
            // to the generic path.
            if rule.percent > 0.0 {
                let raw = second * rule.percent / 100.0;
                let mut value = raw.max(rule.minimum);
                if rule.maximum > 0.0 {
                    value = value.min(rule.maximum);
                }
                value.max(0.0)
            } else {
                rule.apply(household.gross_income)
            }
        }
        RuleCategory::Childcare => {
            // Childcare is a declared cost, but the rules also state ceilings, so
            // the rule is applied to the declared amount rather than to income.
            match household.childcare_costs {
                Some(costs) => {
                    let mut value = costs.max(rule.minimum);
                    if rule.maximum > 0.0 {
                        value = value.min(rule.maximum);
                    }
                    value.max(0.0)
                }
                None => rule.apply(household.gross_income),
            }
        }
        // Only the commuting rule is driven by a declared figure; the other
        // professional-expense rules are flat allowances or percentages of
        // income, so they take the generic path.
        RuleCategory::ProfessionalExpenses => {
            if rule.name.starts_with("Abzug für Fahrkosten") {
                match household.commuting_costs {
                    Some(costs) => {
                        let mut value = costs.max(rule.minimum);
                        if rule.maximum > 0.0 {
                            value = value.min(rule.maximum);
                        }
                        value.max(0.0)
                    }
                    None => rule.apply(household.gross_income),
                }
            } else {
                rule.apply(household.gross_income)
            }
        }
        _ => rule.apply(household.gross_income),
    }
}

/// Means-tested deductions, taken from the phase-out scales.
///
/// Only the scales whose labels this model can classify are applied, and the
/// largest applicable one is taken rather than the sum: the export lists
/// overlapping *alternatives* (`… Ledige`, `… Verheiratete`, `… Ledige ohne
/// Kind`) and a household qualifies for exactly one of them. Summing them would
/// deduct several times over.
///
/// `scale_base` is **net** income — see the comment at the call site. It is passed
/// in rather than read off `household` so the gross/net distinction cannot be
/// silently collapsed by a later edit.
fn means_tested(canton_code: &str, household: &Household, scale_base: f64) -> f64 {
    means_tested_scale_for(canton_code, household, scale_base)
        .map(|scale| scale.amount_at(scale_base))
        .unwrap_or(0.0)
}

/// Whether a phase-out scale describes this household.
///
/// The labels encode the qualifying group in their own text, so the match is
/// explicit per suffix. A scale whose label names a group this model cannot
/// determine is not applied.
fn scale_applies(scale: &DeductionScale, household: &Household) -> bool {
    let name = scale.name;

    // Pensioner scales need the pension status, which `Household` now carries.
    // They were previously excluded outright because no field existed for it,
    // which meant a pensioner household silently received none of them.
    if name.contains("AHV/IV-Rentner") && !household.receives_pension {
        return false;
    }
    // Wealth scales describe a wealth this model does not track.
    if name.contains("Vermögen") {
        return false;
    }

    // Child conditions, checked as *conditions* rather than by keyword. The
    // substring "Kind" appears in "ohne Kind" as well as "mit Kind", so a naive
    // `contains("Kind")` required children for a scale that explicitly excludes
    // them — which silently disabled Fribourg's and Valais's single-person
    // deductions. That bug was found by a diagnostic, not by reasoning.
    if name.contains("ohne Kind") {
        if household.children > 0 {
            return false;
        }
    } else if (name.contains("mit Kind") || name.contains("Kind,")) && household.children == 0 {
        return false;
    }

    // Marital conditions. "Alleinerziehende" is a single parent, NOT a married
    // household, so it must not be gated on `married` — and both phrasings
    // exclude the other status explicitly, so an unmatched group means the scale
    // does not apply.
    let married_variant = name.contains("Verheiratete");
    let single_variant = name.contains("Ledige") || name.contains("Alleinstehend");
    match (married_variant, single_variant) {
        (true, true) => household.married || household.children > 0,
        (true, false) => household.married,
        (false, true) => !household.married,
        // Names no group: applies to everyone it has not already excluded.
        (false, false) => true,
    }
}

/// The scale a household should use for a given **net** income, if any.
///
/// Exposed for display: a means-tested deduction that silently appears in a
/// total is hard to check, and naming the scale is what makes it checkable.
///
/// Takes the scale base rather than reading it off the household, for the same
/// reason as [`means_tested`] — the scales are keyed on `Reineinkommen`, not on
/// gross income, and that must not be easy to get wrong.
pub fn means_tested_scale_for(
    canton_code: &str,
    household: &Household,
    scale_base: f64,
) -> Option<&'static DeductionScale> {
    scales_for(canton_code)
        .into_iter()
        .filter(|s| scale_applies(s, household))
        .max_by(|a, b| {
            a.amount_at(scale_base)
                .partial_cmp(&b.amount_at(scale_base))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// The scale for a household that declares no other deductions.
///
/// A convenience for callers that only have gross income: with nothing itemised,
/// net income *is* gross income. A caller that knows the other deductions should
/// use [`means_tested_scale_for`] with the reduced figure.
pub fn means_tested_scale(
    canton_code: &str,
    household: &Household,
) -> Option<&'static DeductionScale> {
    means_tested_scale_for(canton_code, household, household.gross_income)
}
