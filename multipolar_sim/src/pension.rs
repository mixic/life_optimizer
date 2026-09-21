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

//! Where the multipolar game meets pension security.
//!
//! # Why this module exists
//!
//! `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` section 2c makes a point the Life
//! Optimizer cannot answer on its own: the Swiss AHV is a **pay-as-you-go
//! intergenerational contract** -- "today's contributions fund today's retirees,
//! not your own future self" -- and "a population that collectively chooses lower
//! work percentages changes the ratio this system depends on." The tool "only ever
//! answers 'what's best for you'; it has no voice for 'what happens if everyone
//! does this'."
//!
//! This module gives it one. The coupling is not a hard link to real pension
//! finance -- it is a documented, illustrative transmission channel, and the
//! parameters are invented like everything else here. What it tests is a
//! *structural* claim rather than a quantitative forecast: that the cooperation
//! level of the international system and the security of an intergenerational
//! pension promise are connected through identifiable channels, and that a
//! Pareto-inferior equilibrium in the first degrades the second.
//!
//! # The two channels
//!
//! 1. **Fiscal.** `MULTIPOLAR_GAME.md` section 8 cites Paul Kennedy's "imperial
//!    overstretch": guardian-class (defence) spending grows to outpace the
//!    producer-class base sustaining it. Competitiveness is what generates that
//!    spending, so it crowds out the transfers a PAYG system needs.
//! 2. **Contributory.** The same chapter's section 7 notes that AI-driven
//!    disintermediation can compress employment. A SYSTEM in which individuals
//!    rationally choose lower work percentages reduces the contributor-to-retiree
//!    ratio that PAYG solvency depends on -- the collective-action problem of
//!    `THEORY_OF_SPARING.md` section 7d, applied to pensions.
//!
//! Both channels are worsened by competition and eased by cooperation. That is
//! the entire mechanism, stated plainly so it can be argued with.

/// Parameters for the pension transmission channel.
///
/// **Illustrative.** These are not Swiss pension figures. `AHV` replacement rates,
/// contribution rates and demographic ratios are real, published quantities; what
/// is invented here is the *elasticity* connecting geopolitical competition to
/// them, which is precisely the thing that cannot be measured from a 50-year
/// counterfactual. Treat the outputs as directional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PensionLink {
    /// Replacement rate a fully cooperative world sustains.
    pub base_replacement_rate: f64,
    /// How many percentage points of replacement rate are lost per unit of
    /// competition intensity (0 cooperative, 1 fully competitive).
    pub fiscal_crowding_per_competition: f64,
    /// How much the contributor-to-retiree ratio is degraded per unit of
    /// competition, as a fraction.
    pub ratio_sensitivity: f64,
    /// The contributor-to-retiree ratio at which the system pays the base rate.
    pub reference_ratio: f64,
    /// Portfolio return drag from fragmentation. Decoupling, sanctions and broken
    /// supply chains reduce the return on a globally diversified pension portfolio
    /// even if the household's own behaviour is unchanged.
    pub fragmentation_return_drag: f64,
    /// Annual real return in a fully cooperative world.
    pub cooperative_return: f64,
}

impl Default for PensionLink {
    fn default() -> Self {
        PensionLink {
            base_replacement_rate: 0.34,
            fiscal_crowding_per_competition: 0.09,
            ratio_sensitivity: 0.45,
            reference_ratio: 2.0,
            fragmentation_return_drag: 0.018,
            cooperative_return: 0.030,
        }
    }
}

/// The pension outcome implied by one simulated world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PensionOutcome {
    /// Mean fraction of pair-years resolved cooperatively, in [0, 1].
    pub cooperation_index: f64,
    /// Mean efficiency loss across pair-years, in [0, 1].
    pub mean_efficiency_loss: f64,
    /// PAYG replacement rate as a fraction of pre-retirement income.
    pub replacement_rate: f64,
    /// Contributor-to-retiree ratio this world supports.
    pub support_ratio: f64,
    /// Annual real return on the pension portfolio.
    pub portfolio_return: f64,
}

impl PensionOutcome {
    /// Expected pension income from the PAYG pillar alone.
    ///
    /// This is the quantity `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` section 2c
    /// says the Life Optimizer cannot supply: not "what should *you* do", but
    /// "what will this promise be worth if the system around you evolves this
    /// way".
    pub fn payg_income(&self, pre_retirement_income: f64) -> f64 {
        pre_retirement_income * self.replacement_rate
    }

    /// A single 0-1 index of pension security, for comparing worlds.
    ///
    /// Combines the three compressions multiplicatively, because they compound:
    /// a lower replacement rate is worse when the support ratio is also thin and
    /// portfolio returns are also dragged. Deliberately simple and documented
    /// rather than a fitted score -- the point is ordering, not precision.
    pub fn security_index(&self, reference: &PensionOutcome) -> f64 {
        if reference.replacement_rate <= 0.0 {
            return 0.0;
        }
        let rate = (self.replacement_rate / reference.replacement_rate).clamp(0.0, 2.0);
        let ratio = if reference.support_ratio > 0.0 {
            (self.support_ratio / reference.support_ratio).clamp(0.0, 2.0)
        } else {
            1.0
        };
        let retention = (1.0 + self.portfolio_return) / (1.0 + reference.portfolio_return);
        (rate * ratio * retention).clamp(0.0, 2.0)
    }
}

impl PensionLink {
    /// Translate a world's cooperation statistics into a pension outcome.
    ///
    /// `cooperation_index` is the mean probability of cooperation across all
    /// dyads and years; `mean_efficiency_loss` is the mean wasted surplus. The two
    /// are separate inputs because they measure different things: a world can
    /// cooperate often and still waste surplus in the dyads where it does not,
    /// and the fiscal channel responds to both the intensity and the frequency of
    /// competition.
    pub fn assess(&self, cooperation_index: f64, mean_efficiency_loss: f64) -> PensionOutcome {
        let cooperation = cooperation_index.clamp(0.0, 1.0);
        let competition = 1.0 - cooperation;

        // Channel 1: fiscal crowding. Competition is what drives the guardian
        // spending Kennedy describes, and it is paid for out of the same budget as
        // transfers.
        let replacement_rate =
            (self.base_replacement_rate - self.fiscal_crowding_per_competition * competition)
                .max(0.0);

        // Channel 2: the contributor-to-retiree ratio. Degraded by competition,
        // which is the collective-action channel: lower work percentages and
        // compressed employment both shrink the contribution base.
        let support_ratio =
            (self.reference_ratio * (1.0 - self.ratio_sensitivity * competition)).max(0.0);

        // Portfolio channel: fragmentation drags returns even for a household whose
        // own behaviour is unchanged.
        let portfolio_return = self.cooperative_return - self.fragmentation_return_drag * competition;

        PensionOutcome {
            cooperation_index: cooperation,
            mean_efficiency_loss,
            replacement_rate,
            support_ratio,
            portfolio_return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monotonicity is the whole claim of this module: more cooperation must mean
    /// more pension security on every channel. If any channel ran the other way the
    /// narrative would be incoherent.
    #[test]
    fn cooperation_improves_every_pension_channel() {
        let link = PensionLink::default();
        let cooperative = link.assess(1.0, 0.0);
        let middling = link.assess(0.5, 0.1);
        let competitive = link.assess(0.0, 0.3);

        assert!(
            cooperative.replacement_rate > middling.replacement_rate
                && middling.replacement_rate > competitive.replacement_rate,
            "replacement rate must fall as competition rises: {} > {} > {}",
            cooperative.replacement_rate,
            middling.replacement_rate,
            competitive.replacement_rate
        );
        assert!(
            cooperative.support_ratio > competitive.support_ratio,
            "support ratio must fall as competition rises"
        );
        assert!(
            cooperative.portfolio_return > competitive.portfolio_return,
            "portfolio return must fall as competition rises"
        );
    }

    /// The security index must order worlds correctly and be 1.0 against itself.
    #[test]
    fn security_index_orders_worlds_and_is_normalised() {
        let link = PensionLink::default();
        let cooperative = link.assess(1.0, 0.0);
        let competitive = link.assess(0.0, 0.3);

        assert!(
            (cooperative.security_index(&cooperative) - 1.0).abs() < 1e-12,
            "a world must score 1.0 against itself"
        );
        assert!(
            competitive.security_index(&cooperative) < 1.0,
            "a competitive world must score below a cooperative one"
        );
        assert!(
            competitive.security_index(&cooperative) > 0.0,
            "and must stay positive, since all channels are floored"
        );
    }

    /// Degenerate inputs must not produce NaN, infinities, or negative pensions.
    #[test]
    fn degenerate_inputs_are_handled() {
        let link = PensionLink::default();
        for (coop, loss) in [(0.0, 0.0), (1.0, 1.0), (-1.0, 0.0), (2.0, 0.0), (0.5, -1.0)] {
            let outcome = link.assess(coop, loss);
            assert!(
                outcome.replacement_rate.is_finite() && outcome.replacement_rate >= 0.0,
                "coop {coop}: replacement {} is invalid",
                outcome.replacement_rate
            );
            assert!(outcome.support_ratio.is_finite() && outcome.support_ratio >= 0.0);
            assert!(outcome.portfolio_return.is_finite());
            assert!(outcome.cooperation_index >= 0.0 && outcome.cooperation_index <= 1.0);
        }

        // A zero base rate must not divide by zero in the index.
        let broken = PensionLink {
            base_replacement_rate: 0.0,
            ..PensionLink::default()
        };
        let zero = broken.assess(0.5, 0.1);
        assert!(zero.security_index(&zero).is_finite());
    }

    /// The PAYG income helper must scale the replacement rate correctly -- it is
    /// the number the Life Optimizer would consume.
    #[test]
    fn payg_income_applies_the_replacement_rate() {
        let link = PensionLink::default();
        let outcome = link.assess(1.0, 0.0);
        let income = 100_000.0;
        assert!(
            (outcome.payg_income(income) - income * outcome.replacement_rate).abs() < 1e-9
        );
        // A competitive world must promise less.
        let competitive = link.assess(0.0, 0.3);
        assert!(competitive.payg_income(income) < outcome.payg_income(income));
    }
}
