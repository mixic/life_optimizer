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

//! Two official sources for the married federal tariff, and they disagree.
//!
//! `source-documents/estv_scales_Bund.xlsx` is the ESTV's Tarife export, from which
//! `FEDERAL_MARRIED_BRACKETS` is generated. `source-documents/Tarif Art. 42 - Abs. 1 -
//! 2026-20XX_d.pdf` is the ESTV's published tariff for married taxpayers from
//! Steuerjahr 2026, in Einheits-Satz form. Both are official; both claim to describe
//! the same tax.
//!
//! At the bottom of the table they do not agree:
//!
//! | Taxable income | PDF rate | PDF simple tax | Tax from the imported ladder |
//! |---:|---:|---:|---:|
//! | CHF 100 | 1.5500% | 1.55 | 0.00 |
//! | CHF 500 | 1.5500% | 7.75 | 0.00 |
//! | CHF 900 | 1.5500% | 13.95 | 0.00 |
//!
//! The imported ladder opens with a zero band: its first two points are `(0.0, 0.0)`
//! and `(29700.0, 0.01)`, so it charges nothing below CHF 29,700. The PDF charges
//! 1.55% from the first CHF 100.
//!
//! # This test pins a defect rather than a property
//!
//! It asserts that the disagreement is still there, with these numbers, because a
//! known contradiction that stops being tested becomes an unknown one. **If this test
//! starts failing because the two now agree, that is the good outcome**: one of the
//! sources was corrected, and `VALIDATION.md` §5.2 plus `tools/verify_federal_pdf.py`
//! need updating and this file deleting.
//!
//! # Why the contradiction is not resolved here
//!
//! Resolving it needs the statute — the DBG, `source-documents/SR 642.11.pdf` — which
//! yields no extractable text at all (it is the Identity-H case documented in
//! `SWISS_TAX_DATA.md`). Between an official export and an official PDF, with the law
//! unreadable, choosing one would be a guess dressed as a fix. The defect is recorded
//! instead, and it is on the list in `VALIDATION.md` §5.2.
//!
//! # Coverage
//!
//! Nine rows, CHF 100 to 900. The PDF lays each page out in three column blocks and
//! the income tokens stop lining up with the rate and tax tokens after the first few
//! rows, so only these extract unambiguously. The rest of the table is **not**
//! compared, and this test does not claim otherwise.

use life_optimizer::federal_tariff_data::FEDERAL_MARRIED_BRACKETS;

const SNAPSHOT: &str = "source-documents/federal_pdf_expected.csv";

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Integrate the marginal bands up to `income`, the way the tax is actually computed.
fn tax_from_ladder(income: f64) -> f64 {
    let mut total = 0.0;
    let bands = FEDERAL_MARRIED_BRACKETS;
    for (index, band) in bands.iter().enumerate() {
        if income <= band.threshold {
            break;
        }
        let next = match bands.get(index + 1) {
            Some(following) => following.threshold,
            None => f64::INFINITY,
        };
        let span = income.min(next) - band.threshold;
        if span > 0.0 {
            total += span * band.rate;
        }
    }
    total
}

fn pdf_rows() -> Vec<(f64, f64, f64)> {
    let text = std::fs::read_to_string(SNAPSHOT).unwrap_or_else(|error| {
        panic!(
            "cannot read {SNAPSHOT}: {error}. Re-run \
             `python tools/verify_federal_pdf.py --emit`."
        )
    });
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            assert_eq!(fields.len(), 3, "malformed snapshot row: {line}");
            (
                fields[0].parse().expect("income"),
                fields[1].parse().expect("rate"),
                fields[2].parse().expect("tax"),
            )
        })
        .collect()
}

/// The extraction is self-consistent: the PDF's simple tax is its own rate applied to
/// the income. This checks the parser, not the tariff, and it is what makes the
/// disagreement below evidence rather than a decoding artefact.
#[test]
fn the_pdf_rows_are_internally_consistent() {
    let rows = pdf_rows();
    assert_eq!(rows.len(), 9, "the snapshot has {} rows", rows.len());
    for (income, rate, tax) in &rows {
        let expected = income * rate / 100.0;
        assert!(
            (tax - expected).abs() < 0.005,
            "income {income}: the PDF's rate {rate}% gives {expected:.2}, but it prints {tax:.2}"
        );
    }
}

/// The imported ladder's opening bands, which are the reason the two disagree.
#[test]
fn the_imported_ladder_opens_with_a_zero_band() {
    let bands = FEDERAL_MARRIED_BRACKETS;
    assert!(close(bands[0].threshold, 0.0));
    assert!(close(bands[0].rate, 0.0));
    assert!(close(bands[1].threshold, 29_700.0));
    assert!(close(bands[1].rate, 0.01));
    // Nothing below the second threshold, by construction.
    assert_eq!(tax_from_ladder(100.0), 0.0);
    assert_eq!(tax_from_ladder(29_700.0), 0.0);
}

/// The pinned contradiction.
#[test]
fn the_pdf_and_the_imported_ladder_disagree_at_low_incomes() {
    let rows = pdf_rows();
    for (income, _rate, pdf_tax) in &rows {
        let imported = tax_from_ladder(*income);
        assert_ne!(
            imported, *pdf_tax,
            "income {income}: the imported ladder now agrees with the PDF ({pdf_tax:.2}). \
             That is good news -- one of the sources was corrected. Update VALIDATION.md \
             §5.2, delete this test, and delete tools/verify_federal_pdf.py's finding."
        );
        // The disagreement is the whole first band, not a rounding difference.
        assert!(
            (imported - pdf_tax).abs() > 1.0,
            "income {income}: the difference is only {:.2}, which is now small enough to \
             be a rounding question rather than a band question",
            (imported - pdf_tax).abs()
        );
    }
}
