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

//! The command-line face of the model.
//!
//! The report carries its own limits on its face: the provenance of every parameter,
//! the pre-registered verdict rule, and the mean-field caveat. A reader who takes a
//! phase diagram from here as a forecast has been misled by the presentation rather
//! than by the arithmetic.

use comprehension_cost::analysis::{breakeven as breakeven_curve, breakeven_threshold, channels};
use comprehension_cost::model::{benefit_curve, crossover_month, run, run_stochastic, Scenario};
use comprehension_cost::params::{Params, Provenance};
use comprehension_cost::sweep::{
    is_falsified, sensitivity, sweep, verdict, CENTRAL_EPSILON, CENTRAL_HALF_LIFE, CENTRAL_SAVINGS,
    EPSILON_LEVELS, HALF_LIFE_LEVELS, SAVINGS_LEVELS,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut verbose = false;
    let mut paths: Option<usize> = None;
    let mut export_dir: Option<std::path::PathBuf> = None;

    // Flags are validated rather than sniffed. An ignored typo would silently run the
    // default report and look like it had done what was asked.
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            // The grid is the default output; `--sweep` is accepted explicitly so the
            // documented invocation matches the task and a reader can be explicit about
            // which part of the report they asked for.
            "--sweep" => index += 1,
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            "--paths" => {
                let value = args.get(index + 1).unwrap_or_else(|| {
                    eprintln!("--paths needs a path count, for example: --paths 300");
                    std::process::exit(2);
                });
                match value.parse::<usize>() {
                    Ok(count) if count > 0 => paths = Some(count),
                    _ => {
                        eprintln!("--paths expects a positive integer, got {value:?}");
                        std::process::exit(2);
                    }
                }
                index += 2;
            }
            "--export" => {
                let value = args.get(index + 1).unwrap_or_else(|| {
                    eprintln!(
                        "--export needs a directory, for example: --export out/comprehension"
                    );
                    std::process::exit(2);
                });
                export_dir = Some(std::path::PathBuf::from(value));
                index += 2;
            }
            other => {
                eprintln!("unknown argument {other:?}");
                eprintln!("usage: comprehension_cost [--sweep] [--verbose] [--paths N]");
                std::process::exit(2);
            }
        }
    }

    let base = Params::default();

    println!("AI authoring and the comprehension debt");
    println!("=======================================\n");
    println!("Question: does the authoring time AI saves survive the cost of");
    println!("understanding, reviewing and maintaining what it wrote?\n");
    println!(
        "Model: {} modules, {} months, {:.0}% AI-authored in the AI scenario.",
        base.modules,
        base.horizon_months,
        base.ai_share * 100.0
    );
    println!("The baseline is all-human and identical in every other respect, so the");
    println!("difference between the two runs is attributable to authoring mode alone.\n");

    let cells = sweep(&base, &SAVINGS_LEVELS, &HALF_LIFE_LEVELS, &EPSILON_LEVELS);
    let decision = verdict(&cells, &SAVINGS_LEVELS);

    println!("Cumulative-benefit phase diagram");
    println!("-------------------------------");
    println!("Entry = the month at which cumulative AI benefit turns negative (AI has");
    println!("cost more hours than it saved). \"—\" = no crossover within the horizon.\n");

    for &s in &SAVINGS_LEVELS {
        let label = if s == 0.0 {
            "AI saves nothing — METR's measured result for experts"
        } else if (s - 0.33).abs() < 1e-9 {
            "AI saves a third of authoring — BairesDev self-report"
        } else {
            "AI saves 15% of authoring — between the two anchors"
        };
        println!("sigma = {s:.2}  ({label})");
        print!("  {:>5} \\ eps", "h_ai");
        for &e in &EPSILON_LEVELS {
            print!("  {:>5.2}", e);
        }
        println!();
        for &h in &HALF_LIFE_LEVELS {
            print!("  {h:>9.0}");
            for &e in &EPSILON_LEVELS {
                let cell = cells
                    .iter()
                    .find(|c| {
                        (c.savings - s).abs() < 1e-9
                            && (c.half_life_ai - h).abs() < 1e-9
                            && (c.epsilon - e).abs() < 1e-9
                    })
                    .expect("every grid point was computed");
                match cell.crossover {
                    Some(month) => print!("  {month:>5}"),
                    None => print!("  {:>5}", "—"),
                }
            }
            println!();
        }

        println!(
            "  Cumulative benefit at month {} (negative = AI cost more hours):",
            base.horizon_months
        );
        print!("  {:>5} \\ eps", "h_ai");
        for &e in &EPSILON_LEVELS {
            print!("  {:>5.2}", e);
        }
        println!();
        for &h in &HALF_LIFE_LEVELS {
            print!("  {h:>9.0}");
            for &e in &EPSILON_LEVELS {
                let cell = cells
                    .iter()
                    .find(|c| {
                        (c.savings - s).abs() < 1e-9
                            && (c.half_life_ai - h).abs() < 1e-9
                            && (c.epsilon - e).abs() < 1e-9
                    })
                    .expect("every grid point was computed");
                print!("  {:+5.0}", cell.benefit_end);
            }
            println!();
        }
        println!();
    }

    println!("Pre-registered verdict");
    println!("----------------------");
    println!("{}", decision.headline());

    let central = cells.iter().find(|c| c.is_central()).expect("the central cell is in the grid");
    println!(
        "\nCentral cell (sigma {:.2}, h_ai {:.0}, eps {:.2}): {}",
        CENTRAL_SAVINGS,
        CENTRAL_HALF_LIFE,
        CENTRAL_EPSILON,
        match central.crossover {
            Some(month) => format!("crosses over in month {month}"),
            None => "no crossover".to_string(),
        }
    );
    println!(
        "  cumulative benefit at month {}: {:+.0} h",
        base.horizon_months, central.benefit_end
    );

    if is_falsified(&cells) {
        println!(
            "\nFALSIFIED: no cell with sigma >= 0.15, h_ai >= 6 and eps <= 0.5 crosses over.\n\
             The hypothesis, as modelled, is refuted."
        );
    }

    // ---- channel decomposition -----------------------------------------------
    println!("\nWhere the difference actually comes from");
    println!("--------------------------------------");
    println!("The same central cell, switching on one mechanism at a time. Each row adds");
    println!("the named channel to the row above, so the delta is that channel's cost.\n");
    println!("  {:<44} {:>11} {:>10}", "configuration", "benefit m36", "delta");
    for row in channels(&base) {
        let delta = match row.delta {
            Some(value) => format!("{value:+.0}"),
            None => "—".to_string(),
        };
        println!("  {:<44} {:+11.0} {:>10}", row.label, row.benefit, delta);
    }
    println!("\n  A negative delta is that channel costing hours.");

    // ---- sensitivity ---------------------------------------------------------
    println!("\nSensitivity: which parameter does the answer depend on?");
    println!("-----------------------------------------------------");
    println!("One at a time, perturbed by half and by half again, on the central cell.");
    println!("Ranked by how far the change moves cumulative benefit at the horizon.\n");
    let rows = sensitivity(&base);
    println!(
        "  {:<32} {:>9} {:>9} {:>9} {:>9}",
        "parameter", "at 0.5x", "at 1.5x", "swing", "share"
    );
    let total_swing: f64 = rows.iter().map(|r| r.swing).sum::<f64>().max(1e-9);
    for row in &rows {
        println!(
            "  {:<32} {:+9.0} {:+9.0} {:>9.0} {:>8.0}%",
            row.name,
            row.low,
            row.high,
            row.swing,
            row.swing / total_swing * 100.0
        );
    }
    println!("\n  The decay asymmetry is the hypothesis, and it is ranked where it is ranked");
    println!("  above: a model whose answer turned on it would show it at the top.");

    // ---- break-even ----------------------------------------------------------
    println!("\nBreak-even: how much comprehension has to survive?");
    println!("------------------------------------------------");
    println!("The verdict is a yes/no. This is the actionable form of it: hold everything");
    println!("else at the central cell and vary only the comprehension an AI-authored");
    println!("module starts with. The question becomes how small the gap must be before");
    println!("the economics stop favouring the human author.\n");
    println!("  {:>8} {:>12} {:>14} {:>11}", "c_ai", "benefit m36", "crossover", "mean c m36");
    let curve = breakeven_curve(&base);
    let first_positive = breakeven_threshold(&curve);
    for row in &curve {
        println!(
            "  {:>8.2} {:+12.0} {:>14} {:>11.3}",
            row.c_ai,
            row.benefit,
            match row.crossover {
                Some(month) => format!("month {month}"),
                None => "none".to_string(),
            },
            row.mean_comprehension
        );
    }
    match first_positive {
        Some(c) => println!(
            "\n  Break-even at c_ai = {c:.2}: AI has to leave the engineer understanding about\n  \
             {:.0}% of what authoring the code would have left, for the run to pay for itself.",
            c * 100.0
        ),
        None => println!(
            "\n  No break-even from the initial comprehension gap alone: even at c_ai = 0.85\n  \
             the run loses, because the retention asymmetry and the defect multiplier are\n  \
             still switched on. Reaching break-even needs the comprehension gap AND the\n  \
             retention asymmetry to be small at once. That the gap alone cannot pay for\n  \
             itself is the sharpest result here, and it is reported as such."
        ),
    }

    // ---- the control that proves the model can say "no" ----------------------
    println!("\nControl: remove the comprehension asymmetry and the result must flip");
    println!("------------------------------------------------------------------");
    println!("If AI-authored code started at the same comprehension as human code and");
    println!("carried no defect penalty, the only remaining difference would be the");
    println!("authoring saving — and the benefit should stay positive throughout.\n");
    for &s in &SAVINGS_LEVELS {
        let mut symmetric = base.clone();
        symmetric.savings = s;
        symmetric.c_ai = symmetric.c_human;
        symmetric.half_life_ai = symmetric.half_life_human;
        symmetric.ai_defect_multiplier = 0.0;
        let human = run(&symmetric, Scenario::Human);
        let ai = run(&symmetric, Scenario::Ai);
        let benefit = benefit_curve(&human, &ai);
        println!(
            "  sigma {s:.2}: benefit at month {} = {:+.0} h, crossover = {}",
            symmetric.horizon_months,
            benefit.last().copied().unwrap_or(0.0),
            match crossover_month(&human, &ai) {
                Some(month) => format!("month {month}"),
                None => "none".to_string(),
            }
        );
    }
    println!("\n  With sigma = 0 the two runs are identical, so the benefit is exactly zero:");
    println!("  a model that reported a loss there would be double-charging something.");

    if verbose {
        println!("\nMonth-by-month, central cell");
        println!("----------------------------");
        let mut params = base.clone();
        params.savings = CENTRAL_SAVINGS;
        params.half_life_ai = CENTRAL_HALF_LIFE;
        params.ai_defect_multiplier = CENTRAL_EPSILON;
        let human = run(&params, Scenario::Human);
        let ai = run(&params, Scenario::Ai);
        println!(
            "  {:>5}  {:>10}  {:>10}  {:>10}  {:>10}  {:>12}",
            "month", "human h/mo", "AI h/mo", "human cum", "AI cum", "benefit"
        );
        let hc = human.cumulative();
        let ac = ai.cumulative();
        for (idx, (h, a)) in human.months.iter().zip(ai.months.iter()).enumerate() {
            println!(
                "  {:>5}  {:>10.1}  {:>10.1}  {:>10.0}  {:>10.0}  {:>+12.0}",
                h.month,
                h.total(),
                a.total(),
                hc[idx],
                ac[idx],
                hc[idx] - ac[idx]
            );
        }
        println!(
            "\n  mean comprehension at month {}: human {:.3}, AI {:.3}",
            base.horizon_months,
            human.months.last().map(|m| m.comprehension_mean).unwrap_or(0.0),
            ai.months.last().map(|m| m.comprehension_mean).unwrap_or(0.0)
        );
    }

    if let Some(paths) = paths {
        println!("\nMonte Carlo, central cell ({paths} paths)");
        println!("---------------------------------------");
        println!("The mean-field diagram charges expected events every month. Real");
        println!("arrivals are lumpy; this is how much that matters.\n");
        let mut params = base.clone();
        params.savings = CENTRAL_SAVINGS;
        params.half_life_ai = CENTRAL_HALF_LIFE;
        params.ai_defect_multiplier = CENTRAL_EPSILON;
        let human = run(&params, Scenario::Human); // deterministic baseline
        let mut crossings = Vec::new();
        let mut ends = Vec::new();
        for seed in 0..paths as u64 {
            let ai = run_stochastic(&params, Scenario::Ai, seed);
            let benefit = benefit_curve(&human, &ai);
            ends.push(benefit.last().copied().unwrap_or(0.0));
            if let Some(month) = crossover_month(&human, &ai) {
                crossings.push(month);
            }
        }
        ends.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let share = crossings.len() as f64 / paths as f64;
        println!("  paths with a crossover: {:.0}% ({}/{paths})", share * 100.0, crossings.len());
        if !crossings.is_empty() {
            crossings.sort_unstable();
            println!(
                "  crossover month: median {}, 10th {}, 90th {}",
                crossings[crossings.len() / 2],
                crossings[crossings.len() / 10],
                crossings[9 * crossings.len() / 10]
            );
        }
        println!(
            "  benefit at month {}: median {:+.0} h, 10th {:+.0} h, 90th {:+.0} h",
            base.horizon_months,
            ends[ends.len() / 2],
            ends[ends.len() / 10],
            ends[9 * ends.len() / 10]
        );
        println!("  mean-field value for comparison: {:+.0} h", central.benefit_end);
    }

    // ---- provenance ---------------------------------------------------------
    println!("\nParameters and provenance");
    println!("-------------------------");
    println!("  {:<38} {:<16} {}", "parameter", "value", "provenance");
    for (name, value, provenance) in base.provenance() {
        let mark = if provenance.is_sourced() { "Sourced" } else { "Declared" };
        println!("  {name:<38} {value:<16} {mark}");
    }
    println!("\n  Sourced:");
    for (name, _value, provenance) in base.provenance() {
        if let Provenance::Sourced(citation) = provenance {
            println!("    {name}: {citation}");
        }
    }
    println!("\n  Everything not marked Sourced is this model's own hypothesis. The decay");
    println!("  asymmetry is the least anchored quantity in it and the one the");
    println!("  conclusion is most sensitive to — see AI_AUTHORING_AND_COMPREHENSION.md.");
    println!("\n  This is a hypothesis-generating exercise, not a measurement of any");
    println!("  developer. A phase diagram says where to look.");

    if let Some(dir) = &export_dir {
        match comprehension_cost::export::write_all(dir, &base, paths.unwrap_or(300)) {
            Ok(files) => {
                println!("\nExported {} file(s) to {}", files.len(), dir.display());
                println!(
                    "  Draw the figures with: python tools/plot_comprehension_cost.py {}",
                    dir.display()
                );
            }
            Err(error) => {
                eprintln!("could not write the export to {}: {error}", dir.display());
                std::process::exit(1);
            }
        }
    }
}
