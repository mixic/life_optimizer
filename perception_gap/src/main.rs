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

//! The report: when the gap is largest, and when the estimate of it stops working.
//!
//! The two answers are different and the report keeps them apart. The gap grows monotonically and
//! is largest at the end of the horizon — a fact the closed form gives without a simulation. Whether
//! the gap can be *estimated* is a question about the observation design, and it is the one where
//! the answer is not obvious.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use perception_gap::analysis::{gap_profile, sweep, sweep_with_mismatch};
use perception_gap::{observability, Design, Params};

const ADOPTIONS: [f64; 4] = [0.25, 0.50, 0.75, 1.00];
const GAMMAS: [f64; 6] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5];
const DESIGNS: [Design; 2] = [Design::Full, Design::BehaviouralOnly];
const SEEDS: u64 = 200;

fn main() {
    let export =
        std::env::args().position(|a| a == "--export").and_then(|i| std::env::args().nth(i + 1));

    let params = Params::default();

    println!("The perception gap: believed competence minus true competence");
    println!("=============================================================\n");
    println!("State: [tau, beta] — true competence and believed competence.");
    println!("Gap:   g = beta − tau. Never observed; only inferred.");
    println!("Input: adoption share a, known, so a standard Kalman filter is exact.\n");
    println!("Every parameter is Declared. Nothing here is a measurement of any person.\n");

    // ---- 1. when is the gap largest? ------------------------------------------
    println!("1. When is the gap largest?");
    println!("---------------------------");
    println!("Both latents converge geometrically, so the gap has a closed form:\n");
    println!(
        "  {:<10} {:>12} {:>14} {:>14}",
        "adoption", "steady gap", "half-life mo", "gap at m36"
    );
    for &a in &ADOPTIONS {
        let (steady, half_life) = perception_gap::steady_state(&params, a);
        let profile = gap_profile(&params, a, 50);
        let end = profile.gap.last().copied().unwrap_or(0.0);
        println!("  {:<10.2} {:>12.3} {:>14.1} {:>14.3}", a, steady, half_life, end);
    }
    println!(
        "\n  The gap is monotone in time: it is largest at the last month, always. The\n  \
         non-obvious quantity is when it grows fastest.\n"
    );
    println!(
        "  {:<10} {:>14} {:>18} {:>16}",
        "adoption", "peak month", "fastest growth mo", "growth there"
    );
    for &a in &ADOPTIONS {
        let profile = gap_profile(&params, a, 50);
        println!(
            "  {:<10.2} {:>14} {:>18} {:>16.4}",
            a, profile.peak_month, profile.fastest_month, profile.fastest_growth
        );
    }

    // ---- 2. can it be estimated? ---------------------------------------------
    println!("\n2. Can the gap be estimated at all?");
    println!("----------------------------------");
    println!(
        "Rank of [H; H F]. Rank 2 means both latents are identifiable; rank 1 means\n\
             some direction never affects any measurement and no estimator recovers it.\n"
    );
    println!("  {:<20} {:>6} {:>8} {:>16}", "design", "gamma", "rank", "condition");
    for design in DESIGNS {
        for &gamma in &GAMMAS {
            let mut local = params.clone();
            local.gamma = gamma;
            let obs = observability(&local, design, 1.0);
            let condition = if obs.condition.is_finite() {
                format!("{:.1}", obs.condition)
            } else {
                "infinite".to_string()
            };
            println!("  {:<20} {:>6.2} {:>8} {:>16}", design.label(), gamma, obs.rank, condition);
        }
    }
    println!(
        "\n  Read this before the estimator table below. Behaviour has only two channels and\n  \
         both load on true competence; belief enters them only through gamma. At gamma = 0\n  \
         the behavioural design cannot see believed competence at all, and a filter will\n  \
         still return a number for it."
    );

    // ---- 3. the estimators ---------------------------------------------------
    let cells = sweep(&params, &ADOPTIONS, &GAMMAS, &DESIGNS, SEEDS);
    println!("\n3. Does the filter beat subtraction?");
    println!("-----------------------------------");
    println!("RMSE of the estimated gap against the simulated truth, 200 seeds, adoption 0.5.\n");
    println!(
        "  {:<20} {:>6} {:>10} {:>10} {:>10} {:>8} {:>9}",
        "design", "gamma", "kalman", "subtract", "static", "ratio", "coverage"
    );
    for cell in cells.iter().filter(|c| (c.adoption - 0.5).abs() < 1e-9) {
        println!(
            "  {:<20} {:>6.2} {:>10.3} {:>10.3} {:>10.3} {:>8.2} {:>9.2}",
            cell.design.label(),
            cell.gamma,
            cell.kalman_rmse,
            cell.subtraction_rmse,
            cell.static_rmse,
            cell.gain_over_subtraction(),
            cell.kalman_coverage
        );
    }
    println!(
        "\n  ratio below 1.0 means the filter helps. coverage is the share of months where\n  \
         the truth lay inside the one-sigma band; a nominal 0.68 band that covers 0.40 is\n  \
         the failure mode this table exists to expose."
    );

    // ---- 4. is the filter reading the data, or reciting its own model? --------
    println!("\n4. Is the filter reading the data, or reciting its own model?");
    println!("-------------------------------------------------------------");
    println!("In a simulation the truth is generated by the same transition the filter");
    println!("assumes, so at zero mismatch the filter is rigged to win: its process model");
    println!("already knows the answer. The prior-only column is the filter with no");
    println!("observations at all. If kalman ~ prior, the observations are contributing");
    println!("nothing and the apparent accuracy is the model, not the measurement.\n");
    let mismatched =
        sweep_with_mismatch(&params, &[0.5], &[0.0, 0.3], &DESIGNS, SEEDS, &[0.0, 0.5, 1.0]);
    println!(
        "  {:<20} {:>6} {:>10} {:>8} {:>10} {:>9} {:>10}",
        "design", "gamma", "mismatch", "kalman", "prior-only", "ratios", "coverage"
    );
    for cell in &mismatched {
        println!(
            "  {:<20} {:>6.2} {:>10.2} {:>8.3} {:>10.3} {:>9.2} {:>10.2}",
            cell.design.label(),
            cell.gamma,
            cell.mismatch,
            cell.kalman_rmse,
            cell.prior_only_rmse,
            cell.gain_over_prior(),
            cell.kalman_coverage
        );
    }
    println!("\n  A ratio near 1.0 means the filter adds nothing to its own model.");

    // ---- 5. the headline -----------------------------------------------------
    let worst = cells
        .iter()
        .filter(|c| !c.unidentifiable())
        .max_by(|a, b| a.kalman_rmse.partial_cmp(&b.kalman_rmse).unwrap());
    let best = cells
        .iter()
        .filter(|c| !c.unidentifiable())
        .min_by(|a, b| a.gain_over_subtraction().partial_cmp(&b.gain_over_subtraction()).unwrap());
    println!("\n5. Where the estimate is best and worst");
    println!("--------------------------------------");
    if let Some(best) = best {
        println!(
            "  best:  {} at gamma {:.2}, adoption {:.2} — ratio {:.2}, coverage {:.2}",
            best.design.label(),
            best.gamma,
            best.adoption,
            best.gain_over_subtraction(),
            best.kalman_coverage
        );
    }
    if let Some(worst) = worst {
        println!(
            "  worst: {} at gamma {:.2}, adoption {:.2} — ratio {:.2}, coverage {:.2}",
            worst.design.label(),
            worst.gamma,
            worst.adoption,
            worst.gain_over_subtraction(),
            worst.kalman_coverage
        );
    }
    println!(
        "\n  The unidentifiable cells are not ranked. A filter reports a band there too, and\n  \
         the band is the reason this table is not enough on its own."
    );

    // ---- 5. export -----------------------------------------------------------
    if let Some(dir) = export {
        match write_all(Path::new(&dir), &params, &cells) {
            Ok(files) => {
                println!("\nExported {} file(s) to {dir}", files);
                println!("  Draw the figures with: python tools/plot_perception_gap.py {dir}");
            }
            Err(error) => {
                eprintln!("could not write the export to {dir}: {error}");
                std::process::exit(1);
            }
        }
    }

    println!("\n  This is a simulation whose truth is synthetic. It establishes whether the");
    println!("  estimator works on its own assumptions, which is all a simulation can");
    println!("  establish, and it is not evidence about any engineer.");
}

fn write_all(
    dir: &Path,
    params: &Params,
    cells: &[perception_gap::analysis::Cell],
) -> std::io::Result<usize> {
    fs::create_dir_all(dir)?;
    let mut written = 0;

    // The gap profile per adoption share.
    let mut body = String::from("adoption,month,gap,growth\n");
    for &a in &ADOPTIONS {
        let profile = gap_profile(params, a, 50);
        for (index, value) in profile.gap.iter().enumerate() {
            let growth = profile.growth.get(index).copied().unwrap_or(f64::NAN);
            writeln!(body, "{a:.2},{},{value:.6},{growth:.6}", index + 1).unwrap();
        }
    }
    fs::write(dir.join("gap_profile.csv"), body)?;
    written += 1;

    // The observability table.
    let mut body = String::from("design,gamma,rank,condition\n");
    for design in DESIGNS {
        for &gamma in &GAMMAS {
            let mut local = params.clone();
            local.gamma = gamma;
            let obs = observability(&local, design, 1.0);
            writeln!(
                body,
                "{},{gamma:.2},{},{:.6}",
                design.label(),
                obs.rank,
                if obs.condition.is_finite() { obs.condition } else { 1e9 }
            )
            .unwrap();
        }
    }
    fs::write(dir.join("observability.csv"), body)?;
    written += 1;

    // The estimator grid.
    let mut body = String::from(
        "design,adoption,gamma,true_gap_end,rank,condition,kalman_rmse,subtraction_rmse,\
         static_rmse,prior_only_rmse,kalman_coverage\n",
    );
    for cell in cells {
        writeln!(
            body,
            "{},{:.2},{:.2},{:.6},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            cell.design.label(),
            cell.adoption,
            cell.gamma,
            cell.true_gap_end,
            cell.rank,
            if cell.condition.is_finite() { cell.condition } else { 1e9 },
            cell.kalman_rmse,
            cell.subtraction_rmse,
            cell.static_rmse,
            cell.prior_only_rmse,
            cell.kalman_coverage
        )
        .unwrap();
    }
    fs::write(dir.join("estimator_grid.csv"), body)?;
    written += 1;

    // The calibration grid: one adoption share, three mismatch levels, both designs.
    let calibration =
        sweep_with_mismatch(params, &[0.5], &[0.0, 0.3], &DESIGNS, SEEDS, &[0.0, 0.5, 1.0]);
    let mut body = String::from(
        "design,gamma,mismatch,kalman_rmse,prior_only_rmse,subtraction_rmse,coverage\n",
    );
    for cell in &calibration {
        writeln!(
            body,
            "{},{:.2},{:.2},{:.6},{:.6},{:.6},{:.6}",
            cell.design.label(),
            cell.gamma,
            cell.mismatch,
            cell.kalman_rmse,
            cell.prior_only_rmse,
            cell.subtraction_rmse,
            cell.kalman_coverage
        )
        .unwrap();
    }
    fs::write(dir.join("calibration.csv"), body)?;
    written += 1;

    Ok(written)
}
