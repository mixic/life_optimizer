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

//! The grid sweep, and the verdict rule that was fixed before the grid was run.
//!
//! The rule lives here rather than in the report so that it is code a reader can check
//! against `AI_AUTHORING_AND_COMPREHENSION.md` §1.5, and so that changing it changes
//! a test rather than a sentence.

use crate::model::{benefit_curve, crossover_month, run, Scenario};
use crate::params::Params;

/// The pre-registered central cell.
pub const CENTRAL_SAVINGS: f64 = 0.15;
pub const CENTRAL_HALF_LIFE: f64 = 9.0;
pub const CENTRAL_EPSILON: f64 = 0.1;

/// The pre-registered grid axes, shared by the report and the export so a figure
/// cannot be drawn on a different grid from the one the verdict was computed on.
pub const SAVINGS_LEVELS: [f64; 3] = [0.00, 0.15, 0.33];
pub const HALF_LIFE_LEVELS: [f64; 5] = [6.0, 9.0, 12.0, 18.0, 24.0];
pub const EPSILON_LEVELS: [f64; 4] = [0.00, 0.10, 0.20, 0.50];

/// The central cell: `base` with the pre-registered sweep values applied.
///
/// One definition. The report, the analyses and the CSV export all go through it, which
/// is the only way the printed table and the PNG can be guaranteed to describe the same
/// run.
pub fn central_cell(base: &Params) -> Params {
    let mut params = base.clone();
    params.savings = CENTRAL_SAVINGS;
    params.half_life_ai = CENTRAL_HALF_LIFE;
    params.ai_defect_multiplier = CENTRAL_EPSILON;
    params
}

fn same(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// One cell of the grid.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub savings: f64,
    pub half_life_ai: f64,
    pub epsilon: f64,
    /// First month at which cumulative benefit turns negative.
    pub crossover: Option<usize>,
    /// Cumulative benefit at the horizon. Negative means AI cost more over the run.
    pub benefit_end: f64,
}

impl Cell {
    pub fn is_central(&self) -> bool {
        same(self.savings, CENTRAL_SAVINGS)
            && same(self.half_life_ai, CENTRAL_HALF_LIFE)
            && same(self.epsilon, CENTRAL_EPSILON)
    }

    /// `σ ≥ 0.15`, `h_ai ≥ 9`, `ε ≤ 0.2` — the region the pre-registration calls moderate.
    pub fn is_moderate(&self) -> bool {
        self.savings >= CENTRAL_SAVINGS - 1e-9
            && self.half_life_ai >= CENTRAL_HALF_LIFE - 1e-9
            && self.epsilon <= 0.2 + 1e-9
    }

    /// `σ = 0`, `h_ai = 6`, or `ε = 0.5` — the corner that does not count as evidence.
    pub fn is_extreme(&self) -> bool {
        same(self.savings, 0.0) || same(self.half_life_ai, 6.0) || same(self.epsilon, 0.5)
    }
}

/// The pre-registered verdict.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// The central cell crosses over.
    Supported,
    /// The central cell does not, but some moderate cell does.
    ConditionallySupported { sigma_star: Option<f64> },
    /// Only the extreme corner crosses, or nothing does.
    NotSupported,
}

impl Verdict {
    pub fn headline(&self) -> String {
        match self {
            Verdict::Supported => {
                "SUPPORTED — cumulative AI benefit turns negative in the central cell".to_string()
            }
            Verdict::ConditionallySupported { sigma_star: Some(sigma) } => format!(
                "CONDITIONALLY SUPPORTED — no crossover in the central cell, but some moderate \
                 cell crosses over. Above sigma = {sigma:.2} the hypothesis fails across the \
                 moderate region."
            ),
            Verdict::ConditionallySupported { sigma_star: None } => {
                "CONDITIONALLY SUPPORTED — no crossover in the central cell, but some moderate \
                 cell crosses over."
                    .to_string()
            }
            Verdict::NotSupported => {
                "NOT SUPPORTED — crossover occurs only in the extreme corner, or not at all"
                    .to_string()
            }
        }
    }
}

/// Run the grid.
///
/// The baseline does not depend on `savings`, `half_life_ai` or `epsilon` — it is
/// all-human — so it is computed once per `half_life_human` and reused. That is not an
/// optimisation but a correctness property: it guarantees the two runs differ only in
/// authoring mode.
pub fn sweep(base: &Params, savings: &[f64], half_lives: &[f64], epsilons: &[f64]) -> Vec<Cell> {
    let human = run(base, Scenario::Human);
    let mut cells = Vec::with_capacity(savings.len() * half_lives.len() * epsilons.len());

    for &s in savings {
        for &h in half_lives {
            for &e in epsilons {
                let mut params = base.clone();
                params.savings = s;
                params.half_life_ai = h;
                params.ai_defect_multiplier = e;

                let ai = run(&params, Scenario::Ai);
                let benefit = benefit_curve(&human, &ai);
                cells.push(Cell {
                    savings: s,
                    half_life_ai: h,
                    epsilon: e,
                    crossover: crossover_month(&human, &ai),
                    benefit_end: benefit.last().copied().unwrap_or(0.0),
                });
            }
        }
    }
    cells
}

/// Apply the pre-registered decision rule.
pub fn verdict(cells: &[Cell], savings_levels: &[f64]) -> Verdict {
    if cells.iter().any(|c| c.is_central() && c.crossover.is_some()) {
        return Verdict::Supported;
    }

    let any_moderate = cells.iter().any(|c| c.is_moderate() && c.crossover.is_some());
    if !any_moderate {
        return Verdict::NotSupported;
    }

    // sigma* is the smallest level at or above which nothing in the moderate region
    // crosses over: the point past which the hypothesis stops holding.
    let mut sigma_star = None;
    let mut levels: Vec<f64> =
        savings_levels.iter().copied().filter(|s| *s >= CENTRAL_SAVINGS - 1e-9).collect();
    levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for level in levels {
        let crosses = cells.iter().any(|c| {
            same(c.savings, level)
                && c.half_life_ai >= CENTRAL_HALF_LIFE - 1e-9
                && c.epsilon <= 0.2 + 1e-9
                && c.crossover.is_some()
        });
        if !crosses {
            sigma_star = Some(level);
            break;
        }
    }

    Verdict::ConditionallySupported { sigma_star }
}

/// The falsification condition from §1.5, as a predicate rather than a sentence.
///
/// True when nothing in the admissible region crosses over: then the hypothesis, as
/// modelled, is refuted.
pub fn is_falsified(cells: &[Cell]) -> bool {
    !cells.iter().any(|c| {
        c.savings >= CENTRAL_SAVINGS - 1e-9
            && c.half_life_ai >= 6.0 - 1e-9
            && c.epsilon <= 0.5 + 1e-9
            && c.crossover.is_some()
    })
}

/// One row of the one-at-a-time sensitivity analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SensitivityRow {
    pub name: &'static str,
    pub base: f64,
    pub low: f64,
    pub high: f64,
    /// Change in cumulative benefit at the horizon between the two perturbations.
    pub swing: f64,
}

/// Benefit at the horizon for one parameter set, on the central cell.
fn benefit_at_horizon(p: &Params) -> f64 {
    let human = run(p, Scenario::Human);
    let ai = run(p, Scenario::Ai);
    benefit_curve(&human, &ai).last().copied().unwrap_or(0.0)
}

/// Perturb each parameter by a factor and see how much the answer moves.
///
/// One at a time, ±50%, on the central cell. This is the answer to "which parameter
/// does the conclusion depend on", and it is reported instead of a point estimate
/// because the parameters are Declared and their values are not knowledge.
pub fn sensitivity(base: &Params) -> Vec<SensitivityRow> {
    let central = central_cell(base);

    type Apply = fn(&mut Params, f64);
    let knobs: Vec<(&'static str, f64, Apply)> = vec![
        ("c_ai (initial comprehension)", central.c_ai, |p, v| p.c_ai = v),
        ("c_human (initial comprehension)", central.c_human, |p, v| p.c_human = v),
        ("catch_threshold", central.catch_threshold, |p, v| p.catch_threshold = v),
        ("leak_hazard", central.leak_hazard, |p, v| p.leak_hazard = v),
        ("half_life_ai (months)", central.half_life_ai, |p, v| p.half_life_ai = v),
        ("half_life_human (months)", central.half_life_human, |p, v| p.half_life_human = v),
        ("ai_defect_multiplier", central.ai_defect_multiplier, |p, v| p.ai_defect_multiplier = v),
        ("diag_slope", central.diag_slope, |p, v| p.diag_slope = v),
        ("review_slope", central.review_slope, |p, v| p.review_slope = v),
        ("review_gain", central.review_gain, |p, v| p.review_gain = v),
        ("hazard", central.hazard, |p, v| p.hazard = v),
        ("review_hours_per_unit", central.review_hours_per_unit, |p, v| {
            p.review_hours_per_unit = v
        }),
        ("diag_hours_per_unit", central.diag_hours_per_unit, |p, v| p.diag_hours_per_unit = v),
        ("fix_hours_per_unit", central.fix_hours_per_unit, |p, v| p.fix_hours_per_unit = v),
        ("alpha (authoring h/unit)", central.alpha, |p, v| p.alpha = v),
        ("savings (authoring fraction)", central.savings, |p, v| p.savings = v),
    ];

    let baseline = benefit_at_horizon(&central);
    let mut rows = Vec::with_capacity(knobs.len());
    for (name, value, apply) in knobs {
        let mut low_params = central.clone();
        apply(&mut low_params, (value * 0.5).max(0.0));
        let mut high_params = central.clone();
        apply(&mut high_params, value * 1.5);

        // Comprehensions are probabilities; a 50% perturbation must not leave [0, 1].
        let clamp = |p: &mut Params| {
            p.c_ai = p.c_ai.clamp(0.0, 1.0);
            p.c_human = p.c_human.clamp(0.0, 1.0);
            p.catch_threshold = p.catch_threshold.clamp(0.01, 1.0);
            p.review_gain = p.review_gain.clamp(0.0, 1.0);
            p.half_life_ai = p.half_life_ai.max(0.5);
            p.half_life_human = p.half_life_human.max(0.5);
        };
        clamp(&mut low_params);
        clamp(&mut high_params);

        let low = benefit_at_horizon(&low_params);
        let high = benefit_at_horizon(&high_params);
        rows.push(SensitivityRow { name, base: baseline, low, high, swing: (high - low).abs() });
    }
    rows.sort_by(|a, b| b.swing.partial_cmp(&a.swing).unwrap());
    rows
}
