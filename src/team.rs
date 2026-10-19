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

//! The team externality: what a reduced schedule does to the people who absorb it.
//!
//! # Why this module exists
//!
//! Every other part of this tool prices a decision the user makes for themselves. This one prices
//! a decision they make *about someone else*, and the distinction is why the default is the way it
//! is.
//!
//! Until this module existed, a reduced work percentage was modelled as producing proportionally
//! less output, with the README saying that reduced availability was treated as having "no effect
//! on colleagues". That is not a neutral simplification. In the ordinary case a colleague absorbs
//! the work, so the tool was pricing a schedule that pushes cost onto other people as though it
//! were costless — and therefore **recommending it over a schedule that genuinely produces less**.
//! An incomplete model with no bias is a limitation; an incomplete model that systematically
//! favours externalising cost is a wrong answer, and this is the only such bias in the tool.
//!
//! # What it models, and what it refuses to model
//!
//! Three declared quantities: how many colleagues would absorb the work (`colleagues`), what share
//! of the reduction lands on them (`redistribution`), and optionally what a moved hour is worth to
//! the user (`externality_weight`).
//!
//! It computes hours moved and hours per colleague. It does **not** add them to the user's utility
//! unless the user supplies a weight, because that weight is a moral position. A tool that decided
//! a colleague-hour was worth 0.4 utility points would be smuggling a normative commitment in as
//! arithmetic — the same failure the direction framework in `knowledge_work` refuses. The default
//! weight is zero and is documented as a choice, not an omission.
//!
//! What the tool does instead is refuse to stay silent. Every result with a reduction carries a
//! [`Severity`], and the presentation layer prints a warning whenever the externality is either
//! material or unmodelled.
//!
//! # What this cannot tell you
//!
//! - Whether the work is genuinely redistributed or genuinely dropped. `redistribution` is declared,
//!   and the honest default when a team exists is 1.0 — the pessimistic case — because silence
//!   should not be read as "nothing happened".
//! - Anything about morale, resentment, or what the colleague's manager does about it.
//! - Whether the colleague's own workload was already sustainable.
//!
//! This is a crude model by design. It exists to stop the tool answering a question it was not
//! asked while leaving a more important one unanswered.

use crate::optimizer::WorkScenario;

/// Declared inputs for the externality calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeamParameters {
    /// Colleagues who would absorb redistributed work, **excluding the user**.
    ///
    /// Zero means "not modelled", which is the default and is reported as such rather than as
    /// "no effect". The two are different claims and the tool must not conflate them.
    pub colleagues: u32,
    /// Share of the reduction that lands on the team rather than simply not being done.
    ///
    /// Declared: the world does not tell us this. When a team exists the default is 1.0, because a
    /// tool that silently assumed 0.0 would be assuming away the problem.
    pub redistribution: f64,
    /// Utility points per hour per week moved, as the user values them.
    ///
    /// Zero by default. Any positive value is a normative commitment supplied by the user, and the
    /// tool prints it beside the number so it can be argued with.
    pub externality_weight: f64,
}

impl Default for TeamParameters {
    fn default() -> Self {
        TeamParameters::not_modelled()
    }
}

impl TeamParameters {
    /// The default: no team parameters supplied, so the externality is not quantified.
    pub fn not_modelled() -> Self {
        TeamParameters { colleagues: 0, redistribution: 1.0, externality_weight: 0.0 }
    }

    /// A team of `colleagues` absorbing a share `redistribution` of the reduction.
    pub fn with_team(colleagues: u32, redistribution: f64) -> Self {
        TeamParameters {
            colleagues,
            redistribution: redistribution.clamp(0.0, 1.0),
            externality_weight: 0.0,
        }
    }

    /// The same parameters, with the user's own valuation of a moved hour attached.
    pub fn weighed(mut self, externality_weight: f64) -> Self {
        self.externality_weight = externality_weight.max(0.0);
        self
    }

    /// Whether any colleague could absorb work at all.
    pub fn is_modelled(&self) -> bool {
        self.colleagues > 0
    }

    /// Compute the externality for a given work percentage.
    ///
    /// `full_time_hours` is the contract the percentage applies to, so the hours moved are measured
    /// against the job the user would otherwise be doing rather than against some other baseline.
    pub fn evaluate(&self, work_percentage: f64, full_time_hours: f64) -> TeamExternality {
        let reduction = (1.0 - work_percentage.clamp(0.0, 1.0)).max(0.0);
        let moved = reduction * full_time_hours.max(0.0) * self.redistribution.clamp(0.0, 1.0);
        let per_colleague =
            if self.colleagues > 0 { moved / f64::from(self.colleagues) } else { 0.0 };
        let severity = if reduction <= 0.0 {
            Severity::NoReduction
        } else if !self.is_modelled() {
            Severity::NotModelled
        } else if per_colleague >= SEVERE_HOURS {
            Severity::Severe
        } else if per_colleague >= MATERIAL_HOURS {
            Severity::Material
        } else {
            Severity::Negligible
        };
        TeamExternality {
            colleagues: self.colleagues,
            redistribution: self.redistribution,
            externality_weight: self.externality_weight,
            potential_hours_per_week: moved,
            hours_moved_per_week: if self.is_modelled() { moved } else { 0.0 },
            hours_per_colleague_per_week: per_colleague,
            weighed_penalty: self.externality_weight * moved,
            severity,
        }
    }

    /// Every parameter with its provenance, in the shape the rest of the repository uses.
    pub fn provenance_table(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "colleagues",
                "declared: the number of people who would absorb redistributed work is an \
                 organisational fact the user knows and the tool does not",
            ),
            (
                "redistribution",
                "declared: whether reduced output is absorbed by others or simply not produced \
                 cannot be observed from here; the default when a team exists is the pessimistic \
                 one",
            ),
            (
                "externality weight",
                "declared and zero by default: what a colleague-hour is worth is a moral position, \
                 not a measurement, so the tool reports the hours and lets the user price them",
            ),
        ]
    }
}

/// Hours per colleague per week at or above which the externality is called material.
///
/// Declared. One hour a week is roughly a working week a year, which is the point at which a
/// redistribution stops being a rounding error in someone else's calendar.
pub const MATERIAL_HOURS: f64 = 1.0;
/// Hours per colleague per week at or above which it is called severe: half a day a week.
pub const SEVERE_HOURS: f64 = 4.0;

/// How serious the externality is, including the case where it was not modelled at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    /// The work percentage is full time; nothing is moved and nothing needs saying.
    NoReduction,
    /// A reduction with no team parameters: the externality exists and is not quantified.
    NotModelled,
    /// Quantified and small.
    Negligible,
    /// Quantified and at least [`MATERIAL_HOURS`] per colleague per week.
    Material,
    /// Quantified and at least [`SEVERE_HOURS`] per colleague per week.
    Severe,
}

impl Severity {
    /// Whether the presentation layer must print a warning for this severity.
    ///
    /// `Negligible` is still printed as a figure; it does not need to shout. Everything else does,
    /// including `NotModelled` — which is the case the tool used to be silent about.
    pub fn requires_warning(self) -> bool {
        !matches!(self, Severity::NoReduction | Severity::Negligible)
    }

    pub fn label(self) -> &'static str {
        match self {
            Severity::NoReduction => "no reduction",
            Severity::NotModelled => "not modelled",
            Severity::Negligible => "negligible",
            Severity::Material => "material",
            Severity::Severe => "severe",
        }
    }
}

/// What a reduced schedule moves onto colleagues.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TeamExternality {
    pub colleagues: u32,
    pub redistribution: f64,
    pub externality_weight: f64,
    /// Hours per week that a team would absorb if one were modelled. Always computed, so a
    /// user who has not named a team can still see the size of the exposure.
    pub potential_hours_per_week: f64,
    /// Hours per week actually attributed to colleagues. Zero when no team is modelled, because
    /// "hours moved onto colleagues" is not a claim that can be made about a team that was never
    /// named.
    pub hours_moved_per_week: f64,
    /// The same, per colleague.
    pub hours_per_colleague_per_week: f64,
    /// The user's own valuation of the moved hours, in utility points. Zero unless they set a
    /// weight, and printed so the value can be argued with.
    pub weighed_penalty: f64,
    pub severity: Severity,
}

impl TeamExternality {
    /// The "nothing is known and nothing was moved" value, for full-time scenarios and for callers
    /// that supply no team parameters.
    pub fn none() -> Self {
        TeamParameters::not_modelled().evaluate(1.0, 0.0)
    }

    /// Whether a warning must accompany any presentation of this scenario.
    pub fn requires_warning(&self) -> bool {
        self.severity.requires_warning()
    }

    /// One sentence stating the externality, for use wherever a scenario is summarised.
    pub fn summary(&self) -> String {
        match self.severity {
            Severity::NoReduction => "No work is moved onto colleagues.".to_string(),
            Severity::NotModelled => format!(
                "NOT MODELLED: this reduction is {:.1} h/week if a team absorbs it, and this \
                 result does not account for that.",
                self.potential_hours_per_week
            ),
            _ => format!(
                "{:.1} h/week moved onto {} colleague{} ({:.2} h/week each): {}",
                self.hours_moved_per_week,
                self.colleagues,
                if self.colleagues == 1 { "" } else { "s" },
                self.hours_per_colleague_per_week,
                self.severity.label()
            ),
        }
    }
}

/// The externality attached to a scenario, or the unmodelled value when none was supplied.
///
/// Kept as a function so the optimizer has one place to call and the "not modelled" fallback
/// cannot be forgotten at a construction site.
pub fn for_scenario(
    parameters: &TeamParameters,
    work_percentage: f64,
    full_time_hours: f64,
) -> TeamExternality {
    parameters.evaluate(work_percentage, full_time_hours)
}

/// The externality of a built scenario, which is what the presentation layer has.
pub fn of(scenario: &WorkScenario) -> TeamExternality {
    scenario.team_externality
}
