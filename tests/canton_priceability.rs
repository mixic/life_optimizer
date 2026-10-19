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

//! Which cantons the tax engine can actually price, and why the rest are refused.
//!
//! An external review of this repository said the tool "remains Bern-accurate and
//! Bern-adjacent-approximate" and asked for real schedules for Zürich, Vaud, Geneva, Basel-Stadt
//! and Zug. Checking that claim found it substantially wrong: the ESTV imports cover the whole
//! country, and `TaxSchedule::from_canton_scale` was already wired to every canton that has both a
//! scale and a Steuerfuss. Four of the five cantons the review named have worked all along.
//!
//! What remains true is smaller and more interesting: three cantons cannot be priced, and for a
//! reason that is a deliberate choice rather than an omission. Their source cells are ambiguous, and
//! the importer refuses to guess. That refusal is the same discipline this repository applies
//! everywhere else — no fallback, no substitution of another canton's numbers — so the test below
//! pins it rather than "fixing" it.

use life_optimizer::cantons::{self, Canton};
use life_optimizer::tax::TaxSchedule;

/// The cantons whose official source cell is unusable, and therefore deliberately unpriced.
///
/// - **Geneva**: the cantonal Steuerfuss cell reads `"147.5%9)"`. That is a value with a footnote
///   marker fused to it, or a percentage-of-another-base; the importer cannot tell which, and a
///   wrong Steuerfuss is a wrong tax for every Geneva household.
/// - **Fribourg** and **Valais**: the cells are blank in the source export.
///
/// `tools/generate_steuerfuss.py` documents the refusal at the point where it happens. This list is
/// the same decision, expressed where a test can see it.
const UNPRICED: [&str; 3] = ["FR", "GE", "VS"];

/// The five cantons the review asked for. Four of them are priced; the fifth is Geneva.
const REVIEW_NAMED: [&str; 5] = ["ZH", "VD", "GE", "BS", "ZG"];

fn is_unpriced(canton: Canton) -> bool {
    UNPRICED.contains(&canton.code())
}

/// Exactly three cantons may be unpriced. A fourth appearing, or one of these three becoming
/// priceable without the source being resolved, must fail here rather than surface as a quietly
/// different tax bill.
#[test]
fn exactly_three_cantons_are_unpriced_and_they_are_the_documented_ones() {
    let mut unpriced = Vec::new();
    let mut priced = 0;
    for canton in cantons::ALL_CANTONS {
        if cantons::is_priceable(*canton) {
            priced += 1;
        } else {
            unpriced.push(canton.code());
        }
    }
    assert_eq!(
        unpriced, UNPRICED,
        "the set of unpriced cantons changed: got {unpriced:?}, expected {UNPRICED:?}"
    );
    assert_eq!(priced, 23, "the priced count changed from 23");
    assert_eq!(priced + unpriced.len(), 26);
}

/// Every priced canton must produce a real schedule, not a placeholder.
#[test]
fn every_priced_canton_produces_a_usable_schedule() {
    for canton in cantons::ALL_CANTONS {
        if !cantons::is_priceable(*canton) {
            continue;
        }
        // Bern is priced from its own official table rather than from the ESTV scale, which is why
        // `from_canton_scale` returns a schedule for it with no tax in it: the legacy path is the
        // one that carries Bern's numbers, and `LEGACY_TABLE_CANTONS` is what says so.
        let schedule = if cantons::LEGACY_TABLE_CANTONS.contains(canton) {
            TaxSchedule::bern_city_default(false, 0)
        } else {
            TaxSchedule::from_canton_scale(*canton, false, 0)
                .unwrap_or_else(|| panic!("{} is priced but produced no schedule", canton.code()))
        };
        // A schedule that charged nothing anywhere would satisfy the type and nothing else.
        let low = schedule.tax_rate_on_taxable(40_000.0);
        let high = schedule.tax_rate_on_taxable(200_000.0);
        assert!(
            low >= 0.0 && low < 1.0,
            "{} charges {:.1}% at CHF 40,000",
            canton.code(),
            low * 100.0
        );
        assert!(
            high > 0.0,
            "{} charges nothing at CHF 200,000, so the schedule is empty",
            canton.code()
        );
        assert!(
            high >= low - 1e-9,
            "{} is regressive: {:.2}% at 40k against {:.2}% at 200k",
            canton.code(),
            low * 100.0,
            high * 100.0
        );
    }
}

/// Four of the five cantons the review named are priced, and the fifth is refused loudly rather
/// than approximated.
///
/// This is the test that answers the review: the claim was that the tool could not price them, and
/// the code says it can, with one documented exception.
#[test]
fn four_of_the_five_review_named_cantons_are_priced() {
    let mut priced = 0;
    for code in REVIEW_NAMED {
        let canton = Canton::from_code(code).unwrap_or_else(|| panic!("{code} is not a canton"));
        if cantons::is_priceable(canton) {
            priced += 1;
            assert!(
                TaxSchedule::from_canton_scale(canton, false, 0).is_some(),
                "{code} is priced but produced no schedule"
            );
        } else {
            assert_eq!(code, "GE", "an unexpected canton is unpriced: {code}");
        }
    }
    assert_eq!(priced, 4, "expected four of the five review-named cantons to be priced");
}

/// An unpriced canton must return `None` from `from_canton_scale` rather than a Bern-shaped
/// schedule, because a silent substitution is the failure this repository treats as dangerous.
#[test]
fn an_unpriced_canton_refuses_rather_than_substituting_another_cantons_numbers() {
    for code in UNPRICED {
        let canton = Canton::from_code(code).expect("a real canton");
        assert!(!cantons::is_priceable(canton), "{code} should not be priceable");
        assert!(
            TaxSchedule::from_canton_scale(canton, false, 0).is_none(),
            "{code} is unpriced but produced a schedule, which means something substituted for it"
        );
    }
}

/// The cantonal schedules must actually differ from Bern's, or "supported" would be true in the
/// type system and false in the answer.
#[test]
fn cantonal_schedules_differ_from_bern() {
    let taxable = 150_000.0;
    let bern = TaxSchedule::bern_city_default(false, 0).tax_rate_on_taxable(taxable);
    let mut differences = 0;
    for code in ["ZH", "VD", "BS", "ZG"] {
        let canton = Canton::from_code(code).expect("a real canton");
        let schedule = TaxSchedule::from_canton_scale(canton, false, 0).expect("priced");
        let rate = schedule.tax_rate_on_taxable(taxable);
        if (rate - bern).abs() > 1e-6 {
            differences += 1;
        }
    }
    assert_eq!(
        differences, 4,
        "at least one cantonal schedule reproduces Bern's rate exactly, which suggests a fallback"
    );
}
