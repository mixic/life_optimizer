// Multipolar World Simulator
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

//! CSV export, so the extrapolation can be *looked at* rather than only read.
//!
//! # Why this exists
//!
//! An ensemble of a fifty-year extrapolation printed as tables hides the thing that
//! matters most about it: the spread. Two worlds with the same mean cooperation and
//! different dispersion are different worlds, and a mean line cannot tell them apart.
//! This module writes the distributions out -- per-year quantiles for the
//! trajectories, and one row per run for the end states -- and
//! `tools/plot_multipolar.py` draws them.
//!
//! # What it deliberately does not do
//!
//! It does not plot, and it adds no dependency in order to plot: the numbers leave as
//! text and the picture is drawn by a script that can live outside the crate. Nothing
//! about the numbers changes by being exported -- the growth rates, elasticities and
//! conversion coefficients are invented exactly as they are everywhere else here --
//! so `config.txt` is written alongside the data, and a figure can be traced back to
//! the parameterisation that produced it rather than floating free of it.
//!
//! # The columns a reader should trust and the ones they should not
//!
//! Every `p10`/`p50`/`p90` is a quantile *across runs*, at a fixed year. It is a
//! statement about this parameterisation's own dispersion, not a confidence interval
//! about the world: the uncertainty that matters here is in the parameters, and this
//! file holds those fixed while the shocks vary.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::simulation::{Config, Ensemble};
/// The quantiles written for every trajectory band.
const QUANTILES: [(&str, f64); 3] = [("p10", 0.10), ("p50", 0.50), ("p90", 0.90)];

/// Linear-interpolated quantile of an already-sorted slice.
fn quantile(sorted: &[f64], p: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0],
        len => {
            let position = p * (len - 1) as f64;
            let lower = position.floor() as usize;
            let upper = position.ceil() as usize;
            if lower == upper {
                return sorted[lower];
            }
            let weight = position - lower as f64;
            sorted[lower] * (1.0 - weight) + sorted[upper] * weight
        }
    }
}

fn sorted(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values
}

fn band(values: &[f64]) -> String {
    QUANTILES
        .iter()
        .map(|(_, p)| format!("{:.6}", quantile(values, *p)))
        .collect::<Vec<_>>()
        .join(",")
}

fn write_file(path: PathBuf, contents: String) -> io::Result<PathBuf> {
    fs::write(&path, contents)?;
    Ok(path)
}

/// One row of the scenario comparison, for the export.
pub struct ScenarioRow {
    pub label: String,
    pub premise: String,
    pub cooperation: f64,
    pub tension: f64,
    pub trap: f64,
    pub pareto_loss: f64,
    pub pension: f64,
    pub leading: String,
    pub top_share: f64,
    /// End-state share per bloc, in `config.blocs` order.
    pub shares: Vec<f64>,
    pub bloc_names: Vec<String>,
    pub start_shares: Vec<f64>,
}

/// Write every CSV for one world, and return the paths written.
pub fn write_bundle(dir: &Path, config: &Config, ensemble: &Ensemble) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    let mut files = Vec::new();

    // ---- trajectories, as quantiles across runs ------------------------------
    let mut shares = String::from("year,bloc,p10,p50,p90\n");
    for year in 0..ensemble.horizon {
        for (index, bloc) in config.blocs.iter().enumerate() {
            let values = sorted(
                ensemble
                    .share_trajectories
                    .iter()
                    .filter_map(|run| run.get(year))
                    .filter_map(|year_shares| year_shares.get(index))
                    .copied()
                    .collect(),
            );
            if values.is_empty() {
                continue;
            }
            writeln!(shares, "{year},{},{}", bloc.name, band(&values)).ok();
        }
    }
    files.push(write_file(dir.join("shares-bands.csv"), shares)?);

    let mut world = String::from(
        "year,cooperation_p10,cooperation_p50,cooperation_p90,tension_p10,tension_p50,\
         tension_p90,loss_p10,loss_p50,loss_p90\n",
    );
    for year in 0..ensemble.horizon {
        let at = |runs: &Vec<Vec<f64>>| -> Vec<f64> {
            sorted(
                runs.iter()
                    .filter_map(|run| run.get(year))
                    .copied()
                    .collect(),
            )
        };
        let cooperation = at(&ensemble.cooperation_trajectories);
        let tension = at(&ensemble.tension_trajectories);
        let loss = at(&ensemble.loss_trajectories);
        if cooperation.is_empty() {
            continue;
        }
        writeln!(
            world,
            "{year},{},{},{}",
            band(&cooperation),
            band(&tension),
            band(&loss)
        )
        .ok();
    }
    files.push(write_file(dir.join("world-bands.csv"), world)?);

    // ---- end states, one row per run ----------------------------------------
    let mut runs = String::from(
        "run,mean_cooperation,mean_efficiency_loss,trap_fraction,final_tension,\
         final_recession_risk,top_share,dominant,asymmetric_fraction,polarity\n",
    );
    let dominance = ensemble.dominance_probabilities(config.blocs.len());
    for (index, outcome) in ensemble.outcomes.iter().enumerate() {
        let dominant = outcome
            .dominant
            .and_then(|i| config.blocs.get(i))
            .map(|b| b.name.as_str())
            .unwrap_or("none");
        writeln!(
            runs,
            "{index},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{dominant},{:.6},{}",
            outcome.mean_cooperation,
            outcome.mean_efficiency_loss,
            outcome.trap_fraction,
            outcome.final_tension,
            outcome.final_recession_risk,
            outcome.top_share,
            outcome.asymmetric_fraction,
            outcome.final_polarity.label()
        )
        .ok();
    }
    files.push(write_file(dir.join("world-runs.csv"), runs)?);

    // ---- per-bloc summary, including what the fan chart ends at --------------
    let mut summary =
        String::from("bloc,start_share,end_p10,end_p50,end_p90,end_mean,dominance_probability\n");
    for (index, bloc) in config.blocs.iter().enumerate() {
        let ends = sorted(
            ensemble
                .outcomes
                .iter()
                .filter_map(|outcome| outcome.final_shares.get(index))
                .copied()
                .collect(),
        );
        let mean = if ends.is_empty() {
            0.0
        } else {
            ends.iter().sum::<f64>() / ends.len() as f64
        };
        writeln!(
            summary,
            "{},{:.6},{},{:.6},{:.6}",
            bloc.name,
            bloc.power_share,
            band(&ends),
            mean,
            dominance[index]
        )
        .ok();
    }
    files.push(write_file(dir.join("blocs-summary.csv"), summary)?);

    // ---- the pension channel, one row per run --------------------------------
    //
    // This is the Switzerland-facing output. `security_index` is measured against the
    // fully cooperative reference, exactly as the console report measures it, so the
    // two cannot drift apart.
    let reference = config.pension.assess(1.0, 0.0);
    let mut pension = String::from(
        "run,cooperation,efficiency_loss,replacement_rate,support_ratio,portfolio_return,\
         security_index\n",
    );
    for (index, outcome) in ensemble.outcomes.iter().enumerate() {
        let p = outcome.pension;
        writeln!(
            pension,
            "{index},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            p.cooperation_index,
            p.mean_efficiency_loss,
            p.replacement_rate,
            p.support_ratio,
            p.portfolio_return,
            p.security_index(&reference)
        )
        .ok();
    }
    files.push(write_file(dir.join("pension-runs.csv"), pension)?);

    // ---- the asymmetry each bloc faces --------------------------------------
    let economy = &config.economy;
    let mut exposure = String::from(
        "bloc,reserve_share,import_dependence,export_dependence,disruption_exposure\n",
    );
    for (index, bloc) in config.blocs.iter().enumerate() {
        let e = economy.energy.get(index);
        writeln!(
            exposure,
            "{},{:.6},{:.6},{:.6},{:.6}",
            bloc.name,
            economy.reserve_shares.get(index).copied().unwrap_or(0.0),
            e.map(|e| e.import_dependence).unwrap_or(0.0),
            e.map(|e| e.export_dependence).unwrap_or(0.0),
            e.map(|e| e.disruption_exposure()).unwrap_or(0.0)
        )
        .ok();
    }
    files.push(write_file(dir.join("exposure.csv"), exposure)?);

    // The leverage matrix is what makes a Europe panel possible without hard-coding
    // "the United States holds leverage over Europe" into the plotting script.
    let mut leverage = String::from("holder,subject,leverage\n");
    for holder in 0..config.blocs.len() {
        for subject in 0..config.blocs.len() {
            if holder == subject {
                continue;
            }
            writeln!(
                leverage,
                "{},{},{:.6}",
                config.blocs[holder].name,
                config.blocs[subject].name,
                economy.monetary_leverage(holder, subject)
            )
            .ok();
        }
    }
    files.push(write_file(dir.join("monetary-leverage.csv"), leverage)?);

    let mut flows = String::from("from,to,share\n");
    for flow in economy.flows.iter() {
        if let (Some(from), Some(to)) = (config.blocs.get(flow.from), config.blocs.get(flow.to)) {
            writeln!(flows, "{},{},{:.6}", from.name, to.name, flow.share).ok();
        }
    }
    files.push(write_file(dir.join("energy-flows.csv"), flows)?);

    // ---- the parameterisation, so a figure can be traced back ----------------
    files.push(write_file(dir.join("config.txt"), describe(config))?);

    Ok(files)
}

/// Write the scenario comparison as two CSVs: world metrics, and end shares.
pub fn write_scenarios(dir: &Path, rows: &[ScenarioRow]) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    let mut files = Vec::new();

    let mut world = String::from(
        "scenario,cooperation,tension,trap,pareto_loss,pension,leader,top_share,premise\n",
    );
    for row in rows {
        writeln!(
            world,
            "{},{:.6},{:.6},{:.6},{:.6},{:.6},{},{:.6},\"{}\"",
            row.label,
            row.cooperation,
            row.tension,
            row.trap,
            row.pareto_loss,
            row.pension,
            row.leading,
            row.top_share,
            row.premise
        )
        .ok();
    }
    files.push(write_file(dir.join("scenarios-world.csv"), world)?);

    let mut shares = String::from("scenario,bloc,start_share,end_share\n");
    for row in rows {
        for (index, name) in row.bloc_names.iter().enumerate() {
            writeln!(
                shares,
                "{},{},{:.6},{:.6}",
                row.label,
                name,
                row.start_shares.get(index).copied().unwrap_or(0.0),
                row.shares.get(index).copied().unwrap_or(0.0)
            )
            .ok();
        }
    }
    files.push(write_file(dir.join("scenarios-shares.csv"), shares)?);

    Ok(files)
}

/// A plain-text record of what produced the numbers beside it.
fn describe(config: &Config) -> String {
    let mut out = String::new();
    out.push_str(
        "multipolar_sim export\n\
         ====================\n\n\
         Everything in this directory comes from invented parameters. The blocs, their\n\
         growth biases, the payoff coefficients, the energy and monetary elasticities and\n\
         the pension conversion are chosen to be plausible in sign and rough magnitude so\n\
         that a mechanism can be explored. They are not measurements, and nothing here is\n\
         a forecast. Read the differences between worlds, not the levels in them.\n\n",
    );
    writeln!(out, "seed: {}", config.seed).ok();
    writeln!(out, "runs: {}", config.runs).ok();
    writeln!(out, "horizon (years): {}", config.horizon).ok();
    writeln!(
        out,
        "years are zero-based: year 0 is the first simulated year"
    )
    .ok();
    writeln!(out).ok();

    out.push_str("blocs (name, power_share, growth_bias, volatility, cooperation_affinity, cooperation_valuation)\n");
    for bloc in config.blocs.iter() {
        writeln!(
            out,
            "  {},{:.4},{:.4},{:.4},{:.4},{:.4}",
            bloc.name,
            bloc.power_share,
            bloc.growth_bias,
            bloc.volatility,
            bloc.cooperation_affinity,
            bloc.cooperation_valuation
        )
        .ok();
    }
    writeln!(out).ok();

    let game = &config.game;
    out.push_str("game parameters\n");
    writeln!(out, "  cooperation_gain: {}", game.cooperation_gain).ok();
    writeln!(out, "  defection_temptation: {}", game.defection_temptation).ok();
    writeln!(out, "  exploitation_cost: {}", game.exploitation_cost).ok();
    writeln!(out, "  parity_pressure: {}", game.parity_pressure).ok();
    writeln!(out, "  tension_pressure: {}", game.tension_pressure).ok();
    writeln!(out, "  conflict_wear: {}", game.conflict_wear).ok();
    writeln!(out, "  tension_per_conflict: {}", game.tension_per_conflict).ok();
    writeln!(out, "  tension_decay: {}", game.tension_decay).ok();
    writeln!(out).ok();

    out.push_str("pension link (illustrative: these are not Swiss pension figures)\n");
    let p = &config.pension;
    writeln!(out, "  base_replacement_rate: {}", p.base_replacement_rate).ok();
    writeln!(
        out,
        "  fiscal_crowding_per_competition: {}",
        p.fiscal_crowding_per_competition
    )
    .ok();
    writeln!(out, "  ratio_sensitivity: {}", p.ratio_sensitivity).ok();
    writeln!(out, "  reference_ratio: {}", p.reference_ratio).ok();
    writeln!(
        out,
        "  fragmentation_return_drag: {}",
        p.fragmentation_return_drag
    )
    .ok();
    writeln!(out, "  cooperative_return: {}", p.cooperative_return).ok();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Config;

    /// Every file is written, and the columns the plotting script reads are stable.
    ///
    /// A renamed column would not fail anything else: `tools/plot_multipolar.py` would
    /// raise a `KeyError` at best, and a figure drawn from a mis-read column would look
    /// exactly like a figure drawn from a correct one. So the header lines are pinned
    /// here rather than only in the script.
    #[test]
    fn the_export_writes_every_table_with_the_columns_the_plots_read() {
        let config = Config {
            runs: 4,
            horizon: 3,
            ..Config::default()
        };
        let ensemble = Ensemble::run(&config);
        let dir = std::env::temp_dir().join("multipolar_sim_export_test");
        let _ = fs::remove_dir_all(&dir);

        let files = write_bundle(&dir, &config, &ensemble).expect("the export must write");
        assert_eq!(files.len(), 9, "one table per file, plus config.txt");

        let header = |name: &str| -> String {
            fs::read_to_string(dir.join(name))
                .unwrap_or_else(|error| panic!("{name} must be readable: {error}"))
                .lines()
                .next()
                .unwrap_or_default()
                .to_string()
        };

        assert_eq!(header("shares-bands.csv"), "year,bloc,p10,p50,p90");
        assert!(
            header("world-bands.csv").contains("cooperation_p50")
                && header("world-bands.csv").contains("tension_p90"),
            "the world bands must name the quantiles the plots read: {}",
            header("world-bands.csv")
        );
        assert!(header("blocs-summary.csv").contains("dominance_probability"));
        assert!(header("pension-runs.csv").ends_with("security_index"));
        assert!(header("exposure.csv").contains("disruption_exposure"));
        assert_eq!(header("monetary-leverage.csv"), "holder,subject,leverage");

        // And the data has to be there, not merely the header.
        let shares = fs::read_to_string(dir.join("shares-bands.csv")).expect("readable");
        assert!(
            shares.lines().count() == 1 + 3 * config.blocs.len(),
            "one row per year per bloc: {} lines",
            shares.lines().count()
        );
        let pension = fs::read_to_string(dir.join("pension-runs.csv")).expect("readable");
        assert_eq!(
            pension.lines().count(),
            1 + config.runs,
            "one pension row per run"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
