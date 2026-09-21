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

mod ai;
mod blocks;
mod economy;
mod game;
mod pension;
mod report;
mod simulation;

use ai::{AiParams, AiRole, AI_ACTOR_NAME};
use blocks::{GameParams, PowerBloc};
use economy::Economy;
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
    /// Starting share for the AI actor, when AI is modelled as a player.
    ai_share: f64,
    /// Growth bias for the AI actor, when AI is modelled as a player. The hinge the
    /// player hypothesis turns on.
    ai_growth: f64,
    /// Annual power growth bought by one unit of AI lead, when AI is modelled as a
    /// capability the blocs own. The hinge the tool hypothesis turns on.
    ai_lead_effect: f64,
    /// How much AI leadership changes what cooperation is worth. Zero by default,
    /// deliberately -- see `ai.rs`.
    ai_cooperation: f64,
    sweep: bool,
    compare: bool,
    ai: bool,
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
            ai_share: ai::AI_STARTING_SHARE,
            ai_growth: ai::AI_GROWTH_BIAS,
            ai_lead_effect: ai::DEFAULT_LEAD_GROWTH_EFFECT,
            ai_cooperation: ai::DEFAULT_LEAD_COOPERATION_EFFECT,
            sweep: false,
            compare: false,
            ai: false,
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
            "--ai-share" => &mut self.ai_share,
            "--ai-growth" => &mut self.ai_growth,
            "--ai-lead-effect" => &mut self.ai_lead_effect,
            "--ai-cooperation" => &mut self.ai_cooperation,
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
            eprintln!("warning: --bloc expects Name:share:bias:volatility:affinity, got {spec:?}");
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
        // Every vector in `Economy` is indexed by bloc position, so it has to be
        // rebuilt for the blocs actually being run. It used to be inherited from
        // `Config::default()`, which is built for the five default blocs: any bloc
        // added through `--bloc` therefore had no energy position and no reserve
        // share, and since the disruption loop iterates the economy's vector rather
        // than the bloc list, such a bloc was silently immune to energy disruption.
        config.economy = Economy::for_blocs(&config.blocs);
        config
    }

    /// Apply this run's CLI overrides to an AI layer.
    ///
    /// Kept separate from the world constructors so the sweep can move one parameter
    /// at a time while everything else stays at the documented default, exactly as
    /// `run_sweep` does for the game parameters.
    fn apply_ai_overrides(&self, layer: &mut AiParams) {
        layer.actor.power_share = self.ai_share;
        layer.actor.growth_bias = self.ai_growth;
        layer.lead_growth_effect = self.ai_lead_effect;
        layer.lead_cooperation_effect = self.ai_cooperation;
    }

    /// Build an AI layer of the given role with this run's overrides applied.
    ///
    /// The three worlds come from `ai::ai_worlds` rather than being rebuilt here, so
    /// there is exactly one definition of what each hypothesis is.
    fn ai_params(&self, role: AiRole) -> AiParams {
        let mut layer = ai::ai_worlds(&self.blocs)
            .into_iter()
            .find(|layer| layer.role == role)
            .expect("every AI role is one of the three worlds");
        self.apply_ai_overrides(&mut layer);
        layer
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
            "--ai" => {
                args.ai = true;
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
  multipolar_sim [--sweep | --compare | --ai] [options]

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
  --ai                       run AI as a player and AI as a tool side by side,
                             then sweep for the hinge the answer turns on
  --ai-share <x>             AI actor's starting share            (default {ai_share:.3})
  --ai-growth <x>            AI actor's growth bias               (default {ai_growth:.3})
  --ai-lead-effect <x>       power per unit of AI lead, when AI   (default {ai_lead_effect:.3})
                             is a capability the blocs own
  --ai-cooperation <x>       how much AI leadership changes what  (default {ai_cooperation:.2})
                             cooperation is worth; zero by default
                             because the sign is genuinely disputed
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

  `--ai` is not a sixth bloc and not a multiplier: it is both, run as two
  rival hypotheses about what AI is, against the existing five-bloc model
  as a control. MULTIPOLAR_GAME.md section 7 lists four live answers to who
  captures the gains from AI and declines to pick one; this mode builds the
  two that are structurally different and reports which one the numbers
  support, and how much that depends on parameters nobody has measured.

  When AI is modelled as a player it is added as a bloc named {ai_actor},
  so `--bloc {ai_actor}:...` edits it like any other bloc. The layer
  rebuilds that actor from the `--ai-*` flags, so those win where the two
  disagree; and the control and tool worlds contain no actor at all, since
  a stray one would contradict the premise they are reported under.

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
        ai_share = d.ai_share,
        ai_growth = d.ai_growth,
        ai_lead_effect = d.ai_lead_effect,
        ai_cooperation = d.ai_cooperation,
        ai_actor = AI_ACTOR_NAME,
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
    println!(
        "  {:<16} {:>14} {:>18}",
        "", "cooperative", "non-cooperative"
    );
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
        .filter(|&(index, bloc)| {
            ends[0].get(index).copied().unwrap_or(0.0) > bloc.power_share + 0.01
        })
        .map(|(_, bloc)| bloc.name.as_str())
        .collect();
    let losing: Vec<&str> = blocs
        .iter()
        .enumerate()
        .filter(|&(index, bloc)| {
            ends[0].get(index).copied().unwrap_or(0.0) < bloc.power_share - 0.01
        })
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
    let world_improves =
        coop[0] > coop[1] && trap[0] < trap[1] && loss[0] < loss[1] && pension[0] > pension[1];
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

/// The figures every hinge row reports, so the rows differ only in what they vary.
struct HingePoint {
    cooperation: f64,
    trap: f64,
    top_share: f64,
    /// Which bloc holds the top share, so the reader can see a change of leader
    /// rather than having to infer it from a non-monotone column.
    leading: String,
    pension: f64,
    /// Present only when the world has an AI actor.
    actor_share: Option<f64>,
    actor_start: Option<f64>,
    actor_dominance: Option<f64>,
}

/// Extract the hinge figures from one finished world.
fn hinge_point(config: &Config, ensemble: &Ensemble, actor: Option<usize>) -> HingePoint {
    let (cooperation, _, _) = ensemble.summarize(|o| o.mean_cooperation);
    let (trap, _, _) = ensemble.summarize(|o| o.trap_fraction);
    let (observed, reference) = ensemble.pension_summary(&config.pension);
    let shares = ensemble
        .mean_shares_by_year
        .last()
        .cloned()
        .unwrap_or_default();
    let leading = shares
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .and_then(|(index, _)| config.blocs.get(index))
        .map(|bloc| bloc.name.clone())
        .unwrap_or_else(|| "-".to_string());

    HingePoint {
        cooperation,
        trap,
        top_share: shares.iter().cloned().fold(0.0_f64, f64::max),
        leading,
        pension: observed.security_index(&reference),
        actor_share: actor.and_then(|index| shares.get(index).copied()),
        actor_start: actor.and_then(|index| config.blocs.get(index).map(|bloc| bloc.power_share)),
        actor_dominance: actor.and_then(|index| {
            ensemble
                .dominance_probabilities(config.blocs.len())
                .get(index)
                .copied()
        }),
    }
}

/// Run the three AI worlds from the same shock draws, then locate the hinge.
fn run_ai(base: &Args) {
    let mut params = Vec::new();
    let mut configs = Vec::new();
    let mut ensembles = Vec::new();

    for mut layer in ai::ai_worlds(&base.blocs) {
        base.apply_ai_overrides(&mut layer);
        let mut args = base.clone();
        args.blocs = layer.apply(&base.blocs);
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        params.push(layer);
        configs.push(config);
        ensembles.push(ensemble);
    }

    report::print_ai_report(&params, &configs, &ensembles);
    run_ai_hinge(base);
}

/// Where each hypothesis's answer flips.
///
/// This is the half of `--ai` that respects the honesty constraint. The worlds above
/// are illustrative by construction, so their levels carry no information; what
/// carries information is how far a parameter has to move before the verdict changes.
fn run_ai_hinge(base: &Args) {
    println!();
    println!("{}", "=".repeat(78));
    println!("WHERE THE ANSWER FLIPS");
    println!("{}", "=".repeat(78));
    println!("  Every parameter in this layer is invented, so the level of any outcome is");
    println!("  not a finding. What follows is: how much the ANSWER depends on the one");
    println!("  parameter each hypothesis actually turns on. A verdict that holds across");
    println!("  the whole range is one worth taking seriously; one that flips in the middle");
    println!("  of it is one that rests on a guess.");
    println!();

    // ---- 1. the player's growth advantage ----------------------------------
    println!("  1. AI as a PLAYER: how far must it outgrow the field to become hegemon?");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:>8}  {:>9}  {:>8}  {:>9}  {:>8}  {:<12}",
        "growth", "effective", "AI end", "dominance", "coop", "fate"
    );
    for value in [0.000, 0.004, 0.007, 0.010, 0.013, 0.017, 0.025, 0.040] {
        let mut args = base.clone();
        args.ai_growth = value;
        let layer = args.ai_params(AiRole::SixthPower);
        args.blocs = layer.apply(&base.blocs);
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        let actor = layer.actor_index(&config.blocs);
        let point = hinge_point(&config, &ensemble, actor);
        let n = config.blocs.len();

        let (Some(start), Some(end), Some(dominance)) =
            (point.actor_start, point.actor_share, point.actor_dominance)
        else {
            continue;
        };
        let fate = ai::actor_fate(start, end, dominance);
        println!(
            "  {:>8.4}  {:>8.2}%  {:>7.1}%  {:>8.1}%  {:>8.3}  {:<12}",
            value,
            ai::effective_annual_advantage(value, n) * 100.0,
            end * 100.0,
            dominance * 100.0,
            point.cooperation,
            fate.label()
        );
    }
    println!();
    println!("  `effective` is the annual growth the nominal bias actually buys. The model");
    println!("  applies a bloc's growth bias once per dyad *and* once more in the annual");
    println!("  drift step, so a nominal rate is compounded once per bloc per year and its");
    println!("  real meaning depends on how many blocs exist. The conventional blocs land");
    println!("  near 3% a year, which is the column's comparison point. That multiplicity");
    println!("  is a property of the existing model, not of this layer, and it is reported");
    println!("  here rather than silently corrected, because correcting it would move every");
    println!("  published --compare and --sweep result.");
    println!();
    println!("  Read the fate column for the answer. Where it turns from ABSORBED or");
    println!("  ASCENDANT into HEGEMON is the growth advantage the hypothesis requires --");
    println!("  and it is that number, not the share printed next to it, that is worth");
    println!("  arguing about. Note what the default row says on the way there: at the");
    println!("  fastest-growing bloc's own growth rate the actor is ABSORBED, ending below");
    println!("  where it started, because in a mostly uncooperative world it absorbs the");
    println!("  maximum sanction drag from every currency issuer in the system and applies");
    println!("  none. Growing faster than every bloc is not by itself enough to hold.");
    println!();
    println!("  Two things the fate column does not measure, so that it is not read as more");
    println!("  than it is. The dominance threshold is the same 45% the polarity classifier");
    println!("  uses, so 'hegemon' here and 'unipolar' there are one claim, not two. And the");
    println!("  band between ASCENDANT and HEGEMON is wide: at 0.0130 the actor reaches 14%");
    println!("  of world power -- a major pole by any reading -- while the verdict still says");
    println!("  ASCENDANT, because it is not a hegemon. The verdict answers 'does it come to");
    println!("  dominate', not 'does it matter'.");

    // ---- 2. the tool's ownership payoff ------------------------------------
    println!();
    println!("  2. AI as a TOOL: how uneven must ownership be to move the hierarchy?");
    println!("  {}", "-".repeat(74));

    let control_config = {
        let mut args = base.clone();
        args.blocs = base.ai_params(AiRole::Absent).apply(&base.blocs);
        args.to_config()
    };
    let control_ensemble = Ensemble::run(&control_config);
    let control = hinge_point(&control_config, &control_ensemble, None);

    println!(
        "  {:<12} {:>10}  {:>10}  {:>8}  {:<14} leader",
        "lead effect", "top share", "vs no AI", "coop", "effect"
    );
    for value in [0.000, 0.005, 0.010, 0.020, 0.040, 0.080] {
        let mut args = base.clone();
        args.ai_lead_effect = value;
        args.blocs = args.ai_params(AiRole::WieldedInstrument).apply(&base.blocs);
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        let point = hinge_point(&config, &ensemble, None);
        let effect = ai::instrument_effect(control.top_share, point.top_share);
        println!(
            "  {:<12.4} {:>9.1}%  {:>+9.1}pp  {:>8.3}  {:<14} {}",
            value,
            point.top_share * 100.0,
            (point.top_share - control.top_share) * 100.0,
            point.cooperation,
            effect.label(),
            point.leading
        );
    }
    println!();
    println!("  The 0.000 row reproduces the control exactly, which is what makes the rest of",);
    println!("  the column readable: every other row is that same world with the lead term");
    println!(
        "  switched on. The comparison point is the no-AI top share of {:.1}%.",
        control.top_share * 100.0
    );

    // The control the whole hypothesis rests on, measured rather than asserted: a
    // lead every bloc holds equally should leave the hierarchy almost exactly where
    // it was, because a common lift to everyone's growth is very nearly a no-op once
    // shares are renormalised.
    let uniform = {
        let mut args = base.clone();
        let mut layer = ai::AiParams::uniform_lead(&base.blocs);
        args.apply_ai_overrides(&mut layer);
        args.blocs = layer.apply(&base.blocs);
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        hinge_point(&config, &ensemble, None)
    };
    println!();
    println!("  CONTROL -- every bloc given the SAME lead of 1.00:");
    println!(
        "    top share {:.1}%  ({:+.1}pp against no AI at all), cooperation {:.3}",
        uniform.top_share * 100.0,
        (uniform.top_share - control.top_share) * 100.0,
        uniform.cooperation
    );
    println!("  A lead that everyone holds equally barely moves the hierarchy, which is the");
    println!("  finding the column above depends on: what AI ownership buys is decided by");
    println!("  the *spread* of the lead vector, not by its level. It is barely rather than");
    println!("  exactly, because the payoff feedback inside a year is not proportional");
    println!("  across blocs, so a common lift is not perfectly neutral.");
    println!();
    println!("  One caveat on the top-share column: it is NOT a dose-response curve, and it");
    println!("  is not monotone. Two different things can make it fall while the lead term");
    println!("  grows, and the leader column only shows the second of them.");
    println!();
    println!("  First, top share measures *concentration*, not inequality. A bigger lead");
    println!("  effect lets the two frontier leaders both pull away from the laggards, so");
    println!("  the leaders converge on each other while the spread between leaders and");
    println!("  the rest widens -- and the top share falls, because the gap between the top");
    println!("  two has closed. That is what happens between 0.0200 and 0.0400, where the");
    println!("  leader is unchanged and the top share drops by 17 points: nothing has gone");
    println!("  wrong, the measure is simply answering a different question than the one it");
    println!("  looks like it answers.");
    println!();
    println!("  Second, past a large enough lead the ranking itself flips -- at 0.0800 the");
    println!("  leader becomes Atlantic, whose lead is the largest -- and the new winner's");
    println!("  trajectory is then computed from a different path altogether.");
    println!();
    println!("  So read this column as 'uneven AI ownership moves the hierarchy a great");
    println!("  deal', which the whole range shows, and not as 'this much ownership buys");
    println!("  this much concentration', which it does not show at any point.");

    // ---- 3. the disputed sign ----------------------------------------------
    println!();
    println!("  3. AI as a TOOL: can this model even express 'AI makes cooperation worth more'?");
    println!("  {}", "-".repeat(74));
    println!("  This is a NULL TEST, and it is included because it fails.");
    println!();
    println!(
        "  {:<12} {:>8}  {:>9}  {:>10}  {:>9}",
        "coefficient", "coop", "trap yrs", "top share", "pension"
    );
    for value in [-0.30, -0.15, 0.00, 0.15, 0.30] {
        let mut args = base.clone();
        args.ai_cooperation = value;
        args.blocs = args.ai_params(AiRole::WieldedInstrument).apply(&base.blocs);
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        let point = hinge_point(&config, &ensemble, None);
        println!(
            "  {:<12.2} {:>8.3}  {:>8.1}%  {:>9.1}%  {:>9.3}",
            value,
            point.cooperation,
            point.trap * 100.0,
            point.top_share * 100.0,
            point.pension
        );
    }
    println!();
    println!("  The cooperation column does not move at all, and that is not a null result");
    println!("  about AI -- it is a limitation of this model, which is worth stating plainly");
    println!("  rather than dressing up as a finding. `cooperation_affinity` scales the");
    println!("  payoff a bloc *banks* when it cooperates. It never enters the solved payoff");
    println!("  matrix, because the 2x2 is symmetric: one matrix describes both sides, so");
    println!("  there is nowhere to put 'cooperation is worth more to this bloc'. Changing");
    println!("  the coefficient therefore moves how power is distributed after the fact and");
    println!("  cannot move whether a dyad cooperates in the first place.");
    println!();
    println!("  So MULTIPOLAR_GAME.md section 4's disputed sign -- realist AI as one more");
    println!("  axis of zero-sum rivalry versus AI competition that need not be zero-sum --");
    println!("  is not currently expressible in this model. Making the 2x2 itself");
    println!("  asymmetric is what would make it expressible, and that is the structural");
    println!("  step README.md already records as outstanding. The coefficient defaults to");
    println!("  zero because the sign is disputed, and the row is here because a flag that");
    println!("  silently cannot do what its name suggests is worse than one that says so.");

    println!();
    println!("  {}", "-".repeat(74));
    println!("  Every number in this sweep comes from invented parameters. Its value is not");
    println!("  that any row is right, but that the rows show which claims are load-bearing");
    println!("  and which are free.");
    println!("{}", "=".repeat(78));
}

fn main() {
    let args = parse_args();

    if args.ai {
        run_ai(&args);
        return;
    }

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

    /// `cooperation_affinity` must not be able to move the cooperation rate.
    ///
    /// The 2x2 is symmetric -- one payoff matrix describes both sides -- so there is
    /// nowhere in the solved matrix to put "cooperation is worth more to this bloc".
    /// Affinity scales the payoff a bloc *banks* when it cooperates, downstream of the
    /// solver, so it can redistribute power after the fact but cannot change whether a
    /// dyad cooperates in the first place.
    ///
    /// This is the invariant that makes `--ai`'s third sweep row a *proof* of a
    /// limitation rather than a null result. If it ever stopped holding, that flat
    /// column would become a bug report, and the mode's explanation of why
    /// MULTIPOLAR_GAME.md section 4's disputed sign is not currently expressible would
    /// become false.
    #[test]
    fn cooperation_affinity_cannot_move_the_cooperation_rate() {
        let blocs = blocks::default_blocs();
        let cooperativeness = |effect: f64| -> f64 {
            let mut layer = ai::AiParams::wielded_instrument(&blocs);
            layer.lead_cooperation_effect = effect;
            let config = Config {
                blocs: layer.apply(&blocs),
                // One year: the shares entering the solver are identical between the
                // two runs, so any difference would have to come from affinity itself.
                horizon: 1,
                runs: 20,
                ..Config::default()
            };
            Ensemble::run(&config).summarize(|o| o.mean_cooperation).0
        };

        let suppressed = cooperativeness(-0.50);
        let amplified = cooperativeness(2.00);
        assert!(
            (suppressed - amplified).abs() < 1e-12,
            "affinity has no route into the payoff matrix, so it cannot move the \
             cooperation rate: {suppressed} vs {amplified}"
        );

        // Over a full horizon it can move it only through the power feedback, which is
        // an indirect and much weaker path -- so the near-invariance must survive, but
        // as near rather than exact.
        let long_run = |effect: f64| -> f64 {
            let mut layer = ai::AiParams::wielded_instrument(&blocs);
            layer.lead_cooperation_effect = effect;
            let config = Config {
                blocs: layer.apply(&blocs),
                horizon: 50,
                runs: 20,
                ..Config::default()
            };
            Ensemble::run(&config).summarize(|o| o.mean_cooperation).0
        };
        let over_time = (long_run(-0.50) - long_run(2.00)).abs();
        assert!(
            over_time < 0.02,
            "even over 50 years the indirect path must stay weak, got a difference of \
             {over_time}"
        );
    }

    /// `--ai` is only meaningful if its hypotheses produce different *outcomes*, not
    /// merely different bloc lists.
    ///
    /// If a change to the model made the worlds behave identically, the mode would
    /// quietly become a comparison of nothing while still printing a plausible report
    /// -- the dangerous kind of failure, and the same one `--compare` guards against.
    #[test]
    fn the_ai_hypotheses_produce_different_outcomes() {
        let base = Args {
            horizon: 30,
            runs: 30,
            ..Args::default()
        };
        let measure = |role: AiRole| -> (f64, f64) {
            let layer = base.ai_params(role);
            let mut args = base.clone();
            args.blocs = layer.apply(&base.blocs);
            let config = args.to_config();
            let ensemble = Ensemble::run(&config);
            let shares = ensemble
                .mean_shares_by_year
                .last()
                .cloned()
                .unwrap_or_default();
            (
                ensemble.summarize(|o| o.mean_cooperation).0,
                shares.iter().cloned().fold(0.0_f64, f64::max),
            )
        };

        let (coop_control, top_control) = measure(AiRole::Absent);
        let (_, top_tool) = measure(AiRole::WieldedInstrument);
        let (coop_player, top_player) = measure(AiRole::SixthPower);

        // The tool world's claimed effect, measured rather than assumed: uneven
        // ownership must actually concentrate the system.
        assert!(
            top_tool > top_control + 0.02,
            "owning AI unevenly must concentrate power: {top_tool:.3} vs {top_control:.3}"
        );
        // And the player world must differ from the control, or it is not a world.
        assert!(
            (coop_player - coop_control).abs() > 1e-6,
            "the player world must not behave exactly like the control: \
             {coop_player:.3} vs {coop_control:.3}"
        );
        // The two hypotheses must lead to materially different hierarchies, which is
        // the comparison the mode exists to make. If they converged, the report's
        // headline claim -- that the difference between them is structural -- would be
        // false.
        assert!(
            (top_player - top_tool).abs() > 0.02,
            "the two hypotheses must not end with the same hierarchy: \
             {top_player:.3} as a player vs {top_tool:.3} as a tool"
        );
    }
}
