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

//! Power blocs, and the payoff structure their interactions produce.
//!
//! # The parameters are illustrative, and that matters
//!
//! Nothing in this module is calibrated to measured economic or military data.
//! The growth rates, volatilities, cooperation gains and conflict costs are
//! chosen to be *plausible in sign and rough magnitude* so that the mechanism can
//! be explored -- not to forecast anything. `MULTIPOLAR_GAME.md` section 5 is
//! explicit that there is "no general theorem guaranteeing a multipolar system
//! converges to a stable equilibrium at all, let alone a date by which it will",
//! and section 7 that "genuine uncertainty is the most defensible position".
//!
//! A simulator that produced confident-looking 50-year numbers from invented
//! parameters would be exactly the "plausible narrative wrapped around unfitted
//! parameters" that `FutureWork.md` section 7 warns against. So the honest use of
//! this tool is *sensitivity*: which qualitative outcomes are robust across wide
//! parameter ranges, and which flip on small changes. The binary prints a
//! sensitivity sweep for that reason, and says so in its own output.

use crate::game::Payoffs;

/// One power bloc.
#[derive(Debug, Clone, PartialEq)]
pub struct PowerBloc {
    pub name: String,
    /// Share of total system power, in [0, 1]. Shares are renormalised each year,
    /// so this is a *relative* position rather than an absolute capability.
    pub power_share: f64,
    /// Annual growth bias: a persistent advantage (or disadvantage) beyond the
    /// shared growth rate. Positive means the bloc tends to gain share.
    pub growth_bias: f64,
    /// Annual volatility of this bloc's growth. `MULTIPOLAR_GAME.md` section 4
    /// notes that more independent powers means more opportunities for
    /// miscalculation, so dispersion here is a direct input to instability.
    pub volatility: f64,
    /// How much this bloc gains from cooperation relative to the baseline. A
    /// high value models a bloc deeply embedded in global supply chains, which is
    /// the structural feature the optimistic case in section 4 rests on.
    ///
    /// This scales the *power* the bloc banks when it cooperates, downstream of the
    /// solver. It deliberately does **not** enter the payoff matrix, so it cannot
    /// change what the bloc chooses to do -- see `cooperation_valuation` for that
    /// channel, and the note there on why they are two knobs rather than one.
    pub cooperation_affinity: f64,
    /// How much this bloc values the cooperative outcome, relative to a baseline of
    /// `1.0`. This *does* enter the payoff matrix, so it changes what the bloc
    /// chooses.
    ///
    /// # Why this is separate from `cooperation_affinity`
    ///
    /// They are the two halves of "cooperation is worth more to this bloc", and
    /// conflating them would build in an assumption rather than report a finding: a
    /// bloc that *gains* more from cooperation need not be a bloc that *wants* it
    /// more, and the whole point of `MULTIPOLAR_GAME.md` section 4's open dispute is
    /// that those can come apart.
    ///
    /// * `cooperation_affinity` -- what cooperation pays, once it has been chosen.
    /// * `cooperation_valuation` -- what cooperation is worth, when choosing.
    ///
    /// The second is what the solver needs in order to express the chapter's
    /// optimistic case at all. Until the two sides could face different matrices, a
    /// claim like "AI competition need not be zero-sum *for this bloc*" had nowhere
    /// to live, and `--ai`'s cooperation sweep moved nothing because of it.
    ///
    /// `1.0` means neutral. Every default bloc is neutral, which is what keeps the
    /// model's already-published figures reproducible: with every valuation at 1.0
    /// the two matrices are identical and the general solver reduces to the
    /// symmetric one exactly.
    pub cooperation_valuation: f64,
}

impl PowerBloc {
    pub fn new(
        name: &str,
        power_share: f64,
        growth_bias: f64,
        volatility: f64,
        cooperation_affinity: f64,
    ) -> Self {
        PowerBloc {
            name: name.to_string(),
            power_share,
            growth_bias,
            volatility,
            cooperation_affinity,
            // Neutral by default, so a bloc defined the old way behaves the old way.
            cooperation_valuation: 1.0,
        }
    }

    /// Set how much this bloc values cooperation in its own strategic calculation.
    ///
    /// Builder-style rather than a sixth constructor argument, so every existing
    /// `PowerBloc::new` call keeps compiling and keeps its meaning.
    pub fn with_cooperation_valuation(mut self, valuation: f64) -> Self {
        self.cooperation_valuation = valuation.max(0.0);
        self
    }
}

/// The default five-pole system.
///
/// The names follow `MULTIPOLAR_GAME.md` section 4's description of the current
/// transition: a US-led Atlantic bloc, a China-centred Sinic one, a Eurasian one,
/// an Indo-Pacific one, and the non-aligned middle powers that section 4 notes are
/// now "pursuing independent strategic optimization rather than aligning within a
/// single hegemonic order".
///
/// Starting shares are roughly the shape a five-pole system would have, not
/// measured figures. They are editable, which is the point.
///
/// # The growth biases are annual rates, and they were re-stated to stay that way
///
/// `Sinic 0.040` means four percent a year of relative gain, and that is what the
/// simulation applies -- once, in the annual drift step. The figures here come from
/// converting an earlier convention: `simulation.rs` used to add
/// `power * growth_bias` inside the dyad loop *as well as* in the drift step, so in
/// the five-bloc default world a nominal `0.008` compounded five times a year and
/// was really about 4.1% a year. The conversion is `(1 + b)^5 - 1`, rounded to three
/// decimals. That is why these numbers are larger than the ones this file used to
/// carry, and it is also why no published result moves much: what was held fixed is
/// the *model's behaviour*, and what changed is the *meaning* of the parameter.
///
/// The re-parameterisation is not cosmetic. A per-dyad application makes the annual
/// growth rate a function of how many blocs exist, so any comparison between systems
/// of different sizes -- which is exactly what adding a region to this model is --
/// would have been measuring the bloc count rather than the region.
pub fn default_blocs() -> Vec<PowerBloc> {
    vec![
        PowerBloc::new("Atlantic", 0.30, 0.000, 0.020, 1.00),
        PowerBloc::new("Sinic", 0.26, 0.040, 0.025, 0.95),
        PowerBloc::new("Eurasian", 0.16, 0.010, 0.035, 0.70),
        PowerBloc::new("Indo-Pacific", 0.14, 0.050, 0.030, 0.90),
        PowerBloc::new("Non-Aligned", 0.14, 0.020, 0.040, 1.05),
    ]
}

/// The parameters that shape the interaction, as distinct from the blocs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameParams {
    /// Payoff from mutual cooperation at perfect power parity and zero tension.
    pub cooperation_gain: f64,
    /// Payoff from mutual competition.
    pub conflict_payoff: f64,
    /// The temptation to compete while the other cooperates.
    pub defection_temptation: f64,
    /// The cost of being the one who cooperates against competition -- the
    /// "sucker's payoff", and the quantity the security dilemma turns on.
    pub exploitation_cost: f64,
    /// How strongly a *power gap* pushes the pair toward competition. A hegemonic
    /// pair is less of a security dilemma than a near-peer one, because the weaker
    /// side cannot credibly threaten the stronger -- this is the mechanism behind
    /// the bipolar-stability argument in `MULTIPOLAR_GAME.md` section 4.
    pub parity_pressure: f64,
    /// How strongly accumulated global tension pushes the pair toward
    /// competition. This is what makes the system path-dependent: today's
    /// competition raises tomorrow's.
    pub tension_pressure: f64,
    /// How strongly accumulated tension erodes the payoff of *mutual* competition.
    ///
    /// An arms race that runs long enough stops paying for itself: both sides
    /// exhaust the surplus they were competing over. This is the "imperial
    /// overstretch" mechanism `MULTIPOLAR_GAME.md` section 8 describes, and it is
    /// what lets the system leave the pure dilemma at all. Without it,
    /// `cooperation_gain` and `defection_temptation` are both constant in tension,
    /// so whether `Compete` dominates is fixed at `tension = 0` for all time -- and
    /// a simulator that cannot change its answer cannot produce a distribution.
    pub conflict_wear: f64,
    /// Tension added per competing dyad, and how fast tension decays. Together
    /// these set whether the system can de-escalate at all.
    pub tension_per_conflict: f64,
    pub tension_decay: f64,
}

impl Default for GameParams {
    fn default() -> Self {
        GameParams {
            cooperation_gain: 3.0,
            conflict_payoff: 1.0,
            defection_temptation: 4.2,
            exploitation_cost: -0.8,
            parity_pressure: 1.1,
            tension_pressure: 0.9,
            conflict_wear: 0.9,
            tension_per_conflict: 0.03,
            tension_decay: 0.10,
        }
    }
}

impl GameParams {
    /// Build the payoff matrix for one pair, given their relative power gap and
    /// the current global tension.
    ///
    /// `power_gap` is `|a - b| / (a + b)`, so 0 means perfect parity and 1 means
    /// total dominance. `tension` is the accumulated system-wide tension.
    ///
    /// # How the structure moves
    ///
    /// * **Parity raises the stakes.** Near-peer rivals face a genuine security
    ///   dilemma; a dominant power and a small one do not. So `parity_pressure`
    ///   scales the temptation to defect AND the cost of being exploited, both of
    ///   which are what make competition rational.
    /// * **Cooperation payoff falls with tension.** Accumulated tension erodes the
    ///   gains from cooperation -- sanctions, decoupling, broken supply chains --
    ///   which is the feedback loop that makes an arms race self-reinforcing.
    /// * **Mutual competition is eroded by that same tension.** An arms race that
    ///   runs long enough consumes the surplus it was fought over. This is the one
    ///   channel by which sustained conflict becomes self-limiting rather than
    ///   self-reinforcing, and it is the only way the system can leave the pure
    ///   dilemma; see `conflict_wear`.
    /// * **Exploitation gets worse as the gap widens.** Cooperating against a much
    ///   stronger competitor is more costly than against a peer.
    /// * **Interdependence raises what cooperation is worth.** Two blocs that trade
    ///   heavily have more to lose from a rupture, which is the standard argument
    ///   that commerce dampens conflict.
    ///
    /// `interdependence` comes from `economy.rs` and is `0.0` for a pair that trades
    /// no energy. It multiplies the cooperation payoff, so a valuable trading
    /// relationship makes mutual cooperation worth more.
    ///
    /// # What interdependence deliberately does *not* do
    ///
    /// It leaves the temptation to defect alone, and that is a judgement worth
    /// stating rather than burying. A richer relationship is worth more to *capture*
    /// as well as more to sustain, so the sign of its effect on `dc` is genuinely
    /// ambiguous. Pushing it up would make trade a source of predation; pushing it
    /// down would make trade a source of restraint. This model picks neither, which
    /// means the `--sweep` output shows the restraint channel on its own rather than
    /// the sum of two opposing ones.
    ///
    /// The coefficient is **illustrative**: nobody has measured how a one-point
    /// change in an energy-trade share changes the value of cooperation in a
    /// 50-year counterfactual.
    ///
    /// `valuation` is the *own side's* [`PowerBloc::cooperation_valuation`]: what
    /// cooperation is worth to the bloc this matrix describes. It scales the gross
    /// cooperation gain before tension erodes it, and it is the only term in this
    /// function that can make two sides' matrices differ. Passing `1.0` reproduces
    /// the pre-asymmetric model exactly.
    pub fn payoffs_for(
        &self,
        power_gap: f64,
        tension: f64,
        interdependence: f64,
        valuation: f64,
    ) -> Payoffs {
        /// Illustrative: how strongly a fully interdependent pair values cooperation.
        const INTERDEPENDENCE_GAIN: f64 = 0.6;

        let parity = (1.0 - power_gap).clamp(0.0, 1.0);
        let interdependence = interdependence.clamp(0.0, 1.0);
        let valuation = valuation.max(0.0);

        // Cooperation is worth more between equals, worth more between trading
        // partners, worth more to a bloc that values it more, and worth less as
        // tension accumulates. Floored at zero: cooperation cannot become actively
        // harmful, which would be a different game.
        let cc = (self.cooperation_gain
            * (0.55 + 0.45 * parity)
            * (1.0 + INTERDEPENDENCE_GAIN * interdependence)
            * valuation
            - self.tension_pressure * tension)
            .max(0.0);

        // Mutual competition is the arms-race outcome. It is not zero -- blocs
        // still exist and function -- but it wastes surplus, and the longer the
        // race runs the more it wastes.
        let dd = self.conflict_payoff - self.conflict_wear * tension;

        // The temptation to defect: compete while the other cooperates. Rises with
        // parity, because against a peer the defector takes a decisive advantage,
        // whereas against a much weaker partner there is less to take.
        let dc = self.defection_temptation * (0.45 + 0.55 * parity);

        // The sucker's payoff: cooperate while the other competes. This is where
        // the security dilemma bites, and it worsens as the gap widens because the
        // stronger competitor can impose more.
        let cd = self.exploitation_cost * (0.5 + 0.5 * power_gap);

        Payoffs { cc, cd, dc, dd }
    }

    /// The base case: no economic interdependence and a neutral valuation of
    /// cooperation.
    ///
    /// Test-only. The simulator always carries an economic layer and always passes a
    /// valuation, so the binary has no use for a shorthand, but the payoff tests read
    /// considerably better without arguments that are always zero and one.
    #[cfg(test)]
    pub fn payoffs(&self, power_gap: f64, tension: f64) -> Payoffs {
        self.payoffs_for(power_gap, tension, 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::solve;

    /// The default parameters must produce a genuine security dilemma between
    /// near-peers -- otherwise the simulator would be modelling something other
    /// than what `MULTIPOLAR_GAME.md` section 4 describes.
    #[test]
    fn default_parameters_produce_a_dilemma_between_near_peers() {
        let params = GameParams::default();
        let payoffs = params.payoffs(0.05, 0.0);
        let solution = solve(&payoffs);

        assert_eq!(
            solution.equilibrium,
            crate::game::Equilibrium::Dilemma,
            "near-peers under default parameters should face the security dilemma, \
             got {:?} with payoffs {payoffs:?}",
            solution.equilibrium
        );
        assert!(
            solution.efficiency_loss > 0.0,
            "a dilemma means surplus is lost, got {}",
            solution.efficiency_loss
        );
    }

    /// The dilemma must *weaken* with the power gap: a dominant power and a small
    /// one are not near-peer rivals, so cooperation should become more attainable.
    /// This is the mechanism behind the bipolar-stability argument, so it is
    /// pinned.
    #[test]
    fn wide_power_gaps_are_more_cooperative_than_parity() {
        let params = GameParams::default();
        let at_parity = solve(&params.payoffs(0.0, 0.0));
        let lopsided = solve(&params.payoffs(0.9, 0.0));

        assert!(
            lopsided.equilibrium_payoff >= at_parity.equilibrium_payoff,
            "a lopsided pair should do at least as well as near-peers: \
             {:.3} vs {:.3}",
            lopsided.equilibrium_payoff,
            at_parity.equilibrium_payoff
        );
    }

    /// Accumulated tension must erode cooperation, which is the feedback loop that
    /// makes an arms race self-reinforcing rather than self-correcting.
    #[test]
    fn tension_erodes_the_cooperation_payoff() {
        let params = GameParams::default();
        let calm = params.payoffs(0.1, 0.0);
        let tense = params.payoffs(0.1, 1.0);
        assert!(
            tense.cc < calm.cc,
            "tension must reduce what cooperation is worth: {} vs {}",
            calm.cc,
            tense.cc
        );
        // And it is floored, not allowed to go negative.
        let extreme = params.payoffs(0.1, 100.0);
        assert!(extreme.cc >= 0.0, "cooperation payoff must not go negative");
    }

    /// Sustained tension must be able to *break* the pure dilemma, so the system
    /// has an outcome to reach other than permanent universal competition.
    ///
    /// This pins the defect that made the Monte Carlo degenerate: with
    /// `conflict_wear` at zero the dilemma is inescapable, because `Compete`
    /// strictly dominates at every tension level and the simulator can only ever
    /// report one answer. The escape must be a *mixed* equilibrium rather than a
    /// jump to universal cooperation -- "neither side can commit" is the
    /// interesting result, and it is also what gives the ensemble its variance.
    #[test]
    fn sustained_tension_breaks_the_pure_dilemma() {
        let params = GameParams::default();

        // The pure dilemma holds while the system is calm.
        let calm = solve(&params.payoffs(0.05, 0.0));
        assert_eq!(
            calm.equilibrium,
            crate::game::Equilibrium::Dilemma,
            "near-peers at low tension should still face the pure dilemma"
        );
        assert_eq!(calm.cooperate_probability, 0.0);

        // Somewhere along the escalation path it must give way.
        let broken = (1..=60)
            .map(|k| k as f64 * 0.1)
            .map(|tension| (tension, solve(&params.payoffs(0.05, tension))))
            .find(|(_, solution)| solution.equilibrium != crate::game::Equilibrium::Dilemma)
            .expect(
                "a tense system must eventually stop being a pure dilemma, or conflict \
                 has no self-limiting mechanism at all",
            );
        let (tension, solution) = broken;
        assert!(
            tension < 5.0,
            "the escape must happen at a tension the simulation actually reaches, \
             got {tension}"
        );
        assert!(
            solution.cooperate_probability > 0.0 && solution.cooperate_probability < 1.0,
            "the dilemma should break into a mixed equilibrium, got {:?} with q={}",
            solution.equilibrium,
            solution.cooperate_probability
        );
        assert!(
            solution.efficiency_loss > 0.0,
            "a broken dilemma is still not cooperation, so surplus is still lost"
        );
    }

    /// The escape must come from conflict wear specifically, not from the passage
    /// of tension through some other term. This is the direct regression test for
    /// the inescapable-dilemma defect.
    #[test]
    fn without_conflict_wear_the_dilemma_is_inescapable() {
        let params = GameParams {
            conflict_wear: 0.0,
            ..GameParams::default()
        };

        for step in 0..=60 {
            let tension = step as f64 * 0.1;
            assert_eq!(
                solve(&params.payoffs(0.05, tension)).equilibrium,
                crate::game::Equilibrium::Dilemma,
                "with no conflict wear, competing dominates at every tension; \
                 failed at tension {tension}"
            );
        }
    }

    /// High enough cooperation gain must be able to *escape* the dilemma entirely
    /// -- the optimistic case in `MULTIPOLAR_GAME.md` section 4, where AI
    /// competition need not be zero-sum. If no parameter setting could escape it,
    /// the model would be rigged rather than exploratory.
    #[test]
    fn large_cooperation_gains_escape_the_dilemma() {
        let params = GameParams {
            cooperation_gain: 9.0,
            // Deep interdependence: cooperation is worth more than defecting would
            // be, so the temptation cannot exceed the cooperation gain.
            defection_temptation: 4.0,
            ..GameParams::default()
        };
        let payoffs = params.payoffs(0.0, 0.0);
        let solution = solve(&payoffs);
        assert_ne!(
            solution.equilibrium,
            crate::game::Equilibrium::Dilemma,
            "with large enough cooperation gains the dilemma must be escapable, \
             got {:?} with {payoffs:?}",
            solution.equilibrium
        );
        assert_eq!(
            solution.cooperate_probability, 1.0,
            "the cooperative equilibrium should be reached"
        );
    }

    /// Shares must be positive and sum to roughly one, or "power share" is not a
    /// share of anything.
    #[test]
    fn default_blocs_form_a_valid_distribution() {
        let blocs = default_blocs();
        assert_eq!(blocs.len(), 5, "the default system is five-pole");
        let total: f64 = blocs.iter().map(|b| b.power_share).sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "shares must sum to 1, got {total}"
        );
        for bloc in &blocs {
            assert!(
                bloc.power_share > 0.0,
                "{} has a non-positive share",
                bloc.name
            );
            assert!(bloc.volatility >= 0.0, "{} has negative volatility", bloc.name);
        }
    }

    /// Payoffs must stay finite for any input the simulator can produce, including
    /// a degenerate single-bloc system and a saturated tension.
    #[test]
    fn payoffs_are_finite_across_the_input_range() {
        let params = GameParams::default();
        for gap in [0.0, 0.1, 0.5, 0.9, 1.0] {
            for tension in [0.0, 0.5, 1.0, 5.0, 50.0] {
                let p = params.payoffs(gap, tension);
                for value in [p.cc, p.cd, p.dc, p.dd] {
                    assert!(
                        value.is_finite(),
                        "gap {gap} tension {tension} produced {value}"
                    );
                }
            }
        }
    }
}
