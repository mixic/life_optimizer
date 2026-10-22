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

//! Parameters, and where each one comes from.
//!
//! Every field carries a [`Provenance`]. Exactly one thing in this module is sourced
//! from measured data — the authoring saving, and only as a *range* because two
//! published sources disagree about its sign. Everything else is the model's own
//! hypothesis and is labelled so at the point of use, not in a footnote.

/// Where a parameter's value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Measured in published work, with the citation rendered by [`Provenance::cite`].
    Sourced(&'static str),
    /// The model's own hypothesis. Plausible in sign and rough magnitude, and nothing
    /// more. A reader who treats a Declared figure as measured has been misled by the
    /// presentation, not by the arithmetic.
    Declared,
}

impl Provenance {
    pub fn cite(self) -> &'static str {
        match self {
            Provenance::Sourced(source) => source,
            Provenance::Declared => "Declared — a hypothesis, not a measurement",
        }
    }

    pub fn is_sourced(self) -> bool {
        matches!(self, Provenance::Sourced(_))
    }
}

/// The complete parameter set. Nothing here is hidden inside a function body, because a
/// constant buried in an expression is a constant that cannot be argued with.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    // ---- experimental design -------------------------------------------------
    /// Number of modules in the codebase.
    pub modules: usize,
    /// Horizon in months.
    pub horizon_months: usize,
    /// Complexity is spread evenly over this range across the modules. The mean is 1.6.
    pub complexity_min: f64,
    pub complexity_max: f64,
    /// Share of modules that AI authors in the AI scenario. The baseline authors none.
    pub ai_share: f64,

    // ---- authoring -----------------------------------------------------------
    /// Hours to author one complexity unit by hand.
    pub alpha: f64,
    /// Fraction of authoring time that AI removes. `0.0` means AI saves nothing.
    ///
    /// This is the one contested quantity in the model, and it is swept rather than
    /// fixed: METR measured a 19% *slowdown* for experienced developers in early 2025
    /// and between an 18% speed-up and a 4% slow-down a year later, while a vendor
    /// survey reports 13 hours a week saved. `sigma = 0.0` matches the measured result
    /// for experts; `sigma = 0.33` matches the self-report on a 40-hour week.
    pub savings: f64,

    // ---- comprehension -------------------------------------------------------
    /// Comprehension on creation when a human wrote the module.
    pub c_human: f64,
    /// Comprehension on creation when a machine wrote it.
    pub c_ai: f64,
    /// Fraction of the remaining gap closed by one review or one fix.
    pub review_gain: f64,
    /// Retention half-life in months for human-authored code, untouched.
    pub half_life_human: f64,
    /// Retention half-life in months for AI-authored code, untouched. The asymmetry
    /// between this and `half_life_human` **is** the hypothesis under test.
    pub half_life_ai: f64,

    // ---- review --------------------------------------------------------------
    /// Review hours per complexity unit at full comprehension.
    pub review_hours_per_unit: f64,
    /// How much review cost rises as comprehension falls. At `1.0` a review of
    /// zero-comprehension code costs twice a review of fully understood code.
    pub review_slope: f64,
    /// Comprehension at which the review gate catches everything.
    pub catch_threshold: f64,

    // ---- maintenance ---------------------------------------------------------
    /// Baseline maintenance events per module-month, per complexity unit.
    pub hazard: f64,
    /// Extra maintenance hazard carried by AI-authored code.
    pub ai_defect_multiplier: f64,
    /// Extra hazard from defects a failed review let through.
    pub leak_hazard: f64,
    /// Diagnosis hours per complexity unit at full comprehension.
    pub diag_hours_per_unit: f64,
    /// How much diagnosis cost rises as comprehension falls.
    pub diag_slope: f64,
    /// Fix hours per complexity unit, independent of comprehension.
    pub fix_hours_per_unit: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            modules: 10,
            horizon_months: 36,
            complexity_min: 0.2,
            complexity_max: 3.0,
            ai_share: 0.5,

            alpha: 10.0,
            savings: 0.15,

            c_human: 0.85,
            c_ai: 0.25,
            review_gain: 0.25,
            half_life_human: 24.0,
            half_life_ai: 9.0,

            review_hours_per_unit: 2.0,
            review_slope: 1.0,
            catch_threshold: 0.70,

            hazard: 0.08,
            ai_defect_multiplier: 0.1,
            leak_hazard: 0.10,
            diag_hours_per_unit: 3.0,
            diag_slope: 2.0,
            fix_hours_per_unit: 2.0,
        }
    }
}

impl Params {
    /// Complexity of the `index`-th module, spread evenly across the configured range.
    pub fn complexity(&self, index: usize) -> f64 {
        if self.modules <= 1 {
            return (self.complexity_min + self.complexity_max) / 2.0;
        }
        let t = index as f64 / (self.modules - 1) as f64;
        self.complexity_min + (self.complexity_max - self.complexity_min) * t
    }

    /// Whether the `index`-th module is AI-authored in the AI scenario.
    ///
    /// Spread across the complexity range rather than taking the first `ai_share` of
    /// them. The first version did the latter, and because complexity rises with index
    /// it handed AI the *cheapest* half of the codebase — which understated the
    /// authoring saving by about half and biased the whole model against the hypothesis.
    /// Found by noticing that the authoring-only channel produced +6 h where
    /// `10 x mean_complexity x 0.15` predicted +12 h. Recorded as a correction in
    /// `AI_AUTHORING_AND_COMPREHENSION.md` §2.1.
    pub fn is_ai_authored(&self, index: usize) -> bool {
        let before = (index as f64 * self.ai_share).floor();
        let after = ((index + 1) as f64 * self.ai_share).floor();
        after > before
    }

    /// The provenance of each parameter, in the order they appear in the struct.
    pub fn provenance(&self) -> Vec<(&'static str, String, Provenance)> {
        let d = |name: &'static str, value: String| (name, value, Provenance::Declared);
        vec![
            d("modules", self.modules.to_string()),
            d("horizon_months", self.horizon_months.to_string()),
            d("complexity range", format!("{} – {}", self.complexity_min, self.complexity_max)),
            d("ai_share", format!("{:.2}", self.ai_share)),
            d("alpha (authoring h/unit)", format!("{:.1}", self.alpha)),
            (
                "savings (authoring fraction removed)",
                format!("{:.2}", self.savings),
                Provenance::Sourced(
                    "Contested: METR 2025 measured 19% slower (CI +2%..+39%); METR 2026 \
                     measured -18% (CI -38%..+9%) and -4% (CI -15%..+9%); BairesDev Q3 2026 \
                     self-reports 13 h/week of a 40 h week. Swept, not fixed.",
                ),
            ),
            d("c_human (initial comprehension)", format!("{:.2}", self.c_human)),
            d("c_ai (initial comprehension)", format!("{:.2}", self.c_ai)),
            d("review_gain", format!("{:.2}", self.review_gain)),
            d("half_life_human (months)", format!("{:.1}", self.half_life_human)),
            d("half_life_ai (months)", format!("{:.1}", self.half_life_ai)),
            d("review_hours_per_unit", format!("{:.1}", self.review_hours_per_unit)),
            d("review_slope", format!("{:.1}", self.review_slope)),
            d("catch_threshold", format!("{:.2}", self.catch_threshold)),
            d("hazard (events/module-month)", format!("{:.3}", self.hazard)),
            d("ai_defect_multiplier", format!("{:.2}", self.ai_defect_multiplier)),
            d("leak_hazard", format!("{:.3}", self.leak_hazard)),
            d("diag_hours_per_unit", format!("{:.1}", self.diag_hours_per_unit)),
            d("diag_slope", format!("{:.1}", self.diag_slope)),
            d("fix_hours_per_unit", format!("{:.1}", self.fix_hours_per_unit)),
        ]
    }
}
