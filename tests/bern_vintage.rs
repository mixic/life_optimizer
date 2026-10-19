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

//! How much the 2024 Bern table can be wrong in 2025, in one measurable part.
//!
//! `TaxSchedule::bern_city_default` is a Steuerjahr **2024** table, and the published
//! figures it is compared against are **2025**. `VALIDATION.md` §5 called that gap
//! "inside the band" without saying how much of the band it explains. This file
//! computes the part that can be computed: the cantonal multiplier.
//!
//! # What changed between the two years
//!
//! The Steuerfuss is a multiplier on the simple tax, and it is in
//! `source-documents/steuerfuesse-np-1995-2026.xlsx` for every year — imported,
//! verified, and available in `canton_steuerfuss_data`:
//!
//! | Year | Cantonal | Municipal (Stadt Bern) | Combined |
//! |---|---:|---:|---:|
//! | 2022 | 3.025 | 1.540 | 4.565 |
//! | 2023 | 3.025 | 1.540 | 4.565 |
//! | 2024 | 3.025 | 1.540 | 4.565 |
//! | 2025 | 2.975 | 1.540 | 4.515 |
//! | 2026 | 2.975 | 1.540 | 4.515 |
//!
//! Bern cut the cantonal multiplier by 0.05 of the simple tax from Steuerjahr 2025 and
//! has not restored it. A 2024 table therefore overstates a 2025 liability by
//! 0.05 / 4.565 = **1.095%** of the cantonal-plus-municipal tax, provided the simple
//! tax itself did not change.
//!
//! At the table's top average rate of 24.73% that is **0.27 percentage points**: 24.73%
//! under 2024 rules against 24.46% under 2025 rules.
//!
//! # What this does and does not bound
//!
//! It bounds the multiplier component, and no more. Two things are outside it:
//!
//! - **The simple tax itself.** If Bern also changed its base tariff between 2024 and
//!   2025, the table is wrong by that amount too, and nothing in this repository can
//!   measure it. `SWISS_TAX_DATA.md` records a serious attempt to decode the Stadt Bern
//!   tariff PDF which recovered every income and tax figure but not the rate column.
//! - **Church tax.** The table's header names Kanton, Gemeinde and Kirche. A separate
//!   church multiplier would make the cantonal share smaller and the drift
//!   proportionally smaller, so 0.27 pp is an upper bound rather than a point estimate.
//!
//! The comparison against published figures in `tests/published_tax_reference.rs`
//! measures a gap of 5.7 to 12.4 pp. This file accounts for about 0.3 pp of it, which
//! is the useful result: the vintage is not the explanation, and the missing federal
//! tax — deliberately excluded from the model column — is.

use life_optimizer::canton_steuerfuss_data::steuerfuss_row;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn multiplier(year: u16) -> (f64, f64) {
    let row = steuerfuss_row("BE", year)
        .unwrap_or_else(|| panic!("no Steuerfuss row for Bern in {year}"));
    (
        row.cantonal.expect("Bern has a cantonal multiplier"),
        row.municipal.expect("Stadt Bern has a municipal multiplier"),
    )
}

/// The multiplier schedule the drift is computed from.
#[test]
fn the_bern_multiplier_changed_once_between_2022_and_2026() {
    // Three years at 3.025, then two at 2.975.
    for year in [2022, 2023, 2024] {
        let (cantonal, municipal) = multiplier(year);
        assert!(close(cantonal, 3.025), "{year}: cantonal {cantonal}");
        assert!(close(municipal, 1.540), "{year}: municipal {municipal}");
    }
    for year in [2025, 2026] {
        let (cantonal, municipal) = multiplier(year);
        assert!(close(cantonal, 2.975), "{year}: cantonal {cantonal}");
        assert!(close(municipal, 1.540), "{year}: municipal {municipal}");
    }
}

/// The Steuerjahr 2024 table applied to 2025 rules, and how far that moves it.
#[test]
fn the_2024_vintage_overstates_the_2025_multiplier_by_one_percent() {
    let (c2024, m2024) = multiplier(2024);
    let (c2025, m2025) = multiplier(2025);
    let combined_2024 = c2024 + m2024;
    let combined_2025 = c2025 + m2025;

    // 4.565 down to 4.515.
    assert!(close(combined_2024, 4.565), "2024 combined {combined_2024}");
    assert!(close(combined_2025, 4.515), "2025 combined {combined_2025}");

    // The relative reduction: 0.05 of 4.565, which is 1.0953%.
    let reduction = (combined_2024 - combined_2025) / combined_2024;
    assert!(
        (reduction - 0.010_953).abs() < 1e-6,
        "the multiplier reduction is {reduction}, not 1.0953%"
    );

    // Applied to the table's top average rate. The table says 24.73% at and above
    // CHF 200,000 (asserted in `tests/validation.rs`); under 2025 rules the same
    // income is taxed at 24.73% x 4.515 / 4.565.
    let top_rate_2024 = 0.2473_f64;
    let top_rate_2025 = top_rate_2024 * combined_2025 / combined_2024;
    assert!(
        (top_rate_2025 - 0.244_59).abs() < 1e-4,
        "the 2025-equivalent top rate is {top_rate_2025:.5}, not 0.24459"
    );

    let drift_pp = (top_rate_2024 - top_rate_2025) * 100.0;
    assert!(
        (drift_pp - 0.271).abs() < 0.002,
        "the vintage drift is {drift_pp:.3} pp, not 0.271 pp"
    );

    // And the point of the exercise: this is a small part of the 5.7-12.4 pp gap the
    // published comparison reports. If a future Bern tariff makes the drift larger than
    // a percentage point, the assumption that "vintage is not the explanation" stops
    // holding and the comparison needs rewriting.
    assert!(
        drift_pp < 1.0,
        "the vintage drift has grown to {drift_pp:.2} pp, which is no longer negligible \
         against the published gap"
    );
}

/// The 2024 table is wrong for 2025 by this multiplier change and never low: the
/// cantonal multiplier fell, so a 2024 table can only overstate.
#[test]
fn the_vintage_error_runs_in_the_safe_direction() {
    let (c2024, m2024) = multiplier(2024);
    let (c2025, m2025) = multiplier(2025);
    assert!(
        c2024 + m2024 > c2025 + m2025,
        "the 2025 multiplier is not below the 2024 one, so the direction assumption \
         in the drift bound is wrong"
    );
}
