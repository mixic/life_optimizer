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

//! The cantonal scales, checked against a snapshot derived independently from the ESTV workbooks.
//!
//! `VALIDATION.md` §5 said the base scales were "not checked against anything outside the
//! repository". That was wrong about the sourcing and right about the checking:
//! `estv_scales_data.rs` is generated from the official Tarife workbooks in `source-documents/`,
//! but nothing re-derived the constants from them, so a generated file that only its own generator
//! had ever read was sourced and unverified at once.
//!
//! `tools/verify_estv_import.py` is the independent derivation. It reads the workbooks with
//! openpyxl, applies the tariff arithmetic itself, and writes `estv_scales_expected.csv`. This test
//! consumes that snapshot, so the comparison runs in CI without needing Python or the spreadsheets.
//!
//! # What the snapshot covers, and what it does not
//!
//! Forty-four cantonal ladders -- twenty-two cantons, single and married -- plus the two federal
//! ladders (`BUND_SINGLE`, `BUND_MARRIED`, which live outside `estv_scales_data.rs` in
//! `federal_tariff_data.rs` and are compared by the third test here), totalling 635 bracket rows.
//! Six cantonal ladders are absent by construction: Basel-Landschaft publishes algebraic formulae
//! rather than bands, and Obwalden and Uri carry no `Einkommen`/`Kanton` band rows in their
//! exports. Those are listed in `SNAPSHOT_EXEMPT` and asserted to be the *only* exemptions, so a
//! canton silently dropping out of the verification fails here.

use life_optimizer::cantons::{self, ALL_CANTONS};

const SNAPSHOT: &str = "source-documents/estv_scales_expected.csv";

/// Ladders the export cannot supply, each for a stated reason.
const SNAPSHOT_EXEMPT: [&str; 6] = [
    "BL_SINGLE",
    "BL_MARRIED", // formula tariff, published as expressions
    "OW_SINGLE",
    "OW_MARRIED", // no Einkommen/Kanton band rows in the export
    "UR_SINGLE",
    "UR_MARRIED", // same
];

/// The Bern table is not part of the ESTV import; `bern_city_default` carries it.
const IMPORT_EXEMPT: [&str; 1] = ["BE"];

struct Row {
    threshold: f64,
    rate: f64,
}

#[test]
fn the_generated_cantonal_scales_match_the_estv_workbooks() {
    let text = std::fs::read_to_string(SNAPSHOT).unwrap_or_else(|error| {
        panic!(
            "cannot read {SNAPSHOT}: {error}. Re-run \
             `python tools/verify_estv_import.py --emit`."
        )
    });

    let mut expected: std::collections::BTreeMap<String, Vec<Row>> = Default::default();
    for line in text.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields.len(), 4, "malformed snapshot row: {line}");
        let rows = expected.entry(fields[0].to_string()).or_default();
        rows.push(Row {
            threshold: fields[2].parse().expect("threshold"),
            rate: fields[3].parse().expect("rate"),
        });
    }
    assert!(
        expected.len() >= 46,
        "the snapshot only contains {} ladders, which suggests it was truncated",
        expected.len()
    );

    let mut compared = 0;
    for canton in ALL_CANTONS {
        if IMPORT_EXEMPT.contains(&canton.code()) {
            continue;
        }
        for married in [false, true] {
            let key = format!("{}_{}", canton.code(), if married { "MARRIED" } else { "SINGLE" });
            let Some(rows) = expected.get(&key) else {
                assert!(
                    SNAPSHOT_EXEMPT.contains(&key.as_str()),
                    "{key} is missing from the snapshot and is not a documented exemption"
                );
                continue;
            };

            let scale = cantons::imported_scale(*canton).unwrap_or_else(|| {
                panic!("{key} is in the snapshot but {} has no imported scale", canton.code())
            });
            let generated = scale.brackets(married);
            assert_eq!(
                generated.len(),
                rows.len(),
                "{key}: the import has {} brackets, the workbook has {}",
                generated.len(),
                rows.len()
            );
            for (index, (bracket, row)) in generated.iter().zip(rows.iter()).enumerate() {
                assert!(
                    (bracket.threshold - row.threshold).abs() < 1e-9,
                    "{key} bracket {index}: threshold {} in the import, {} in the workbook",
                    bracket.threshold,
                    row.threshold
                );
                assert!(
                    (bracket.rate - row.rate).abs() < 1e-6,
                    "{key} bracket {index}: rate {} in the import, {} in the workbook",
                    bracket.rate,
                    row.rate
                );
            }
            compared += 1;
        }
    }
    assert_eq!(compared, 44, "expected 44 ladders to be compared, got {compared}");
}

/// Every canton the tax engine prices must either come from the ESTV import or be the Bern table.
///
/// This is the check that stops the verification quietly covering less than it claims: a canton
/// added to the engine without an imported scale would be priced from somewhere unverified.
#[test]
fn every_priced_canton_is_covered_by_the_import_or_by_bern() {
    for canton in ALL_CANTONS {
        if !cantons::is_priceable(*canton) {
            continue;
        }
        if IMPORT_EXEMPT.contains(&canton.code()) {
            continue;
        }
        assert!(
            cantons::imported_scale(*canton).is_some(),
            "{} is priced but has no imported ESTV scale, so its bands are unverified",
            canton.code()
        );
    }
}

/// The federal tariff, re-derived from `estv_scales_Bund.xlsx` and compared row by row.
///
/// The federal constants were the least checked numbers in the repository: `federal_tariff_data.rs`
/// is generated from the workbook, and until this test only its own generator had read it. That
/// mattered, because the file's header records that it replaced a hand-entered table whose top
/// marginal rate was 7.39% against the statutory 13.2% -- a defect that no test caught at the time.
///
/// # The descending top rates are in the tariff, not an import bug
///
/// Both ladders end with a rate that *falls*: 13.2% to 11.5% for a single taxpayer at CHF 793,300,
/// and 13.0% to 11.5% for a married taxpayer at CHF 940,800. A reader who expected a monotone
/// ladder would read that as corruption. It is the published tariff: the workbook itself carries
/// those two thresholds and those two rates, so the comparison below reproduces them rather than
/// excusing them. The step-down is what caps the federal average rate; deleting it as a
/// "non-monotonic segment" would change every high earner's bill.
#[test]
fn the_federal_tariff_matches_the_estv_workbook() {
    use life_optimizer::federal_tariff_data::{FEDERAL_MARRIED_BRACKETS, FEDERAL_SINGLE_BRACKETS};

    let text = std::fs::read_to_string(SNAPSHOT).unwrap_or_else(|error| {
        panic!(
            "cannot read {SNAPSHOT}: {error}. Re-run \
             `python tools/verify_estv_import.py --emit`."
        )
    });

    let mut expected: std::collections::BTreeMap<String, Vec<Row>> = Default::default();
    for line in text.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(fields.len(), 4, "malformed snapshot row: {line}");
        if !fields[0].starts_with("BUND_") {
            continue;
        }
        expected.entry(fields[0].to_string()).or_default().push(Row {
            threshold: fields[2].parse().expect("threshold"),
            rate: fields[3].parse().expect("rate"),
        });
    }

    for (key, source) in
        [("BUND_SINGLE", FEDERAL_SINGLE_BRACKETS), ("BUND_MARRIED", FEDERAL_MARRIED_BRACKETS)]
    {
        let rows =
            expected.get(key).unwrap_or_else(|| panic!("{key} is missing from the snapshot"));
        assert_eq!(
            source.len(),
            rows.len(),
            "{key}: the constants have {} brackets, the workbook has {}",
            source.len(),
            rows.len()
        );
        for (index, (bracket, row)) in source.iter().zip(rows.iter()).enumerate() {
            assert!(
                (bracket.threshold - row.threshold).abs() < 1e-9,
                "{key} bracket {index}: threshold {} in the constants, {} in the workbook",
                bracket.threshold,
                row.threshold
            );
            assert!(
                (bracket.rate - row.rate).abs() < 1e-6,
                "{key} bracket {index}: rate {} in the constants, {} in the workbook",
                bracket.rate,
                row.rate
            );
        }
    }

    // The top of the single ladder, asserted explicitly so that a future "fix" that smooths the
    // step-down has to argue with a test rather than with a comment.
    let single = FEDERAL_SINGLE_BRACKETS;
    let last = single[single.len() - 1];
    let penultimate = single[single.len() - 2];
    assert_eq!(penultimate.threshold, 793_300.0);
    assert_eq!(last.threshold, 793_400.0);
    assert!((penultimate.rate - 0.132).abs() < 1e-9);
    assert!((last.rate - 0.115).abs() < 1e-9);
}
