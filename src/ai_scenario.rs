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

//! Named AI-adoption scenarios, so the common case does not need eight coefficients.
//!
//! # The problem this solves
//!
//! The tool exposed eight flags for the AI-productivity model, every one of them a quantity its
//! own documentation calls illustrative. Asking a user to supply eight unmeasured coefficients is
//! a way of transferring the modeller's uncertainty onto the person least able to resolve it, and
//! the review was right that this is a lot of surface for coefficients that predict nothing.
//!
//! The answer is not to delete the model — the question is real — but to give it a front door:
//! three named scenarios whose numbers are printed when they are used. The individual flags
//! remain for anyone who wants to move one coefficient without moving the others.
//!
//! # Where the numbers come from
//!
//! `AI_AS_CATALYST.md` §1 and §4 record the published range for AI-assisted software work: a
//! spread of **75 percentage points, from 19% slower to 56% faster**, with METR's developers
//! forecasting 24% and Google's own study running from 3% slower to 40% faster. The three
//! scenarios bracket that range rather than sitting inside it at a favourite point:
//!
//! | scenario | productivity gain | where it sits |
//! |---|---|---|
//! | sceptical | 10% | below most published estimates |
//! | neutral | 25% | near METR's pre-study forecast |
//! | optimistic | 45% | near the top of the published range |
//!
//! The *anchors* are sourced from that document; the mapping from "published gain" to this model's
//! coefficients is declared, because the studies measure a different quantity under different
//! conditions. Stating which is which is the whole point of the table.

/// One named scenario: the coefficients it sets, and what it is meant to represent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scenario {
    /// The name used on the command line and printed in the report.
    pub name: &'static str,
    /// One line describing the position it represents.
    pub description: &'static str,
    /// Expected productivity gain, as a decimal. The anchor from `AI_AS_CATALYST.md`.
    pub productivity_gain: f64,
    /// The optimistic end of the same gain's range.
    pub productivity_gain_high: f64,
    /// Share of the gain that survives verification and rework.
    pub quality_retention: f64,
    /// Employer's required output relative to the pre-AI baseline.
    pub required_output_index: f64,
    /// How sensitive quality is to compressing the schedule.
    pub compression_quality_sensitivity: f64,
}

impl Scenario {
    /// The coefficients with their declared justifications, for the provenance note.
    pub fn provenance_table(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "productivity gain",
                "anchored to the published range in AI_AS_CATALYST.md 4; the mapping to this \
                 model's coefficient is declared",
            ),
            (
                "quality retention",
                "declared: the share of the gain surviving verification is the quantity \
                 AI_AS_CATALYST.md reports as unmeasured",
            ),
            (
                "required output index",
                "declared: what an employer asks for is a choice, not a measurement",
            ),
            (
                "compression sensitivity",
                "declared: no published estimate exists for it",
            ),
        ]
    }
}

/// The three scenarios, in order from least to most optimistic.
pub const SCENARIOS: [Scenario; 3] = [
    Scenario {
        name: "sceptical",
        description: "AI delivers less than most published estimates, and the gain does not survive review",
        productivity_gain: 0.10,
        productivity_gain_high: 0.20,
        quality_retention: 0.60,
        required_output_index: 1.10,
        compression_quality_sensitivity: 0.50,
    },
    Scenario {
        name: "neutral",
        description: "the published mid-range, with a gain that mostly survives review",
        productivity_gain: 0.25,
        productivity_gain_high: 0.45,
        quality_retention: 0.85,
        required_output_index: 1.20,
        compression_quality_sensitivity: 0.75,
    },
    Scenario {
        name: "optimistic",
        description: "near the top of the published range, with the gain largely intact",
        productivity_gain: 0.45,
        // 55%, not 60%: AI_AS_CATALYST.md records the published spread as topping out at 56%
        // faster, and a preset that asserts more than the evidence supports is the error this
        // whole round is about. The test caught this constant at 0.60.
        productivity_gain_high: 0.55,
        quality_retention: 0.95,
        required_output_index: 1.35,
        compression_quality_sensitivity: 0.90,
    },
];

/// Look a scenario up by name, case-insensitively.
///
/// An unknown name returns `None` rather than a default, so a typo cannot silently change the
/// model under the user: the caller reports it and refuses.
pub fn by_name(name: &str) -> Option<Scenario> {
    let wanted = name.trim().to_ascii_lowercase();
    SCENARIOS
        .iter()
        .find(|scenario| scenario.name == wanted)
        .copied()
}

/// Every scenario name, for help text and error messages.
pub fn names() -> Vec<&'static str> {
    SCENARIOS.iter().map(|scenario| scenario.name).collect()
}

/// A one-line summary of what a scenario sets, for printing beside the result.
pub fn describe(scenario: &Scenario) -> String {
    format!(
        "{}: gain {:.0}% (up to {:.0}%), {:.0}% of it surviving review, employer asking {:.0}% of \
         the old output",
        scenario.name,
        scenario.productivity_gain * 100.0,
        scenario.productivity_gain_high * 100.0,
        scenario.quality_retention * 100.0,
        scenario.required_output_index * 100.0
    )
}

/// Whether a name is a known scenario, for validating a flag before it is used.
pub fn is_known(name: &str) -> bool {
    by_name(name).is_some()
}
