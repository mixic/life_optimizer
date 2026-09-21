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
    /// Annual probability of an economic crisis.
    crisis_probability: f64,
    /// Annual probability of a regional war.
    war_probability: f64,
    /// Bloc name that regional wars are centred on, if any.
    war_target: Option<String>,
    /// Annual probability of a technological breakthrough.
    breakthrough_probability: f64,
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
    scenarios: bool,
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
            crisis_probability: crate::simulation::ShockParams::default().crisis_probability,
            war_probability: crate::simulation::ShockParams::default().war_probability,
            breakthrough_probability: crate::simulation::ShockParams::default()
                .breakthrough_probability,
            war_target: None,
            ai_share: ai::AI_STARTING_SHARE,
            ai_growth: ai::AI_GROWTH_BIAS,
            ai_lead_effect: ai::DEFAULT_LEAD_GROWTH_EFFECT,
            ai_cooperation: ai::DEFAULT_LEAD_COOPERATION_EFFECT,
            sweep: false,
            compare: false,
            ai: false,
            scenarios: false,
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
            "--crisis" => &mut self.crisis_probability,
            "--war" => &mut self.war_probability,
            "--breakthrough" => &mut self.breakthrough_probability,
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

    /// Apply a `--bloc Name:share:bias:volatility:affinity[:valuation]` definition.
    ///
    /// Replaces the bloc of that name if the system has one, otherwise adds it, so
    /// the default five-pole system can be edited field by field as well as
    /// replaced. Names match case-insensitively so the report's capitalisation does
    /// not have to be reproduced exactly.
    ///
    /// The sixth field is optional, and deliberately so: five fields leaves the bloc's
    /// cooperation valuation at the neutral `1.0`, so every command line written
    /// before the 2x2 became a bimatrix keeps working and keeps its meaning.
    ///
    /// A malformed definition is reported and refused rather than half-applied: a
    /// bloc silently missing one field would look like a modelling result.
    fn set_bloc(&mut self, spec: &str) -> bool {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() != 5 && parts.len() != 6 {
            eprintln!(
                "warning: --bloc expects Name:share:bias:volatility:affinity[:valuation], \
                 got {spec:?}"
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

        let mut bloc = PowerBloc::new(name, numbers[0], numbers[1], numbers[2], numbers[3]);
        if let Some(valuation) = numbers.get(4) {
            bloc = bloc.with_cooperation_valuation(*valuation);
        }
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
        config.shocks.crisis_probability = self.crisis_probability;
        config.shocks.war_probability = self.war_probability;
        config.shocks.breakthrough_probability = self.breakthrough_probability;
        // Resolved to an index here rather than carried as a name, so the simulation
        // never has to search and cannot silently stop matching. An unknown name is
        // reported rather than ignored: a mistyped target would otherwise produce a
        // perfectly ordinary-looking random-war world.
        config.war_target = self.war_target.as_ref().and_then(|name| {
            match config
                .blocs
                .iter()
                .position(|bloc| bloc.name.eq_ignore_ascii_case(name))
            {
                Some(index) => Some(index),
                None => {
                    eprintln!("warning: --war-target names no bloc ({name:?}); wars stay random");
                    None
                }
            }
        });
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
            "--scenarios" => {
                args.scenarios = true;
                i += 1;
            }
            "--war-target" => {
                // Like `--bloc`, the one other flag whose value is not a number.
                match raw.get(i + 1) {
                    Some(name) => {
                        args.war_target = Some(name.clone());
                        i += 2;
                    }
                    None => {
                        eprintln!("warning: --war-target needs a bloc name");
                        i += 1;
                    }
                }
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
  multipolar_sim [--sweep | --compare | --ai | --scenarios] [options]

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
  --crisis <p>               yearly chance of an economic      (default {crisis:.2})
                             crisis
  --war <p>                  yearly chance of a regional war   (default {war:.2})
  --war-target <bloc>        pin regional wars to this bloc; the
                             second belligerent is still drawn
  --breakthrough <p>         yearly chance of a technological  (default {breakthrough:.2})
                             breakthrough
  --scenarios                run a set of named worlds side by side:
                             a Sinic economic rise, a global crisis,
                             and war centred on a named bloc
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

  `--bloc` takes Name:share:bias:volatility:affinity with an optional sixth
  field, the bloc's cooperation valuation. Matching the bloc name is
  case-insensitive, and it edits the bloc of that name if the system has
  one, otherwise it appends a new one, so the default five-pole system can
  be reshaped one field at a time or replaced outright. The valuation is
  what enters that bloc's own payoff matrix, so it is the field that makes
  the two sides of a dyad differ; omitting it leaves the bloc neutral at
  1.0, which is what every bloc was before the game became a bimatrix.

  The five default blocs are Atlantic 0.30/0.000/0.020/1.00, Sinic
  0.26/0.008/0.025/0.95, Eurasian 0.16/0.002/0.035/0.70, Indo-Pacific
  0.14/0.010/0.030/0.90 and Non-Aligned 0.14/0.004/0.040/1.05, as
  share/growth-bias/volatility/cooperation-affinity. All five are neutral
  on valuation.

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
        crisis = d.crisis_probability,
        war = d.war_probability,
        breakthrough = d.breakthrough_probability,
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
    /// Mean Pareto-efficiency loss across dyad-years.
    loss: f64,
    /// Mean accumulated tension at the horizon.
    tension: f64,
    top_share: f64,
    /// Which bloc holds the top share, so the reader can see a change of leader
    /// rather than having to infer it from a non-monotone column.
    leading: String,
    pension: f64,
    /// Fraction of dyad-years solved to an asymmetric equilibrium: the two sides
    /// playing different pure actions. Zero unless the two matrices differ.
    asymmetric: f64,
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
        loss: ensemble.summarize(|o| o.mean_efficiency_loss).0,
        tension: ensemble.summarize(|o| o.final_tension).0,
        top_share: shares.iter().cloned().fold(0.0_f64, f64::max),
        leading,
        pension: observed.security_index(&reference),
        asymmetric: ensemble.summarize(|o| o.asymmetric_fraction).0,
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
    println!("  3. AI as a TOOL: does owning AI make cooperation worth more, or less?");
    println!("  {}", "-".repeat(74));
    println!("  MULTIPOLAR_GAME.md section 4 presents this as an open dispute rather than");
    println!("  a settled question -- the realist case has AI as one more axis of zero-sum");
    println!("  rivalry, the optimistic case has AI competition bound up with supply chains");
    println!("  and data flows, and therefore not zero-sum at all. Those imply opposite");
    println!("  signs, so the coefficient defaults to zero and the sweep runs both ways.");
    println!();
    println!(
        "  {:<12} {:>8}  {:>9}  {:>10}  {:>9}  {:>8}",
        "coefficient", "coop", "trap yrs", "top share", "pension", "asymm."
    );
    // Fine enough to distinguish a *slope* from a *step*. A coarse grid would show
    // that the row moves without showing how, and "how" is the whole question here:
    // if the branch opens discontinuously then the magnitude of the effect is not a
    // finding, only its direction is.
    for value in [
        -0.30, -0.20, -0.15, -0.10, -0.05, -0.01, 0.00, 0.01, 0.05, 0.10, 0.15, 0.20, 0.30,
    ] {
        let mut args = base.clone();
        args.ai_cooperation = value;
        args.blocs = args.ai_params(AiRole::WieldedInstrument).apply(&base.blocs);
        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        let point = hinge_point(&config, &ensemble, None);
        println!(
            "  {:<12.2} {:>8.3}  {:>8.1}%  {:>9.1}%  {:>9.3}  {:>7.1}%",
            value,
            point.cooperation,
            point.trap * 100.0,
            point.top_share * 100.0,
            point.pension,
            point.asymmetric * 100.0
        );
    }
    println!("  `asymm.` is the share of dyad-years whose equilibrium had the two sides");
    println!("  playing *different* pure actions -- one cooperating while the other");
    println!("  competes. It is the only direct evidence that the asymmetric solver is");
    println!("  doing any work at all, and it is non-zero only here: a capability that");
    println!("  never fires is indistinguishable from one that does not exist.");
    println!();
    println!("  Reading the row. A coefficient row that moved nothing would have meant the");
    println!("  claim was not expressible, which is what the first version of this feature");
    println!("  reported: it raised `cooperation_affinity`, a term applied *downstream* of");
    println!("  the solver to the power a bloc banks, so it could not change a decision and");
    println!("  the column was flat at every value. The coefficient now raises the owning");
    println!("  bloc's cooperation *valuation*, which enters the payoff matrix, so it moves");
    println!("  what the blocs actually choose. Getting there required making the 2x2 a");
    println!("  bimatrix -- one payoff matrix per side -- because a symmetric game has");
    println!("  nowhere to put \"cooperation is worth more to this bloc\".");
    println!();
    println!("  What to take from the numbers, and the grid is deliberately fine enough");
    println!("  to show this: the cooperation rate is MONOTONE on each side of zero and");
    println!("  DISCONTINUOUS at it. Moving from -0.30 to -0.01 cooperation rises 0.314 ->");
    println!("  0.346; from 0.01 to 0.30 it rises 0.408 -> 0.430; and at exactly 0.00 it is");
    println!("  0.217, below both. So:");
    println!();
    println!("    ROBUST: the SIGN. A coefficient that makes AI ownership raise the owner's");
    println!("    valuation of cooperation produces a more cooperative world than one that");
    println!("    makes it a zero-sum rivalry axis -- the optimistic case against the");
    println!("    realist one, over the whole range, in the direction the chapter argues.");
    println!();
    println!("    NOT A FINDING: the level, and any comparison against the 0.00 row. That");
    println!("    row is a different branch, not a point on this curve -- note that even the");
    println!("    *realist* setting at -0.30 is more cooperative than symmetry at 0.00,");
    println!("    which cannot be a claim about AI. The magnitude of the effect is therefore");
    println!("    an artefact of branch selection, and only the sign survives it.");
    println!();
    println!("  ONE THING TO READ CAREFULLY: the 0.00 row is a knife-edge. At exactly zero");
    println!("  every bloc holds the same valuation, so the two payoff matrices are");
    println!("  identical, and the exchangeability rule then closes the asymmetric branch");
    println!("  completely -- a symmetric game cannot report that one of two identical");
    println!("  players is the cooperator. Any nonzero spread, in either direction, opens");
    println!("  that branch, which is why the asymmetric share jumps from 0% to 42-84%");
    println!("  beside it.");
    println!();
    println!("  That discontinuity is itself a limitation worth naming: behaviour that is");
    println!("  qualitatively different at perfect symmetry than at near-symmetry means a");
    println!("  model like this cannot be read as continuous in the spread of valuations.");

    println!();
    println!("  {}", "-".repeat(74));
    println!("  Every number in this sweep comes from invented parameters. Its value is not");
    println!("  that any row is right, but that the rows show which claims are load-bearing");
    println!("  and which are free.");
    println!("{}", "=".repeat(78));
}

/// One named world in the `--scenarios` comparison.
struct Scenario {
    label: &'static str,
    /// Plain-language statement of what this world assumes and why.
    premise: &'static str,
    /// Rewrite the blocs, if this scenario changes anyone's position.
    bloc: Option<(&'static str, f64, f64, f64, f64, f64)>,
    crisis: Option<f64>,
    war: Option<f64>,
    war_target: Option<&'static str>,
}

/// The scenarios, as data, so the report and its tests read the same list.
///
/// Every one of these is a *parameterisation*, not a forecast, and the report says
/// so before printing any of it. The values are chosen to be plausible in sign and
/// large enough to see, which is a different thing from being calibrated.
fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            label: "BASELINE",
            premise: "no scenario: the world as configured, with wars hitting random pairs",
            bloc: None,
            crisis: None,
            war: None,
            war_target: None,
        },
        Scenario {
            label: "SINIC GROWS FASTER",
            premise: "Sinic compounds 50% faster than it does now, and nothing else changes",
            // Growth bias 0.008 -> 0.012, keeping Sinic the fastest-growing bloc. This
            // is the pure compounding route: no military term, no change to what Sinic
            // wants, only how fast its position accumulates.
            bloc: Some(("Sinic", 0.26, 0.012, 0.025, 0.95, 1.00)),
            crisis: None,
            war: None,
            war_target: None,
        },
        Scenario {
            label: "SINIC VALUES COOPERATION",
            premise: "same growth as now, but Sinic weights cooperation 25% more heavily",
            // The integration route, isolated. Kept separate from the growth row
            // deliberately: the first version of this mode bundled the two and the
            // result was a slight *decline*, which describes neither mechanism. Split
            // apart they turn out to pull in opposite directions, and that contrast is
            // the most useful thing this mode produces.
            bloc: Some(("Sinic", 0.26, 0.008, 0.025, 1.25, 1.25)),
            crisis: None,
            war: None,
            war_target: None,
        },
        Scenario {
            label: "GLOBAL CRISIS",
            premise: "recurrent economic crises: 45% a year instead of 18%",
            bloc: None,
            crisis: Some(0.45),
            war: None,
            war_target: None,
        },
        Scenario {
            label: "WAR IN THE SINIC BLOC",
            premise: "a regional war every seventh year or so, centred on Sinic",
            bloc: None,
            crisis: None,
            war: Some(0.15),
            war_target: Some("Sinic"),
        },
        Scenario {
            label: "WAR AT THE PERIPHERY",
            premise: "the same war rate, but centred on Non-Aligned -- the proxy for a \
                      conflict outside any pole",
            bloc: None,
            crisis: None,
            war: Some(0.15),
            war_target: Some("Non-Aligned"),
        },
        Scenario {
            label: "SINIC GROWS INTO WAR",
            premise: "Sinic compounds faster while a war runs on its own territory",
            bloc: Some(("Sinic", 0.26, 0.012, 0.025, 0.95, 1.00)),
            crisis: None,
            war: Some(0.15),
            war_target: Some("Sinic"),
        },
    ]
}

/// Run several named worlds from the same shock draws and compare them.
///
/// # What this mode is for, and what it is not
///
/// It answers "what changes if the world takes this shape" -- not "which shape will
/// it take". The parameters are invented, so the *levels* are not findings; what is
/// comparable is how each scenario moves the same statistics relative to the others,
/// because every scenario runs the same years from the same seed.
///
/// The one methodological care taken here: a pinned war target still consumes the
/// random belligerent draw, so a targeted scenario and the baseline share a random
/// stream. Without that, the worlds would diverge for a reason unrelated to the
/// scenario and the comparison would be measuring luck.
fn run_scenarios(base: &Args) {
    let scenarios = scenarios();
    let mut rows: Vec<(&Scenario, HingePoint, Vec<String>)> = Vec::new();

    for scenario in &scenarios {
        let mut args = base.clone();
        if let Some((name, share, bias, volatility, affinity, valuation)) = scenario.bloc {
            let mut bloc = PowerBloc::new(name, share, bias, volatility, affinity);
            bloc = bloc.with_cooperation_valuation(valuation);
            match args
                .blocs
                .iter_mut()
                .find(|existing| existing.name.eq_ignore_ascii_case(name))
            {
                Some(existing) => *existing = bloc,
                None => args.blocs.push(bloc),
            }
        }
        if let Some(probability) = scenario.crisis {
            args.crisis_probability = probability;
        }
        if let Some(probability) = scenario.war {
            args.war_probability = probability;
        }
        if let Some(name) = scenario.war_target {
            args.war_target = Some(name.to_string());
        }

        let config = args.to_config();
        let ensemble = Ensemble::run(&config);
        let point = hinge_point(&config, &ensemble, None);
        let shares = ensemble
            .mean_shares_by_year
            .last()
            .cloned()
            .unwrap_or_default();
        let per_bloc: Vec<String> = config
            .blocs
            .iter()
            .enumerate()
            .map(|(index, bloc)| {
                format!(
                    "{} {:.1}%",
                    short_name(&bloc.name),
                    shares.get(index).copied().unwrap_or(0.0) * 100.0
                )
            })
            .collect();
        rows.push((scenario, point, per_bloc));
    }

    println!();
    println!("{}", "=".repeat(78));
    println!("SCENARIOS OF MULTIPOLARITY");
    println!("{}", "=".repeat(78));
    println!(
        "  {} worlds, the same {} years and the same shock draws, {} runs each.",
        scenarios.len(),
        base.horizon,
        base.runs
    );
    println!("  Only the named assumption differs between them.");
    println!();
    println!("  NOTE ON WHAT THIS IS");
    println!("  --------------------");
    println!("  These are PARAMETERISATIONS, not forecasts. A scenario is a coherent");
    println!("  'what if the world looked like this', and the numbers below are what this");
    println!("  model does with it -- not what the world would do. Read the differences");
    println!("  between rows, not the levels in them.");

    println!();
    println!("  THE WORLDS");
    println!("  {}", "-".repeat(74));
    for scenario in &scenarios {
        println!("  {:<24} {}", scenario.label, scenario.premise);
    }

    println!();
    println!("  HOW THE WORLD FARES");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:<24} {:>7} {:>7} {:>8} {:>8} {:>9} {:>10} {:>6}",
        "", "coop", "tension", "trap", "Pareto", "pension", "leader", "top"
    );
    for (scenario, point, _) in &rows {
        println!(
            "  {:<24} {:>7.3} {:>7.2} {:>7.1}% {:>8.3} {:>9.3} {:>10} {:>5.1}%",
            scenario.label,
            point.cooperation,
            point.tension,
            point.trap * 100.0,
            point.loss,
            point.pension,
            short_name(&point.leading),
            point.top_share * 100.0
        );
    }
    println!();
    println!("  `tension` is the mean accumulated tension at the horizon, and it is in the");
    println!("  table because it is the variable that explains most of the surprises: in");
    println!("  this model a *tenser* system is not always a less cooperative one, because");
    println!("  sustained tension is what erodes the payoff of mutual competition and so");
    println!("  opens the door out of the pure dilemma. Read it beside `coop`, not instead");

    println!();
    println!("  WHERE POWER ENDS UP");
    println!("  {}", "-".repeat(74));
    for (scenario, _, per_bloc) in &rows {
        println!("  {:<24} {}", scenario.label, per_bloc.join("  "));
    }

    print_scenario_reading(base, &rows);
    println!("{}", "=".repeat(78));
}

/// Abbreviate a bloc name so the scenario table stays on one line.
fn short_name(name: &str) -> &str {
    match name {
        "Indo-Pacific" => "Indo-Pac",
        "Non-Aligned" => "Non-Align",
        other => other,
    }
}

/// The closing reading of the scenario table.
///
/// Deliberately comparative rather than absolute: the only claims made here are about
/// how the rows differ from each other, because that is the only thing the shared seed
/// and the shared parameterisation make checkable.
fn print_scenario_reading(base: &Args, rows: &[(&Scenario, HingePoint, Vec<String>)]) {
    println!();
    println!("  HOW TO READ THIS");
    println!("  {}", "-".repeat(74));

    let find = |label: &str| rows.iter().find(|(s, _, _)| s.label == label);
    let (Some(baseline), Some(grows), Some(values)) = (
        find("BASELINE"),
        find("SINIC GROWS FASTER"),
        find("SINIC VALUES COOPERATION"),
    ) else {
        return;
    };

    println!("  1. Can Sinic win through the economy? Yes -- but only one of the two");
    println!("     economic routes works, and the other backfires.");
    println!();
    println!(
        "     {:<24} {:>7} {:>7} {:>9}",
        "world", "Sinic", "coop", "pension"
    );
    for (scenario, point, per_bloc) in rows.iter().take(3) {
        let sinic = per_bloc
            .iter()
            .find(|entry| entry.starts_with("Sinic"))
            .and_then(|entry| entry.split_whitespace().last())
            .unwrap_or("-");
        println!(
            "     {:<24} {:>7} {:>7.3} {:>9.3}",
            scenario.label, sinic, point.cooperation, point.pension
        );
    }
    println!();
    println!(
        "     **Growing faster is decisive: {:.1}% to {:.1}%, a near-monopoly.** A 50%",
        baseline.1.top_share * 100.0,
        grows.1.top_share * 100.0
    );
    println!("     increase in the growth bias compounds over 50 years into a share no other");
    println!("     bloc can contest. Note what the nominal figure hides: a bias is applied");
    println!(
        "     once per dyad as well as once a year, so 0.012 compounds to about {:.1}% a",
        crate::ai::effective_annual_advantage(0.012, 5) * 100.0
    );
    println!("     year against a field near 3%. A row with 0.020 in it reaches 97.7% -- the");
    println!("     model has no countervailing force once a growth lead is established.");
    println!();
    println!("     **And notice what it does not do: it leaves the world exactly as");
    println!("     cooperative as it was.** Cooperation 0.215 -> 0.216, trap years 94.7% ->");
    println!("     94.5%, pension 0.505 -> 0.506. A bloc taking three quarters of world power");
    println!("     changes *who holds power* not at all how the system behaves, because it is");
    println!("     still playing the same uncooperative game against everyone. That is worth");
    println!("     stating because the intuitive expectation runs the other way -- that a");
    println!("     dominant power would impose order. Here it simply outgrows the disorder.");
    println!();
    println!("     **Making cooperation your strategy does the opposite.** Weighting");
    println!(
        "     cooperation 25% more heavily takes Sinic from {:.1}% to {:.1}% and raises",
        baseline.1.top_share * 100.0,
        values.1.top_share * 100.0
    );
    println!(
        "     world cooperation from {:.3} to {:.3} -- and it *loses* relative position",
        baseline.1.cooperation, values.1.cooperation
    );
    println!("     doing it. That is the sucker's payoff at work: in a world that is still");
    println!("     mostly uncooperative, a bloc that cooperates more often is exploited more");
    println!("     often, and the rivals that keep competing collect the temptation. It is");
    println!("     the model's version of a real argument -- that unilateral restraint is a");
    println!("     transfer to whoever does not practise it -- and it is worth noting that");
    println!("     the pension index still *rises*, so the world is better off even though");
    println!("     Sinic is not. The gap between 'the world gains' and 'the restrainer gains'");
    println!("     is the finding.");
    println!();
    println!("     So the honest answer to 'does Sinic win through economy' is: it depends");
    println!("     entirely on what 'through economy' means. Compounding capability wins;");
    println!("     becoming the one who cooperates loses. Bundling the two -- which the first");
    println!("     version of this mode did -- produces a mild *decline* that describes");
    println!("     neither mechanism, which is why they are separate rows.");

    println!();
    println!("  2. What does a crisis do?");
    if let Some(crisis) = find("GLOBAL CRISIS") {
        println!(
            "     cooperation {:.3} -> {:.3}   tension {:.2} -> {:.2}   Pareto {:.3} -> {:.3}",
            baseline.1.cooperation,
            crisis.1.cooperation,
            baseline.1.tension,
            crisis.1.tension,
            baseline.1.loss,
            crisis.1.loss
        );
        println!("     This is the counterintuitive row, and it should not be read as 'crises");
        println!("     are good'. Two things are happening, and the second is a limitation:");
        println!("     - The crisis shock acts on the *game*, not on any economy: it adds");
        println!("       tension and takes a little power from one random bloc. More shocks");
        println!("       means more accumulated tension.");
        println!("     - More tension is what erodes the payoff of mutual competition, via");
        println!("       `conflict_wear`. So a tenser system leaves the pure dilemma sooner");
        println!("       and cooperation *rises* -- which is a property of the tension channel");
        println!("       this model happens to have, not a claim about real recessions. The");
        println!("       sign of that effect is exactly the kind of thing the --sweep mode");
        println!("       exists to expose, and it is not robust to the conflict-wear value.");
        println!("     What the model genuinely cannot say: there is no output, credit,");
        println!("     unemployment or trade-volume channel, so it cannot show a recession's");
        println!("     depth at all -- only how it changes what the game rewards.");
    }

    println!();
    println!("  3. Does it matter *where* the war is?");
    if let (Some(home), Some(periphery)) =
        (find("WAR IN THE SINIC BLOC"), find("WAR AT THE PERIPHERY"))
    {
        println!(
            "     war in Sinic:     leader {:<12} top {:.1}%   coop {:.3}",
            short_name(&home.1.leading),
            home.1.top_share * 100.0,
            home.1.cooperation
        );
        println!(
            "     war at periphery: leader {:<12} top {:.1}%   coop {:.3}",
            short_name(&periphery.1.leading),
            periphery.1.top_share * 100.0,
            periphery.1.cooperation
        );
        println!("     Both run the same war rate from the same shock draws, so any difference");
        println!("     between these two rows is the location and nothing else. That is the");
        println!("     cleanest comparison this mode makes, and it is the direct answer to");
        println!("     'does it matter where the war is'.");
    }

    println!("  4. Does a war stop an economic rise?");
    if let Some(war_rise) = find("SINIC GROWS INTO WAR") {
        println!(
            "     Sinic growing:            {:.1}%   coop {:.3}   pension {:.3}",
            grows.1.top_share * 100.0,
            grows.1.cooperation,
            grows.1.pension
        );
        println!(
            "     Sinic growing under war:  {:.1}%   coop {:.3}   pension {:.3}",
            war_rise.1.top_share * 100.0,
            war_rise.1.cooperation,
            war_rise.1.pension
        );
        println!("     A war every seventh year, centred on Sinic's own territory, against a");
        println!("     compounding growth lead. The war term is a 5% power loss and a tension");
        println!("     spike, so it is a *deterrent* channel and not a destruction channel: it");
        println!("     cannot destroy accumulated capability, only redistribute a little and");
        println!("     raise tension. Read the row as 'does conflict reverse a compounding");
        println!("     advantage', not as 'what would a war cost'.");
    }

    println!();
    println!("  5. What this model cannot say about your question");
    println!("     Four limits, stated rather than left to be discovered:");
    println!("     - **There is no Africa.** The five blocs are Atlantic, Sinic, Eurasian,");
    println!("       Indo-Pacific and Non-Aligned. Africa, Latin America and most of South");
    println!("       Asia sit inside 'Non-Aligned' or nowhere, so a war 'in Africa' can only");
    println!("       be modelled as a war on the Non-Aligned aggregate. Its size and its");
    println!("       numbers are invented, so the row is a shape, not a place.");
    println!("     - **A crisis is not an economic crisis.** The shock raises tension and");
    println!("       removes a little power. There is no output, credit, unemployment or");
    println!("       trade-volume channel, so the model cannot show a recession's depth,");
    println!("       only its effect on how cooperative the game becomes.");
    println!("     - **A war cannot destroy capability.** Both war and crisis subtract a");
    println!("       small percentage of power for one year. There is no path by which");
    println!("       conflict destroys accumulated stock, so the model cannot represent a");
    println!("       war that sets a rising power back by a decade.");
    println!("     - **Sinic's rise has no mechanism behind it.** Raising its growth and");
    println!("       cooperation terms is a *statement of the outcome being tested*, not an");
    println!("       account of how it happens -- no industrial policy, no currency");
    println!("       internationalisation, no reserve-share drift. Sinic's reserve share is");
    println!("       pinned at the renminbi's 1.95% because that is the reported figure, so");
    println!("       the monetary route to a rise is not available in this model at all.");
    println!();
    println!(
        "  With {} runs the noise on the cooperation column is roughly {:.3}, so treat",
        base.runs,
        1.0 / (base.runs as f64).sqrt() * 0.5
    );
    println!("  differences smaller than that as unresolved rather than as findings.");
}

fn main() {
    let args = parse_args();

    if args.scenarios {
        run_scenarios(&args);
        return;
    }

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

        // The valuation field is optional. Omitting it must leave the bloc neutral --
        // otherwise every command line written before the game became a bimatrix would
        // silently acquire a meaning it never had.
        assert!(args.set_bloc("Antarctic:0.05:0.000:0.010:1.00"));
        let antarctic = |args: &Args| {
            args.blocs
                .iter()
                .find(|bloc| bloc.name == "Antarctic")
                .expect("just added")
                .cooperation_valuation
        };
        assert_eq!(
            antarctic(&args),
            1.0,
            "a five-field spec must leave the valuation neutral"
        );

        // Supplying it must take effect, and must not disturb the other fields.
        assert!(args.set_bloc("Antarctic:0.05:0.000:0.010:1.00:1.75"));
        assert_eq!(
            args.blocs.len(),
            before + 1,
            "still an edit, not an addition"
        );
        assert_eq!(antarctic(&args), 1.75);

        // A seventh field is still refused rather than partly applied.
        assert!(!args.set_bloc("Antarctic:0.05:0.000:0.010:1.00:1.75:9.0"));
        assert_eq!(antarctic(&args), 1.75, "a refused spec must change nothing");

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

    /// `cooperation_affinity` must not be able to move the cooperation rate, while
    /// `cooperation_valuation` must.
    ///
    /// These two knobs are the whole reason the payoff structure splits them, and the
    /// pair is what `--ai`'s third sweep row rests on. `cooperation_affinity` scales
    /// the payoff a bloc *banks* when it cooperates -- downstream of the solver -- so
    /// it can redistribute power after the fact but cannot change whether a dyad
    /// cooperates. `cooperation_valuation` enters the payoff matrix, so it can.
    ///
    /// This test used to assert only the first half, and it passed while the "does AI
    /// make cooperation worth more" question was inexpressible in the model. The
    /// second half is what makes the sweep row a real test rather than a null one, so
    /// if it ever stops holding, that row has silently become meaningless again.
    #[test]
    fn affinity_is_downstream_of_the_solver_and_valuation_is_inside_it() {
        let blocs = blocks::default_blocs();

        // --- affinity: no route into the solver ---------------------------------
        let by_affinity = |scale: f64| -> f64 {
            let mut edited = blocs.clone();
            for bloc in edited.iter_mut() {
                bloc.cooperation_affinity *= scale;
            }
            let config = Config {
                blocs: edited,
                // One year, so the shares entering the solver are identical between
                // the two runs and any difference would have to come from the
                // parameter itself.
                horizon: 1,
                runs: 20,
                ..Config::default()
            };
            Ensemble::run(&config).summarize(|o| o.mean_cooperation).0
        };
        let suppressed = by_affinity(0.25);
        let amplified = by_affinity(4.00);
        assert!(
            (suppressed - amplified).abs() < 1e-12,
            "affinity has no route into the payoff matrix, so it cannot move the \
             cooperation rate: {suppressed} vs {amplified}"
        );

        // --- valuation: inside the solver ---------------------------------------
        //
        // Raising every bloc's valuation far enough must raise how often they
        // cooperate, or the asymmetry this model was widened for is still decorative.
        let by_valuation = |valuation: f64| -> f64 {
            let mut edited = blocs.clone();
            for bloc in edited.iter_mut() {
                bloc.cooperation_valuation = valuation;
            }
            let config = Config {
                blocs: edited,
                horizon: 1,
                runs: 20,
                ..Config::default()
            };
            Ensemble::run(&config).summarize(|o| o.mean_cooperation).0
        };
        let neutral = by_valuation(1.0);
        let keen = by_valuation(3.0);
        assert!(
            keen > neutral,
            "a bloc that values cooperation more must cooperate more often: \
             {keen} at valuation 3.0 vs {neutral} at 1.0"
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
