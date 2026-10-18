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

//! How much the recommended work percentage depends on assumptions the user never chose.
//!
//! # The question this answers
//!
//! The tool reports *an* optimum, and the review's sharpest usability point is that a single
//! optimum is not a decision: "80% scores 31.75" says nothing about whether 75% would have scored
//! 31.7 or 25. A recommendation is only as good as its margin, and the margin is only as good as
//! the assumptions it was computed under.
//!
//! So this module re-runs the search under assumptions the user did not set — each utility weight
//! halved and doubled, and the whole consumption basket moved by the same factors — and reports
//! what happens to the answer. The output is either a stable optimum (worth acting on) or a range
//! (worth arguing about), and it says which.
//!
//! # Why the weights are the right thing to perturb
//!
//! Because they are the only inputs in the model with no empirical content whatsoever. The salary
//! is observed, the tax schedule is sourced, the canton is chosen. The weights are a declaration
//! about what the user values, and the review is right that a tool whose selling point is
//! auditability should show what happens when the declarations move.
//!
//! # What this is not
//!
//! It is not a confidence interval. Nothing here samples an uncertainty distribution, because
//! nobody has one for "how much do you value family time". It is a **sensitivity check on declared
//! inputs**: it tells you which declarations the answer rests on, and it cannot tell you how likely
//! any of them is to be wrong.

use crate::optimizer::{LifeOptimizer, OptimizerConfig};
use crate::requirements::{PersonalRequirements, PreferenceWeights};

/// The five utility criteria, in the order [`PreferenceWeights`] stores them.
pub const CRITERIA: [&str; 5] = ["consumption", "leisure", "family", "health", "security"];

/// The factors each declared input is moved by.
///
/// Halving and doubling is a big move on purpose: a test that finds no change under a factor of
/// 1.05 has found that the model is insensitive to the assumption, which is not the same claim.
pub const FACTORS: [f64; 2] = [0.5, 2.0];

/// Whether the perturbed input was a utility weight or a consumption requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// One of the five utility weights.
    Weight,
    /// The household's consumption basket, which is the needs-like input the optimizer does use.
    Requirement,
}

impl InputKind {
    pub fn label(self) -> &'static str {
        match self {
            InputKind::Weight => "weight",
            InputKind::Requirement => "requirement",
        }
    }
}

/// One re-run of the search under a moved assumption.
#[derive(Debug, Clone, PartialEq)]
pub struct Perturbation {
    pub kind: InputKind,
    pub name: &'static str,
    pub factor: f64,
    /// The optimal work percentage under the moved assumption, or `None` when nothing was feasible.
    pub optimal: Option<f64>,
}

impl Perturbation {
    /// How far this perturbation moved the answer from the baseline, in percentage points.
    pub fn shift(&self, base: Option<f64>) -> Option<f64> {
        match (base, self.optimal) {
            (Some(base), Some(optimal)) => Some((optimal - base) * 100.0),
            _ => None,
        }
    }
}

/// The whole reading.
#[derive(Debug, Clone)]
pub struct RobustnessReport {
    /// The weights the answer was computed under, so they can be printed rather than assumed.
    pub weights: PreferenceWeights,
    pub base: Option<f64>,
    pub perturbations: Vec<Perturbation>,
    /// The smallest and largest optimal percentage seen across every perturbation.
    pub range: Option<(f64, f64)>,
    /// Whether any perturbation changed the answer.
    pub stable: bool,
    /// The lowest and highest percentage the search was allowed to consider.
    ///
    /// Carried because an optimum that sits on this edge is not a finding: it says "the lowest
    /// percentage we tried", and no perturbation of the weights can move it inward, so reporting
    /// it as stable would be the most confident possible statement about the least informative
    /// possible answer.
    pub searched: (f64, f64),
}

impl RobustnessReport {
    /// Whether any perturbation produced no feasible scenario at all, which is a harder finding
    /// than a shifted percentage: the recommendation does not exist under that assumption.
    pub fn any_infeasible(&self) -> bool {
        self.perturbations
            .iter()
            .any(|perturbation| perturbation.optimal.is_none())
    }

    /// The widest shift, in percentage points.
    pub fn widest_shift(&self) -> Option<f64> {
        self.perturbations
            .iter()
            .filter_map(|perturbation| perturbation.shift(self.base))
            .fold(None, |acc: Option<f64>, shift| {
                Some(acc.map_or(shift.abs(), |best: f64| best.max(shift.abs())))
            })
    }

    /// Whether the baseline optimum sits on the edge of the searched range.
    pub fn at_search_edge(&self) -> bool {
        match self.base {
            Some(base) => {
                (base - self.searched.0).abs() < 1e-9 || (base - self.searched.1).abs() < 1e-9
            }
            None => false,
        }
    }

    /// One sentence stating whether the recommendation survives its own assumptions.
    pub fn verdict(&self) -> String {
        let Some(base) = self.base else {
            return "No feasible percentage was found, so there is nothing to stress.".to_string();
        };
        if self.at_search_edge() {
            return format!(
                "The answer is {:.0}%, which is the {} of the percentages tested ({:.0}% to \
                 {:.0}%). An optimum on the edge is not a finding -- it may mean the best answer \
                 lies outside the search -- and the weights cannot move it inward, so the margin \
                 above is uninformative until the range is widened.",
                base * 100.0,
                if (base - self.searched.0).abs() < 1e-9 { "low end" } else { "high end" },
                self.searched.0 * 100.0,
                self.searched.1 * 100.0
            );
        }
        if self.any_infeasible() {
            return format!(
                "The answer is {:.0}% here, but at least one plausible change to your stated \
                 preferences leaves no feasible percentage at all.",
                base * 100.0
            );
        }
        match self.range {
            Some((low, high)) if (high - low).abs() < 1e-9 => format!(
                "The answer is {:.0}% and it does not move under any perturbation tested. That is \
                 the strongest statement this tool can make.",
                base * 100.0
            ),
            Some((low, high)) => {
                let widest = self.widest_shift().unwrap_or(0.0);
                format!(
                    "The answer is {:.0}%, but it moves between {:.0}% and {:.0}% when a single \
                     declared input is moved (widest shift {:.0} points). The table says which \
                     one. Treat the margin, not the leader, as the finding.",
                    base * 100.0,
                    low * 100.0,
                    high * 100.0,
                    widest
                )
            }
            None => "No perturbation produced a feasible percentage.".to_string(),
        }
    }
}

/// The five weights as an array, in [`CRITERIA`] order.
fn weights_array(weights: &PreferenceWeights) -> [f64; 5] {
    [
        weights.consumption,
        weights.leisure,
        weights.family,
        weights.health,
        weights.security,
    ]
}

/// Build weights from an array, renormalising so the vector still sums to one.
///
/// Renormalising matters: doubling one weight without renormalising would change the *scale* of
/// the total utility as well as its composition, and every scenario would shift together. The
/// question is what happens when the user cares twice as much about family relative to the rest,
/// which is a change in ratio, not in magnitude.
fn weights_from(values: [f64; 5]) -> PreferenceWeights {
    let total: f64 = values.iter().map(|value| value.max(0.0)).sum();
    let scale = if total > 0.0 { 1.0 / total } else { 0.0 };
    PreferenceWeights {
        consumption: values[0].max(0.0) * scale,
        leisure: values[1].max(0.0) * scale,
        family: values[2].max(0.0) * scale,
        health: values[3].max(0.0) * scale,
        security: values[4].max(0.0) * scale,
    }
}

/// Move one weight by `factor` and renormalise.
pub fn perturb_weights(weights: &PreferenceWeights, index: usize, factor: f64) -> PreferenceWeights {
    let mut values = weights_array(weights);
    if index < values.len() {
        values[index] = (values[index] * factor).max(0.0);
    }
    weights_from(values)
}

/// Scale every money field of the consumption basket by `factor`.
///
/// This is the needs-like input the optimizer actually uses: the mandatory floor that decides
/// whether a percentage is feasible at all. It is not the retirement-needs figure, which lives in
/// the adequacy report and cannot move a work-percentage optimum — a distinction the tool now
/// states rather than leaves for the reader to discover.
pub fn perturb_requirements(
    requirements: &PersonalRequirements,
    factor: f64,
) -> PersonalRequirements {
    let mut scaled = requirements.clone();
    scaled.housing *= factor;
    scaled.food *= factor;
    scaled.transport *= factor;
    scaled.insurance *= factor;
    scaled.childcare *= factor;
    scaled.healthcare *= factor;
    scaled.education *= factor;
    scaled.vacation *= factor;
    scaled.savings_goal *= factor;
    scaled.discretionary *= factor;
    scaled
}

/// Run the base search and every perturbation, and report what moved.
pub fn analyse(config: &OptimizerConfig, candidates: &[f64]) -> RobustnessReport {
    let search = |config: &OptimizerConfig| -> Option<f64> {
        let optimizer = LifeOptimizer::new(config.clone());
        optimizer
            .search_outcome(candidates)
            .map(|outcome| outcome.scenario.work_percentage)
    };

    let base = search(config);
    let mut perturbations = Vec::new();

    for (index, name) in CRITERIA.iter().enumerate() {
        for factor in FACTORS {
            let mut moved = config.clone();
            moved.preferences = perturb_weights(&config.preferences, index, factor);
            perturbations.push(Perturbation {
                kind: InputKind::Weight,
                name,
                factor,
                optimal: search(&moved),
            });
        }
    }

    for factor in FACTORS {
        let mut moved = config.clone();
        moved.requirements = perturb_requirements(&config.requirements, factor);
        perturbations.push(Perturbation {
            kind: InputKind::Requirement,
            name: "consumption basket",
            factor,
            optimal: search(&moved),
        });
    }

    let optimal: Vec<f64> = perturbations
        .iter()
        .filter_map(|perturbation| perturbation.optimal)
        .collect();
    let range = if optimal.is_empty() {
        None
    } else {
        let low = optimal.iter().cloned().fold(f64::INFINITY, f64::min);
        let high = optimal.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Some((low.min(base.unwrap_or(high)), high.max(base.unwrap_or(low))))
    };
    let stable = match (base, range) {
        (Some(_), Some((low, high))) => (high - low).abs() < 1e-9,
        _ => false,
    };

    let searched = (
        candidates.iter().cloned().fold(f64::INFINITY, f64::min),
        candidates.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    );
    RobustnessReport {
        weights: config.preferences.clone(),
        base,
        perturbations,
        range,
        stable,
        searched,
    }
}

/// Where each perturbed input comes from, for the provenance note beside the block.
pub fn provenance_note() -> &'static str {
    "The weights are declared, not measured: nothing tells the tool how much you value family \
     time. Halving and doubling each one, and moving the whole consumption basket the same way, is \
     a check on how much the answer rests on those declarations -- not a confidence interval."
}
