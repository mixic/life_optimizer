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

//! The analyses the report prints and the figures draw.
//!
//! These live in the library rather than in `main.rs` for one reason: a figure and a
//! table that disagree are worse than either alone, and the only way to make that
//! impossible is for both to call the same function. `tools/plot_comprehension_cost.py`
//! reads the CSV that `--export` writes from these functions, so the PNGs and the
//! printed report are the same numbers.

use crate::model::{benefit_curve, crossover_month, run, run_stochastic, Scenario};
use crate::params::Params;
use crate::sweep::{central_cell, CENTRAL_EPSILON, CENTRAL_HALF_LIFE, CENTRAL_SAVINGS};

/// Cumulative benefit at the horizon, on one parameter set.
pub fn benefit_at_horizon(params: &Params) -> f64 {
    let human = run(params, Scenario::Human);
    let ai = run(params, Scenario::Ai);
    benefit_curve(&human, &ai).last().copied().unwrap_or(0.0)
}

/// One step of the channel decomposition.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRow {
    pub label: &'static str,
    pub benefit: f64,
    /// Change from the previous step. `None` on the first row, which has no predecessor.
    pub delta: Option<f64>,
}

/// The central cell with the comprehension channels switched on one at a time.
///
/// The first row starts neutral — AI code understood and retained exactly as well as
/// human code — so it isolates the authoring saving. Each later row restores one
/// mechanism, and the delta is that mechanism's cost. This is the decomposition the
/// report quotes, and it exists because the total says nothing about where it came from.
pub fn channels(base: &Params) -> Vec<ChannelRow> {
    let mut staged = central_cell(base);
    // Neutral to begin with: AI code understood and retained exactly as well as human code,
    // so the first row isolates the authoring saving.
    staged.c_ai = staged.c_human;
    staged.half_life_ai = staged.half_life_human;
    staged.ai_defect_multiplier = 0.0;
    staged.leak_hazard = 0.0;

    // The comprehension gap costs hours through two separate channels, and an earlier version
    // of this function ran them together in one row labelled "review". A referee asked whether
    // review was incremental or reallocated, and the answer exposed the label: review excess is
    // about 11 hours, while the row it sat in was 61. Holding the diagnosis slope at zero while
    // the gap opens isolates the review term; restoring it adds the diagnosis term.
    let diagnosis_slope = staged.diag_slope;

    let mut rows: Vec<ChannelRow> = Vec::new();
    let mut previous: Option<f64> = None;
    let push = |label: &'static str,
                params: &Params,
                rows: &mut Vec<ChannelRow>,
                previous: &mut Option<f64>| {
        let benefit = benefit_at_horizon(params);
        rows.push(ChannelRow { label, benefit, delta: previous.map(|p| benefit - p) });
        *previous = Some(benefit);
    };

    push("authoring saving only", &staged, &mut rows, &mut previous);
    staged.c_ai = base.c_ai;
    staged.diag_slope = 0.0;
    push("+ review of code understood less well", &staged, &mut rows, &mut previous);
    staged.diag_slope = diagnosis_slope;
    push("+ diagnosis of code understood less well", &staged, &mut rows, &mut previous);
    staged.half_life_ai = base.half_life_ai;
    push("+ faster retention loss", &staged, &mut rows, &mut previous);
    staged.ai_defect_multiplier = base.ai_defect_multiplier;
    push("+ AI defect multiplier", &staged, &mut rows, &mut previous);
    staged.leak_hazard = base.leak_hazard;
    push("+ review leakage", &staged, &mut rows, &mut previous);

    rows
}

/// One row of the break-even sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct BreakevenRow {
    pub c_ai: f64,
    pub benefit: f64,
    pub crossover: Option<usize>,
    /// Mean comprehension across modules at the horizon, AI scenario.
    pub mean_comprehension: f64,
}

/// Vary only the comprehension an AI-authored module starts with.
pub fn breakeven(base: &Params) -> Vec<BreakevenRow> {
    let mut rows = Vec::with_capacity(16);
    for step in 0..=15 {
        let c_ai = 0.10 + step as f64 * 0.05;
        let mut params = central_cell(base);
        params.c_ai = c_ai;

        let human = run(&params, Scenario::Human);
        let ai = run(&params, Scenario::Ai);
        rows.push(BreakevenRow {
            c_ai,
            benefit: benefit_curve(&human, &ai).last().copied().unwrap_or(0.0),
            crossover: crossover_month(&human, &ai),
            mean_comprehension: ai.months.last().map(|m| m.comprehension_mean).unwrap_or(0.0),
        });
    }
    rows
}

/// The first `c_ai` at which the run stops losing, if any does.
pub fn breakeven_threshold(rows: &[BreakevenRow]) -> Option<f64> {
    rows.iter().find(|row| row.benefit >= 0.0).map(|row| row.c_ai)
}

/// One seeded path of the Monte Carlo.
#[derive(Debug, Clone, PartialEq)]
pub struct McRow {
    pub seed: u64,
    pub crossover: Option<usize>,
    pub benefit: f64,
}

/// Draw real Poisson arrivals rather than charging expected values.
pub fn monte_carlo(base: &Params, paths: usize) -> Vec<McRow> {
    let params = central_cell(base);
    let human = run(&params, Scenario::Human);
    (0..paths as u64)
        .map(|seed| {
            let ai = run_stochastic(&params, Scenario::Ai, seed);
            McRow {
                seed,
                crossover: crossover_month(&human, &ai),
                benefit: benefit_curve(&human, &ai).last().copied().unwrap_or(0.0),
            }
        })
        .collect()
}

/// The central cell as printed, for callers that want the numbers behind a figure.
pub fn central_summary(base: &Params) -> (f64, Option<usize>) {
    let params = central_cell(base);
    let human = run(&params, Scenario::Human);
    let ai = run(&params, Scenario::Ai);
    (benefit_curve(&human, &ai).last().copied().unwrap_or(0.0), crossover_month(&human, &ai))
}

// Keeps the constants in one place for the documentation links above.
#[allow(dead_code)]
const _: (f64, f64, f64) = (CENTRAL_SAVINGS, CENTRAL_HALF_LIFE, CENTRAL_EPSILON);

/// Review hours in each arm, so the accounting question has a number attached.
///
/// A referee asked whether review is *incremental* to the AI arm or a *reallocation* of the
/// review a human would have done anyway. In this model review is charged to **both** arms:
/// every module is reviewed on creation whether a human or a machine wrote it, and the only
/// difference is that reviewing code the reviewer understands less of costs more hours and
/// catches fewer defects. This function makes that checkable rather than asserted.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewAccounting {
    pub human_review: f64,
    pub ai_review: f64,
    pub human_authoring: f64,
    pub ai_authoring: f64,
    pub human_maintenance: f64,
    pub ai_maintenance: f64,
}

impl ReviewAccounting {
    /// Review hours the AI arm spends *in excess of* the human arm. This is the quantity the
    /// channel decomposition attributes to comprehension, not the whole review bill.
    pub fn excess_review(&self) -> f64 {
        self.ai_review - self.human_review
    }

    /// True when both arms are actually reviewed, which is the property that makes the
    /// comparison a comparison rather than a one-sided charge.
    pub fn both_arms_reviewed(&self) -> bool {
        self.human_review > 0.0 && self.ai_review > 0.0
    }
}

/// Sum the ledger by activity, for each arm, on the central cell.
pub fn review_accounting(base: &Params) -> ReviewAccounting {
    let params = central_cell(base);
    let human = run(&params, Scenario::Human);
    let ai = run(&params, Scenario::Ai);
    let total = |ledger: &crate::model::Ledger, pick: fn(&crate::model::MonthLedger) -> f64| {
        ledger.months.iter().map(pick).sum::<f64>()
    };
    ReviewAccounting {
        human_review: total(&human, |month| month.review),
        ai_review: total(&ai, |month| month.review),
        human_authoring: total(&human, |month| month.authoring),
        ai_authoring: total(&ai, |month| month.authoring),
        human_maintenance: total(&human, |month| month.maintenance),
        ai_maintenance: total(&ai, |month| month.maintenance),
    }
}
