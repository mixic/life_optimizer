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

//! Write the model's numbers to CSV, so the figures are drawn from the model itself.
//!
//! `tools/plot_comprehension_cost.py` reads these files and refuses to draw anything if
//! they are inconsistent. That check is worth having only because the data comes from
//! the same functions the report prints: a figure drawn from a separate reimplementation
//! would agree with the table only by luck.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::analysis::{breakeven, breakeven_threshold, channels, monte_carlo, review_accounting};
use crate::global_sensitivity::model_indices;
use crate::model::{benefit_curve, crossover_month, run, Scenario};
use crate::params::Params;
use crate::sweep::{
    central_cell, sensitivity, sweep, EPSILON_LEVELS, HALF_LIFE_LEVELS, SAVINGS_LEVELS,
};

fn write_file(dir: &Path, name: &str, body: &str) -> io::Result<PathBuf> {
    let path = dir.join(name);
    fs::write(&path, body)?;
    Ok(path)
}

/// Write every table the figures need. Returns the paths written.
pub fn write_all(dir: &Path, base: &Params, paths: usize) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    let mut written = Vec::new();

    // ---- the phase grid ------------------------------------------------------
    let cells = sweep(base, &SAVINGS_LEVELS, &HALF_LIFE_LEVELS, &EPSILON_LEVELS);
    let mut body = String::from("savings,half_life_ai,epsilon,crossover_month,benefit_end\n");
    for cell in &cells {
        let month = cell.crossover.map(|m| m.to_string()).unwrap_or_else(String::new);
        writeln!(
            body,
            "{:.2},{:.0},{:.2},{},{:.4}",
            cell.savings, cell.half_life_ai, cell.epsilon, month, cell.benefit_end
        )
        .unwrap();
    }
    written.push(write_file(dir, "phase.csv", &body)?);

    // ---- the central cell, month by month ------------------------------------
    let params = central_cell(base);
    let human = run(&params, Scenario::Human);
    let ai = run(&params, Scenario::Ai);
    let human_cumulative = human.cumulative();
    let ai_cumulative = ai.cumulative();
    let benefit = benefit_curve(&human, &ai);
    let mut body = String::from(
        "month,human_total,human_cumulative,ai_total,ai_cumulative,benefit,\
         human_comprehension,ai_comprehension\n",
    );
    for (index, (h, a)) in human.months.iter().zip(ai.months.iter()).enumerate() {
        writeln!(
            body,
            "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            h.month,
            h.total(),
            human_cumulative[index],
            a.total(),
            ai_cumulative[index],
            benefit[index],
            h.comprehension_mean,
            a.comprehension_mean
        )
        .unwrap();
    }
    written.push(write_file(dir, "central.csv", &body)?);

    // ---- the channel decomposition -------------------------------------------
    let mut body = String::from("configuration,benefit,delta\n");
    for row in channels(base) {
        let delta = row.delta.map(|d| format!("{d:.4}")).unwrap_or_else(String::new);
        writeln!(body, "{},{:.4},{}", row.label, row.benefit, delta).unwrap();
    }
    written.push(write_file(dir, "channels.csv", &body)?);

    // ---- the sensitivity ranking ---------------------------------------------
    let mut body = String::from("parameter,low,high,swing\n");
    for row in sensitivity(base) {
        writeln!(body, "\"{}\",{:.4},{:.4},{:.4}", row.name, row.low, row.high, row.swing).unwrap();
    }
    written.push(write_file(dir, "sensitivity.csv", &body)?);

    // ---- global sensitivity --------------------------------------------------
    let sobol = model_indices(base, 1024, 2024);
    let mut body = String::from("parameter,first_order,total_order,interaction\n");
    for index in sobol.indices.iter() {
        writeln!(
            body,
            "\"{}\",{:.6},{:.6},{:.6}",
            index.name,
            index.first,
            index.total,
            index.interaction()
        )
        .unwrap();
    }
    written.push(write_file(dir, "sobol.csv", &body)?);
    written.push(write_file(
        dir,
        "sobol_interaction_share.txt",
        &format!("{:.6}\n", sobol.interaction_share()),
    )?);

    // ---- review accounting ---------------------------------------------------
    let accounting = review_accounting(base);
    let mut body = String::from("activity,human,ai,excess\n");
    for (name, human, ai) in [
        ("authoring", accounting.human_authoring, accounting.ai_authoring),
        ("review", accounting.human_review, accounting.ai_review),
        ("maintenance", accounting.human_maintenance, accounting.ai_maintenance),
    ] {
        writeln!(body, "{name},{human:.4},{ai:.4},{:.4}", ai - human).unwrap();
    }
    written.push(write_file(dir, "review_accounting.csv", &body)?);

    // ---- break-even ----------------------------------------------------------
    let mut body = String::from("c_ai,benefit,crossover_month,mean_comprehension\n");
    for row in breakeven(base) {
        let month = row.crossover.map(|m| m.to_string()).unwrap_or_else(String::new);
        writeln!(
            body,
            "{:.2},{:.4},{},{:.4}",
            row.c_ai, row.benefit, month, row.mean_comprehension
        )
        .unwrap();
    }
    written.push(write_file(dir, "breakeven.csv", &body)?);
    if let Some(threshold) = breakeven_threshold(&breakeven(base)) {
        written.push(write_file(dir, "breakeven_threshold.txt", &format!("{threshold:.4}\n"))?);
    }

    // ---- the Monte Carlo -----------------------------------------------------
    let mut body = String::from("seed,crossover_month,benefit\n");
    for row in monte_carlo(base, paths) {
        let month = row.crossover.map(|m| m.to_string()).unwrap_or_else(String::new);
        writeln!(body, "{},{},{:.4}", row.seed, month, row.benefit).unwrap();
    }
    written.push(write_file(dir, "paths.csv", &body)?);

    // ---- the control: no comprehension asymmetry ------------------------------
    let mut body = String::from("savings,benefit_end,crossover_month\n");
    for &savings in &SAVINGS_LEVELS {
        let mut params = base.clone();
        params.savings = savings;
        params.c_ai = params.c_human;
        params.half_life_ai = params.half_life_human;
        params.ai_defect_multiplier = 0.0;
        params.leak_hazard = 0.0;
        let human = run(&params, Scenario::Human);
        let ai = run(&params, Scenario::Ai);
        let end = benefit_curve(&human, &ai).last().copied().unwrap_or(0.0);
        let month = crossover_month(&human, &ai).map(|m| m.to_string()).unwrap_or_else(String::new);
        writeln!(body, "{savings:.2},{end:.4},{month}").unwrap();
    }
    written.push(write_file(dir, "control.csv", &body)?);

    Ok(written)
}
