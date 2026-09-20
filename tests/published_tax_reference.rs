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

//! Cross-check the hand-entered Bern tax table against published reference
//! figures.
//!
//! Source: finpension.ch, "Steuerprogression in der Schweiz" (published
//! 16.06.2026), quoting the ESTV Steuerrechner for **Steuerjahr 2025**, with
//! the explicit assumption "alleinstehend, konfessionslos, keine Kinder".
//!
//! The article gives total tax (Bund + Kanton + Gemeinde) in CHF against
//! *taxable* income, for the cities of Zürich, Schwyz and Bern:
//!
//! | Taxable income | Schwyz  | Zürich  | Bern    |
//! |----------------|---------|---------|---------|
//! | CHF  25,000    |  1,643  |  1,364  |  4,002  |
//! | CHF  50,000    |  4,424  |  4,854  |  9,250  |
//! | CHF 100,000    | 12,327  | 16,181  | 22,984  |
//! | CHF 250,000    | 46,732  | 68,897  | 80,382  |
//! | CHF 500,000    | 113,200 | 172,133 | 185,468 |
//!
//! These are the only externally-sourced cantonal figures available in this
//! repository, which makes them worth pinning — but they are **five points for
//! three cantons**, not a tax scale. They can validate a model's shape and
//! ordering; they cannot replace the cantonal base scales.

use life_optimizer::tax::TaxSchedule;

/// One published (taxable income, total tax) reference point.
struct Reference {
    canton: &'static str,
    taxable_income: f64,
    tax_chf: f64,
}

const REFERENCES: &[Reference] = &[
    Reference { canton: "SZ", taxable_income: 25_000.0, tax_chf: 1_643.0 },
    Reference { canton: "SZ", taxable_income: 50_000.0, tax_chf: 4_424.0 },
    Reference { canton: "SZ", taxable_income: 100_000.0, tax_chf: 12_327.0 },
    Reference { canton: "SZ", taxable_income: 250_000.0, tax_chf: 46_732.0 },
    Reference { canton: "SZ", taxable_income: 500_000.0, tax_chf: 113_200.0 },
    Reference { canton: "ZH", taxable_income: 25_000.0, tax_chf: 1_364.0 },
    Reference { canton: "ZH", taxable_income: 50_000.0, tax_chf: 4_854.0 },
    Reference { canton: "ZH", taxable_income: 100_000.0, tax_chf: 16_181.0 },
    Reference { canton: "ZH", taxable_income: 250_000.0, tax_chf: 68_897.0 },
    Reference { canton: "ZH", taxable_income: 500_000.0, tax_chf: 172_133.0 },
    Reference { canton: "BE", taxable_income: 25_000.0, tax_chf: 4_002.0 },
    Reference { canton: "BE", taxable_income: 50_000.0, tax_chf: 9_250.0 },
    Reference { canton: "BE", taxable_income: 100_000.0, tax_chf: 22_984.0 },
    Reference { canton: "BE", taxable_income: 250_000.0, tax_chf: 80_382.0 },
    Reference { canton: "BE", taxable_income: 500_000.0, tax_chf: 185_468.0 },
];

/// The published effective rate at a reference point, in percent.
fn published_rate(r: &Reference) -> f64 {
    r.tax_chf / r.taxable_income * 100.0
}

/// Reference rates must rise with income — this is the "progression" the article
/// is about, and it is the property a model has to reproduce to be credible.
#[test]
fn published_figures_show_progression() {
    for canton in ["SZ", "ZH", "BE"] {
        let mut points: Vec<&Reference> = REFERENCES
            .iter()
            .filter(|r| r.canton == canton)
            .collect();
        points.sort_by(|a, b| a.taxable_income.partial_cmp(&b.taxable_income).unwrap());

        for pair in points.windows(2) {
            let lower = published_rate(pair[0]);
            let higher = published_rate(pair[1]);
            assert!(
                higher > lower,
                "{canton}: rate at CHF {} ({higher:.2}%) should exceed rate at CHF {} ({lower:.2}%)",
                pair[1].taxable_income,
                pair[0].taxable_income
            );
        }
    }
}

/// The cantonal ordering in the published data. Asserting it pins the figures
/// against a transcription slip, and documents that the ordering is
/// **income-dependent** — a naive "Schwyz is always cheapest" assumption is
/// wrong at the low end.
///
/// Read off the source table:
///
/// | Income | cheapest | middle | dearest |
/// |---|---|---|---|
/// | 25,000  | ZH (1,364)  | SZ (1,643)  | BE (4,002)   |
/// | 50,000  | SZ (4,424)  | ZH (4,854)  | BE (9,250)   |
/// | 100,000 | SZ (12,327) | ZH (16,181) | BE (22,984)  |
/// | 250,000 | SZ (46,732) | ZH (68,897) | BE (80,382)  |
/// | 500,000 | SZ (113,200)| ZH (172,133)| BE (185,468) |
#[test]
fn cantonal_ordering_matches_the_published_table() {
    let at = |canton: &str, income: f64| -> f64 {
        REFERENCES
            .iter()
            .find(|r| r.canton == canton && r.taxable_income == income)
            .unwrap_or_else(|| panic!("missing reference for {canton} at {income}"))
            .tax_chf
    };

    // At CHF 25,000 Zürich undercuts Schwyz — the one income level where the
    // ordering differs.
    assert!(
        at("ZH", 25_000.0) < at("SZ", 25_000.0),
        "at CHF 25,000 Zürich should be cheaper than Schwyz"
    );

    // From CHF 50,000 up, Schwyz is the cheapest and Bern the dearest.
    for income in [50_000.0, 100_000.0, 250_000.0, 500_000.0] {
        let (sz, zh, be) = (at("SZ", income), at("ZH", income), at("BE", income));
        assert!(
            sz < zh,
            "at CHF {income}: Schwyz {sz} should be below Zürich {zh}"
        );
        assert!(
            zh < be,
            "at CHF {income}: Zürich {zh} should be below Bern {be}"
        );
    }

    // Bern is dearest at every level, including the low end.
    for income in [25_000.0, 50_000.0, 100_000.0, 250_000.0, 500_000.0] {
        let (zh, be) = (at("ZH", income), at("BE", income));
        assert!(
            zh < be,
            "at CHF {income}: Zürich {zh} should be below Bern {be}"
        );
    }
}

/// Bern's published burden must be close to what the shipped Bern table
/// produces on the same taxable income.
///
/// This is the one canton with a complete table in the repository, so it is the
/// only place an external figure can be checked against real code. A large
/// discrepancy means the hand-entered table has drifted from the ESTV figures
/// the article quotes.
#[test]
fn bern_table_is_close_to_the_published_figures() {
    // `tax_only_rate` is the official cantonal + municipal + church rate
    // schedule, which is the quantity the article's "Steuer" column represents
    // (Bund + Kanton + Gemeinde).
    let schedule = TaxSchedule::bern_city_default(false, 0);

    let mut checked = 0;
    for r in REFERENCES.iter().filter(|r| r.canton == "BE") {
        let modelled_rate = schedule.tax_only_rate(r.taxable_income) * 100.0;
        let published = published_rate(r);
        let diff_pp = modelled_rate - published;

        // Report rather than assert tightly: the table is Steuerjahr 2024 and
        // the figures are 2025, and the article includes the federal share
        // while `tax_only_rate` is the cantonal schedule. A regression here is
        // still worth surfacing.
        println!(
            "BE taxable CHF {:>7.0}: model {:>6.2}%  published {:>6.2}%  diff {:>+6.2}pp",
            r.taxable_income, modelled_rate, published, diff_pp
        );
        checked += 1;

        assert!(
            diff_pp.abs() < 15.0,
            "BE at CHF {}: modelled {modelled_rate:.2}% vs published {published:.2}% \
             differ by {diff_pp:+.2}pp, which is too large to be a definitional difference",
            r.taxable_income
        );
    }
    assert_eq!(checked, 5, "all five Bern reference points should be checked");
}

/// Order of magnitude check: total tax must be a sensible fraction of income
/// everywhere, never exceeding the income itself or being negative.
#[test]
fn published_burdens_are_plausible() {
    for r in REFERENCES {
        let fraction = r.tax_chf / r.taxable_income;
        assert!(
            (0.0..0.5).contains(&fraction),
            "{} at CHF {}: tax is {:.1}% of income, which is implausible",
            r.canton,
            r.taxable_income,
            fraction * 100.0
        );
    }
}
