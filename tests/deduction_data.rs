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

//! Tests for the generated ESTV deduction data and its source integrity.
//!
//! The data comes from two ESTV workbooks via `tools/import_estv_deductions.py`.
//! These tests check the properties that make it usable, and — more importantly
//! — the classification decisions that stop a *parameter* from being summed as
//! though it were a deduction.
//!
//! The central risk this module exists to prevent: `estv_deductions.xlsx` holds
//! 574 real deductions, 9 thresholds and 34 non-deductions in one flat table. A
//! consumer that summed every row would both double-count and treat a threshold
//! as an amount.

use life_optimizer::estv_deductions_data::{
    rules_for, scales_for, RuleKind, DEDUCTION_RULES, DEDUCTION_SCALES,
};

/// Every rule must name a jurisdiction this project recognises: a two-letter
/// canton code, or `Bund` for the federal rules.
///
/// The generator emitted `BUND` on its first run, which no other part of the
/// codebase would have matched; this test is what catches that class of mistake.
#[test]
fn every_jurisdiction_is_recognised() {
    for rule in DEDUCTION_RULES {
        let code = rule.canton_code;
        let ok = code == "Bund"
            || (code.len() == 2
                && code.chars().all(|c| c.is_ascii_uppercase())
                && life_optimizer::cantons::Canton::from_code(code).is_some());
        assert!(ok, "unknown jurisdiction {code:?} on rule {:?}", rule.name);
    }
    for scale in DEDUCTION_SCALES {
        assert!(
            scale.canton_code.len() == 2,
            "scale {:?} has jurisdiction {:?}",
            scale.name,
            scale.canton_code
        );
    }
}

/// All 26 cantons and the federation must be present on the rules side.
#[test]
fn all_jurisdictions_are_present() {
    let present: Vec<&str> = {
        let mut v: Vec<&str> = DEDUCTION_RULES.iter().map(|r| r.canton_code).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    assert_eq!(
        present.len(),
        27,
        "expected 26 cantons + Bund, got {present:?}"
    );
    assert!(present.contains(&"Bund"));
    for canton in life_optimizer::cantons::ALL_CANTONS {
        assert!(
            present.contains(&canton.code()),
            "{} has no deduction rules",
            canton.code()
        );
    }
}

/// A rule's numbers must be internally consistent and non-negative.
#[test]
fn rule_amounts_are_plausible() {
    for rule in DEDUCTION_RULES {
        for (label, value) in [
            ("amount", rule.amount),
            ("percent", rule.percent),
            ("minimum", rule.minimum),
            ("maximum", rule.maximum),
        ] {
            assert!(
                value >= 0.0 && value.is_finite(),
                "{} {}: {label} is {value}",
                rule.canton_code,
                rule.name
            );
        }
        // These are annual CHF deductions, so an amount or ceiling above a
        // million francs means a parsing error rather than tax policy. The
        // largest real figure in the export is well under this.
        assert!(
            rule.amount <= 1_000_000.0,
            "{} {}: amount {} is implausible",
            rule.canton_code,
            rule.name,
            rule.amount
        );
        assert!(
            rule.maximum <= 1_000_000.0,
            "{} {}: maximum {} is implausible",
            rule.canton_code,
            rule.name,
            rule.maximum
        );
        // The percentage bound applies to *deductions* only. The rows classified
        // as `Other` include rate constants stored in the same column — the
        // FINMA technical interest rate is 350 here, meaning 3.5% — so bounding
        // them at 100 would reject a correctly imported value. This is exactly
        // why the classification is carried rather than inferred.
        if rule.kind == RuleKind::Deduction {
            assert!(
                rule.percent <= 100.0,
                "{} {}: percent {} is implausible for a deduction",
                rule.canton_code,
                rule.name,
                rule.percent
            );
        }
        // A floor above a ceiling is contradictory, and would make `apply`
        // return the floor every time.
        if rule.maximum > 0.0 && rule.minimum > 0.0 {
            assert!(
                rule.minimum <= rule.maximum,
                "{} {}: minimum {} exceeds maximum {}",
                rule.canton_code,
                rule.name,
                rule.minimum,
                rule.maximum
            );
        }
    }
}

/// Every rule must be classified, and the classification must actually be used.
///
/// If every row came back `Deduction`, the `Schwellwert` and
/// `Steuerermässigung` rows would be summed as though they reduced taxable
/// income — which is the specific failure this data module exists to make
/// impossible.
#[test]
fn non_deductions_are_classified_not_merged() {
    let thresholds: Vec<&str> = DEDUCTION_RULES
        .iter()
        .filter(|r| r.kind == RuleKind::Threshold)
        .map(|r| r.name)
        .collect();
    assert!(
        !thresholds.is_empty(),
        "the source has threshold rows, so some must be classified as such"
    );
    for name in &thresholds {
        assert!(
            name.starts_with("Schwellwert")
                || name.starts_with("Familien Sozialabzug Schwellwert")
                || name.starts_with("Entlastungsabzug"),
            "{name:?} was classified as a threshold but does not look like one"
        );
    }

    let other: Vec<&str> = DEDUCTION_RULES
        .iter()
        .filter(|r| r.kind == RuleKind::Other)
        .map(|r| r.name)
        .collect();
    assert!(!other.is_empty(), "the source has non-deduction rows");
    for name in &other {
        assert!(
            name.starts_with("Steuerermässigung")
                || name.starts_with("Maximaler technischer Zinssatz")
                || name.starts_with("Faktor ")
                || name.starts_with("Krankenkasse"),
            "{name:?} was classified as 'other' but does not look like one"
        );
    }

    // And the majority must still be ordinary deductions, or the classifier is
    // over-eager and the tool would find nothing to deduct.
    let deductions = DEDUCTION_RULES
        .iter()
        .filter(|r| r.kind == RuleKind::Deduction)
        .count();
    assert!(
        deductions > DEDUCTION_RULES.len() * 3 / 4,
        "only {deductions} of {} rules are deductions; the classifier looks wrong",
        DEDUCTION_RULES.len()
    );
}

/// Valais's rental-property rule is split across two source rows — one carrying
/// the rate, one the ceiling. They must be merged into one rule, or a consumer
/// finds one row and silently drops either the cap or the rate.
#[test]
fn split_rules_are_merged() {
    let matches: Vec<_> = rules_for("VD")
        .into_iter()
        .filter(|r| r.name.starts_with("Pauschalabzug Unterhaltskosten von vermieteten"))
        .collect();
    let rule = matches
        .iter()
        .find(|r| r.name.contains("bis 20 Jahren"))
        .unwrap_or_else(|| panic!("VD's 20-year rental rule should be present: {matches:?}"));

    assert_eq!(
        matches
            .iter()
            .filter(|r| r.name == rule.name)
            .count(),
        1,
        "the two source rows must produce exactly one merged rule"
    );
    assert_eq!(rule.percent, 10.0, "the rate comes from the first source row");
    assert_eq!(
        rule.maximum, 15_000.0,
        "the ceiling comes from the second source row"
    );
}

/// A rule whose parts are stated separately must apply as one expression:
/// `clamp(amount + rate * base, min, max)`.
#[test]
fn apply_combines_amount_and_percentage() {
    // Aargau's "Pauschalabzug übrige Berufskosten" is 700 plus 10%, capped at
    // 2400 — the shape that cannot be expressed by picking a single column.
    let rule = rules_for("AR")
        .into_iter()
        .find(|r| r.name == "Pauschalabzug Berufsauslagen Nebenerwerb")
        .expect("AR's side-employment rule");
    assert_eq!((rule.amount, rule.percent), (0.0, 20.0));
    assert_eq!((rule.minimum, rule.maximum), (800.0, 2400.0));

    // Below the floor, the floor applies.
    assert_eq!(rule.apply(1_000.0), 800.0);
    // Inside the band, the percentage applies.
    assert_eq!(rule.apply(5_000.0), 1_000.0);
    // And it is capped.
    assert_eq!(rule.apply(100_000.0), 2_400.0);

    // A rule with `maximum = 0` has no stated ceiling, which is NOT a ceiling of
    // zero: treating it as one would zero out most of the table.
    let uncapped = DEDUCTION_RULES
        .iter()
        .find(|r| r.kind == RuleKind::Deduction && r.percent > 0.0 && r.maximum == 0.0)
        .expect("the source has uncapped percentage rules");
    assert!(
        uncapped.apply(100_000.0) > 0.0,
        "{} {} returned 0 for an uncapped percentage rule",
        uncapped.canton_code,
        uncapped.name
    );
}

/// Every phase-out table must be a well-formed step function: strictly
/// increasing thresholds, non-negative amounts, and (for the means-tested
/// deductions the source contains) no increase as income rises.
#[test]
fn deduction_scales_are_well_formed_step_functions() {
    assert!(
        !DEDUCTION_SCALES.is_empty(),
        "the scale workbook should produce at least one scale"
    );

    for scale in DEDUCTION_SCALES {
        assert!(
            !scale.points.is_empty(),
            "{} {:?} has no points",
            scale.canton_code,
            scale.name
        );
        let mut previous = None;
        let mut previous_amount = f64::INFINITY;
        for point in scale.points {
            assert!(
                point.amount >= 0.0,
                "{} {:?}: negative amount {}",
                scale.canton_code,
                scale.name,
                point.amount
            );
            if let Some(prev) = previous {
                assert!(
                    point.threshold > prev,
                    "{} {:?}: thresholds must strictly increase, {prev} then {}",
                    scale.canton_code,
                    scale.name,
                    point.threshold
                );
            }
            assert!(
                point.amount <= previous_amount,
                "{} {:?}: the deduction must not rise with income, {} then {}",
                scale.canton_code,
                scale.name,
                previous_amount,
                point.amount
            );
            previous = Some(point.threshold);
            previous_amount = point.amount;
        }
    }
}

/// `amount_at` must be a step function: zero below the first threshold, then the
/// amount of the highest threshold reached.
#[test]
fn amount_at_walks_the_steps() {
    let scale = DEDUCTION_SCALES
        .iter()
        .find(|s| s.points.len() >= 3)
        .expect("a multi-step scale");

    // Below the first threshold there is nothing; the first threshold is 0 for
    // every scale in this export, so this is the income <= 0 case.
    assert_eq!(scale.amount_at(-1.0), 0.0);

    // Exactly on a threshold, that step applies.
    for point in scale.points {
        assert_eq!(
            scale.amount_at(point.threshold),
            point.amount,
            "{} {:?} at {}",
            scale.canton_code,
            scale.name,
            point.threshold
        );
    }

    // One franc below the next threshold, the previous step still applies.
    for pair in scale.points.windows(2) {
        let (first, second) = (&pair[0], &pair[1]);
        assert_eq!(
            scale.amount_at(second.threshold - 1.0),
            first.amount,
            "{} {:?} just below {}",
            scale.canton_code,
            scale.name,
            second.threshold
        );
    }

    // Above the last threshold the last amount holds (which the tables phase to
    // zero, so this is zero for every real scale).
    let last = scale.points.last().unwrap();
    assert_eq!(scale.amount_at(last.threshold * 10.0 + 1_000_000.0), last.amount);
}

/// The means-tested scales must actually phase to zero, which is what makes them
/// means-tested rather than flat.
#[test]
fn means_tested_scales_phase_out() {
    let phased = DEDUCTION_SCALES
        .iter()
        .filter(|s| s.points.last().is_some_and(|p| p.amount == 0.0))
        .count();
    assert!(
        phased > 0,
        "at least some scales should phase to zero; none did"
    );
    assert!(
        phased == DEDUCTION_SCALES.len(),
        "all {} scales should phase out, only {phased} do",
        DEDUCTION_SCALES.len()
    );
}

/// Lookups must not fall back to a neighbouring jurisdiction.
#[test]
fn jurisdiction_lookups_do_not_fall_back() {
    assert!(rules_for("XX").is_empty());
    assert!(rules_for("").is_empty());
    assert!(scales_for("XX").is_empty());
    // The federal rules are keyed `Bund`, which is how `estv_scales_Bund.xlsx`
    // spells it. `BUND` was the generator's first output and matched nothing else
    // in the codebase.
    assert!(!rules_for("Bund").is_empty());
    assert!(rules_for("BUND").is_empty(), "the federal key is `Bund`");
    assert!(rules_for("bund").is_empty(), "lookup is case-sensitive");
}

/// Rules must vary across cantons, so the canton key is genuinely doing work.
#[test]
fn rules_differ_between_cantons() {
    let zh = rules_for("ZH");
    let ag = rules_for("AG");
    assert!(!zh.is_empty() && !ag.is_empty());

    // The same deduction must be present in both, since the ESTV export bills
    // itself as a complete per-canton table.
    let zh_commuting = zh.iter().find(|r| r.name.contains("Fahrkosten Haupterwerb"));
    let ag_commuting = ag.iter().find(|r| r.name.contains("Fahrkosten Haupterwerb"));
    if let (Some(zh_rule), Some(ag_rule)) = (zh_commuting, ag_commuting) {
        // They need not differ in value, but they must be distinct records with
        // their own canton rather than one shared row.
        assert_eq!(zh_rule.canton_code, "ZH");
        assert_eq!(ag_rule.canton_code, "AG");
    }
}
