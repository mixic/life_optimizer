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

//! The Monte Carlo simulation.
//!
//! One *run* simulates the system forward for `horizon` years. Each year every
//! unordered pair of blocs plays the 2x2 game from [`crate::game`], with payoffs
//! derived from their current power gap and the accumulated global tension. The
//! realized outcome is sampled from the solved equilibrium, so a mixed equilibrium
//! genuinely randomises rather than being rounded to one action.
//!
//! Many runs produce a distribution, because `MULTIPOLAR_GAME.md` section 7 is
//! explicit that "genuine uncertainty is the most defensible position": there is
//! no single answer to where a multipolar system settles, and a point estimate
//! would misrepresent that.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};

use crate::blocks::{GameParams, PowerBloc};
use crate::economy::Economy;
use crate::game::{solve_pair, Action, Equilibrium, Payoffs};
use crate::information::{attack_payoff, InformationParams};
use crate::pension::{PensionLink, PensionOutcome};

/// The number of dyads `GameParams::tension_per_conflict` is calibrated against.
///
/// A fully competitive ten-pair system adds `10 * tension_per_conflict` per year, and
/// a system partitioned more finely adds the same total rather than more. Ten pairs is
/// the five-bloc default, so this is a unit convention and not a claim: it fixes the
/// scale of a system-level index so that the index does not move when the world is
/// described in more detail.
///
/// It is the same class of defect as the per-dyad growth bias corrected in
/// `blocks.rs`: a quantity that describes the *system* applied once per *pair*. Both
/// were invisible until the bloc list changed size, and both would have been read as
/// findings about whatever new region appeared.
const REFERENCE_DYADS: f64 = 10.0;

/// The factor that turns a per-pair tension feed into a system-level one.
///
/// See [`REFERENCE_DYADS`]. The invariant this must satisfy -- and the one its test
/// pins -- is that `tension_feed_scale(n) * pairs(n)` is the same for every `n`: a
/// fully competitive system adds the same tension however many blocs it is described
/// as having. At the ten-pair reference size the factor is exactly `1.0`, which is
/// what keeps the five-bloc figures reproducible.
fn tension_feed_scale(blocs: usize) -> f64 {
    let pairs = blocs.saturating_mul(blocs.saturating_sub(1)) / 2;
    if pairs > 0 {
        REFERENCE_DYADS / pairs as f64
    } else {
        1.0
    }
}

/// Kind of annual shock. The categories follow `MULTIPOLAR_GAME.md` section 4's
/// discussion of what makes multipolar transitions dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockKind {
    /// A crisis: tension jumps, and the involved blocs lose a little power.
    Crisis,
    /// A technological breakthrough: a power gain for whoever achieves it. The
    /// chapter's section 5 notes AI is now a primary axis of competition.
    Breakthrough,
    /// A regional war: the most damaging, and the most tension-generating.
    RegionalWar,
}

/// Configuration for the shock process.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockParams {
    pub crisis_probability: f64,
    pub breakthrough_probability: f64,
    pub war_probability: f64,
    /// Tension added by each kind.
    pub crisis_tension: f64,
    pub war_tension: f64,
    /// Power lost by an involved bloc, as a fraction.
    pub crisis_power_loss: f64,
    pub war_power_loss: f64,
    /// Power gained by the beneficiary of a breakthrough.
    pub breakthrough_gain: f64,
    /// Probability that the energy network is disrupted in a given year, and how
    /// much of its power a fully exposed bloc loses when it is.
    ///
    /// These two are **illustrative**. The mechanism is the reported one -- an
    /// importer loses supply, an exporter loses revenue -- and *which* blocs pay is
    /// taken from the sourced exposure figures in `economy.rs`. What is invented is
    /// the scale that connects a modelled disruption to a change in relative power,
    /// because no such relationship is measurable.
    pub energy_disruption_probability: f64,
    pub energy_disruption_damage: f64,
}

impl Default for ShockParams {
    fn default() -> Self {
        ShockParams {
            crisis_probability: 0.18,
            breakthrough_probability: 0.10,
            war_probability: 0.03,
            crisis_tension: 0.15,
            war_tension: 0.50,
            crisis_power_loss: 0.01,
            war_power_loss: 0.05,
            breakthrough_gain: 0.05,
            energy_disruption_probability: 0.10,
            energy_disruption_damage: 0.04,
        }
    }
}

/// Whole-simulation configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub blocs: Vec<PowerBloc>,
    pub game: GameParams,
    pub shocks: ShockParams,
    pub pension: PensionLink,
    /// Monetary standing, energy trade and financial conditions.
    ///
    /// Defaults are built for `blocs`; a caller that replaces the bloc set should
    /// rebuild this with [`Economy::for_blocs`] rather than reuse a stale one,
    /// because every vector in it is indexed by bloc position.
    pub economy: Economy,
    pub horizon: u32,
    pub runs: usize,
    /// Base seed for the ensemble. Run `n` uses `seed + n * 2_654_435_761`, so the
    /// whole ensemble is reproducible and two parameter settings can be compared
    /// against the *same* shock draws rather than against different luck. That
    /// matters here more than usual: with illustrative parameters, most of the
    /// difference between two configurations is noise unless the seeds are held
    /// fixed.
    pub seed: u64,
    /// Tension above which a year counts as a "conflict trap".
    pub trap_tension_threshold: f64,
    /// Cooperation below which a year counts as a "conflict trap".
    pub trap_cooperation_threshold: f64,
    /// Bloc that regional wars are pinned to, if any.
    ///
    /// `None` means a war damages a random pair, which is the default and the honest
    /// one for a baseline. Setting it answers a question the random version cannot:
    /// *where* a war happens is not a detail in a model with seven asymmetric blocs,
    /// because a war inside the largest bloc and a war in Africa have different
    /// consequences for who ends up on top -- and before the regional split the second
    /// of those had no referent at all.
    ///
    /// It pins only the *first* belligerent; the second is still drawn, so a targeted
    /// war is a war centred on that bloc rather than a scripted outcome.
    pub war_target: Option<usize>,
    /// The information layer of `INFORMATION_WARFARE.md`.
    ///
    /// Disabled by default, and that is load-bearing rather than tidy: the only thing the
    /// layer does is replace the exogenous war draw with an endogenous one, so with it off
    /// every published figure must be reproduced exactly. A test pins that.
    pub information: InformationParams,
}

/// The default ensemble base seed.
pub const DEFAULT_SEED: u64 = 0x5EED_0000;

impl Default for Config {
    fn default() -> Self {
        let blocs = crate::blocks::default_blocs();
        let economy = Economy::for_blocs(&blocs);
        Config {
            game: GameParams::default(),
            shocks: ShockParams::default(),
            pension: PensionLink::default(),
            economy,
            blocs,
            horizon: 50,
            runs: 400,
            seed: DEFAULT_SEED,
            trap_tension_threshold: 0.6,
            trap_cooperation_threshold: 0.5,
            war_target: None,
            information: InformationParams::default(),
        }
    }
}

/// How power is distributed in one year, once the system has settled into a shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// One bloc holds a clear majority of system power.
    Unipolar,
    /// Two blocs together dominate and are comparable to each other.
    Bipolar,
    /// Three or more comparable poles, with no dominant pair.
    Balanced,
    /// Power is spread thinly across many small blocs.
    Fragmented,
}

impl Polarity {
    pub fn label(self) -> &'static str {
        match self {
            Polarity::Unipolar => "unipolar",
            Polarity::Bipolar => "bipolar",
            Polarity::Balanced => "balanced",
            Polarity::Fragmented => "fragmented",
        }
    }
}

/// One year's state, kept for trajectory output.
#[derive(Debug, Clone, PartialEq)]
pub struct YearRecord {
    pub year: u32,
    pub shares: Vec<f64>,
    pub tension: f64,
    /// Realized cooperation in this year, across all dyads.
    pub cooperation: f64,
    /// Efficiency loss realized this year.
    pub efficiency_loss: f64,
    /// The most common equilibrium type among this year's dyads.
    pub dominant_equilibrium: &'static str,
    pub shocks: Vec<ShockKind>,
    /// Whether the energy network was disrupted this year: 0.0 or 1.0.
    pub energy_disruption: f64,
    /// Global financial-conditions index at year end, in [0, 1].
    pub recession_risk: f64,
}

/// What the information layer produced, when it was switched on.
///
/// Reported separately from [`RunOutcome`]'s own fields because the layer's whole claim
/// is that it *derives* something the baseline assumes: with the layer on, the war rate
/// is an output. Printing it beside the assumed rate is what makes that comparison
/// possible. `INFORMATION_WARFARE.md` sections 3.2-3.7.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InformationOutcome {
    /// Wars triggered by an attack payoff that crossed zero -- the endogenous onset.
    pub legitimated_wars: u32,
    /// Wars that would have occurred with no justification at all (`V >= 0` at `b = 0`).
    /// Theorem 1(iv): in these dyads the narrative is decoration, not cause.
    pub bare_wars: u32,
    /// Years in which some dyad was in Corollary 1.1's contestable band -- inside the
    /// region where disinformation can change the outcome at all.
    pub contestable_years: u32,
    /// Times a bloc's sustained assertions were exposed, across the whole run.
    pub detected_lies: u32,
    /// Mean belief in culpability at the horizon, across blocs.
    pub mean_belief: f64,
    /// Mean credibility at the horizon, across blocs. Theorem 2's stock.
    pub mean_credibility: f64,
}

/// What one complete run produced.
#[derive(Debug, Clone, PartialEq)]
pub struct RunOutcome {
    /// Index of the most powerful bloc at the horizon, or `None` if no bloc holds
    /// a clear lead.
    pub dominant: Option<usize>,
    pub final_polarity: Polarity,
    /// Mean realized cooperation across all dyad-years.
    pub mean_cooperation: f64,
    /// Mean efficiency loss across all dyad-years.
    pub mean_efficiency_loss: f64,
    /// Fraction of years classified as a conflict trap.
    pub trap_fraction: f64,
    /// Tension at the horizon.
    pub final_tension: f64,
    pub pension: PensionOutcome,
    /// Final power shares, for distribution summaries.
    pub final_shares: Vec<f64>,
    /// Share index of the leading bloc at the horizon.
    pub top_share: f64,
    /// Financial-conditions index at the horizon, in [0, 1].
    pub final_recession_risk: f64,
    /// How many shocks of each kind occurred.
    pub shock_counts: (u32, u32, u32),
    /// Fraction of dyad-years whose equilibrium had the two sides playing
    /// *different* pure actions.
    ///
    /// Zero unless the two sides faced different matrices, which requires a
    /// non-neutral [`PowerBloc::cooperation_valuation`]. It is reported because it is
    /// the only direct evidence that the asymmetric solver is doing any work: a
    /// capability that never fires is indistinguishable from one that does not exist,
    /// and this is the number that tells the two apart.
    pub asymmetric_fraction: f64,
    /// The information layer's read-out, or `None` when the layer was off.
    pub information: Option<InformationOutcome>,
}

/// Classify a power distribution into a polarity.
///
/// The thresholds are documented conventions, not derived quantities. The
/// bipolar test deliberately requires *both* that two blocs together dominate and
/// that they are comparable to each other, because a 40/35/25 split is not bipolar
/// in the sense the stability literature means -- it is three comparable poles.
fn classify_polarity(mut shares: Vec<f64>) -> Polarity {
    if shares.is_empty() {
        return Polarity::Fragmented;
    }
    shares.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let top = shares[0];
    let second = shares.get(1).copied().unwrap_or(0.0);
    let third = shares.get(2).copied().unwrap_or(0.0);

    if top > 0.45 {
        Polarity::Unipolar
    } else if top + second > 0.60 && second > 0.20 {
        Polarity::Bipolar
    } else if third > 0.10 {
        Polarity::Balanced
    } else {
        Polarity::Fragmented
    }
}

/// Normalize shares so they sum to one, flooring at zero.
///
/// Power shares are relative by construction, so this runs every year. Without it,
/// a run of positive growth biases would inflate the total without changing the
/// balance, and the polarity thresholds would stop meaning anything.
fn renormalise(shares: &mut [f64]) {
    for s in shares.iter_mut() {
        if !s.is_finite() || *s < 0.0 {
            *s = 0.0;
        }
    }
    let total: f64 = shares.iter().sum();
    if total > 0.0 {
        for s in shares.iter_mut() {
            *s /= total;
        }
    } else {
        // Everything collapsed: fall back to an even split rather than producing
        // NaN shares that would poison every downstream calculation.
        let even = 1.0 / shares.len().max(1) as f64;
        for s in shares.iter_mut() {
            *s = even;
        }
    }
}

/// Simulate one run.
pub fn simulate_run(config: &Config, seed: u64) -> (RunOutcome, Vec<YearRecord>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let n = config.blocs.len();
    let mut power: Vec<f64> = config.blocs.iter().map(|b| b.power_share).collect();
    renormalise(&mut power);

    let mut tension = 0.1_f64;
    // A local copy: financial conditions evolve during the run, and `config` is
    // borrowed immutably. The initial state comes from the configuration, so two
    // configurations still differ only in what the caller set.
    let mut economy = config.economy.clone();
    let mut records: Vec<YearRecord> = Vec::with_capacity(config.horizon as usize);

    let mut total_cooperation = 0.0;
    let mut total_loss = 0.0;
    let mut dyad_years = 0u32;
    let mut trap_years = 0u32;
    let mut shock_counts = (0u32, 0u32, 0u32);
    let mut asymmetric_dyads = 0u32;

    // --- the information layer's own state ----------------------------------
    //
    // `belief` is what the audience believes about each bloc's culpability; `credibility`
    // is each bloc's stock of standing as a source. Both are meaningless unless the layer
    // is on, and when it is off they are never read.
    let information_on = config.information.enabled
        && config.information.truth.len() == n
        && config.information.effort.len() == n
        && n >= 2;
    let mut belief = vec![0.0_f64; n];
    let mut credibility = vec![config.information.credibility.ceiling; n];
    let mut information_outcome = InformationOutcome {
        legitimated_wars: 0,
        bare_wars: 0,
        contestable_years: 0,
        detected_lies: 0,
        mean_belief: 0.0,
        mean_credibility: 0.0,
    };
    // Credibility only moves for blocs that are actually spending it.
    let assertive: Vec<bool> = if information_on {
        config.information.effort.iter().map(|e| *e > 0.0).collect()
    } else {
        vec![false; n]
    };

    for year in 0..config.horizon {
        // --- the information layer: belief and credibility -------------------
        //
        // Updated *before* the attack decision, because the decision is taken on the
        // belief the audience currently holds. Equation (3) of `INFORMATION_WARFARE.md`,
        // reduced to the simulation's granularity: each bloc's assertions raise belief
        // about its target, weighted by its own credibility, while verification pulls
        // belief back toward the truth.
        if information_on {
            let params = &config.information;
            let previous_belief = belief.clone();

            // `effort[i]` is bloc `i`'s *own* total sustained assertion mass, one meaning
            // and one meaning only, so that the quantity which damages its credibility is
            // the same quantity that moves belief. Targeting is uniform across the other
            // blocs: no available source says who talks about whom, and inventing a
            // targeting matrix would make the layer's output a property of that guess
            // rather than of the model. What makes one dyad differ from another is then
            // the payoff structure, which is the model's own.
            let reach = (n - 1) as f64;
            for j in 0..n {
                let injection: f64 = (0..n)
                    .filter(|i| *i != j)
                    .map(|i| params.effort[i] / reach * credibility[i])
                    .sum();
                // Cross-contamination: suspicion about one bloc transferring to another.
                // Zero unless the configured influence matrix says otherwise.
                let contamination: f64 = (0..n)
                    .filter(|m| *m != j)
                    .map(|m| params.influence.contamination[m][j] * previous_belief[m])
                    .sum();
                belief[j] = ((1.0 - params.verification) * previous_belief[j]
                    + params.verification * params.truth[j]
                    + injection
                    + contamination)
                    .clamp(0.0, 1.0);
            }

            // Credibility: detection and damage, per bloc. Theorem 2 of the paper, and
            // the reason a sustained campaign is self-limiting.
            for i in 0..n {
                if !assertive[i] {
                    continue;
                }
                let detected = rng.gen::<f64>() < params.credibility.detection_probability(params.effort[i]);
                if detected {
                    information_outcome.detected_lies += 1;
                }
                credibility[i] = params.credibility.step(credibility[i], params.effort[i], detected);
            }
        }

        // --- shocks ---------------------------------------------------------
        let mut year_shocks = Vec::new();

        // With the layer off this is the original exogenous draw, unchanged and
        // consuming exactly the same random numbers, so the published figures reproduce.
        // With it on, onset is *derived*: the war rate becomes an output of the model
        // rather than an assumption in it. `INFORMATION_WARFARE.md` section 3.2.
        let war_probability_draw = rng.gen::<f64>();
        let war_triggered = if information_on {
            let mut trigger: Option<(usize, usize, bool)> = None;
            let mut best_payoff = f64::NEG_INFINITY;
            let mut contestable = false;
            for i in 0..n {
                for j in 0..n {
                    if i == j {
                        continue;
                    }
                    let total = power[i] + power[j];
                    let gap = if total > 0.0 {
                        (power[i] - power[j]).abs() / total
                    } else {
                        0.0
                    };
                    let interdependence = economy.interdependence(i, j);
                    let payoffs = config.game.payoffs_for(
                        gap,
                        tension,
                        interdependence,
                        config.blocs[i].cooperation_valuation,
                    );
                    // The spoils are the model's own prize for defection -- what the
                    // aggressor gains by taking advantage of a cooperator rather than
                    // settling for mutual competition. Nothing new is invented here.
                    let spoils = (payoffs.dc - payoffs.dd).max(0.0);
                    let cost = payoffs.cc.max(0.0) + config.information.force_cost;
                    let legitimation = &config.information.legitimation;
                    let bare = attack_payoff(spoils, cost, config.information.coalition_channel, legitimation, 0.0);
                    let at_belief = attack_payoff(
                        spoils,
                        cost,
                        config.information.coalition_channel,
                        legitimation,
                        belief[j],
                    );
                    // Corollary 1.1: the dyad is contestable when justification is what
                    // decides it -- not needed at zero belief, and sufficient at the
                    // belief actually held.
                    if bare < 0.0 && at_belief >= 0.0 {
                        contestable = true;
                    }
                    if at_belief > best_payoff {
                        best_payoff = at_belief;
                        trigger = Some((i, j, bare >= 0.0));
                    }
                }
            }
            if contestable {
                information_outcome.contestable_years += 1;
            }
            match trigger {
                Some((_, _, bare)) if best_payoff >= 0.0 => {
                    if bare {
                        information_outcome.bare_wars += 1;
                    } else {
                        information_outcome.legitimated_wars += 1;
                    }
                    true
                }
                _ => false,
            }
        } else {
            war_probability_draw < config.shocks.war_probability
        };

        if war_triggered {
            year_shocks.push(ShockKind::RegionalWar);
            shock_counts.2 += 1;
            tension += config.shocks.war_tension;
            // A war damages a pair, not everyone equally.
            if n >= 2 {
                // The random pair is drawn *unconditionally*, even when a target is
                // pinned, so that a targeted scenario consumes exactly the same random
                // numbers as the baseline. Otherwise the two worlds would diverge for a
                // reason that has nothing to do with the war -- the whole downstream
                // stream shifts -- and the comparison would be contaminated by luck
                // rather than by the scenario.
                let random_a = rng.gen_range(0..n);
                let a = config.war_target.unwrap_or(random_a).min(n - 1);
                let mut b = rng.gen_range(0..n);
                if b == a {
                    b = (b + 1) % n;
                }
                // Theorem 5(i) inside the simulation: a war between deeply interdependent
                // blocs destroys more, because there was more to sever. The baseline --
                // severance_scale at zero, or the layer off -- is unaffected.
                let severance = if information_on {
                    1.0 + config.information.severance_scale * economy.interdependence(a, b)
                } else {
                    1.0
                };
                power[a] *= 1.0 - config.shocks.war_power_loss * severance;
                power[b] *= 1.0 - config.shocks.war_power_loss * severance;
            }
        }
        if rng.gen::<f64>() < config.shocks.crisis_probability {
            year_shocks.push(ShockKind::Crisis);
            shock_counts.0 += 1;
            tension += config.shocks.crisis_tension;
            if n > 0 {
                let a = rng.gen_range(0..n);
                power[a] *= 1.0 - config.shocks.crisis_power_loss;
            }
        }
        if rng.gen::<f64>() < config.shocks.breakthrough_probability {
            year_shocks.push(ShockKind::Breakthrough);
            shock_counts.1 += 1;
            if n > 0 {
                let a = rng.gen_range(0..n);
                power[a] *= 1.0 + config.shocks.breakthrough_gain;
            }
        }

        // --- energy disruption ----------------------------------------------
        //
        // Financial strain makes a disruption more likely, and the sourced exposure
        // figures decide who pays. This is the door into the model that acts on
        // *capability* rather than on choices: an importer loses supply and an
        // exporter loses revenue, so one event moves blocs in opposite directions --
        // which the payoff matrix cannot express, because the matrix is about what a
        // bloc decides, not about what it is able to do.
        let disruption_probability = (config.shocks.energy_disruption_probability
            * (1.0 + economy.recession_risk))
            .clamp(0.0, 1.0);
        let energy_disruption = if rng.gen::<f64>() < disruption_probability {
            1.0
        } else {
            0.0
        };
        if energy_disruption > 0.0 {
            for (index, exposure) in economy.energy.iter().enumerate() {
                if index < n {
                    power[index] *= 1.0
                        - exposure.disruption_exposure() * config.shocks.energy_disruption_damage;
                }
            }
        }

        // --- play every dyad ------------------------------------------------
        let mut year_cooperation = 0.0;
        let mut year_loss = 0.0;
        let mut dyads = 0u32;
        let mut equilibrium_votes: Vec<&'static str> = Vec::new();

        // `tension` is a system-level index -- the report calls it the mean accumulated
        // tension -- so a competitive pair must feed it in proportion to how much of the
        // system that pair is. Feeding it a flat amount per pair makes the index depend
        // on how finely the world is partitioned: seven blocs have 21 pairs against five
        // blocs' 10, so the same behaviour would have doubled the tension index, and the
        // model would have reported that *naming a continent* made the world twice as
        // tense. See `REFERENCE_DYADS`.
        let feed_scale = tension_feed_scale(n);

        for i in 0..n {
            for j in (i + 1)..n {
                let total = power[i] + power[j];
                let gap = if total > 0.0 {
                    (power[i] - power[j]).abs() / total
                } else {
                    0.0
                };

                // Interdependence is the economic layer's contribution to the game
                // itself: a pair that trades heavily has more to lose from a
                // rupture, which raises what cooperation is worth to both sides.
                //
                // Each side gets its *own* matrix, which is what lets a bloc's own
                // valuation of cooperation change what it chooses. With every
                // valuation at the neutral 1.0 the two matrices are identical and this
                // is exactly the symmetric game it replaced.
                let interdependence = economy.interdependence(i, j);
                let mine_matrix = config.game.payoffs_for(
                    gap,
                    tension,
                    interdependence,
                    config.blocs[i].cooperation_valuation,
                );
                let theirs_matrix = config.game.payoffs_for(
                    gap,
                    tension,
                    interdependence,
                    config.blocs[j].cooperation_valuation,
                );
                let solution = solve_pair(&mine_matrix, &theirs_matrix);

                // Realize the equilibrium. A pure equilibrium is deterministic; at a
                // mixed one neither side can commit, so the two sides draw
                // *independently*. That is what "a mixed equilibrium genuinely
                // randomises rather than being rounded to one action" has to mean --
                // drawing once for the pair would make the two sides' choices
                // perfectly correlated, which is precisely the commitment a mixed
                // equilibrium says neither side can make.
                //
                // The two probabilities are separate because the two matrices are: at
                // a mixed equilibrium of an asymmetric game the sides mix at different
                // rates, and using one rate for both would erase the very asymmetry
                // the solver was widened to find.
                let draw = |rng: &mut StdRng, probability: f64| {
                    if rng.gen::<f64>() < probability {
                        Action::Cooperate
                    } else {
                        Action::Compete
                    }
                };
                let mine = draw(&mut rng, solution.cooperate_probability);
                let theirs = draw(&mut rng, solution.opponent_cooperate_probability);
                let both_cooperated = mine == Action::Cooperate && theirs == Action::Cooperate;

                // Realized cooperation for the dyad: the share of the two sides that
                // cooperated. Its expectation is the mean of the two probabilities, so
                // the statistic keeps the meaning and scale it had before.
                let cooperated_sides =
                    u32::from(mine == Action::Cooperate) + u32::from(theirs == Action::Cooperate);
                year_cooperation += f64::from(cooperated_sides) / 2.0;
                year_loss += solution.efficiency_loss;
                equilibrium_votes.push(solution.equilibrium.label());
                if solution.equilibrium == Equilibrium::Asymmetric {
                    asymmetric_dyads += 1;
                }
                dyads += 1;

                // --- payoffs feed back into power ---------------------------
                //
                // Realized payoffs shift relative power: cooperation creates joint
                // gains, competition wastes surplus. Each side banks its *own*
                // realized payoff, from its *own* matrix, rather than the equilibrium
                // expectation, so a defector actually collects the temptation it took
                // rather than the average. The affinity term scales the payoff a bloc
                // gets when it is the one cooperating, because a bloc embedded in
                // global supply chains has more to gain from cooperation -- and it is
                // deliberately separate from that bloc's *valuation*, which has already
                // done its work inside the matrix by deciding whether it cooperates at
                // all.
                //
                // The coefficient is half what it was when the feedback used the
                // equilibrium expectation, because that change roughly doubled the
                // spread of the quantity being fed back -- realized payoffs range
                // over `[cd, dc]` where expectations ranged over `[dd, cc]`. Halving
                // it keeps the coupling at the strength it had before, rather than
                // letting an unrelated change silently amplify power concentration.
                // The coefficient is stated **per unit of system mean power**: at the
                // five-bloc reference the mean share is 0.20, so a banked payoff of 1.0
                // moves a bloc by 0.5% of itself, which is what an absolute 0.001 moved
                // the average bloc. It is proportional rather than an absolute increment
                // because the absolute form transferred power from small blocs to large
                // ones for identical behaviour: at the old sizes (14% to 30%) that was a
                // factor of two, but with a 4% region in the system it drains that region
                // to nothing while barely touching anyone else. A granularity artefact
                // again -- see `REFERENCE_DYADS` and `default_blocs()`.
                const PAYOFF_TO_POWER: f64 = 0.005;
                let banked = |matrix: &Payoffs, me: Action, them: Action, bloc: &PowerBloc| {
                    let base = matrix.payoff(me, them);
                    if me == Action::Cooperate {
                        base * bloc.cooperation_affinity
                    } else {
                        base
                    }
                };
                // Only the realized payoff acts here. `growth_bias` deliberately does
                // **not**: it is an *annual* rate and is applied once, in the drift
                // step below. Applying it per dyad as well made a bloc's growth
                // depend on how many other blocs existed -- a five-bloc world
                // compounded every bias five times a year and a seven-bloc world
                // seven times. That is a property of the bloc count, not of any bloc,
                // and it is exactly the kind of artefact a reader comparing systems
                // of different sizes would have taken for a finding.
                power[i] *= (1.0
                    + banked(&mine_matrix, mine, theirs, &config.blocs[i]) * PAYOFF_TO_POWER)
                    .max(0.0);
                power[j] *= (1.0
                    + banked(&theirs_matrix, theirs, mine, &config.blocs[j]) * PAYOFF_TO_POWER)
                    .max(0.0);

                // Competition raises tension; this is the feedback loop that makes
                // an arms race self-reinforcing. A dyad counts as competitive unless
                // it reached mutual cooperation.
                if !both_cooperated {
                    tension += config.game.tension_per_conflict * feed_scale;

                    // Money is leverage. When a pair turns adversarial, the bloc
                    // whose currency the other depends on can restrict access to it,
                    // so the bloc with less monetary leverage absorbs more of the
                    // cost. `monetary_leverage(j, i)` is j's leverage *over* i, which
                    // is why it is the drag on i.
                    const SANCTION_DRAG: f64 = 0.004;
                    power[i] *= 1.0 - economy.monetary_leverage(j, i) * SANCTION_DRAG;
                    power[j] *= 1.0 - economy.monetary_leverage(i, j) * SANCTION_DRAG;
                }
            }
        }

        total_cooperation += year_cooperation;
        total_loss += year_loss;
        dyad_years += dyads;

        // The modal equilibrium is the most legible summary of a year.
        equilibrium_votes.sort_unstable();
        let dominant = equilibrium_votes
            .chunk_by(|a, b| a == b)
            .max_by_key(|chunk| chunk.len())
            .and_then(|chunk| chunk.first().copied())
            .unwrap_or("none");

        // --- drift and stabilisation ----------------------------------------
        //
        // The **one** place a bloc's growth bias is applied, so that it means what
        // its name says: an annual rate, independent of how many blocs exist. See
        // the note in the dyad loop for what it used to do instead.
        for (bloc, share) in config.blocs.iter().zip(power.iter_mut()) {
            let noise = Normal::new(0.0, bloc.volatility)
                .map(|d| d.sample(&mut rng))
                .unwrap_or(0.0);
            // Volatility enters multiplicatively so a bloc cannot be driven to a
            // negative share by a large draw.
            *share *= (1.0 + bloc.growth_bias + noise).max(0.0);
        }
        tension = (tension * (1.0 - config.game.tension_decay)).max(0.0);
        // Financial conditions carry into next year, where they gate how likely a
        // disruption is -- which is what makes this a mechanism rather than a figure.
        economy.step_financial_conditions(tension, energy_disruption);
        renormalise(&mut power);

        let year_coop_rate = if dyads > 0 {
            year_cooperation / dyads as f64
        } else {
            1.0
        };
        let year_loss_rate = if dyads > 0 {
            year_loss / dyads as f64
        } else {
            0.0
        };

        // A "conflict trap" year is one where the system is both tense and
        // uncooperative. Requiring both avoids counting a tense-but-cooperating
        // year (deterrence working) as a trap.
        if tension > config.trap_tension_threshold
            && year_coop_rate < config.trap_cooperation_threshold
        {
            trap_years += 1;
        }

        records.push(YearRecord {
            year,
            shares: power.clone(),
            tension,
            cooperation: year_coop_rate,
            efficiency_loss: year_loss_rate,
            dominant_equilibrium: dominant,
            shocks: year_shocks,
            energy_disruption,
            recession_risk: economy.recession_risk,
        });
    }

    let mean_cooperation = if dyad_years > 0 {
        total_cooperation / dyad_years as f64
    } else {
        1.0
    };
    let mean_efficiency_loss = if dyad_years > 0 {
        total_loss / dyad_years as f64
    } else {
        0.0
    };
    let trap_fraction = if config.horizon > 0 {
        trap_years as f64 / config.horizon as f64
    } else {
        0.0
    };

    let final_shares = power.clone();
    let mut ranked = power.clone();
    ranked.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let top_share = ranked.first().copied().unwrap_or(0.0);

    // "Dominant" requires a clear lead, not merely first place: in a balanced
    // system the largest bloc is not a hegemon, and reporting one would contradict
    // the polarity result.
    let dominant = {
        let best = power
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);
        match best {
            Some(i) if top_share > 0.45 => Some(i),
            _ => None,
        }
    };

    let information = if information_on {
        information_outcome.mean_belief = belief.iter().sum::<f64>() / n as f64;
        information_outcome.mean_credibility = credibility.iter().sum::<f64>() / n as f64;
        Some(information_outcome)
    } else {
        None
    };

    let outcome = RunOutcome {
        dominant,
        final_polarity: classify_polarity(final_shares.clone()),
        mean_cooperation,
        mean_efficiency_loss,
        trap_fraction,
        final_tension: tension,
        pension: config.pension.assess(mean_cooperation, mean_efficiency_loss),
        final_shares,
        top_share,
        final_recession_risk: economy.recession_risk,
        shock_counts,
        asymmetric_fraction: if dyad_years > 0 {
            f64::from(asymmetric_dyads) / f64::from(dyad_years)
        } else {
            0.0
        },
        information,
    };

    (outcome, records)
}

/// Aggregate results across many runs.
#[derive(Debug, Clone)]
pub struct Ensemble {
    pub outcomes: Vec<RunOutcome>,
    /// Cooperation share by year, for trajectory bands.
    pub cooperation_by_year: Vec<f64>,
    /// Mean power share per bloc per year.
    pub mean_shares_by_year: Vec<Vec<f64>>,
    /// Tension by year.
    pub tension_by_year: Vec<f64>,
    /// Number of years recorded per run.
    pub horizon: usize,
    /// Power shares per run, per year, per bloc. `--export` draws its fan charts from
    /// these; everything else in the binary uses the means above.
    pub share_trajectories: Vec<Vec<Vec<f64>>>,
    /// Realized cooperation per run, per year.
    pub cooperation_trajectories: Vec<Vec<f64>>,
    /// Accumulated tension per run, per year.
    pub tension_trajectories: Vec<Vec<f64>>,
    /// Realized Pareto-efficiency loss per run, per year.
    pub loss_trajectories: Vec<Vec<f64>>,
}

impl Ensemble {
    /// Run the simulation `config.runs` times with distinct seeds.
    pub fn run(config: &Config) -> Self {
        let mut outcomes = Vec::with_capacity(config.runs);
        let horizon = config.horizon as usize;
        let mut cooperation_by_year = vec![0.0; horizon];
        let mut tension_by_year = vec![0.0; horizon];
        let n = config.blocs.len();
        let mut mean_shares_by_year = vec![vec![0.0; n]; horizon];

        // The same trajectories, kept per run rather than averaged. A mean line cannot
        // show how wide the spread around it is, and the spread is most of what a
        // forty-run ensemble has to say about a fifty-year extrapolation: two worlds
        // with the same mean but different dispersion are different worlds. The cost is
        // a few megabytes at the default settings.
        let mut share_trajectories: Vec<Vec<Vec<f64>>> = Vec::with_capacity(config.runs);
        let mut cooperation_trajectories: Vec<Vec<f64>> = Vec::with_capacity(config.runs);
        let mut tension_trajectories: Vec<Vec<f64>> = Vec::with_capacity(config.runs);
        let mut loss_trajectories: Vec<Vec<f64>> = Vec::with_capacity(config.runs);

        for run in 0..config.runs {
            // Distinct, reproducible seeds derived from the configured base, so two
            // parameter settings can be compared against the same shock draws.
            let seed = config
                .seed
                .wrapping_add(run as u64 * 2_654_435_761);
            let (outcome, records) = simulate_run(config, seed);

            let mut run_shares = Vec::with_capacity(horizon);
            let mut run_cooperation = Vec::with_capacity(horizon);
            let mut run_tension = Vec::with_capacity(horizon);
            let mut run_loss = Vec::with_capacity(horizon);

            for (year, record) in records.iter().enumerate() {
                if year < horizon {
                    cooperation_by_year[year] += record.cooperation;
                    tension_by_year[year] += record.tension;
                    for (i, share) in record.shares.iter().enumerate() {
                        if i < n {
                            mean_shares_by_year[year][i] += share;
                        }
                    }
                    run_shares.push(record.shares.clone());
                    run_cooperation.push(record.cooperation);
                    run_tension.push(record.tension);
                    run_loss.push(record.efficiency_loss);
                }
            }
            share_trajectories.push(run_shares);
            cooperation_trajectories.push(run_cooperation);
            tension_trajectories.push(run_tension);
            loss_trajectories.push(run_loss);
            outcomes.push(outcome);
        }

        let runs = config.runs.max(1) as f64;
        for year in 0..horizon {
            cooperation_by_year[year] /= runs;
            tension_by_year[year] /= runs;
            for share in mean_shares_by_year[year].iter_mut() {
                *share /= runs;
            }
        }

        Ensemble {
            outcomes,
            cooperation_by_year,
            mean_shares_by_year,
            tension_by_year,
            horizon,
            share_trajectories,
            cooperation_trajectories,
            tension_trajectories,
            loss_trajectories,
        }
    }

    /// Fraction of runs ending in each polarity.
    pub fn polarity_distribution(&self) -> Vec<(Polarity, f64)> {
        let total = self.outcomes.len().max(1) as f64;
        [
            Polarity::Unipolar,
            Polarity::Bipolar,
            Polarity::Balanced,
            Polarity::Fragmented,
        ]
        .iter()
        .map(|p| {
            let count = self
                .outcomes
                .iter()
                .filter(|o| o.final_polarity == *p)
                .count();
            (*p, count as f64 / total)
        })
        .collect()
    }

    /// Probability each bloc ends dominant, in bloc order.
    pub fn dominance_probabilities(&self, n: usize) -> Vec<f64> {
        let total = self.outcomes.len().max(1) as f64;
        (0..n)
            .map(|i| {
                self.outcomes
                    .iter()
                    .filter(|o| o.dominant == Some(i))
                    .count() as f64
                    / total
            })
            .collect()
    }

    /// Probability that no bloc ends dominant -- the balanced outcome.
    pub fn no_clear_leader_probability(&self) -> f64 {
        let total = self.outcomes.len().max(1) as f64;
        self.outcomes.iter().filter(|o| o.dominant.is_none()).count() as f64 / total
    }

    /// Mean and quantiles of a per-run statistic, for reporting bands.
    pub fn summarize<F: Fn(&RunOutcome) -> f64>(&self, f: F) -> (f64, f64, f64) {
        let mut values: Vec<f64> = self.outcomes.iter().map(&f).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let q = |p: f64| -> f64 {
            let idx = ((values.len() - 1) as f64 * p).round() as usize;
            values[idx.min(values.len() - 1)]
        };
        (mean, q(0.10), q(0.90))
    }

    /// Mean pension outcome across runs, and the cooperative reference it is
    /// measured against.
    pub fn pension_summary(&self, link: &PensionLink) -> (PensionOutcome, PensionOutcome) {
        let n = self.outcomes.len().max(1) as f64;
        let mean_coop = self.outcomes.iter().map(|o| o.mean_cooperation).sum::<f64>() / n;
        let mean_loss = self.outcomes.iter().map(|o| o.mean_efficiency_loss).sum::<f64>() / n;
        let observed = link.assess(mean_coop, mean_loss);
        // The reference is total cooperation: the best the channel permits.
        let reference = link.assess(1.0, 0.0);
        (observed, reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> Config {
        Config {
            horizon: 10,
            runs: 12,
            ..Config::default()
        }
    }

    /// The same seed must produce the same run, or the ensemble is not
    /// reproducible and parameter comparisons are meaningless.
    ///
    /// The divergence check compares the whole outcome rather than
    /// `mean_cooperation`. That was the original assertion and it was the wrong
    /// one: cooperation is a *bounded* statistic that can legitimately saturate at
    /// `0.0` for an entire family of seeds once the security dilemma dominates, so
    /// comparing it asserts something about the parameter regime rather than about
    /// the seed. Under the old degenerate defaults it failed for exactly that
    /// reason. Tension accumulates the shock draws, so it is seed-dependent in every
    /// regime.
    #[test]
    fn runs_are_reproducible_from_a_seed() {
        let config = small_config();
        let (a, _) = simulate_run(&config, 42);
        let (b, _) = simulate_run(&config, 42);
        assert_eq!(a, b, "the same seed must give the same outcome");

        let (c, _) = simulate_run(&config, 43);
        assert_ne!(a, c, "different seeds must give a different run");
        assert_ne!(
            a.final_tension, c.final_tension,
            "tension accumulates the shock draws, so it cannot be seed-independent"
        );
    }

    /// Shares must remain a valid distribution at every step. If renormalisation
    /// failed, polarity thresholds would silently stop meaning anything.
    #[test]
    fn shares_stay_normalised_throughout() {
        let config = small_config();
        let (_, records) = simulate_run(&config, 7);
        for record in &records {
            let total: f64 = record.shares.iter().sum();
            assert!(
                (total - 1.0).abs() < 1e-9,
                "year {}: shares sum to {total}",
                record.year
            );
            for share in &record.shares {
                assert!(
                    share.is_finite() && *share >= 0.0,
                    "year {}: invalid share {share}",
                    record.year
                );
            }
        }
    }

    /// All reported statistics must be finite and in range, for every run.
    #[test]
    fn run_statistics_are_finite_and_in_range() {
        let config = small_config();
        for seed in 0..20u64 {
            let (o, _) = simulate_run(&config, seed);
            assert!((0.0..=1.0).contains(&o.mean_cooperation), "seed {seed}");
            assert!((0.0..=1.0).contains(&o.mean_efficiency_loss), "seed {seed}");
            assert!((0.0..=1.0).contains(&o.trap_fraction), "seed {seed}");
            assert!(o.final_tension.is_finite() && o.final_tension >= 0.0, "seed {seed}");
            assert!(o.top_share.is_finite() && o.top_share > 0.0, "seed {seed}");
            assert!(o.pension.replacement_rate.is_finite(), "seed {seed}");
        }
    }

    /// Polarity classification must reflect the distribution, and the documented
    /// third-pole convention must be applied consistently.
    ///
    /// The case that used to sit here -- `[0.25, 0.24, 0.09, 0.08]` asserted as
    /// `Balanced` -- was wrong on the module's own rule. Two poles of about a
    /// quarter with nothing else above 10% is not three comparable poles, so the
    /// classifier's `Fragmented` was right and the expectation was the bug. The two
    /// cases below now straddle the 10% convention exactly, so the boundary is
    /// pinned instead of implied.
    #[test]
    fn polarity_classification_reflects_the_distribution() {
        assert_eq!(classify_polarity(vec![0.80, 0.10, 0.05, 0.05]), Polarity::Unipolar);
        assert_eq!(classify_polarity(vec![0.35, 0.32, 0.20, 0.13]), Polarity::Bipolar);
        assert_eq!(classify_polarity(vec![0.30, 0.28, 0.25, 0.17]), Polarity::Balanced);
        // Three comparable poles, the smallest of them just above the convention.
        assert_eq!(
            classify_polarity(vec![0.35, 0.24, 0.11, 0.10, 0.10, 0.10]),
            Polarity::Balanced
        );
        // The same shape with the third pole exactly *at* the convention, not above
        // it: `third > 0.10` is strict, so this falls through to fragmented.
        assert_eq!(
            classify_polarity(vec![0.36, 0.24, 0.10, 0.10, 0.10, 0.10]),
            Polarity::Fragmented
        );
        // Two large poles and two marginal ones: there is no third pole.
        assert_eq!(classify_polarity(vec![0.25, 0.24, 0.09, 0.08]), Polarity::Fragmented);
        // Empty must not panic.
        assert_eq!(classify_polarity(vec![]), Polarity::Fragmented);
    }

    /// The ensemble must be internally consistent: probabilities sum to one and
    /// trajectory means lie in range.
    #[test]
    fn ensemble_aggregates_are_well_formed() {
        let config = small_config();
        let ensemble = Ensemble::run(&config);

        let total: f64 = ensemble.polarity_distribution().iter().map(|(_, p)| p).sum();
        assert!((total - 1.0).abs() < 1e-9, "polarities sum to {total}");

        let dominance: f64 = ensemble.dominance_probabilities(config.blocs.len()).iter().sum();
        let no_leader = ensemble.no_clear_leader_probability();
        assert!(
            (dominance + no_leader - 1.0).abs() < 1e-9,
            "dominance {dominance} + no-leader {no_leader} must sum to 1"
        );

        for year in 0..ensemble.horizon {
            assert!((0.0..=1.0).contains(&ensemble.cooperation_by_year[year]));
            assert!(ensemble.tension_by_year[year] >= 0.0);
            let share_total: f64 = ensemble.mean_shares_by_year[year].iter().sum();
            assert!(
                (share_total - 1.0).abs() < 1e-9,
                "year {year}: mean shares sum to {share_total}"
            );
        }
    }

    /// Competition must be able to produce a worse pension outcome than
    /// cooperation *within the simulated system* -- otherwise the coupling claimed
    /// in `pension.rs` is not actually exercised end to end.
    #[test]
    fn cooperation_level_drives_the_pension_outcome_end_to_end() {
        let cooperative = Config {
            game: GameParams {
                cooperation_gain: 9.0,
                defection_temptation: 3.0,
                ..GameParams::default()
            },
            horizon: 15,
            runs: 10,
            ..Config::default()
        };
        let competitive = Config {
            game: GameParams {
                cooperation_gain: 1.0,
                defection_temptation: 8.0,
                exploitation_cost: -3.0,
                ..GameParams::default()
            },
            horizon: 15,
            runs: 10,
            ..Config::default()
        };

        let good = Ensemble::run(&cooperative);
        let bad = Ensemble::run(&competitive);
        let (good_outcome, _) = good.pension_summary(&cooperative.pension);
        let (bad_outcome, _) = bad.pension_summary(&competitive.pension);

        assert!(
            good_outcome.cooperation_index > bad_outcome.cooperation_index,
            "the cooperative parameterisation must actually cooperate more: \
             {:.3} vs {:.3}",
            good_outcome.cooperation_index,
            bad_outcome.cooperation_index
        );
        assert!(
            good_outcome.security_index(&good_outcome) >= bad_outcome.security_index(&good_outcome),
            "and must yield at least as secure a pension"
        );
    }

    /// A zero-horizon or zero-run configuration must not panic or divide by zero.
    #[test]
    fn degenerate_configurations_are_handled() {
        let zero_horizon = Config {
            horizon: 0,
            runs: 3,
            ..Config::default()
        };
        let ensemble = Ensemble::run(&zero_horizon);
        assert_eq!(ensemble.horizon, 0);
        let (mean, lo, hi) = ensemble.summarize(|o| o.mean_cooperation);
        assert!(mean.is_finite() && lo.is_finite() && hi.is_finite());

        let zero_runs = Config {
            horizon: 3,
            runs: 0,
            ..Config::default()
        };
        let ensemble = Ensemble::run(&zero_runs);
        assert!(ensemble.outcomes.is_empty());
        // Summarizing nothing must be finite, not NaN.
        let (mean, _, _) = ensemble.summarize(|o| o.final_tension);
        assert!(mean.is_finite());

        let single_bloc = Config {
            blocs: vec![PowerBloc::new("Solo", 1.0, 0.0, 0.0, 1.0)],
            horizon: 5,
            runs: 3,
            ..Config::default()
        };
        let (outcome, _) = simulate_run(&single_bloc, 1);
        assert!(outcome.final_shares[0].is_finite());
        assert!((outcome.final_shares[0] - 1.0).abs() < 1e-9);
    }

    /// The finding this simulator exists to produce.
    ///
    /// The parameters are invented, so no single trajectory is informative. What
    /// *is* informative is how strongly the qualitative outcome depends on the
    /// cooperation/competition payoff balance -- the balance that decides whether
    /// the security dilemma binds at all, and the question `MULTIPOLAR_GAME.md`
    /// section 4 says the literature is split on. This asserts that dependence end
    /// to end, on the same quantities the report prints.
    ///
    /// Both configurations share the default seed, so they are compared against the
    /// same shock draws rather than against different luck.
    #[test]
    fn outcomes_are_sensitive_to_the_cooperation_competition_balance() {
        let at = |cooperation_gain: f64, defection_temptation: f64| {
            let config = Config {
                game: GameParams {
                    cooperation_gain,
                    defection_temptation,
                    ..GameParams::default()
                },
                horizon: 50,
                runs: 40,
                ..Config::default()
            };
            let ensemble = Ensemble::run(&config);
            let (cooperation, _, _) = ensemble.summarize(|o| o.mean_cooperation);
            let (trap, _, _) = ensemble.summarize(|o| o.trap_fraction);
            let (loss, _, _) = ensemble.summarize(|o| o.mean_efficiency_loss);
            let polarity = ensemble
                .polarity_distribution()
                .into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(polarity, _)| polarity)
                .expect("a 40-run ensemble must classify at least one polarity");
            (cooperation, trap, loss, polarity)
        };

        // Cooperation is worth clearly more than defecting: the dilemma need not bind.
        let (coop_good, trap_good, loss_good, polarity_good) = at(8.0, 4.2);
        // Defecting is worth clearly more than cooperating: the dilemma binds hard.
        let (coop_bad, trap_bad, loss_bad, polarity_bad) = at(1.0, 8.0);

        assert!(
            coop_good > coop_bad + 0.3,
            "a cooperative payoff balance must produce materially more cooperation: \
             {coop_good:.3} vs {coop_bad:.3}"
        );
        assert!(
            trap_good < trap_bad - 0.3,
            "and materially fewer conflict-trap years: {trap_good:.3} vs {trap_bad:.3}"
        );
        assert!(
            loss_good < loss_bad,
            "and less Pareto-efficiency loss: {loss_good:.3} vs {loss_bad:.3}"
        );

        // And the direction has to be right: it is the conflict-locked configuration
        // that concentrates the system.
        //
        // The assertion is that the world *concentrates*, not that it lands on one
        // particular label. With the Atlantic split the contrast the earlier version of
        // this test pinned came back, and with the opposite sign to what a reader might
        // expect: the *conflict-locked* configuration is the concentrated one, and the
        // cooperative world is merely `Balanced` -- cooperation spreads power, a hard
        // dilemma pools it. The labels are the classifier's, and which pair of them
        // appears is a property of the bloc list; that they differ is not.
        assert!(
            matches!(polarity_bad, Polarity::Unipolar | Polarity::Bipolar),
            "a hard dilemma should concentrate the system, got {polarity_bad:?}"
        );
        assert_ne!(
            polarity_good, polarity_bad,
            "the cooperation/competition balance must change the polarity the system \
             settles into: {polarity_good:?} vs {polarity_bad:?}"
        );
    }

    /// "Five editable power blocs" only means something if editing one changes what
    /// happens. This edits the weakest bloc into the strongest and checks the run
    /// notices, which is the concrete form of the editability claim.
    #[test]
    fn editing_a_bloc_changes_the_outcome() {
        let baseline = Config {
            horizon: 50,
            runs: 20,
            ..Config::default()
        };
        let mut tilted = baseline.clone();
        let edited = tilted.blocs.len() - 1;
        // The bias is an annual rate, so "decisive over 50 years" means a large annual
        // number: 0.30 a year is 1.3^50, which no share difference survives. A figure
        // like 0.03 would be decisive only under the old per-dyad compounding this
        // model used to do, and would now lose to the blocs that start ahead.
        tilted.blocs[edited].growth_bias = 0.30;

        let (before, _) = simulate_run(&baseline, 11);
        let (after, _) = simulate_run(&tilted, 11);

        assert!(
            after.final_shares[edited] > before.final_shares[edited],
            "raising a bloc's growth bias must raise its share: {:.4} vs {:.4}",
            after.final_shares[edited],
            before.final_shares[edited]
        );

        let strongest = after
            .final_shares
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index);
        assert_eq!(
            strongest,
            Some(edited),
            "a bloc given a decisive compounding advantage should end up the strongest"
        );
    }

    /// A bloc's growth bias must buy exactly the annual rate it names, and that rate
    /// must not depend on how many blocs share the system.
    ///
    /// This is the regression test for the per-dyad multiplicity: `simulate_run` used
    /// to add `power * growth_bias` inside the dyad loop *and* in the annual drift
    /// step, so a nominal bias of `b` compounded `n` times a year in an `n`-bloc
    /// world. The exponent was a property of the system's size rather than of the
    /// bloc, so any comparison across different bloc counts -- which is what adding a
    /// region to the model is -- would have been measuring the artefact.
    ///
    /// The measurement is of the *dynamics* rather than of a helper: two worlds differ
    /// only in how many blocs they contain, and the bloc under test must grow at the
    /// same annual rate in both. The residual difference is what the extra dyads do
    /// to tension and to the payoff feedback, which is second-order at this bias and
    /// is why the tolerance is stated rather than assumed.
    #[test]
    fn growth_bias_is_an_annual_rate_in_systems_of_any_size() {
        let growth = 0.05;

        let rate_in = |extra: usize| -> f64 {
            let mut config = Config {
                horizon: 1,
                runs: 1,
                ..Config::default()
            };
            for bloc in config.blocs.iter_mut() {
                bloc.volatility = 0.0;
                bloc.growth_bias = 0.0;
            }
            // The bloc under test is a mid-sized one rather than the first, so the
            // result cannot come from an ordering accident.
            let tested = 2;
            config.blocs[tested].growth_bias = growth;
            let start = config.blocs[tested].power_share;
            for k in 0..extra {
                // A zero-share, zero-volatility, zero-bias bloc: it adds dyads and
                // nothing else, which is the smallest possible change in bloc count.
                config
                    .blocs
                    .push(PowerBloc::new(&format!("Filler{k}"), 0.0, 0.0, 0.0, 1.0));
            }
            config.economy = Economy::for_blocs(&config.blocs);
            let (outcome, _) = simulate_run(&config, 3);
            outcome.final_shares[tested] / start
        };

        let five = rate_in(0);
        let seven = rate_in(2);

        assert!(
            five > 1.0 + growth * 0.5 && five < 1.0 + growth * 1.5,
            "one year at a {growth} bias must be roughly one application, got {five}"
        );
        assert!(
            (five - seven).abs() < 0.02,
            "the same bias must buy the same annual growth in a seven-bloc world as in \
             a five-bloc one, or the model's growth rate is a property of the bloc \
             count: {five} vs {seven}"
        );
    }

    /// Tension is a system-level index, so partitioning the world more finely must not
    /// make it tenser.
    ///
    /// This is the regression test for the second of the two per-pair feed defects: the
    /// tension index was fed a flat `tension_per_conflict` for every competing dyad, so
    /// a seven-bloc world -- 21 pairs against the reference ten -- would have run at
    /// roughly twice the tension for identical behaviour. The model would then have
    /// reported that naming a continent made the world twice as tense, and every
    /// downstream quantity (cooperation, trap years, pension index) with it.
    ///
    /// The invariant is what the test checks: the per-pair feed times the pair count is
    /// constant in the bloc count, and exact at the ten-pair reference.
    #[test]
    fn the_tension_feed_does_not_depend_on_how_finely_the_world_is_partitioned() {
        let pairs = |n: usize| n * n.saturating_sub(1) / 2;
        let total_feed = |n: usize| tension_feed_scale(n) * pairs(n) as f64;

        assert_eq!(
            tension_feed_scale(5),
            1.0,
            "the factor must be exactly one at the ten-pair reference, or the published \
             five-bloc figures would move"
        );
        for n in 2..=12 {
            assert!(
                (total_feed(n) - REFERENCE_DYADS).abs() < 1e-12,
                "a fully competitive {n}-bloc system must add the reference tension, got {}",
                total_feed(n)
            );
        }
        // A degenerate list has no pairs and no tension to feed.
        assert_eq!(tension_feed_scale(0), 1.0);
        assert_eq!(tension_feed_scale(1), 1.0);
    }

    /// The information layer must be inert when it is off, and "inert" has to mean
    /// *byte-identical*, not merely similar -- `INFORMATION_WARFARE.md` section 8 promises
    /// the published figures reproduce, and every figure in this crate is a snapshot that
    /// would silently drift if the draw sequence moved by one.
    ///
    /// A golden value pins the whole chain: the number of random draws per year, the order
    /// they are consumed in, and every payoff and update downstream of them. If the layer
    /// ever takes a draw while disabled, this fails.
    #[test]
    fn the_information_layer_is_off_by_default_and_leaves_the_baseline_untouched() {
        let mut config = Config::default();
        assert!(
            !config.information.enabled,
            "the layer must default off, or the published figures change meaning"
        );
        config = Config {
            runs: 8,
            seed: 12345,
            ..config
        };
        let ensemble = Ensemble::run(&config);
        assert!(
            ensemble.outcomes.iter().all(|o| o.information.is_none()),
            "a disabled layer must report nothing"
        );
        // Golden value at 8 runs, seed 12345: the baseline this crate published before the
        // layer existed, recorded from the pre-change binary.
        let cooperation = ensemble.summarize(|o| o.mean_cooperation).0;
        assert!(
            (cooperation - 0.207).abs() < 0.0005,
            "baseline cooperation moved to {cooperation}: the information layer must not \
             draw a random number while it is disabled"
        );
    }

    /// And with it on, it must actually be doing something -- a knob that changes nothing
    /// is indistinguishable from one that does not exist.
    #[test]
    fn enabling_the_layer_changes_war_onset_and_reports_its_own_state() {
        let mut config = Config {
            runs: 24,
            ..Config::default()
        };
        let baseline = Ensemble::run(&config);

        let n = config.blocs.len();
        config.information = InformationParams {
            enabled: true,
            effort: vec![0.2; n],
            truth: vec![0.3; n],
            influence: crate::information::Influence::ring(n, 0.1),
            ..InformationParams::default()
        };
        let layered = Ensemble::run(&config);

        assert!(
            layered.outcomes.iter().all(|o| o.information.is_some()),
            "an enabled layer must report its state on every run"
        );
        let reported = layered.outcomes[0].information.expect("present");
        assert!(
            reported.mean_credibility > 0.0 && reported.mean_credibility <= 1.0,
            "credibility is a stock in [0, 1], got {}",
            reported.mean_credibility
        );
        let _ = reported;
        assert_ne!(
            baseline
                .outcomes
                .iter()
                .map(|o| o.shock_counts.2)
                .sum::<u32>(),
            layered
                .outcomes
                .iter()
                .map(|o| o.shock_counts.2)
                .sum::<u32>(),
            "endogenous onset must produce a different war count than the assumed rate"
        );
    }
}
