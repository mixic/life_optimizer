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

//! Monte Carlo simulator for a multipolar world, and what its equilibria imply
//! for pension security.
//!
//! Built to give the Life Optimizer something `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD`
//! section 2c says it lacks: a voice for "what happens if everyone does this".
//! The theory this operationalises is in `MULTIPOLAR_GAME.md` sections 4 and 7 and
//! `THEORY_OF_SPARING.md` section 7d.
//!
//! # Read the sensitivity sweep, not the headline
//!
//! The parameters are invented (see `blocks.rs`). A single run's 50-year
//! trajectory therefore carries almost no information on its own. What carries
//! information is *which qualitative outcomes survive across parameter ranges*,
//! which is why `--sweep` is the most useful mode and why the report prints its
//! caveat before any numbers.

mod blocks;
mod economy;
mod game;
mod pension;
mod report;
mod simulation;

use blocks::{GameParams, PowerBloc};
use simulation::{Config, Ensemble};

/// Options, parsed from `--flag value` pairs without an argument-parsing crate.
#[derive(Debug, Clone)]
struct Args {
    blocs: Vec<PowerBloc>,
    horizon: u32,
    runs: usize,
    seed: u64,
    cooperation_gain: f64,
    defection_temptation: f64,
    exploitation_cost: f64,
    parity_pressure: f64,
    tension_pressure: f64,
    conflict_wear: f64,
    volatility_scale: f64,
    energy_disruption_probability: f64,
    sweep: bool,
    compare: bool,
}

impl Default for Args {
    fn default() -> Self {
        let params = GameParams::default();
        Args {
            blocs: blocks::default_blocs(),
            horizon: 50,
            runs: 400,
            seed: simulation::DEFAULT_SEED,
            cooperation_gain: params.cooperation_gain,
            defection_temptation: params.defection_temptation,
            exploitation_cost: params.exploitation_cost,
            parity_pressure: params.parity_pressure,
            tension_pressure: params.tension_pressure,
            conflict_wear: params.conflict_wear,
            volatility_scale: 1.0,
            energy_disruption_probability: crate::simulation::ShockParams::default()
                .energy_disruption_probability,
            sweep: false,
            compare: false,
        }
    }
}

impl Args {
    /// Apply a `--flag` with its numeric value. Unknown keys are reported by the
    /// caller rather than silently ignored here.
    ///
    /// Integer-valued flags go through `f64` because the whole parser is numeric;
    /// `--seed` therefore cannot express values above 2^53, which is far beyond any
    /// seed this tool needs.
    fn set(&mut self, key: &str, value: f64) -> bool {
        let target = match key {
            "--cooperation-gain" => &mut self.cooperation_gain,
            "--temptation" => &mut self.defection_temptation,
            "--exploitation-cost" => &mut self.exploitation_cost,
            "--parity-pressure" => &mut self.parity_pressure,
            "--tension-pressure" => &mut self.tension_pressure,
            "--conflict-wear" => &mut self.conflict_wear,
            "--energy-disruption" => &mut self.energy_disruption_probability,
            "--volatility" => &mut self.volatility_scale,
            "--seed" => {
                self.seed = value.max(0.0) as u64;
                return true;
            }
            "--horizon" => {
                self.horizon = value.max(1.0) as u32;
                return true;
            }
            "--runs" => {
                self.runs = value.max(1.0) as usize;
                return true;
            }
            _ => return false,
        };
        *target = value;
        true
    }

    /// Apply a `--bloc Name:share:bias:volatility:affinity` definition.
    ///
    /// Replaces the bloc of that name if the system has one, otherwise adds it, so
    /// the default five-pole system can be edited field by field as well as
    /// replaced. Names match case-insensitively so the report's capitalisation does
    /// not have to be reproduced exactly.
    ///
    /// A malformed definition is reported and refused rather than half-applied: a
    /// bloc silently missing one field would look like a modelling result.
    fn set_bloc(&mut self, spec: &str) -> bool {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() != 5 {
            eprintln!(
                "warning: --bloc expects Name:share:bias:volatility:affinity, got {spec:?}"
            );
            return false;
        }
        let name = parts[0].trim();
        if name.is_empty() {
            eprintln!("warning: --bloc needs a non-empty name: {spec:?}");
            return false;
        }
        let parsed: Option<Vec<f64>> = parts[1..]
            .iter()
            .map(|field| field.trim().parse::<f64>().ok())
            .collect();
        let Some(numbers) = parsed else {
            eprintln!("warning: --bloc has a non-numeric field: {spec:?}");
            return false;
        };
        if numbers.iter().any(|value| !value.is_finite()) {
            eprintln!("warning: --bloc has a non-finite field: {spec:?}");
            return false;
        }

        let bloc = PowerBloc::new(name, numbers[0], numbers[1], numbers[2], numbers[3]);
        match self
            .blocs
            .iter_mut()
            .find(|existing| existing.name.eq_ignore_ascii_case(name))
        {
            Some(existing) => *existing = bloc,
            None => self.blocs.push(bloc),
        }
        true
    }

    fn to_config(&self) -> Config {
        let mut config = Config {
            blocs: self.blocs.clone(),
            horizon: self.horizon,
            runs: self.runs,
            seed: self.seed,
            game: GameParams {
                cooperation_gain: self.cooperation_gain,
                defection_temptation: self.defection_temptation,
                exploitation_cost: self.exploitation_cost,
                parity_pressure: self.parity_pressure,
                tension_pressure: self.tension_pressure,
                conflict_wear: self.conflict_wear,
                ..GameParams::default()
            },
            ..Config::default()
        };
        if (self.volatility_scale - 1.0).abs() > f64::EPSILON {
            for bloc in config.blocs.iter_mut() {
                bloc.volatility *= self.volatility_scale;
            }
        }
        config.shocks.energy_disruption_probability = self.energy_disruption_probability;
        config
    }
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args::default();

    let mut i = 0;
    while i < raw.len() {
        let key = raw[i].clone();
        match key.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--sweep" => {
                args.sweep = true;
                i += 1;
            }
            "--compare" => {
                args.compare = true;
                i += 1;
            }
            "--bloc" => {
                // The one flag whose value is not a number, so it cannot go through
                // the numeric path below.
                match raw.get(i + 1) {
                    Some(spec) => {
                        args.set_bloc(spec);
                        i += 2;
                    }
                    None => {
                        eprintln!("warning: --bloc needs a value");
                        i += 1;
                    }
                }
            }
            _ => {
                let parsed = raw.get(i + 1).and_then(|v| v.parse::<f64>().ok());
                match parsed {
                    Some(value) if args.set(&key, value) => i += 2,
                    _ => {
                        eprintln!("warning: ignoring unrecognised argument {key:?}");
                        i += 1;
                    }
                }
            }
        }
    }
    args
}

fn print_usage() {
    // Derived from the real defaults rather than written out by hand. They had
    // already drifted once: the help text still advertised `tension-pressure 1.4`
    // after the default became 0.9, which is exactly the kind of stale figure this
    // project does not tolerate elsewhere.
    let d = Args::default();
    println!(
        "\
Multipolar World Simulator

USAGE:
  multipolar_sim [--sweep] [options]

OPTIONS:
  --horizon <years>          simulation length                (default {horizon})
  --runs <n>                 Monte Carlo runs                 (default {runs})
  --seed <n>                 ensemble base seed; same seed    (default {seed})
                             reproduces the same ensemble
  --cooperation-gain <x>     payoff from mutual cooperation   (default {cooperation_gain:.1})
  --temptation <x>           payoff from defecting            (default {temptation:.1})
  --exploitation-cost <x>    sucker's payoff, negative        (default {exploitation_cost:.1})
  --parity-pressure <x>      how much parity raises stakes    (default {parity_pressure:.1})
  --tension-pressure <x>     how much tension erodes coop     (default {tension_pressure:.1})
  --conflict-wear <x>        how much tension erodes mutual   (default {conflict_wear:.1})
                             competition -- the arms-race
                             fatigue that lets conflict end
  --bloc <spec>              add or edit a power bloc; repeatable
  --volatility <scale>       multiply every bloc's volatility (default 1.0)
  --energy-disruption <p>    yearly chance the energy network (default {energy_disruption:.2})
                             is disrupted; exposure decides
                             who pays
  --compare                  run the cooperative and non-cooperative worlds
                             side by side and report who wins and who loses
  --sweep                    run the sensitivity sweep instead of one report
  --help                     this message

  `--seed` is printed above in the decimal form it is parsed from, so the value
  can be copied straight back onto the command line.

  `--bloc` takes Name:share:bias:volatility:affinity, matching the bloc name
  case-insensitively. It edits the bloc of that name if the system has one,
  otherwise it appends a new one, so the default five-pole system can be
  reshaped one field at a time or replaced outright.

  The five default blocs are Atlantic 0.30/0.000/0.020/1.00, Sinic
  0.26/0.008/0.025/0.95, Eurasian 0.16/0.002/0.035/0.70, Indo-Pacific
  0.14/0.010/0.030/0.90 and Non-Aligned 0.14/0.004/0.040/1.05, as
  share/growth-bias/volatility/cooperation-affinity.

The sweep is the informative mode: it shows which qualitative outcomes are robust
across parameter ranges and which flip on small changes. See report.rs.",
        horizon = d.horizon,
        runs = d.runs,
        seed = d.seed,
        cooperation_gain = d.cooperation_gain,
        temptation = d.defection_temptation,
        exploitation_cost = d.exploitation_cost,
        parity_pressure = d.parity_pressure,
        tension_pressure = d.tension_pressure,
        conflict_wear = d.conflict_wear,
        energy_disruption = d.energy_disruption_probability,
    );
}

/// Vary one parameter while holding the rest, and report how the outcomes move.
///
/// This is the deliverable that respects the honesty constraint: since the
/// parameters are invented, the *sensitivity* is the finding, not the level.
fn run_sweep(base: &Args) {
    println!();
    println!("{}", "=".repeat(78));
    println!("SENSITIVITY SWEEP");
    println!("{}", "=".repeat(78));
    println!("  The parameters are illustrative, so the LEVEL of any outcome is not a");
    println!("  finding. What follows is: how strongly the outcome depends on each");
    println!("  parameter. A wide swing means the conclusion is not robust to that");
    println!("  assumption, and should not be reported as one.");
    println!();

    type Apply = fn(&mut Args, f64);
    let sweeps: [(&str, Apply, [f64; 6]); 5] = [
        (
            "cooperation gain",
            |a: &mut Args, v: f64| a.cooperation_gain = v,
            [1.0, 2.0, 3.0, 4.5, 6.0, 8.0],
        ),
        (
            "defection temptation",
            |a: &mut Args, v: f64| a.defection_temptation = v,
            [2.0, 3.0, 4.2, 6.0, 8.0, 10.0],
        ),
        (
            "conflict wear (arms-race fatigue)",
            |a: &mut Args, v: f64| a.conflict_wear = v,
            [0.0, 0.3, 0.6, 0.9, 1.5, 3.0],
        ),
        (
            "energy disruption probability",
            |a: &mut Args, v: f64| a.energy_disruption_probability = v,
            [0.0, 0.05, 0.10, 0.20, 0.35, 0.50],
        ),
        (
            "bloc volatility (scale)",
            |a: &mut Args, v: f64| a.volatility_scale = v,
            [0.5, 1.0, 2.0, 3.0, 4.0, 6.0],
        ),
    ];

    for (name, apply, values) in sweeps {
        println!("  Varying {name}");
        println!("  {}", "-".repeat(74));
        println!(
            "  {:>8}  {:>8}  {:>9}  {:>8}  {:>13}  {:>8}",
            "value", "coop", "trap yrs", "Pareto", "polarity", "pension"
        );

        for value in values {
            let mut args = base.clone();
            apply(&mut args, value);
            let config = args.to_config();
            let ensemble = Ensemble::run(&config);

            let (coop, _, _) = ensemble.summarize(|o| o.mean_cooperation);
            let (trap, _, _) = ensemble.summarize(|o| o.trap_fraction);
            let (loss, _, _) = ensemble.summarize(|o| o.mean_efficiency_loss);
            let (observed, reference) = ensemble.pension_summary(&config.pension);

            // The most likely polarity, as a one-word summary.
            let polarity = ensemble
                .polarity_distribution()
                .into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(p, prob)| format!("{} {:.0}%", p.label(), prob * 100.0))
                .unwrap_or_else(|| "-".to_string());

            println!(
                "  {:>8.2}  {:>8.3}  {:>8.1}%  {:>8.3}  {:>13}  {:>8.3}",
                value,
                coop,
                trap * 100.0,
                loss,
                polarity,
                observed.security_index(&reference)
            );
        }
        println!();
    }

    println!("  HOW TO READ THIS");
    println!("  ----------------");
    println!("  A parameter whose column barely moves is one the qualitative");
    println!("  conclusion does not depend on -- safe to be wrong about. A parameter");
    println!("  that flips the outcome across its plausible range is one the");
    println!("  conclusion IS conditional on, and any claim resting on it should say");
    println!("  so. The temptation and cooperation-gain rows are the pair that decides");
    println!("  whether the security dilemma binds at all, which is exactly the");
    println!("  question MULTIPOLAR_GAME.md section 4 says the literature is split on.");
    println!("  The conflict-wear row is the one that decides whether conflict can ever");
    println!("  stop once it has started: at 0.0 the dilemma is permanent by");
    println!("  construction, and no amount of tension buys an exit from it.");
    println!();
    println!("  One caveat on the Pareto column: it is NOT monotone by construction.");
    println!("  Efficiency loss is normalised by the surplus the game actually offers,");
    println!("  and that span moves with these same parameters, so a row can show more");
    println!("  loss at one setting than another without either being \"worse\". Read it");
    println!("  down a column only alongside the coop and trap columns that explain it.");
}

/// One of the two worlds compared by `--compare`.
struct Mode {
    label: &'static str,
    /// Plain-language statement of what this balance is meant to represent.
    intent: &'static str,
    cooperation_gain: f64,
    defection_temptation: f64,
}

/// The two balances `--compare` uses, and the seed-free way to name them.
///
/// Kept as a function so the test below can assert that these numbers really do
/// produce the regimes they are labelled with, rather than the labels being taken
/// on trust.
fn comparison_modes() -> [Mode; 2] {
    [
        Mode {
            label: "COOPERATIVE",
            intent: "cooperation is worth clearly more than defecting",
            cooperation_gain: 8.0,
            defection_temptation: 4.2,
        },
        Mode {
            label: "NON-COOPERATIVE",
            intent: "defecting is worth clearly more than cooperating",
            cooperation_gain: 1.0,
            defection_temptation: 8.0,
        },
    ]
}

/// Run the same years twice -- once where cooperation pays, once where it does not
/// -- and report who gains, who loses, and whether the ranking changes.
///
/// # Why both a ranking and an aggregate
///
/// "Who wins" has two different answers depending on the question, and a report
/// giving only one of them would mislead. A bloc can hold or extend its *share* of
/// world power while the world it is winning in is poorer, more conflict-ridden,
/// and worse for its own pensioners. So the ranking and the aggregate are printed
/// side by side, from the same shock draws.
///
/// # What is being compared
///
/// The two worlds differ in the whole cooperation/competition balance, not in one
/// coefficient, so this compares *balances* rather than isolating a single
/// parameter. Both share the seed, so the difference between them is the structure
/// of the game rather than luck.
fn run_comparison(base: &Args) {
    let modes = comparison_modes();

    let mut results: Vec<(&Mode, Config, Ensemble)> = Vec::new();
    for mode in &modes {
        let mut args = base.clone();
        args.cooperation_gain = mode.cooperation_gain;
        args.defection_temptation = mode.defection_temptation;
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        results.push((mode, config, ensemble));
    }

    println!();
    println!("{}", "=".repeat(78));
    println!("WHO WINS, AND WHO LOSES");
    println!("{}", "=".repeat(78));
    println!(
        "  The same {} years, run twice, {} Monte Carlo runs each, same seed.",
        base.horizon, base.runs
    );
    println!("  The two worlds differ only in the cooperation/competition payoff");
    println!("  balance, so what separates them is the game's structure, not luck.");

    println!();
    println!("  THE TWO WORLDS");
    println!("  {}", "-".repeat(74));
    for (mode, _, _) in &results {
        println!("  {:<16} {}", mode.label, mode.intent);
        println!(
            "  {:<16} cooperation gain {:.1}, defection temptation {:.1}",
            "", mode.cooperation_gain, mode.defection_temptation
        );
    }

    // Whether each world actually became the mode it is labelled as. Measured
    // rather than assumed: a label that did not hold would make the whole
    // comparison meaningless, so it is printed where a reader can check it.
    println!();
    println!("  Did each world actually become the mode it is labelled as?");
    for (mode, _, ensemble) in &results {
        let (coop, _, _) = ensemble.summarize(|o| o.mean_cooperation);
        let (trap, _, _) = ensemble.summarize(|o| o.trap_fraction);
        println!(
            "    {:<16} realised cooperation {:.3}, conflict-trap years {:.1}%",
            mode.label,
            coop,
            trap * 100.0
        );
    }

    // ---- how the world itself fares ----------------------------------------
    let coop: Vec<f64> = results
        .iter()
        .map(|(_, _, e)| e.summarize(|o| o.mean_cooperation).0)
        .collect();
    let trap: Vec<f64> = results
        .iter()
        .map(|(_, _, e)| e.summarize(|o| o.trap_fraction).0)
        .collect();
    let loss: Vec<f64> = results
        .iter()
        .map(|(_, _, e)| e.summarize(|o| o.mean_efficiency_loss).0)
        .collect();
    let pension: Vec<f64> = results
        .iter()
        .map(|(_, config, ensemble)| {
            let (observed, reference) = ensemble.pension_summary(&config.pension);
            observed.security_index(&reference)
        })
        .collect();

    println!();
    println!("  HOW THE WORLD ITSELF FARES");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:<26} {:>14} {:>16}   better",
        "", "cooperative", "non-coop."
    );
    let rows: [(&str, &[f64], bool); 4] = [
        ("cooperation index", &coop, true),
        ("conflict-trap years", &trap, false),
        ("Pareto-efficiency loss", &loss, false),
        ("pension security index", &pension, true),
    ];
    for (label, values, higher_is_better) in rows {
        let cooperative_better = (values[0] > values[1]) == higher_is_better;
        println!(
            "  {:<26} {:>14.3} {:>16.3}   {}",
            label,
            values[0],
            values[1],
            if cooperative_better {
                "cooperative"
            } else {
                "non-cooperative"
            }
        );
    }

    // ---- who wins and who loses --------------------------------------------
    let blocs = &results[0].1.blocs;
    let n = blocs.len();
    let ends: Vec<Vec<f64>> = results
        .iter()
        .map(|(_, _, e)| e.mean_shares_by_year.last().cloned().unwrap_or_default())
        .collect();
    let dominance: Vec<Vec<f64>> = results
        .iter()
        .map(|(_, _, e)| e.dominance_probabilities(n))
        .collect();

    println!();
    println!("  SHARE OF WORLD POWER -- start versus horizon");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:<16} {:>8} {:>13} {:>10} {:>15}",
        "bloc", "start", "cooperative", "change", "non-cooperative"
    );
    for (index, bloc) in blocs.iter().enumerate() {
        let start = bloc.power_share;
        let with = ends[0].get(index).copied().unwrap_or(0.0);
        let without = ends[1].get(index).copied().unwrap_or(0.0);
        println!(
            "  {:<16} {:>7.1}% {:>12.1}% {:>+9.1}pp {:>14.1}%",
            bloc.name,
            start * 100.0,
            with * 100.0,
            (with - start) * 100.0,
            without * 100.0
        );
    }

    println!();
    println!("  PROBABILITY OF ENDING DOMINANT (above 45% of world power)");
    println!("  {}", "-".repeat(74));
    println!("  {:<16} {:>14} {:>18}", "", "cooperative", "non-cooperative");
    for (index, bloc) in blocs.iter().enumerate() {
        let with = dominance[0].get(index).copied().unwrap_or(0.0);
        let without = dominance[1].get(index).copied().unwrap_or(0.0);
        if with.max(without) > 0.0 {
            println!(
                "  {:<16} {:>13.1}% {:>17.1}%",
                bloc.name,
                with * 100.0,
                without * 100.0
            );
        }
    }

    // ---- what it means -----------------------------------------------------
    let largest = |mode: usize| -> String {
        ends[mode]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .and_then(|(index, _)| blocs.get(index))
            .map(|bloc| bloc.name.clone())
            .unwrap_or_else(|| "-".to_string())
    };
    let top_cooperative = largest(0);
    let top_competitive = largest(1);

    println!();
    println!("  WHAT THIS SAYS");
    println!("  {}", "-".repeat(74));
    println!(
        "  Largest bloc: cooperative -> {top_cooperative},  non-cooperative -> {top_competitive}"
    );
    if top_cooperative == top_competitive {
        println!("  The ranking does NOT change between the two worlds: the same bloc ends");
        println!("  on top either way, so who wins is not decided by the cooperation");
        println!("  question. What is decided by it is how much there is to win.");
    } else {
        println!("  The ranking DOES change between the two worlds, so who comes out on");
        println!("  top depends on whether cooperation pays.");
    }

    let gaining: Vec<&str> = blocs
        .iter()
        .enumerate()
        .filter(|&(index, bloc)| ends[0].get(index).copied().unwrap_or(0.0) > bloc.power_share + 0.01)
        .map(|(_, bloc)| bloc.name.as_str())
        .collect();
    let losing: Vec<&str> = blocs
        .iter()
        .enumerate()
        .filter(|&(index, bloc)| ends[0].get(index).copied().unwrap_or(0.0) < bloc.power_share - 0.01)
        .map(|(_, bloc)| bloc.name.as_str())
        .collect();

    println!();
    println!(
        "  Even in the cooperative world -- cooperation {:.2}, conflict-trap years {:.1}% --",
        coop[0],
        trap[0] * 100.0
    );
    println!("  the shares still move:");
    println!(
        "    gaining: {}",
        if gaining.is_empty() {
            "nobody".to_string()
        } else {
            gaining.join(", ")
        }
    );
    println!(
        "    losing:  {}",
        if losing.is_empty() {
            "nobody".to_string()
        } else {
            losing.join(", ")
        }
    );
    println!();
    println!("  That is the security dilemma written as an outcome rather than as a");
    println!("  mechanism: cooperation is not the same thing as equality, and a");
    println!("  cooperative world can still produce losers. It does.");

    // Which bloc finishes first is the least robust thing on this page, and it would
    // be easy to read as the headline. Say what actually drives it.
    let world_improves = coop[0] > coop[1]
        && trap[0] < trap[1]
        && loss[0] < loss[1]
        && pension[0] > pension[1];
    println!();
    println!("  Which claim here is worth carrying away");
    println!("  {}", "-".repeat(74));
    if world_improves {
        println!("  ROBUST: every aggregate above favours the cooperative world. It is");
        println!("  richer, less conflict-ridden, and better for a pensioner in it -- while");
        println!("  still redistributing power away from three of the five blocs.");
    } else {
        println!("  The aggregates do NOT all point the same way in this configuration,");
        println!("  which is itself worth looking at before reading the ranking.");
    }
    println!("  NOT ROBUST: the name of the largest bloc. It is decided by the starting");
    println!("  shares and growth biases in blocks.rs, every one of which is invented, so");
    println!("  treat cooperative -> {top_cooperative} as a property of this");
    println!("  parameterisation rather than as a prediction about the world.");

    println!();
    println!("  {}", "-".repeat(74));
    println!("  The payoff parameters are invented, so read the DIRECTION of these");
    println!("  differences, not their size. --sweep shows how much the direction itself");
    println!("  depends on the invented numbers.");
    println!("{}", "=".repeat(78));
}

fn main() {
    let args = parse_args();

    if args.compare {
        run_comparison(&args);
        return;
    }

    if args.sweep {
        run_sweep(&args);
        return;
    }

    let config = args.to_config();
    let ensemble = Ensemble::run(&config);
    report::print_report(&config, &ensemble);

    println!();
    println!("  {}", "-".repeat(74));
    println!("  Run with --sweep to see how much of the above depends on parameters");
    println!("  that were invented rather than measured. That is the more useful run.");
    println!("{}", "=".repeat(78));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `--bloc` must edit the named bloc rather than duplicating it, must append an
    /// unknown one, and must refuse a malformed spec outright. "Editable power
    /// blocs" is only a real property if editing one behaves predictably, and a bloc
    /// silently missing a field would look like a modelling result.
    #[test]
    fn bloc_definitions_edit_or_append_but_never_half_apply() {
        let mut args = Args::default();
        let before = args.blocs.len();

        // Editing an existing bloc replaces it in place, field for field.
        assert!(args.set_bloc("Atlantic:0.40:0.001:0.030:0.80"));
        assert_eq!(args.blocs.len(), before, "an edit must not add a bloc");
        let atlantic = args
            .blocs
            .iter()
            .find(|bloc| bloc.name == "Atlantic")
            .expect("Atlantic is one of the default blocs");
        assert_eq!(atlantic.power_share, 0.40);
        assert_eq!(atlantic.growth_bias, 0.001);
        assert_eq!(atlantic.volatility, 0.030);
        assert_eq!(atlantic.cooperation_affinity, 0.80);

        // Matching is case-insensitive and tolerates surrounding whitespace, so the
        // report's capitalisation does not have to be reproduced exactly.
        assert!(args.set_bloc("  sInIc : 0.20 : 0.002 : 0.020 : 0.90"));
        assert_eq!(
            args.blocs.len(),
            before,
            "a case-different name is still an edit, not an addition"
        );
        assert_eq!(
            args.blocs
                .iter()
                .filter(|bloc| bloc.name.eq_ignore_ascii_case("sinic"))
                .count(),
            1,
            "the edit must not have created a second Sinic"
        );

        // An unknown name appends, which is how a sixth pole is introduced.
        assert!(args.set_bloc("Antarctic:0.05:0.000:0.010:1.00"));
        assert_eq!(args.blocs.len(), before + 1);

        // Malformed specs are refused and change nothing.
        let snapshot = args.blocs.len();
        for bad in [
            "Atlantic",
            "Atlantic:0.3:0.0:0.02",
            "Atlantic:0.3:0.0:0.02:1.0:extra",
            ":0.3:0.0:0.02:1.0",
            "Atlantic:share:0.0:0.02:1.0",
            "Atlantic:inf:0.0:0.02:1.0",
        ] {
            assert!(!args.set_bloc(bad), "{bad:?} should be refused");
            assert_eq!(args.blocs.len(), snapshot, "{bad:?} must not be applied");
        }
    }

    /// `--compare` is only meaningful if its two parameterisations really do produce
    /// the regimes they are labelled with. If a change to the defaults or to the
    /// mapping made both modes cooperate equally, the comparison would quietly
    /// become a comparison of nothing -- and the output would still look plausible,
    /// which is the dangerous kind of failure.
    #[test]
    fn the_comparison_modes_produce_the_regimes_they_are_named_for() {
        let modes = comparison_modes();
        assert_eq!(modes.len(), 2, "there are exactly two worlds to compare");
        assert_ne!(
            modes[0].label, modes[1].label,
            "two identical labels would be unreadable"
        );

        let cooperativeness = |mode: &Mode| -> f64 {
            let config = Config {
                game: GameParams {
                    cooperation_gain: mode.cooperation_gain,
                    defection_temptation: mode.defection_temptation,
                    ..GameParams::default()
                },
                horizon: 50,
                runs: 40,
                ..Config::default()
            };
            Ensemble::run(&config).summarize(|o| o.mean_cooperation).0
        };

        let cooperative = cooperativeness(&modes[0]);
        let competitive = cooperativeness(&modes[1]);

        assert!(
            cooperative > 0.9,
            "the COOPERATIVE mode must actually be cooperative, got {cooperative:.3}"
        );
        assert!(
            competitive < 0.3,
            "the NON-COOPERATIVE mode must actually be uncooperative, got {competitive:.3}"
        );
        assert!(
            cooperative > competitive + 0.5,
            "the two modes must be far apart or the comparison means nothing: \
             {cooperative:.3} vs {competitive:.3}"
        );
    }
}
