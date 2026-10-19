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

//! The deduction tables, checked against a snapshot derived from the ESTV workbooks.
//!
//! `VALIDATION.md` §5 recorded that the deduction engine was validated against
//! nothing: `src/estv_deductions_data.rs` is generated from
//! `estv_deductions.xlsx` and `estv_deduction_scales.xlsx`, and until this test
//! existed only its own generator had ever read it. Every deduction the model
//! computes for a Swiss household traced back to a translation step nobody had
//! independently repeated.
//!
//! `tools/verify_estv_deductions_import.py` is the independent repetition. It
//! reads both workbooks with openpyxl, applies the merge the importer documents
//! for rules split across two rows, derives each row's kind from its label, and
//! writes `source-documents/estv_deductions_expected.csv`. The snapshot is
//! written from the *workbook*, never from the generated file, so this test
//! compares the two derivations rather than the file against itself.
//!
//! # What the snapshot covers, and what it does not
//!
//! 617 rules and 22 means-tested scales. The scales workbook also carries 14
//! wealth (`Vermögen`) rows for two Fribourg abatements, which the importer
//! keeps out of the income model by design; they are excluded and counted on
//! the verifier's report rather than silently dropped.
//!
//! # The snapshot is a snapshot
//!
//! Like the tax-tariff snapshot, this one is checked into the repository so CI
//! can run the comparison without Python or the spreadsheets. If the workbooks
//! are re-exported for a later tax year, regenerate it with
//! `python tools/verify_estv_deductions_import.py --emit` and re-import.

use life_optimizer::estv_deductions_data::{
    DeductionRule, RuleKind, DEDUCTION_RULES, DEDUCTION_SCALES,
};

const SNAPSHOT: &str = "source-documents/estv_deductions_expected.csv";

/// ESTV labels differ from the imported names only in runs of whitespace.
///
/// Several labels carry a double space in the source (`Verheiratete mit
/// Beiträgen Säule 2/3a`); the importer collapses them. Comparing with this
/// normalisation on both sides keeps the one documented normalisation from
/// masking the numbers, which are compared exactly.
fn normalise(label: &str) -> String {
    label.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn kind_from(name: &str) -> RuleKind {
    match name {
        "Deduction" => RuleKind::Deduction,
        "Threshold" => RuleKind::Threshold,
        "Other" => RuleKind::Other,
        other => panic!("unknown RuleKind {other:?} in the snapshot"),
    }
}

struct SnapshotRule {
    canton: String,
    amount: f64,
    percent: f64,
    minimum: f64,
    maximum: f64,
    kind: RuleKind,
    label: String,
}

fn snapshot() -> (Vec<SnapshotRule>, Vec<Vec<String>>) {
    let text = std::fs::read_to_string(SNAPSHOT).unwrap_or_else(|error| {
        panic!(
            "cannot read {SNAPSHOT}: {error}. Re-run \
             `python tools/verify_estv_deductions_import.py --emit`."
        )
    });

    let mut rules = Vec::new();
    let mut scales = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        // The label is last and may contain commas, so the split is bounded and
        // the final field absorbs the rest of the line.
        if line.starts_with("rule,") {
            let fields: Vec<&str> = line.splitn(8, ',').collect();
            assert_eq!(fields.len(), 8, "malformed rule row: {line}");
            rules.push(SnapshotRule {
                canton: fields[1].to_string(),
                amount: fields[2].parse().expect("amount"),
                percent: fields[3].parse().expect("percent"),
                minimum: fields[4].parse().expect("minimum"),
                maximum: fields[5].parse().expect("maximum"),
                kind: kind_from(fields[6]),
                label: fields[7].to_string(),
            });
        } else if line.starts_with("scale,") {
            let fields: Vec<&str> = line.splitn(5, ',').collect();
            assert_eq!(fields.len(), 5, "malformed scale row: {line}");
            scales.push(fields.iter().map(|field| field.to_string()).collect());
        } else {
            panic!("unrecognised snapshot row: {line}");
        }
    }
    (rules, scales)
}

#[test]
fn the_imported_deduction_rules_match_the_estv_workbooks() {
    let (rules, _scales) = snapshot();

    let mut compared = 0;
    let mut matched = vec![false; DEDUCTION_RULES.len()];

    for row in &rules {
        let wanted = normalise(&row.label);
        let mut found = None;
        for (index, rule) in DEDUCTION_RULES.iter().enumerate() {
            if rule.canton_code == row.canton && normalise(rule.name) == wanted {
                found = Some((index, rule));
                break;
            }
        }
        let Some((index, rule)) = found else {
            panic!(
                "{} / {:?} is in the workbook but not in the imported tables",
                row.canton, row.label
            );
        };
        matched[index] = true;

        assert_eq!(
            rule.kind, row.kind,
            "{} / {:?}: kind differs from the one the label implies",
            row.canton, row.label
        );
        for (field, actual, expected) in [
            ("amount", rule.amount, row.amount),
            ("percent", rule.percent, row.percent),
            ("minimum", rule.minimum, row.minimum),
            ("maximum", rule.maximum, row.maximum),
        ] {
            assert!(
                (actual - expected).abs() < 1e-9,
                "{} / {:?}: {field} is {actual} in the import, {expected} in the workbook",
                row.canton,
                row.label
            );
        }
        compared += 1;
    }

    assert_eq!(rules.len(), 617, "the snapshot has {} rules", rules.len());
    assert_eq!(
        compared, 617,
        "expected 617 rules to be compared, got {compared}"
    );
    let unmatched: Vec<&str> = DEDUCTION_RULES
        .iter()
        .zip(matched.iter())
        .filter(|(_, seen)| !**seen)
        .map(|(rule, _)| rule.name)
        .collect();
    assert!(
        unmatched.is_empty(),
        "{} imported rule(s) are not in the workbook: {unmatched:?}",
        unmatched.len()
    );
}

#[test]
fn the_imported_means_tested_scales_match_the_estv_workbooks() {
    let (_rules, scales) = snapshot();
    assert_eq!(scales.len(), 22, "the snapshot has {} scales", scales.len());

    let mut compared = 0;
    for row in &scales {
        let canton = &row[1];
        let authority = &row[2];
        let label = normalise(&row[4]);
        let expected: Vec<(f64, f64)> = row[3]
            .split(';')
            .filter(|point| !point.is_empty())
            .map(|point| {
                let (threshold, amount) = point.split_once(':').expect("threshold:amount");
                (
                    threshold.parse().expect("threshold"),
                    amount.parse().expect("amount"),
                )
            })
            .collect();

        let scale = DEDUCTION_SCALES
            .iter()
            .find(|scale| {
                scale.canton_code == canton
                    && scale.authority == authority
                    && normalise(scale.name) == label
            })
            .unwrap_or_else(|| panic!("{canton} / {label:?} is in the workbook but not imported"));

        assert_eq!(
            scale.points.len(),
            expected.len(),
            "{canton} / {label:?}: {} points in the import, {} in the workbook",
            scale.points.len(),
            expected.len()
        );
        for (index, (point, (threshold, amount))) in scale.points.iter().zip(expected.iter()).enumerate()
        {
            assert!(
                (point.threshold - threshold).abs() < 1e-9,
                "{canton} / {label:?} point {index}: threshold {} in the import, {threshold} in the workbook",
                point.threshold
            );
            assert!(
                (point.amount - amount).abs() < 1e-9,
                "{canton} / {label:?} point {index}: amount {} in the import, {amount} in the workbook",
                point.amount
            );
        }
        compared += 1;
    }
    assert_eq!(
        compared, 22,
        "expected 22 scales to be compared, got {compared}"
    );
}

/// The formula the engine applies, checked against hand-computed workbook cases.
///
/// The tests above prove the *numbers* arrived intact. They do not prove the
/// engine turns those numbers into the right amount, because a correct table
/// read through a wrong formula is still wrong. Each case below is one the
/// workbooks actually contain and the arithmetic is written out by hand.
fn rule(canton: &str, label: &str) -> &'static DeductionRule {
    DEDUCTION_RULES
        .iter()
        .find(|rule| rule.canton_code == canton && normalise(rule.name) == normalise(label))
        .unwrap_or_else(|| panic!("no rule for {canton} / {label:?}"))
}

#[test]
fn the_engine_applies_the_workbook_formula() {
    // Appenzell Ausserrhoden, "Pauschalabzug übrige Berufskosten":
    // Betrag 700, 10%, no floor, ceiling 2400. So 700 + 10% of the base,
    // capped: 700 at a zero base, 1700 on 10,000, and the cap on 50,000
    // (700 + 5000 = 5700, cut to 2400).
    let ar = rule("AR", "Pauschalabzug übrige Berufskosten");
    assert_eq!(ar.apply(0.0), 700.0);
    assert_eq!(ar.apply(10_000.0), 1_700.0);
    assert_eq!(ar.apply(50_000.0), 2_400.0);

    // Aargau, same label, a different rule: 3% with a *floor* of 2000 and a
    // ceiling of 4000. On 10,000 the raw 300 is lifted to the floor; on
    // 200,000 the raw 6000 is cut to the ceiling. This is the case that
    // separates a floor applied before a ceiling from one applied after.
    let ag = rule("AG", "Pauschalabzug übrige Berufskosten");
    assert_eq!(ag.apply(10_000.0), 2_000.0);
    assert_eq!(ag.apply(100_000.0), 3_000.0);
    assert_eq!(ag.apply(200_000.0), 4_000.0);

    // A rule whose `maximum` is zero. The source states no ceiling for these,
    // so the deduction must stay proportional instead of being cut to zero --
    // the sentinel that a plausible implementation gets backwards.
    let uncapped = rule(
        "AG",
        "Pauschalabzug Unterhaltskosten Liegenschaften mit Alter bis 10 Jahren",
    );
    assert_eq!(uncapped.maximum, 0.0);
    assert_eq!(uncapped.apply(500_000.0), 50_000.0);

    // Vaud's property-maintenance rule, which the export splits across two
    // rows: one carrying 10% and one carrying a ceiling of 15,000. The merged
    // rule is the conjunction, so 10% of the base up to 15,000.
    let vd = rule(
        "VD",
        "Pauschalabzug Unterhaltskosten von vermieteten Liegenschaften mit Alter bis 20 Jahren",
    );
    assert_eq!(vd.amount, 0.0);
    assert_eq!(vd.percent, 10.0);
    assert_eq!(vd.maximum, 15_000.0);
    assert_eq!(vd.apply(100_000.0), 10_000.0);
    assert_eq!(vd.apply(200_000.0), 15_000.0);

    // A deduction that is a bare ceiling carries no arithmetic of its own:
    // Aargau's childcare deduction is 0 plus 0% capped at 25,000, so the
    // engine returns zero for every base. That is what the workbook says, not
    // a defect in the lookup -- the ceiling applies to costs the taxpayer
    // declares, which this rule does not model.
    let childcare = rule("AG", "Abzug Kinderdrittbetreuungskosten");
    assert_eq!(childcare.apply(0.0), 0.0);
    assert_eq!(childcare.apply(80_000.0), 0.0);
}
