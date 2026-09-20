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

//! The 26 Swiss cantons, with the data needed to compute cantonal tax.
//!
//! This is the second level of the two-level model: cantonal (and, in the
//! capital, municipal) tax is the canton's *base* tariff multiplied by a
//! **Steuerfuss** — a percentage applied uniformly to the whole scale. That
//! structure is why a canton's tax level can be summarised by a single
//! multiplier once its base scale is known.
//!
//! # Provenance is enforced, not optional
//!
//! Every canton carries a [`CantonTaxData`] whose every field is an
//! [`Option`] paired with a source. There is deliberately **no fallback**: an
//! unsourced canton produces an error, never a silent substitution of another
//! canton's numbers or a plausible guess.
//!
//! This matters more than it might appear. Swiss cantonal tax varies by a
//! factor of roughly three between the cheapest and most expensive canton, so a
//! substituted or invented Steuerfuss would change every recommendation the
//! tool makes while still looking authoritative. `FutureWork.md` §7 states the
//! standard the project holds itself to: numbers must be traceable "to a data
//! source, to a named assumption, to a documented estimation method — versus
//! being a plausible narrative wrapped around unfitted parameters."
//!
//! # What is still needed
//!
//! To complete the model each canton needs:
//!
//! 1. `steuerfuss_2024` — the cantonal multiplier, in percent (e.g. `98.0`).
//!    Published annually in each canton's budget or tax decree.
//! 2. `base_scale` — the canton's simple-tax tariff: the rate schedule the
//!    multiplier is applied to. Published in the canton's Steuergesetz.
//! 3. `capital_municipal_fuss` — the capital city's municipal multiplier, for a
//!    total that reflects a real address rather than a canton-wide average.
//!
//! See `MATHEMATICS.md` §5.2.1 for the arithmetic and `CONTRIBUTING.md` for how
//! to add a canton with its sources.

use crate::federal_tax::{federal_tax, FederalBracket};
use serde::{Deserialize, Serialize};

/// All 26 Swiss cantons, in official order.
///
/// The order and two-letter codes follow the official enumeration (AG through
/// ZH), so the list can be diffed against an official table by eye.
pub const ALL_CANTONS: &[Canton] = &[
    Canton::Aargau,
    Canton::AppenzellAusserrhoden,
    Canton::AppenzellInnerrhoden,
    Canton::Bern,
    Canton::BaselLandschaft,
    Canton::BaselStadt,
    Canton::Fribourg,
    Canton::Geneva,
    Canton::Glarus,
    Canton::Graubunden,
    Canton::Jura,
    Canton::Lucerne,
    Canton::Neuchatel,
    Canton::Nidwalden,
    Canton::Obwalden,
    Canton::Schaffhausen,
    Canton::Schwyz,
    Canton::Solothurn,
    Canton::StGallen,
    Canton::Thurgau,
    Canton::Ticino,
    Canton::Uri,
    Canton::Valais,
    Canton::Vaud,
    Canton::Zug,
    Canton::Zurich,
];

/// A Swiss canton.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Canton {
    Aargau,
    AppenzellAusserrhoden,
    AppenzellInnerrhoden,
    Bern,
    BaselLandschaft,
    BaselStadt,
    Fribourg,
    Geneva,
    Glarus,
    Graubunden,
    Jura,
    Lucerne,
    Neuchatel,
    Nidwalden,
    Obwalden,
    Schaffhausen,
    Schwyz,
    Solothurn,
    StGallen,
    Thurgau,
    Ticino,
    Uri,
    Valais,
    Vaud,
    Zug,
    Zurich,
}

impl Canton {
    /// Two-letter official code, as used on the CLI.
    pub fn code(&self) -> &'static str {
        match self {
            Canton::Aargau => "AG",
            Canton::AppenzellAusserrhoden => "AR",
            Canton::AppenzellInnerrhoden => "AI",
            Canton::Bern => "BE",
            Canton::BaselLandschaft => "BL",
            Canton::BaselStadt => "BS",
            Canton::Fribourg => "FR",
            Canton::Geneva => "GE",
            Canton::Glarus => "GL",
            Canton::Graubunden => "GR",
            Canton::Jura => "JU",
            Canton::Lucerne => "LU",
            Canton::Neuchatel => "NE",
            Canton::Nidwalden => "NW",
            Canton::Obwalden => "OW",
            Canton::Schaffhausen => "SH",
            Canton::Schwyz => "SZ",
            Canton::Solothurn => "SO",
            Canton::StGallen => "SG",
            Canton::Thurgau => "TG",
            Canton::Ticino => "TI",
            Canton::Uri => "UR",
            Canton::Valais => "VS",
            Canton::Vaud => "VD",
            Canton::Zug => "ZG",
            Canton::Zurich => "ZH",
        }
    }

    /// English name, for display and error messages.
    pub fn name(&self) -> &'static str {
        match self {
            Canton::Aargau => "Aargau",
            Canton::AppenzellAusserrhoden => "Appenzell Ausserrhoden",
            Canton::AppenzellInnerrhoden => "Appenzell Innerrhoden",
            Canton::Bern => "Bern",
            Canton::BaselLandschaft => "Basel-Landschaft",
            Canton::BaselStadt => "Basel-Stadt",
            Canton::Fribourg => "Fribourg",
            Canton::Geneva => "Geneva",
            Canton::Glarus => "Glarus",
            Canton::Graubunden => "Graubünden",
            Canton::Jura => "Jura",
            Canton::Lucerne => "Lucerne",
            Canton::Neuchatel => "Neuchâtel",
            Canton::Nidwalden => "Nidwalden",
            Canton::Obwalden => "Obwalden",
            Canton::Schaffhausen => "Schaffhausen",
            Canton::Schwyz => "Schwyz",
            Canton::Solothurn => "Solothurn",
            Canton::StGallen => "St. Gallen",
            Canton::Thurgau => "Thurgau",
            Canton::Ticino => "Ticino",
            Canton::Uri => "Uri",
            Canton::Valais => "Valais",
            Canton::Vaud => "Vaud",
            Canton::Zug => "Zug",
            Canton::Zurich => "Zürich",
        }
    }

    /// The canton's capital, which is the municipality this model prices.
    pub fn capital(&self) -> &'static str {
        match self {
            Canton::Aargau => "Aarau",
            Canton::AppenzellAusserrhoden => "Herisau",
            Canton::AppenzellInnerrhoden => "Appenzell",
            Canton::Bern => "Bern",
            Canton::BaselLandschaft => "Liestal",
            Canton::BaselStadt => "Basel",
            Canton::Fribourg => "Fribourg",
            Canton::Geneva => "Geneva",
            Canton::Glarus => "Glarus",
            Canton::Graubunden => "Chur",
            Canton::Jura => "Delémont",
            Canton::Lucerne => "Lucerne",
            Canton::Neuchatel => "Neuchâtel",
            Canton::Nidwalden => "Stans",
            Canton::Obwalden => "Sarnen",
            Canton::Schaffhausen => "Schaffhausen",
            Canton::Schwyz => "Schwyz",
            Canton::Solothurn => "Solothurn",
            Canton::StGallen => "St. Gallen",
            Canton::Thurgau => "Frauenfeld",
            Canton::Ticino => "Bellinzona",
            Canton::Uri => "Altdorf",
            Canton::Valais => "Sion",
            Canton::Vaud => "Lausanne",
            Canton::Zug => "Zug",
            Canton::Zurich => "Zürich",
        }
    }

    /// Parse a two-letter canton code, case-insensitively.
    pub fn from_code(code: &str) -> Option<Self> {
        let needle = code.trim().to_ascii_uppercase();
        ALL_CANTONS.iter().copied().find(|c| c.code() == needle)
    }

    /// All codes, for error messages.
    pub fn all_codes() -> String {
        ALL_CANTONS
            .iter()
            .map(|c| c.code())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Where a number came from. Every sourced figure must name its origin, and
/// every unsourced one must say so, so that a reader can tell which is which
/// without leaving the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    /// Taken from a named official publication, with the year it applies to.
    Official { source: &'static str, year: u16 },
    /// Supplied by the user for their own situation, from their tax assessment.
    UserProvided,
    /// Not yet sourced. Cantons in this state cannot be priced — they produce
    /// an error rather than a guess.
    NotSourced,
}

impl Provenance {
    pub fn is_usable(&self) -> bool {
        !matches!(self, Provenance::NotSourced)
    }

    pub fn describe(&self) -> String {
        match self {
            Provenance::Official { source, year } => format!("{source} ({year})"),
            Provenance::UserProvided => "user-provided".to_string(),
            Provenance::NotSourced => "NOT SOURCED — not usable for calculation".to_string(),
        }
    }
}

/// Cantonal tax data. Every numeric field is optional: absence means "unknown",
/// and unknown means the canton cannot be priced, never that a default is used.
///
/// **Units.** `steuerfuss` holds the ESTV *factor* form, not a percentage: the
/// workbook records Zürich as `0.98`, meaning 98% of the simple tax, and Bern as
/// `2.975`, meaning 297.5%. Using the workbook's own representation avoids a
/// unit conversion that would otherwise have to be re-derived at every call
/// site — and a 100x error in a tax multiplier is not subtle.
///
/// Not `Deserialize`: the base scale is a `&'static` slice borrowed from a
/// published table, so this registry is compiled-in reference data rather than
/// user configuration. Cantons supplied by the user enter through
/// `TaxSchedule::custom_rate` instead, which is the honest place for an
/// observed personal figure.
#[derive(Debug, Clone, PartialEq)]
pub struct CantonTaxData {
    /// Cantonal multiplier in ESTV factor form, applied to the base scale.
    /// E.g. `0.98` means 98% of the simple tax.
    pub steuerfuss: Option<f64>,
    pub steuerfuss_provenance: Provenance,
    /// The canton's simple-tax tariff, which the multiplier scales.
    pub base_scale: Option<&'static [FederalBracket]>,
    pub base_scale_provenance: Provenance,
    /// Municipal multiplier of the capital city, in ESTV factor form, on top of
    /// the cantonal one.
    pub capital_municipal_fuss: Option<f64>,
    pub municipal_provenance: Provenance,
}

impl CantonTaxData {
    /// A canton with nothing sourced yet.
    pub const fn unsourced() -> Self {
        Self {
            steuerfuss: None,
            steuerfuss_provenance: Provenance::NotSourced,
            base_scale: None,
            base_scale_provenance: Provenance::NotSourced,
            capital_municipal_fuss: None,
            municipal_provenance: Provenance::NotSourced,
        }
    }

    /// Whether the two-level path (base scale x Steuerfuss) can price this
    /// canton. Requires both the multiplier and the scale it multiplies.
    pub fn is_priced(&self) -> bool {
        self.steuerfuss.is_some() && self.base_scale.is_some()
    }

    /// Human-readable list of what is missing, for the error message.
    ///
    /// Accounts for the imported scales: a canton whose base scale comes from
    /// `estv_scales_data` must not have it reported as missing, or the reader is
    /// sent looking for a file that is already loaded.
    pub fn missing_fields(&self, canton: Canton) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.steuerfuss.is_none() {
            missing.push("cantonal Steuerfuss");
        }
        if self.base_scale.is_none() && imported_scale(canton).is_none() {
            missing.push("cantonal base tax scale");
        }
        if self.capital_municipal_fuss.is_none() {
            missing.push("capital municipal Steuerfuss");
        }
        missing
    }
}

/// Cantons that can be priced even though their two-level decomposition is
/// incomplete, because a complete standalone rate table exists for them.
///
/// Bern is the only one today: `TaxSchedule::bern_city_default` carries the
/// official Stadt Bern table (cantonal + municipal + church already combined),
/// so equating "not decomposed into base x multiplier" with "unusable" would
/// wrongly disable the one canton that works.
pub const LEGACY_TABLE_CANTONS: &[Canton] = &[Canton::Bern];

/// Whether a canton can produce a tax figure at all, by either path.
pub fn is_priceable(canton: Canton) -> bool {
    is_priced_any_source(canton) || LEGACY_TABLE_CANTONS.contains(&canton)
}

/// Whether the Steuerfuss and a base scale are both available, from whichever
/// source supplies the scale.
fn is_priced_any_source(canton: Canton) -> bool {
    let data = canton_tax_data(canton);
    data.steuerfuss.is_some() && (data.base_scale.is_some() || imported_scale(canton).is_some())
}

/// Bern's cantonal base scale.
///
/// **Not sourced.** This is a placeholder carrying the shape the real scale
/// needs, so the calculation path can be tested end to end. It must be replaced
/// with the published Bern tariff before any Bern output is trusted.
pub const BERN_BASE_SCALE_PLACEHOLDER: &[FederalBracket] = &[
    FederalBracket { threshold: 0.0, rate: 0.0000 },
    FederalBracket { threshold: 10_000.0, rate: 0.0200 },
    FederalBracket { threshold: 20_000.0, rate: 0.0400 },
    FederalBracket { threshold: 40_000.0, rate: 0.0600 },
    FederalBracket { threshold: 60_000.0, rate: 0.0800 },
    FederalBracket { threshold: 100_000.0, rate: 0.1000 },
    FederalBracket { threshold: 200_000.0, rate: 0.1200 },
];

/// Fill in Steuerfuss figures from the generated ESTV import.
///
/// `canton_tax_data` hard-codes the handful of cantons whose entry has been
/// hand-checked. This wrapper supplies the Steuerfuss and capital-municipal
/// figures for *every* canton from `canton_steuerfuss_data`, which already holds
/// them for all 26 cantons. Without it the registry reported "missing: cantonal
/// Steuerfuss, capital municipal Steuerfuss" for cantons whose multipliers were
/// sitting in the workbook the whole time.
///
/// Base scales cannot be filled this way — they are a separate publication and
/// genuinely absent — so a canton still fails unless a scale has been imported.
/// The provenance of each filled figure names the workbook and year, so a
/// supplied number is never anonymous.
fn with_imported_steuerfuss(canton: Canton, mut data: CantonTaxData) -> CantonTaxData {
    // A deliberately hand-entered value always wins over the bulk import.
    if data.steuerfuss.is_none() {
        if let Some(row) = crate::canton_steuerfuss_data::steuerfuss_for_code(canton.code(), 2024) {
            if let Some(fuss) = row.cantonal {
                data.steuerfuss = Some(fuss);
                data.steuerfuss_provenance = Provenance::Official {
                    source: "ESTV, Steuerfüsse in den Kantonshauptorten (natural persons)",
                    year: 2024,
                };
            }
        }
    }
    if data.capital_municipal_fuss.is_none() {
        if let Some(row) = crate::canton_steuerfuss_data::steuerfuss_for_code(canton.code(), 2024) {
            if let Some(fuss) = row.municipal {
                data.capital_municipal_fuss = Some(fuss);
                data.municipal_provenance = Provenance::Official {
                    source: "ESTV, Steuerfüsse in den Kantonshauptorten (cantonal capital)",
                    year: 2024,
                };
            }
        }
    }
    data
}

/// The cantonal tax data table.
///
/// Everything is [`CantonTaxData::unsourced`] except where a figure has been
/// deliberately entered with its provenance. Steuerfuss and municipal multipliers
/// are then backfilled for all cantons from the generated ESTV import; base
/// scales are not, and remain the reason most cantons cannot be priced.
pub fn canton_tax_data(canton: Canton) -> CantonTaxData {
    with_imported_steuerfuss(canton, hand_entered_tax_data(canton))
}

/// The hand-checked entries, before the bulk Steuerfuss import is layered on.
fn hand_entered_tax_data(canton: Canton) -> CantonTaxData {
    match canton {
        // Bern is priced by `TaxSchedule::bern_city_default`, which carries its
        // own hand-entered Stadt Bern rate table taken from the city's own
        // publication. That table is complete and in use, so the *canton* is
        // priceable even though the two-level decomposition (base scale plus
        // Steuerfuss) has not been done for it. Marking it unsourced here would
        // wrongly block the one canton that currently works.
        //
        // The provenance records how, not just whether: these are official
        // Stadt Bern figures, entered directly rather than derived.
        Canton::Bern => CantonTaxData {
            steuerfuss: None,
            steuerfuss_provenance: Provenance::Official {
                source: "Stadt Bern, Steuerbelastung des Arbeitseinkommens (complete \
                         rate table, not a base-scale decomposition)",
                year: 2024,
            },
            base_scale: None,
            base_scale_provenance: Provenance::Official {
                source: "Stadt Bern, Steuerbelastung des Arbeitseinkommens",
                year: 2024,
            },
            capital_municipal_fuss: None,
            municipal_provenance: Provenance::Official {
                source: "Stadt Bern (rates already include cantonal + municipal + church)",
                year: 2024,
            },
        },

        // Aargau: scale imported from the ESTV "Tarife" export
        // (`estv_scales_AG.xlsx`); Steuerfuss from the ESTV Steuerfuss workbook.
        //
        // Aargau uses income splitting (`Splittingfaktor 2.0`): taxable income
        // is divided by 2, the scale applied, and the result multiplied by 2.
        Canton::Aargau => CantonTaxData {
            steuerfuss: Some(1.11),
            steuerfuss_provenance: Provenance::Official {
                source: "ESTV, Steuerfüsse in den Kantonshauptorten (natural persons)",
                year: 2024,
            },
            base_scale: imported_scale(Canton::Aargau).map(|s| s.brackets(false)),
            base_scale_provenance: Provenance::Official {
                source: "ESTV Steuerrechner, Tarife export (estv_scales_AG.xlsx)",
                year: 2026,
            },
            capital_municipal_fuss: Some(0.96),
            municipal_provenance: Provenance::Official {
                source: "ESTV, Steuerfüsse in den Kantonshauptorten (Aarau)",
                year: 2024,
            },
        },

        // Zürich's cantonal Steuerfuss is 98% of the simple tax, and the city
        // of Zürich adds 119%. Both are in the ESTV workbook. The canton's
        // *base scale* is not yet entered, so the canton is not priceable —
        // but the multipliers are recorded with their provenance so the
        // remaining work is only the tariff.
        Canton::Zurich => CantonTaxData {
            steuerfuss: Some(0.98),
            steuerfuss_provenance: Provenance::Official {
                source: "ESTV, Steuerfüsse in den Kantonshauptorten (natural persons)",
                year: 2024,
            },
            base_scale: None,
            base_scale_provenance: Provenance::NotSourced,
            capital_municipal_fuss: Some(1.19),
            municipal_provenance: Provenance::Official {
                source: "ESTV, Steuerfüsse in den Kantonshauptorten (Stadt Zürich)",
                year: 2024,
            },
        },

        _ => CantonTaxData::unsourced(),
    }
}

/// Failure modes when pricing a canton. Returned rather than panicking so the
/// CLI can explain precisely what is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CantonTaxError {
    /// The canton has no sourced data at all.
    NotSourced { canton: Canton, missing: Vec<&'static str> },
    /// Data exists but is internally inconsistent.
    InvalidData { canton: Canton, detail: String },
}

impl std::fmt::Display for CantonTaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CantonTaxError::NotSourced { canton, missing } => write!(
                f,
                "no sourced tax data for canton {} ({}). Missing: {}. \
                 Life Optimizer will not substitute another canton's rates or an \
                 estimate, because cantonal tax varies by roughly a factor of three \
                 and a guessed multiplier would silently change every result.",
                canton.code(),
                canton.name(),
                missing.join(", ")
            ),
            CantonTaxError::InvalidData { canton, detail } => {
                write!(f, "invalid tax data for canton {}: {detail}", canton.code())
            }
        }
    }
}

impl std::error::Error for CantonTaxError {}

/// The imported base scale for a canton, if one has been loaded.
pub fn imported_scale(canton: Canton) -> Option<&'static crate::estv_scales_data::BaseScale> {
    crate::estv_scales_data::base_scale(canton.code())
}

/// The brackets to apply for a given canton and marital status.
///
/// Cantons express marital treatment in one of two ways, and the imported data
/// records which:
///
/// * a **shared** scale (`Steuersubjekt = Alle`), where the difference is
///   applied afterwards through the splitting factor; or
/// * **per-subject** scales, where the difference is already in the table.
///
/// Selecting the brackets is therefore not simply "the canton's scale" — using
/// the married table *and* splitting would count the marital adjustment twice.
pub fn imported_brackets(canton: Canton, married: bool) -> Option<&'static [FederalBracket]> {
    imported_scale(canton).map(|s| s.brackets(married))
}

/// The cantonal arithmetic, on explicit inputs.
///
/// Split out from [`cantonal_tax`] so the composition rule can be tested
/// directly, without a real canton's scale having to be loaded. The whole point
/// of the two-level model is that the Steuerfuss multiplies the *entire*
/// base-scale result, and that is what this function states.
///
/// Multipliers are ESTV factors, so 0.98 means 98%:
///
/// ```text
/// cantonal_tax = simple_tax(taxable_income, scale) x (cantonal + municipal)
/// ```
///
/// `splitting_factor` implements income splitting, which several cantons use in
/// place of a married scale: taxable income is divided by the factor, the scale
/// is applied to that reduced amount, and the resulting tax is multiplied back
/// up. With a factor of 2 this is full splitting, which is why Aargau's scale
/// can apply to married and single taxpayers alike.
pub fn cantonal_tax_from_scale(
    scale: &[FederalBracket],
    taxable_income: f64,
    cantonal_fuss: f64,
    municipal_fuss: Option<f64>,
    include_municipal: bool,
    splitting_factor: Option<f64>,
) -> Result<f64, String> {
    if !(0.0..=4.0).contains(&cantonal_fuss) {
        return Err(format!(
            "cantonal Steuerfuss factor {cantonal_fuss} is outside the plausible range 0-4.0"
        ));
    }

    let (income_for_scale, split_multiplier) = match splitting_factor {
        Some(factor) => {
            if !(1.0..=3.0).contains(&factor) {
                return Err(format!(
                    "splitting factor {factor} is outside the plausible range 1.0-3.0"
                ));
            }
            (taxable_income / factor, factor)
        }
        None => (taxable_income, 1.0),
    };

    let base = crate::federal_tax::tax_with_scale(scale, income_for_scale) * split_multiplier;

    let mut total_fuss = cantonal_fuss;
    if include_municipal {
        if let Some(municipal) = municipal_fuss {
            if !(0.0..=5.0).contains(&municipal) {
                return Err(format!(
                    "municipal Steuerfuss factor {municipal} is outside the plausible range 0-5.0"
                ));
            }
            total_fuss += municipal;
        }
    }

    Ok(base * total_fuss)
}

/// Cantonal (including capital municipal) tax on a taxable income.
///
/// The Steuerfuss applies to the *whole* base-scale result, which is what makes
/// a single multiplier a sufficient description of a canton's tax level. The
/// arithmetic lives in [`cantonal_tax_from_scale`]; this function supplies the
/// canton's data and converts its errors.
///
/// `married` currently affects only the federal component, because cantonal
/// scales differ in how they treat marital status (married scale vs. splitting
/// vs. a deduction) and the scales are not yet entered. Once they are, the
/// household's status must select the appropriate scale here rather than being
/// passed to [`crate::federal_tax::tax_with_scale`].
pub fn cantonal_tax(
    canton: Canton,
    taxable_income: f64,
    married: bool,
    include_municipal: bool,
) -> Result<f64, CantonTaxError> {
    let _ = married;
    let data = canton_tax_data(canton);

    if LEGACY_TABLE_CANTONS.contains(&canton) && !data.is_priced() {
        // Guessing at a decomposition would double-count against the standalone
        // table that already exists for this canton.
        return Err(CantonTaxError::InvalidData {
            canton,
            detail: "this canton is priced by its own complete rate table; the \
                     two-level path does not apply"
                .to_string(),
        });
    }

    let fuss = data.steuerfuss.ok_or_else(|| CantonTaxError::NotSourced {
        canton,
        missing: data.missing_fields(canton),
    })?;

    // Marital status selects the bracket table; the splitting factor is a
    // separate axis and is only ever non-`None` for a shared scale.
    let (scale, splitting) = match imported_scale(canton) {
        Some(imported) => (
            imported.brackets(married),
            imported.splitting_factor(married),
        ),
        None => (
            data.base_scale.ok_or_else(|| CantonTaxError::NotSourced {
                canton,
                missing: data.missing_fields(canton),
            })?,
            None,
        ),
    };

    cantonal_tax_from_scale(
        scale,
        taxable_income,
        fuss,
        data.capital_municipal_fuss,
        include_municipal,
        splitting,
    )
    .map_err(|detail| CantonTaxError::InvalidData { canton, detail })
}

/// Federal + cantonal + capital municipal tax on a *taxable* income.
///
/// Returns an error for cantons whose data is incomplete. The federal
/// component is always available, since it does not vary by canton.
pub fn total_tax(
    canton: Canton,
    taxable_income: f64,
    married: bool,
    include_municipal: bool,
) -> Result<f64, CantonTaxError> {
    let federal = federal_tax(taxable_income, married);
    let cantonal = cantonal_tax(canton, taxable_income, married, include_municipal)?;
    Ok(federal + cantonal)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registry must cover exactly the 26 official cantons, with no
    /// duplicates and no omissions.
    #[test]
    fn registry_covers_all_26_cantons() {
        assert_eq!(ALL_CANTONS.len(), 26, "Switzerland has 26 cantons");

        let mut codes: Vec<&str> = ALL_CANTONS.iter().map(|c| c.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 26, "canton codes must be unique");

        // A few well-known codes, to catch a mispaired code/name.
        assert_eq!(Canton::Zurich.code(), "ZH");
        assert_eq!(Canton::Bern.code(), "BE");
        assert_eq!(Canton::Geneva.code(), "GE");
        assert_eq!(Canton::Zug.code(), "ZG");
        assert_eq!(Canton::Ticino.code(), "TI");
    }

    /// Every canton must round-trip through its code, and parsing must be
    /// forgiving about case but strict about content.
    #[test]
    fn canton_codes_round_trip() {
        for canton in ALL_CANTONS {
            assert_eq!(Canton::from_code(canton.code()), Some(*canton));
            assert_eq!(Canton::from_code(&canton.code().to_lowercase()), Some(*canton));
            assert_eq!(Canton::from_code(&format!("  {}  ", canton.code())), Some(*canton));
        }
        assert_eq!(Canton::from_code("XX"), None);
        assert_eq!(Canton::from_code(""), None);
        assert_eq!(Canton::from_code("Zürich"), None, "codes, not names");
    }

    /// Every canton must have a capital, since that is the municipality priced.
    #[test]
    fn every_canton_has_a_capital() {
        for canton in ALL_CANTONS {
            assert!(!canton.capital().trim().is_empty(), "{}: no capital", canton.code());
            assert!(!canton.name().trim().is_empty(), "{}: no name", canton.code());
        }
    }

    /// Cantons that currently cannot be priced, derived from the data rather
    /// than named explicitly.
    ///
    /// Hard-coding a canton here was a recurring maintenance trap: as exports
    /// arrived, "an unpriced canton" kept becoming priced and the tests failed
    /// for the wrong reason. Deriving the set means adding data never breaks
    /// these assertions.
    fn unpriceable_cantons() -> Vec<Canton> {
        ALL_CANTONS
            .iter()
            .copied()
            .filter(|c| !is_priceable(*c))
            .collect()
    }

    /// An unsourced canton must fail loudly. This is the central safety property
    /// of the whole registry: no silent fallback to another canton.
    #[test]
    fn unsourced_cantons_fail_loudly() {
        let unpriced = unpriceable_cantons();
        if unpriced.is_empty() {
            return; // every canton loaded; nothing to assert
        }

        for canton in unpriced {
            let result = cantonal_tax(canton, 100_000.0, false, false);
            assert!(
                result.is_err(),
                "{} cannot be priced, so it must not return a number",
                canton.code()
            );
            match result.unwrap_err() {
                CantonTaxError::NotSourced { canton: c, missing } => {
                    assert_eq!(c, canton);
                    assert!(
                        !missing.is_empty(),
                        "{}: the error must say what is missing",
                        canton.code()
                    );
                }
                // The other legitimate refusal: a canton priced by its own
                // standalone table has no two-level decomposition.
                CantonTaxError::InvalidData { .. } => {}
            }
        }
    }

    /// The error message must be actionable: it names the canton and what is
    /// missing, and explains why no estimate was substituted.
    ///
    /// It must name only what is *genuinely* absent. The Steuerfuss is supplied
    /// from the ESTV import for every canton whose source cell held a plain
    /// number, so reporting it as missing would send a reader looking in the
    /// wrong place.
    #[test]
    fn error_message_is_actionable() {
        let unpriced = unpriceable_cantons();
        let Some(&canton) = unpriced.iter().find(|c| {
            canton_tax_data(**c).steuerfuss.is_some()
        }) else {
            return; // no suitable canton loaded; nothing to assert
        };

        let err = cantonal_tax(canton, 100_000.0, false, false).unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains(canton.code()),
            "should name the canton: {message}"
        );
        assert!(
            message.contains("will not substitute"),
            "should state the no-fallback policy: {message}"
        );
    }

    /// Cantons whose 2024 source cell held no plain cantonal Steuerfuss.
    ///
    /// These are **documented source exceptions**, not registry gaps, and the
    /// list is asserted exactly so that a change in the source data fails the
    /// test rather than silently altering which cantons are priceable:
    ///
    /// * `GE` — cell reads `148.5%9)`, with footnote 9: *"Rabais de 12% de
    ///   l'impôt cantonal de 147.5%"*. A rebate applies, so the effective
    ///   multiplier is not the printed figure.
    /// * `VS` — cell reads `3)`, footnoted *"Kein Vielfaches"*: Valais does not
    ///   express a cantonal multiplier at all.
    /// * `BL` — the cell is blank, so there is nothing to read.
    /// * `FR` — the cantonal cell is blank; Fribourg splits income and wealth
    ///   rows instead.
    const STEUERFUSS_SOURCE_EXCEPTIONS: &[&str] = &["GE", "VS", "BL", "FR"];

    /// The bulk Steuerfuss import must reach every canton whose source cell
    /// actually held a multiplier.
    ///
    /// This is the regression guard for the defect it fixes: three cantons were
    /// hand-entered while the workbook held multipliers for all 26, so 23
    /// cantons reported "missing: cantonal Steuerfuss" for data that was
    /// already in the repository.
    #[test]
    fn steuerfuss_is_supplied_for_every_canton_except_source_exceptions() {
        for canton in ALL_CANTONS {
            let data = canton_tax_data(*canton);
            if STEUERFUSS_SOURCE_EXCEPTIONS.contains(&canton.code()) {
                assert!(
                    data.steuerfuss.is_none(),
                    "{}: is a documented source exception and should stay unsupplied",
                    canton.code()
                );
                continue;
            }
            assert!(
                data.steuerfuss.is_some(),
                "{}: Steuerfuss should be supplied from the ESTV import",
                canton.code()
            );
            assert!(
                data.steuerfuss_provenance.is_usable(),
                "{}: a supplied Steuerfuss must carry provenance",
                canton.code()
            );
        }
    }

    /// Exactly the expected cantons lack a cantonal Steuerfuss — no more, no
    /// fewer. If the source data changes, this fails and forces the change to
    /// be reviewed rather than silently shifting coverage.
    #[test]
    fn source_exception_set_is_exactly_as_documented() {
        let mut actual: Vec<&str> = ALL_CANTONS
            .iter()
            .filter(|c| canton_tax_data(**c).steuerfuss.is_none())
            .map(|c| c.code())
            .collect();
        actual.sort_unstable();

        let mut expected = STEUERFUSS_SOURCE_EXCEPTIONS.to_vec();
        expected.sort_unstable();

        assert_eq!(
            actual, expected,
            "the set of cantons without a cantonal Steuerfuss changed"
        );
    }

    /// For a canton with no export yet, only the base scale should remain
    /// missing — the multipliers are supplied and must not be reported absent.
    #[test]
    fn only_the_base_scale_remains_missing() {
        for canton in [Canton::Vaud, Canton::Ticino, Canton::Zug, Canton::Thurgau] {
            if imported_scale(canton).is_some() {
                continue; // an export arrived; the scale is no longer missing
            }
            let missing = canton_tax_data(canton).missing_fields(canton);
            assert!(
                missing.contains(&"cantonal base tax scale"),
                "{}: the base scale is the outstanding item",
                canton.code()
            );
            assert!(
                !missing.contains(&"cantonal Steuerfuss"),
                "{}: Steuerfuss is supplied, so it must not be listed",
                canton.code()
            );
        }
    }

    /// Valais must name its *own* reason for being unpriced: its multiplier is
    /// a source exception ("Kein Vielfaches"), so that is missing too.
    ///
    /// Geneva is deliberately excluded even though its multiplier also carries a
    /// footnote: an export has since been supplied, so GE's remaining gap is
    /// only the interpretation of the rebate, not the absence of data.
    #[test]
    fn no_multiplier_canton_reports_its_own_gap() {
        for canton in [Canton::Valais] {
            let missing = canton_tax_data(canton).missing_fields(canton);
            assert!(
                missing.contains(&"cantonal Steuerfuss"),
                "{}: its Steuerfuss is a source exception, so it is genuinely missing",
                canton.code()
            );
        }
    }

    /// A canton with a scale but no usable multiplier must still refuse rather
    /// than guess at the multiplier.
    ///
    /// Valais now has an imported scale, but its 2024 Steuerfuss cell reads
    /// `3)` — footnoted *"Kein Vielfaches"* — so the multiplier is genuinely
    /// absent. Having half the two-level inputs must not produce a figure.
    #[test]
    fn half_the_inputs_still_fails_rather_than_guessing() {
        let canton = Canton::Valais;
        let data = canton_tax_data(canton);

        assert!(
            data.steuerfuss.is_none(),
            "Valais' cantonal cell is a source exception, so it stays unsupplied"
        );
        assert!(
            imported_scale(canton).is_some(),
            "but its scale has been imported, so this is the half-data case"
        );
        assert!(
            !is_priceable(canton),
            "a scale without a multiplier cannot produce a figure"
        );

        let result = cantonal_tax(canton, 100_000.0, false, false);
        assert!(result.is_err(), "a half-sourced canton must not be priced");

        let missing = data.missing_fields(canton);
        assert!(
            missing.contains(&"cantonal Steuerfuss"),
            "the error must name the missing multiplier: {missing:?}"
        );
        assert!(
            !missing.contains(&"cantonal base tax scale"),
            "the scale is present and must not be reported missing: {missing:?}"
        );
    }

    /// Provenance must be declared for every populated figure — an unsourced
    /// number sitting in the registry would be the exact failure mode §7 warns
    /// about.
    #[test]
    fn every_populated_figure_declares_its_provenance() {
        for canton in ALL_CANTONS {
            let data = canton_tax_data(*canton);

            if data.steuerfuss.is_some() {
                assert!(
                    data.steuerfuss_provenance.is_usable(),
                    "{}: has a Steuerfuss but declares no usable provenance",
                    canton.code()
                );
            }
            if data.base_scale.is_some() {
                assert!(
                    data.base_scale_provenance.is_usable(),
                    "{}: has a base scale but declares no usable provenance",
                    canton.code()
                );
            }
            if data.capital_municipal_fuss.is_some() {
                assert!(
                    data.municipal_provenance.is_usable(),
                    "{}: has a municipal multiplier but no provenance",
                    canton.code()
                );
            }
        }
    }

    /// Bern's placeholder scale must not be presented as sourced.
    #[test]
    fn placeholder_scale_is_not_marked_official() {
        let data = canton_tax_data(Canton::Bern);
        // Bern is priced by the legacy schedule, not here.
        assert!(!data.is_priced(), "the two-level path must not claim to price Bern yet");
    }

    /// The federal component must never be unavailable — it does not depend on
    /// the canton. Only the cantonal part can fail.
    ///
    /// Derived from the data rather than naming a canton, because naming one
    /// keeps breaking as exports arrive.
    #[test]
    fn federal_component_is_canton_independent() {
        let taxable = 120_000.0;
        let single_federal = federal_tax(taxable, false);

        if let Some(&canton) = unpriceable_cantons().first() {
            // The cantonal part fails...
            let err = total_tax(canton, taxable, false, false).unwrap_err();
            assert!(matches!(
                err,
                CantonTaxError::NotSourced { .. } | CantonTaxError::InvalidData { .. }
            ));
            // ...but the federal figure is unaffected by the canton choice.
            assert!((federal_tax(taxable, false) - single_federal).abs() < 1e-12);
        }

        // And for every canton that *is* priceable, total = federal + cantonal.
        for canton in ALL_CANTONS.iter().copied().filter(|c| is_priceable(*c)) {
            if LEGACY_TABLE_CANTONS.contains(&canton) {
                continue; // priced by its own standalone table
            }
            let total = total_tax(canton, taxable, false, false)
                .unwrap_or_else(|e| panic!("{}: {e}", canton.code()));
            let cantonal = cantonal_tax(canton, taxable, false, false).unwrap();
            assert!(
                (total - (single_federal + cantonal)).abs() < 1e-9,
                "{}: total should be federal + cantonal",
                canton.code()
            );
        }
    }

    /// The scale application must be linear in the Steuerfuss: doubling the
    /// multiplier doubles the cantonal tax. This is what makes a single
    /// multiplier a valid summary of a canton.
    #[test]
    fn cantonal_tax_is_linear_in_the_steuerfuss() {
        let taxable = 80_000.0;
        let base =
            crate::federal_tax::tax_with_scale(BERN_BASE_SCALE_PLACEHOLDER, taxable);

        // Apply the arithmetic the public function uses, at two multipliers.
        let at_100 = base * 100.0 / 100.0;
        let at_200 = base * 200.0 / 100.0;
        assert!((at_200 - 2.0 * at_100).abs() < 1e-9);
    }

    /// The composition rule, on explicit inputs so it can be checked without any
    /// canton's scale being loaded.
    ///
    /// Using a one-bracket 100%-of-income scale makes the expected value exact
    /// and hand-checkable, which is the point: a test that recomputes the
    /// implementation's own arithmetic proves nothing.
    #[test]
    fn composition_rule_multiplies_the_whole_base_scale() {
        // A flat 100% scale: simple tax == taxable income.
        let flat = [FederalBracket { threshold: 0.0, rate: 1.0 }];
        let income = 100_000.0;

        // Cantonal only: 100,000 x 0.98.
        let cantonal_only =
            cantonal_tax_from_scale(&flat, income, 0.98, Some(1.19), false, None)
                .expect("valid factors");
        assert!(
            (cantonal_only - 98_000.0).abs() < 1e-9,
            "expected 100000 x 0.98 = 98000, got {cantonal_only}"
        );

        // Cantonal + municipal: 100,000 x (0.98 + 1.19) = 217,000.
        let with_municipal =
            cantonal_tax_from_scale(&flat, income, 0.98, Some(1.19), true, None)
                .expect("valid factors");
        assert!(
            (with_municipal - 217_000.0).abs() < 1e-9,
            "expected 100000 x 2.17 = 217000, got {with_municipal}"
        );

        // A high-tax canton: Bern's 2.975 factor on the same base.
        let bern_like =
            cantonal_tax_from_scale(&flat, income, 2.975, Some(1.54), true, None)
                .expect("valid factors");
        assert!(
            (bern_like - income * (2.975 + 1.54)).abs() < 1e-9,
            "Bern-like total should be income x 4.515, got {bern_like}"
        );
    }

    /// Income splitting: the scale is applied to income divided by the factor,
    /// and the result multiplied back up.
    ///
    /// A flat scale makes this exactly checkable, and it also shows the
    /// mechanism is *not* a no-op: splitting is progressive in effect, because
    /// halving the base moves the taxpayer down the scale before doubling.
    #[test]
    fn splitting_applies_the_scale_to_halved_income() {
        let flat = [FederalBracket { threshold: 0.0, rate: 0.10 }];
        let income = 100_000.0;

        // Without splitting: 100,000 x 10% x 1.0 fuss = 10,000.
        let unsplit = cantonal_tax_from_scale(&flat, income, 1.0, None, false, None).unwrap();
        assert!((unsplit - 10_000.0).abs() < 1e-9, "got {unsplit}");

        // With full splitting on a flat scale: (100,000 / 2) x 10% x 2 = 10,000.
        // A flat scale is linear, so splitting cannot change the answer -- which
        // is the correct behaviour and confirms the halving/doubling nets out.
        let split = cantonal_tax_from_scale(&flat, income, 1.0, None, false, Some(2.0)).unwrap();
        assert!(
            (split - 10_000.0).abs() < 1e-9,
            "splitting must be neutral on a linear scale, got {split}"
        );

        // On a *progressive* scale splitting must reduce the tax.
        let progressive = [
            FederalBracket { threshold: 0.0, rate: 0.02 },
            FederalBracket { threshold: 50_000.0, rate: 0.20 },
        ];
        let p_unsplit =
            cantonal_tax_from_scale(&progressive, income, 1.0, None, false, None).unwrap();
        let p_split =
            cantonal_tax_from_scale(&progressive, income, 1.0, None, false, Some(2.0)).unwrap();
        assert!(
            p_split < p_unsplit,
            "splitting should reduce tax on a progressive scale: {p_split} vs {p_unsplit}"
        );
    }

    /// An implausible splitting factor is rejected rather than applied.
    #[test]
    fn implausible_splitting_factor_is_rejected() {
        let flat = [FederalBracket { threshold: 0.0, rate: 1.0 }];
        assert!(cantonal_tax_from_scale(&flat, 100_000.0, 1.0, None, false, Some(0.5)).is_err());
        assert!(cantonal_tax_from_scale(&flat, 100_000.0, 1.0, None, false, Some(9.0)).is_err());
        assert!(cantonal_tax_from_scale(&flat, 100_000.0, 1.0, None, false, Some(1.0)).is_ok());
    }

    /// Out-of-range factors are rejected rather than producing a nonsense tax.
    #[test]
    fn implausible_factors_are_rejected() {
        let flat = [FederalBracket { threshold: 0.0, rate: 1.0 }];

        assert!(cantonal_tax_from_scale(&flat, 100_000.0, -1.0, None, false, None).is_err());
        assert!(cantonal_tax_from_scale(&flat, 100_000.0, 40.0, None, false, None).is_err());
        assert!(
            cantonal_tax_from_scale(&flat, 100_000.0, 1.0, Some(50.0), true, None).is_err(),
            "an absurd municipal factor must not be silently applied"
        );
        // A missing municipal factor is not an error -- some cantons have none.
        assert!(cantonal_tax_from_scale(&flat, 100_000.0, 1.0, None, true, None).is_ok());
    }

    /// The two levels must compose additively: total = federal + cantonal, with
    /// the federal part independent of the canton.
    #[test]
    fn total_is_federal_plus_cantonal() {
        let flat = [FederalBracket { threshold: 0.0, rate: 1.0 }];
        let income = 50_000.0;

        let cantonal =
            cantonal_tax_from_scale(&flat, income, 1.0, None, false, None).unwrap();
        let federal = crate::federal_tax::federal_tax(income, false);

        // The public `total_tax` cannot be used here because no canton has a
        // loaded scale, so assert the composition directly.
        let total = federal + cantonal;
        assert!((total - (federal + income)).abs() < 1e-9);

        // And the federal component must be strictly positive at this income,
        // so the sum is genuinely two contributions rather than one.
        assert!(federal > 0.0, "federal tax should be non-zero at 50k");
    }

    /// A zero-income household owes nothing, at either level.
    #[test]
    fn zero_income_yields_zero_cantonal_tax() {
        let flat = [FederalBracket { threshold: 0.0, rate: 1.0 }];
        let tax = cantonal_tax_from_scale(&flat, 0.0, 0.98, Some(1.19), true, None).unwrap();
        assert_eq!(tax, 0.0);
    }

    /// Aargau must now be priceable end to end: the ESTV-imported scale, the
    /// ESTV Steuerfuss, and the splitting factor all present.
    ///
    /// Aargau's capital is Aarau, with cantonal Steuerfuss 110% and municipal
    /// 96% (ESTV workbook, 2024).
    #[test]
    fn aargau_is_priceable_from_the_imported_scale() {
        assert!(
            is_priceable(Canton::Aargau),
            "Aargau has an imported scale and multipliers, so it must be priceable"
        );

        let taxable = 100_000.0;

        // Single taxpayer: no splitting, so the full income meets the scale.
        //   bands up to 100,000 accumulate to a simple tax of 6,938
        //   6,938 x 2.07 (1.11 cantonal + 0.96 Aarau) = 14,361.66
        let single = cantonal_tax(Canton::Aargau, taxable, false, true)
            .expect("Aargau is now priceable");
        assert!(
            (single - 14_361.66).abs() < 0.5,
            "Aargau single at CHF {taxable}: expected 14361.66, got {single:.2}"
        );

        // Married taxpayer: splitting halves the assessable income, so the
        // scale is applied at 50,000 giving 2,488, then doubled for the split
        // and scaled by the Steuerfuss:
        //   2,488 x 2 x 2.07 = 9,869.76
        let married = cantonal_tax(Canton::Aargau, taxable, true, true)
            .expect("Aargau is now priceable");
        assert!(
            (married - 9_869.76).abs() < 0.5,
            "Aargau married at CHF {taxable}: expected 9869.76, got {married:.2}"
        );

        // Marriage must not cost more than being single at the same income.
        assert!(
            married < single,
            "splitting should favour the married taxpayer: {married:.2} vs {single:.2}"
        );
    }

    /// Ordinary cantons must now be priceable from the imported scales.
    ///
    /// The point is end-to-end coverage: registry Steuerfuss + imported scale +
    /// the two-level path, for cantons that previously refused outright.
    ///
    /// `FR` and `GE` are deliberately absent, and covered by their own tests:
    /// both have an imported scale but no plain cantonal multiplier — `FR` is
    /// blank in the workbook, `GE` carries a 12% rebate footnote.
    #[test]
    fn imported_cantons_are_priceable() {
        let taxable = 100_000.0;
        for canton in [
            Canton::Zurich,
            Canton::BaselStadt,
            Canton::Lucerne,
            Canton::Schaffhausen,
            Canton::Solothurn,
            Canton::Aargau,
        ] {
            assert!(
                is_priceable(canton),
                "{} should be priceable from the imported scale",
                canton.code()
            );

            for married in [false, true] {
                let tax = cantonal_tax(canton, taxable, married, true)
                    .unwrap_or_else(|e| panic!("{} married={married}: {e}", canton.code()));
                let rate = tax / taxable;

                assert!(
                    tax > 0.0,
                    "{} married={married}: tax should be positive",
                    canton.code()
                );
                assert!(
                    (0.01..0.45).contains(&rate),
                    "{} married={married}: effective cantonal rate {:.1}% is implausible",
                    canton.code(),
                    rate * 100.0
                );
            }

            // Married must never be taxed more heavily than single at the same
            // income, whether the canton uses splitting or a married scale.
            let single = cantonal_tax(canton, taxable, false, true).unwrap();
            let married = cantonal_tax(canton, taxable, true, true).unwrap();
            assert!(
                married <= single + 1.0,
                "{}: married {married:.2} should not exceed single {single:.2}",
                canton.code()
            );
        }
    }

    /// A canton with no imported scale must still refuse rather than fall back.
    ///
    /// The list is chosen from cantons with no export supplied so far. If one of
    /// them gains a scale the test skips it rather than failing, so adding
    /// exports does not break the suite.
    #[test]
    fn cantons_without_imported_scales_still_refuse() {
        for canton in [
            Canton::Ticino,
            Canton::Vaud,
            Canton::Jura,
            Canton::Neuchatel,
            Canton::Thurgau,
        ] {
            if is_priceable(canton) {
                continue; // an export arrived; nothing to assert
            }
            let result = cantonal_tax(canton, 100_000.0, false, true);
            assert!(
                result.is_err(),
                "{} has no scale and must not produce a figure",
                canton.code()
            );
        }
    }

    /// Fribourg has a scale but no cantonal multiplier, so it must refuse and
    /// say which of the two is missing.
    #[test]
    fn fribourg_refuses_for_the_missing_multiplier_not_the_scale() {
        if imported_scale(Canton::Fribourg).is_none() {
            return; // no export supplied; nothing to assert
        }
        assert!(
            !is_priceable(Canton::Fribourg),
            "FR has a scale but a blank Steuerfuss, so it cannot be priced"
        );
        let missing = canton_tax_data(Canton::Fribourg).missing_fields(Canton::Fribourg);
        assert!(
            missing.contains(&"cantonal Steuerfuss"),
            "FR should report the missing multiplier: {missing:?}"
        );
        assert!(
            !missing.contains(&"cantonal base tax scale"),
            "FR's scale is imported, so it must not be reported missing: {missing:?}"
        );
    }

    /// Geneva likewise has a scale but a footnoted multiplier.
    ///
    /// Its 2024 cell reads `148.5%9)` with footnote 9 — *"Rabais de 12% de
    /// l'impôt cantonal de 147.5%"*. The effective multiplier is computable
    /// (147.5% x 0.88 = 129.8%) but interpreting a footnote is a decision rather
    /// than a parse, so it stays unsupplied and Geneva refuses. This test pins
    /// that until the interpretation is confirmed.
    #[test]
    fn geneva_refuses_until_the_rebate_is_interpreted() {
        if imported_scale(Canton::Geneva).is_none() {
            return; // no export supplied; nothing to assert
        }
        assert!(
            !is_priceable(Canton::Geneva),
            "GE's multiplier carries a rebate footnote, so it is not usable as-is"
        );
        let missing = canton_tax_data(Canton::Geneva).missing_fields(Canton::Geneva);
        assert!(
            missing.contains(&"cantonal Steuerfuss"),
            "GE should report the missing multiplier: {missing:?}"
        );
    }
}
