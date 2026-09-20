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

//! Tests for the generated Steuerfuss data and its source integrity.
//!
//! The data in `canton_steuerfuss_data.rs` is produced by
//! `tools/generate_steuerfuss.py` from the official ESTV workbook. These tests
//! check the properties that make it usable: full coverage of the 26 cantons,
//! plausible multiplier ranges, and — most importantly — that the cells the
//! source workbook did *not* express as plain multipliers are preserved as
//! exceptions rather than silently substituted.

use life_optimizer::canton_steuerfuss_data::{
    steuerfuss_row, STEUERFUSS_ROWS, STEUERFUSS_YEARS,
};
use life_optimizer::cantons::{Canton, ALL_CANTONS};

/// Every canton must appear in every covered year. A gap would silently drop a
/// canton from the registry rather than failing.
#[test]
fn every_canton_present_in_every_year() {
    for year in STEUERFUSS_YEARS {
        for canton in ALL_CANTONS {
            let row = steuerfuss_row(canton.code(), *year);
            assert!(
                row.is_some(),
                "missing Steuerfuss row for {} in {year}",
                canton.code()
            );
        }
    }
}

/// The row set must be exactly years x cantons, with no duplicates.
#[test]
fn row_count_matches_years_times_cantons() {
    let expected = STEUERFUSS_YEARS.len() * ALL_CANTONS.len();
    assert_eq!(
        STEUERFUSS_ROWS.len(),
        expected,
        "expected {expected} rows for {} years x 26 cantons, got {}",
        STEUERFUSS_YEARS.len(),
        STEUERFUSS_ROWS.len()
    );
}

/// Canton codes in the data must all be real cantons, and the capital recorded
/// must match the canton's official capital.
#[test]
fn codes_and_capitals_are_consistent() {
    for row in STEUERFUSS_ROWS {
        let canton = Canton::from_code(row.canton_code)
            .unwrap_or_else(|| panic!("unknown canton code '{}'", row.canton_code));
        assert_eq!(
            canton.capital(),
            row.capital,
            "{}: capital mismatch in {}",
            row.canton_code,
            row.year
        );
    }
}

/// Multipliers expressed as plain numbers must be in a range Switzerland
/// actually uses. Cantonal Steuerfüsse run from well under 100% (Zug, Glarus)
/// to around 300% (Bern, Obwalden, Nidwalden, Jura); anything outside a
/// generous band means a parsing error rather than a tax policy.
#[test]
fn parsed_multipliers_are_plausible() {
    for row in STEUERFUSS_ROWS {
        if let Some(cantonal) = row.cantonal {
            assert!(
                (0.3..=4.0).contains(&cantonal),
                "{} {}: cantonal multiplier {cantonal} is implausible",
                row.canton_code,
                row.year
            );
        }
        if let Some(municipal) = row.municipal {
            assert!(
                (0.0..=5.0).contains(&municipal),
                "{} {}: municipal multiplier {municipal} is implausible",
                row.canton_code,
                row.year
            );
        }
    }
}

/// When a cell was parsed as a number, the raw text must agree with it.
///
/// Two traps here, both drawn from the real workbook:
///
/// * The raw text is the *source spreadsheet's* rendering, not a canonical
///   form: a cell holding 1.1 is stored by the writer as
///   `1.1000000000000001`. Comparison must be numeric.
/// * Some sheets use a comma decimal separator (`3,2`), which is not valid Rust
///   float syntax, so a naive `parse()` panics on valid data.
#[test]
fn parsed_values_agree_with_raw_source_text() {
    for row in STEUERFUSS_ROWS {
        for (value, raw) in [
            (row.cantonal, row.cantonal_raw),
            (row.municipal, row.municipal_raw),
        ] {
            let Some(value) = value else { continue };
            let raw = raw.trim();
            let normalized = raw.replace(',', ".");
            let as_float: f64 = normalized.parse().unwrap_or_else(|_| {
                panic!(
                    "{} {}: parsed {value} but raw {raw:?} is not a plain number",
                    row.canton_code, row.year
                )
            });
            assert!(
                (as_float - value).abs() < 1e-9,
                "{} {}: parsed {value} differs from raw {raw:?}",
                row.canton_code,
                row.year
            );
        }
    }
}

/// A cell with no numeric value must be *classified*: either explicitly blank
/// (the source says the multiplier does not apply) or unparsed with its original
/// text preserved.
///
/// Collapsing these two would make "Ticino has no church tax" indistinguishable
/// from "we could not read this figure" — the exact confusion this module
/// exists to prevent.
#[test]
fn non_numeric_cells_are_classified_not_silently_dropped() {
    for row in STEUERFUSS_ROWS {
        for (value, blank, raw, label) in [
            (row.cantonal, row.cantonal_blank, row.cantonal_raw, "cantonal"),
            (row.municipal, row.municipal_blank, row.municipal_raw, "municipal"),
        ] {
            if value.is_some() {
                // A parsed value must not also claim to be blank.
                assert!(
                    !blank,
                    "{} {}: {label} is both parsed and marked blank",
                    row.canton_code, row.year
                );
                continue;
            }
            if blank {
                // Blank means "not applicable" — nothing to preserve.
                continue;
            }
            // Otherwise the cell held something we did not interpret, and the
            // source text must survive so a reader can judge it.
            assert!(
                !raw.trim().is_empty(),
                "{} {}: {label} value is None, not blank, and has no raw text — \
                 the source content was lost",
                row.canton_code,
                row.year
            );
        }
    }
}

/// The blank flag must be exercised by the real data — if nothing is ever blank
/// the distinction is untested, and if everything is, it is meaningless.
#[test]
fn blank_classification_is_actually_used() {
    let blank_count = STEUERFUSS_ROWS
        .iter()
        .filter(|r| r.municipal_blank)
        .count();
    let unparsed_count = STEUERFUSS_ROWS
        .iter()
        .filter(|r| r.municipal.is_none() && !r.municipal_blank)
        .count();

    assert!(blank_count > 0, "the source has blank cells, so some rows should be blank");
    assert!(
        unparsed_count > 0,
        "the source has footnoted cells, so some rows should be unparsed-but-present"
    );
    assert!(
        blank_count + unparsed_count < STEUERFUSS_ROWS.len(),
        "most municipal cells should be plain multipliers"
    );
}

/// The documented source exceptions must be present and recognisable, not
/// quietly flattened into numbers. If a future regeneration turns one of these
/// into a value, this test forces the change to be deliberate.
#[test]
fn documented_source_exceptions_are_preserved() {
    // Genève applies a rebate: "Rabais de 12% de l'impôt cantonal de 147,5%".
    // Its cantonal cell is therefore NOT a plain multiplier.
    let geneva = steuerfuss_row("GE", 2024).expect("Genève 2024 present");
    assert!(
        geneva.cantonal.is_none(),
        "Genève's cantonal cell carries a rebate footnote and must not be a plain number"
    );
    assert!(
        geneva.cantonal_raw.contains('%'),
        "Genève's raw text should show the percentage form, got {:?}",
        geneva.cantonal_raw
    );

    // Basel-Stadt: "Die Gemeindesteuer ist in der Kantonssteuer inbegriffen" —
    // municipal tax is included in the cantonal one, so there is no separate
    // municipal multiplier to add.
    let basel = steuerfuss_row("BS", 2024).expect("Basel-Stadt 2024 present");
    assert!(
        basel.municipal.is_none(),
        "Basel-Stadt has no separate municipal multiplier"
    );

    // Sion: "Kein Vielfaches" — no multiplier at all for the cantonal tax.
    let sion = steuerfuss_row("VS", 2024);
    if let Some(row) = sion {
        assert!(
            row.cantonal.is_none(),
            "Valais cantonal cell is marked 'kein Vielfaches'"
        );
    }

    // Chur: municipal expressed as "88% 7)" — a percentage of the cantonal tax,
    // not a multiplier of the simple tax.
    let chur = steuerfuss_row("GR", 2024).expect("Graubünden 2024 present");
    assert!(
        chur.municipal.is_none(),
        "Chur's municipal tax is a percentage of cantonal tax, not a simple-tax multiple"
    );
}

/// Zürich's cantonal multiplier is well known and stable across recent years,
/// which makes it a useful anchor for the whole pipeline.
#[test]
fn zurich_anchor_values_are_correct() {
    let zurich_2024 = steuerfuss_row("ZH", 2024).expect("Zürich 2024 present");
    assert_eq!(zurich_2024.cantonal, Some(0.98), "Zürich cantonal is 98% in 2024");
    assert_eq!(zurich_2024.municipal, Some(1.19), "city of Zürich adds 119%");
}

/// Bern's cantonal multiplier is far above 100%, which is the whole reason a
/// single "assume it is like Zürich" shortcut would be wrong.
#[test]
fn bern_multiplier_reflects_its_high_tax_position() {
    let bern = steuerfuss_row("BE", 2024).expect("Bern 2024 present");
    let cantonal = bern.cantonal.expect("Bern's cantonal multiplier is numeric");
    assert!(
        cantonal > 2.5,
        "Bern's cantonal multiplier should be around 2.975, got {cantonal}"
    );
}

/// Years must be unique per canton, contiguous within the reported range, and
/// sorted newest-first as the generator promises.
#[test]
fn years_are_sorted_and_contiguous() {
    for pair in STEUERFUSS_YEARS.windows(2) {
        assert!(
            pair[0] == pair[1] + 1,
            "years must be contiguous and descending: {} then {}",
            pair[0],
            pair[1]
        );
    }
}

/// An unknown canton or year must return `None` rather than a neighbouring
/// row's data.
#[test]
fn lookups_do_not_fall_back() {
    assert!(steuerfuss_row("XX", 2024).is_none());
    assert!(steuerfuss_row("ZH", 1800).is_none());
    assert!(steuerfuss_row("", 2024).is_none());
}

/// The data must span a meaningful historical range, since the whole point of
/// the ESTV workbook is to show how multipliers have moved over time.
#[test]
fn historical_range_is_substantial() {
    assert!(
        STEUERFUSS_YEARS.len() >= 25,
        "expected at least 25 years of history, got {}",
        STEUERFUSS_YEARS.len()
    );
    let newest = STEUERFUSS_YEARS.first().copied().unwrap_or(0);
    let oldest = STEUERFUSS_YEARS.last().copied().unwrap_or(0);
    assert!(newest >= 2024, "newest year {newest} is unexpectedly old");
    assert!(oldest <= 2000, "oldest year {oldest} is unexpectedly recent");
}

/// Multipliers must actually vary across cantons — if every canton parsed to
/// the same figure, the column mapping would be wrong.
#[test]
fn multipliers_vary_across_cantons() {
    let year = 2024;
    let mut values: Vec<f64> = STEUERFUSS_ROWS
        .iter()
        .filter(|r| r.year == year)
        .filter_map(|r| r.cantonal)
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values.dedup();

    assert!(
        values.len() > 10,
        "only {} distinct cantonal multipliers in {year}; column mapping looks wrong",
        values.len()
    );
    // And the spread should be large, which is the substantive fact about
    // Swiss cantonal tax.
    let lowest = values.first().copied().unwrap_or(0.0);
    let highest = values.last().copied().unwrap_or(0.0);
    assert!(
        highest / lowest > 2.0,
        "cantonal multipliers should span more than a factor of two: {lowest} to {highest}"
    );
}
