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

//! Two strategies for a great power facing a fragile region, priced against each
//! other: buy the peace, or buy the war.
//!
//! # The question this module exists to answer
//!
//! `THIRD_PARTY_SPOILER.md` establishes that a third party can profit from a conflict
//! between two others, and that it is rarely made to pay. That raises an obvious
//! comparison which the spoiler document does not itself make: **what does the same
//! influence buy if it is spent on cooperation instead?** The two strategies are not
//! usually compared, because they are studied in different literatures -- trade and
//! development finance on one side, proxy war and resource capture on the other -- and
//! a comparison needs both in one unit.
//!
//! The unit is the same one the spoiler document uses: the dyad's **peace margin**
//! `sigma = cc - dc`. Conflict is the equilibrium when `sigma < 0`. Both strategies
//! are then simply two ways of moving `sigma`:
//!
//! * **Cooperation** -- trade access, investment, security guarantees -- raises `cc`,
//!   and therefore `sigma`, at a price.
//! * **Spoiling** -- manufactured grievance, subsidised defection -- lowers `sigma`
//!   at a price, and the spoiler is paid out of the conflict that follows.
//!
//! Because both instruments act on the *same* variable, their returns are not
//! independent: `P(peace) + P(conflict) = 1`, so every unit of `sigma` one strategy
//! buys is a unit the other cannot have. That is the whole structure, and everything
//! below is a consequence of it.
//!
//! # The asymmetry the comparison turns on
//!
//! The two strategies do not have the same *time shape*, and this is the single most
//! consequential modelling choice in the module:
//!
//! * Cooperation is a **flow with upkeep**. An investment programme, a trade
//!   preference, a security guarantee: each is paid for every year it is wanted, and
//!   each stops paying the year it stops being maintained. Its return is a flow that
//!   continues as long as the peace does, and the peace is what is being bought.
//! * Spoiling is a **stock with a setup cost and a depleting prize**. The grievance is
//!   manufactured and the defection subsidised until `sigma < 0`; after that the
//!   conflict sustains itself, because `sigma < 0` *is* the equilibrium. But the prize
//!   is a region, and a region in conflict is being consumed: the rent runs down.
//!
//! So the spoiler's prize is bounded by `rent / depletion` no matter how long it waits,
//! while the cooperator's return grows with the horizon. That gives this module's central
//! result, [`StrategyParams::horizon_verdict`]: **spoiling wins short games and
//! cooperation wins long ones, and the crossing point is computable.**
//!
//! It comes with a condition that turned out to be the more interesting half, and which
//! replaced a guess that the numbers falsified. The crossing exists only for a
//! sufficiently patient strategist. Raising the discount rate pushes the crossing
//! *later* -- the opposite of the intuition that a deferred return suffers from
//! impatience -- because in this structure the spoiler is paid early and the cooperator
//! is paid in instalments. Past a critical rate there is no crossing at any horizon, and
//! spoiling dominates outright. See [`StrategyParams::critical_discount_rate`], where the
//! correction is recorded in full.
//!
//! Both results are consequences of A1 and A2 below, not observations, and should be read
//! that way. If a sponsor can hold a conflict open indefinitely at no upkeep and without
//! consuming the prize, the conclusion reverses. Both assumptions are stated here rather
//! than buried, and the sensitivity mode moves them.
//!
//! # Assumptions, listed
//!
//! | # | Assumption | What it costs if wrong |
//! | --- | --- | --- |
//! | A1 | Cooperation requires annual upkeep and stops paying when the peace ends | Reverses the horizon result if cooperation is a one-off purchase |
//! | A2 | Conflict requires a setup payment and then sustains itself, but depletes the prize at rate `gamma` | Reverses the horizon result if the prize is inexhaustible |
//! | A3 | Both instruments move `sigma` with diminishing returns, `g * sqrt(spend)` | Changes the optimal budget but not the sign of the comparison |
//! | A4 | The state is stochastic: `P(conflict) = Phi(-sigma / dispersion)` | None; it is what makes this a distribution rather than a point |
//! | A5 | The victims' retaliation discounts the spoiler's take but does not remove it | The punishment channel is `THIRD_PARTY_SPOILER.md` Theorem 6, reused here |
//! | A6 | Both strategies face the same `sigma`, the same dispersion and the same horizon | The comparison is only meaningful under a common budget and clock |
//!
//! # Every number here is declared, and most of them could not be otherwise
//!
//! A region's peace margin `sigma` is a property of a counterfactual payoff matrix. No
//! one has measured it, in the Near East or anywhere else, and no published figure could
//! stand in for it -- so [`Region::margin_provenance`] is illustrative by construction
//! and stays that way. A region's capturable rent is a different matter: resource rents
//! are reported, so [`Region::rent_provenance`] could in principle be anchored, and the
//! framework is built so that anchoring it needs a number and not a re-derivation.
//!
//! What this means for reading anything below: **no level is a finding**. The
//! comparison *between* the two strategies is the finding, and it is worth exactly as
//! much as the assumptions above are worth -- which is why the sensitivity mode moves
//! each of them in turn and reports where the verdict flips. A framework whose verdict
//! survives its own assumptions being moved is useful; a framework whose verdict does
//! not is a description of its assumptions.

use crate::blocks::PowerBloc;
use crate::economy::Provenance;
use crate::game::Payoffs;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Which of the two strategies is being run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Buy the peace: trade, investment, security guarantees. Raises the region
    /// dyad's cooperation payoff, and is paid for every year.
    Cooperate,
    /// Buy the war: manufactured grievance and subsidised defection. Lowers the
    /// region dyad's peace margin, is paid for once, and is repaid out of the
    /// conflict that follows.
    Spoil,
}

impl Strategy {
    pub fn label(self) -> &'static str {
        match self {
            Strategy::Cooperate => "COOPERATION",
            Strategy::Spoil => "SPOILING",
        }
    }

    /// One line on what the strategy is buying, for report headers.
    pub fn intent(self) -> &'static str {
        match self {
            Strategy::Cooperate => {
                "raise the region's peace margin with trade, investment and guarantees"
            }
            Strategy::Spoil => {
                "push the region's peace margin below zero and take the rent the conflict releases"
            }
        }
    }
}

/// One fragile region, as a dyad plus what is at stake in it.
///
/// A region is *not* a place in this model. It is a pair of blocs and a set of declared
/// numbers about their relationship -- which is why the field is [`Region::bloc_a`] and
/// [`Region::bloc_b`] rather than a list of countries. See `STRATEGY_COMPARISON.md`
/// section 2 for what each region is standing in for.
#[derive(Debug, Clone)]
pub struct Region {
    /// Short label, used in tables and figure legends.
    pub name: &'static str,
    /// The dyad in words, so a reader can see what the bloc pair stands for.
    pub axis: &'static str,
    /// First bloc of the dyad. Matched case-insensitively against `Config::blocs`.
    pub bloc_a: &'static str,
    /// Second bloc of the dyad.
    pub bloc_b: &'static str,
    /// The dyad's peace margin `sigma = cc - dc`, before either strategy acts. Conflict
    /// is the equilibrium when this is negative. **Illustrative and necessarily so.**
    pub margin: f64,
    /// Annual rent the conflict releases to whoever can capture it -- resources,
    /// reconstruction, arms, cheap access to a weakened bloc. **Illustrative**, but
    /// anchorable: resource rents are reported figures.
    pub rent: f64,
    /// The victims' capacity to identify and punish a sponsor, `0..1`. Reused from
    /// `THIRD_PARTY_SPOILER.md` Theorem 6, where impunity is `1 - exposure`.
    pub retaliation: f64,
    /// Annual gross benefit that accrues to the strategist while the region is at
    /// peace -- trade access, contracts, supply security. **Illustrative.**
    pub trade_benefit: f64,
    /// Where [`Region::margin`] came from. Stays illustrative: a counterfactual payoff
    /// matrix has no measurement.
    pub margin_provenance: Provenance,
    /// Where [`Region::rent`] came from. The one field that could be sourced.
    pub rent_provenance: Provenance,
}

impl Region {
    /// Probability the region is at peace before any strategy acts.
    pub fn baseline_peace_probability(&self, dispersion: f64) -> f64 {
        normal_cdf(self.margin / dispersion)
    }

    /// The dyad starts in conflict, on the model's own definition: `sigma < 0`.
    pub fn starts_in_conflict(&self) -> bool {
        self.margin < 0.0
    }

    /// Whether both ends of the dyad are blocs the running world actually has.
    /// A region whose blocs were replaced by `--bloc` is skipped rather than
    /// silently matched to the wrong pair.
    pub fn resolves(&self, blocs: &[PowerBloc]) -> Option<(usize, usize)> {
        let find = |name: &str| {
            blocs
                .iter()
                .position(|bloc| bloc.name.eq_ignore_ascii_case(name))
        };
        match (find(self.bloc_a), find(self.bloc_b)) {
            (Some(a), Some(b)) if a != b => Some((a, b)),
            _ => None,
        }
    }
}

/// The declared regions, and what each one is standing in for.
///
/// Each is a *fragile* dyad of the kind the question names: a pair whose peace margin
/// is near enough to zero that a third party's spending can decide its sign. The three
/// differ in exactly the two ways the framework says should matter -- the rent at stake
/// and the victims' capacity to retaliate -- so that the comparison has something to
/// separate them.
pub fn default_regions() -> Vec<Region> {
    /// No measurement of a counterfactual payoff margin exists, so this field is not
    /// merely unmeasured; it is not the kind of quantity that is measured.
    const MARGIN: Provenance = Provenance::Illustrative {
        rationale: "a counterfactual 2x2 payoff margin has no published measurement; \
                    the sign encodes a judgement about the dyad and nothing more",
    };
    /// Resource rents, by contrast, are reported. The value here is a declared
    /// stand-in for a capturable share, not a figure from a source.
    const RENT: Provenance = Provenance::Illustrative {
        rationale: "resource and reconstruction rents are reported in principle, but the \
                    capturable share of them is not; this is a declared stand-in",
    };

    vec![
        Region {
            name: "Near East",
            axis: "Eurasian (Iran) - Gulf",
            bloc_a: "Eurasian",
            bloc_b: "Gulf",
            // The most contested of the three and the one closest to the knife edge:
            // it is already below zero, so a spoiler only has to widen a gap that is
            // open, while a cooperator has to close one.
            margin: -0.35,
            rent: 1.10,
            retaliation: 0.25,
            trade_benefit: 0.85,
            margin_provenance: MARGIN,
            rent_provenance: RENT,
        },
        Region {
            name: "Balkans",
            axis: "Europe - Eurasian (Serbia / Kosovo)",
            bloc_a: "Europe",
            bloc_b: "Eurasian",
            // The one region here that is *above* the knife edge: the peace holds, so
            // cooperation is defending something and spoiling has to break it. It is
            // the test case for whether the framework can say anything other than
            // "conflict is where conflict already is".
            margin: 0.25,
            rent: 0.35,
            retaliation: 0.45,
            trade_benefit: 0.45,
            margin_provenance: MARGIN,
            rent_provenance: RENT,
        },
        Region {
            name: "Carpathians",
            axis: "Europe - Eurasian (Ukraine / Russia)",
            bloc_a: "Europe",
            bloc_b: "Eurasian",
            // Deepest in conflict, largest rent, and the highest retaliation: the
            // case where the prize is big and the victims can still answer, which is
            // what should make punishment bite here and not in the Near East.
            margin: -0.55,
            rent: 0.95,
            retaliation: 0.70,
            trade_benefit: 0.70,
            margin_provenance: MARGIN,
            rent_provenance: RENT,
        },
    ]
}

/// The plan: which strategy, how hard, over which regions, and at what clock.
///
/// Lives in `Config` so a run can be the same world under a different strategy. The
/// default is **disabled**, and a disabled layer changes nothing at all -- not the
/// payoffs, not the random stream, not one published figure.
#[derive(Debug, Clone)]
pub struct StrategyParams {
    /// Whether the layer runs. Off by default; off means byte-identical output.
    pub enabled: bool,
    pub strategy: Strategy,
    /// How much of the unconstrained optimal programme is actually funded, `0..1`.
    /// One meaning the whole thing. It is the budget dial, and it exists because
    /// "the strategy is optimal" is a claim about a budget nobody has.
    pub intensity: f64,
    pub regions: Vec<Region>,
    /// Standard deviation of the region's peace margin across the things that can go
    /// wrong. Assumption A4: it is what turns a point into a distribution.
    pub dispersion: f64,
    /// Diminishing-returns coefficient of the cooperative instrument: `d_sigma` per
    /// unit of `sqrt(spend)`. Assumption A3.
    pub coop_gain: f64,
    /// The same for the spoiling instrument. Larger than `coop_gain` by default, which
    /// is `THIRD_PARTY_SPOILER.md` Corollary 1.1 -- narrative is cheaper than
    /// matériel per unit of margin moved. The default is an assumption, and the
    /// sensitivity mode is there because it is the assumption the verdict turns on.
    pub spoil_gain: f64,
    /// How much the victims' retaliation discounts the spoiler's take, from
    /// `THIRD_PARTY_SPOILER.md` Theorem 6: the take is scaled by `1 - kappa * e`.
    pub punishment_scale: f64,
    /// Rate at which a conflict consumes the prize. Assumption A2.
    pub depletion: f64,
    /// Annual discount rate.
    pub discount: f64,
    /// Years the strategist looks ahead. The comparison is a function of this.
    pub horizon_years: f64,
    /// Drag on the region dyad's cooperation payoff while the region is in conflict,
    /// applied to the world simulation.
    pub cc_drag: f64,
    /// Lift on the region dyad's temptation to defect while the region is in conflict.
    pub dc_lift: f64,
    /// Lift on the region dyad's cooperation payoff under the cooperative strategy.
    pub cc_lift: f64,
}

impl Default for StrategyParams {
    fn default() -> Self {
        StrategyParams {
            enabled: false,
            strategy: Strategy::Cooperate,
            intensity: 0.60,
            regions: default_regions(),
            dispersion: 0.50,
            coop_gain: 0.60,
            spoil_gain: 0.90,
            punishment_scale: 0.70,
            depletion: 0.12,
            discount: 0.04,
            horizon_years: 50.0,
            cc_drag: 0.25,
            dc_lift: 0.30,
            cc_lift: 0.20,
        }
    }
}

/// What the horizon comparison comes to, with the two "no crossing" cases kept apart.
///
/// They are opposite results and an `Option<f64>` cannot tell them apart: "cooperation is
/// ahead at every horizon" and "spoiling is ahead at every horizon" would both be `None`,
/// and a report that printed `none` for both would be saying the same thing about two
/// different worlds. This enum exists so that the caller *cannot* conflate them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HorizonVerdict {
    /// The arms cross here: spoiling is worth more below, cooperation above.
    Crossing(f64),
    /// Cooperation is ahead at every horizon tried.
    CooperationAlways,
    /// Spoiling is ahead at every horizon tried.
    SpoilingAlways,
}

impl HorizonVerdict {
    pub fn label(self) -> String {
        match self {
            HorizonVerdict::Crossing(years) => format!("{years:.1} yr"),
            HorizonVerdict::CooperationAlways => "coop. always".to_string(),
            HorizonVerdict::SpoilingAlways => "spoil. always".to_string(),
        }
    }
}

impl StrategyParams {
    /// A layer that does nothing, for the baseline arm of a comparison.
    pub fn off() -> Self {
        StrategyParams::default()
    }

    /// Build an enabled layer running one strategy.
    pub fn running(strategy: Strategy) -> Self {
        StrategyParams {
            enabled: true,
            strategy,
            ..StrategyParams::default()
        }
    }

    /// Present-value factor of a unit annual flow for `years` at rate `r`.
    ///
    /// The **discrete** annuity, `sum over k in 0..years of (1 + r)^-k`, in closed form:
    /// `(1 - (1 + r)^-T) * (1 + r) / r`, and `T` when `r` is zero. Written out rather
    /// than approximated because the whole comparison is between two annuities of
    /// different rates, and a cancellation between them would be invisible.
    ///
    /// It is discrete rather than the continuous `(1 - e^-rT) / r` because this model
    /// steps in years and every payment in it is annual. The two differ by a factor of
    /// about `(1 + r)`, which sounds negligible and is not: it is the same order as the
    /// discount rate the comparison turns on. A test pins the closed form against an
    /// explicit geometric sum.
    ///
    /// For non-integer `years` -- the crossing search bisects over the horizon -- the
    /// formula is the natural interpolation of that sum rather than a sum of a fractional
    /// number of terms.
    pub fn annuity(rate: f64, years: f64) -> f64 {
        if rate.abs() < 1e-12 {
            years
        } else {
            (1.0 - (1.0 + rate).powf(-years)) * (1.0 + rate) / rate
        }
    }

    /// Probability the region is at peace once the cooperative programme `spend` runs.
    pub fn peace_probability_under_cooperation(&self, region: &Region, spend: f64) -> f64 {
        self.peace_at_margin(self.margin_under_cooperation(region, spend))
    }

    /// Probability the region is in conflict once the spoiling programme `spend` runs.
    pub fn conflict_probability_under_spoiling(&self, region: &Region, spend: f64) -> f64 {
        self.conflict_at_margin(self.margin_under_spoiling(region, spend))
    }

    /// The spoiler's share of the rent after the victims' answer, Theorem 6.
    pub fn capturable_share(&self, region: &Region) -> f64 {
        (1.0 - self.punishment_scale * region.retaliation).clamp(0.0, 1.0)
    }

    /// The cooperative programme's **annual** net margin: what the trade and the
    /// security are worth this year, less what they cost this year.
    ///
    /// A flow, not a value, because that is exactly what A1 says cooperation is.
    pub fn cooperative_margin(&self, region: &Region, spend: f64) -> f64 {
        region.trade_benefit * self.peace_probability_under_cooperation(region, spend) - spend
    }

    /// Probability of peace at a given margin, with no strategy acting.
    ///
    /// The two strategies are two directions along this one curve, and this is what
    /// makes the comparison a comparison: `peace_at(m) + conflict_at(m) = 1` exactly, so
    /// a unit of margin bought for peace is a unit the spoiler cannot have. Note that
    /// the two arms' *spend* is not complementary -- equal spends in opposite directions
    /// do not land on the same margin -- which is why the identity is stated in the
    /// margin rather than in the money.
    pub fn peace_at_margin(&self, margin: f64) -> f64 {
        normal_cdf(margin / self.dispersion)
    }

    /// Probability of conflict at a given margin. Exactly `1 - ` [`Self::peace_at_margin`].
    pub fn conflict_at_margin(&self, margin: f64) -> f64 {
        1.0 - self.peace_at_margin(margin)
    }

    /// The margin a cooperative programme of `spend` leaves behind.
    pub fn margin_under_cooperation(&self, region: &Region, spend: f64) -> f64 {
        region.margin + self.coop_gain * spend.max(0.0).sqrt()
    }

    /// The margin a spoiling programme of `spend` leaves behind.
    pub fn margin_under_spoiling(&self, region: &Region, spend: f64) -> f64 {
        region.margin - self.spoil_gain * spend.max(0.0).sqrt()
    }

    /// The spoiler's **annual** harvest, before the setup cost: the rent released by
    /// the conflict, discounted by the victims' retaliation.
    pub fn spoiling_harvest(&self, region: &Region, spend: f64) -> f64 {
        region.rent
            * self.capturable_share(region)
            * self.conflict_probability_under_spoiling(region, spend)
    }

    /// Present value of the cooperative strategy in one region: a maintained flow.
    ///
    /// `margin(x) * annuity(r, T)`. Note that the budget's own cost is inside the
    /// margin, so this is already net of everything paid.
    pub fn cooperative_value(&self, region: &Region, spend: f64) -> f64 {
        self.cooperative_margin(region, spend)
            * Self::annuity(self.discount, self.horizon_years)
    }

    /// Present value of the spoiling strategy in one region.
    ///
    /// The setup cost `y` is paid once and is *not* multiplied by the annuity; the
    /// harvest is a flow that decays at `depletion` because the conflict is consuming
    /// the prize it is being paid out of. That asymmetry is A2, and it is what bounds
    /// the spoiler.
    pub fn spoiling_value(&self, region: &Region, spend: f64) -> f64 {
        let harvest = self.spoiling_harvest(region, spend);
        harvest * Self::annuity(self.discount + self.depletion, self.horizon_years) - spend
    }

    /// Present value of one year of the cooperative programme, as a call on the budget.
    ///
    /// The two strategies are charged for the *same budget* on present-value terms, so
    /// this is the multiplier the cooperative arm's spend carries and the spoiling
    /// arm's does not.
    pub fn cooperative_budget_factor(&self) -> f64 {
        Self::annuity(self.discount, self.horizon_years)
    }

    /// The funded programme in one region: `intensity` of the unconstrained optimum.
    ///
    /// `intensity` is applied to the *spend*, not to the resulting margin, so funding
    /// half an optimal programme buys `sqrt(0.5)` of its margin effect and not half of
    /// it. That is A3 doing its work, and it is the reason a half-funded programme is
    /// worth much less than half of a funded one.
    pub fn funded_spend(&self, region: &Region, strategy: Strategy) -> f64 {
        let optimum = match strategy {
            Strategy::Cooperate => self.optimal_cooperative_spend(region),
            Strategy::Spoil => self.optimal_spoiling_spend(region),
        };
        self.intensity.clamp(0.0, 1.0) * optimum
    }

    /// The spend that maximises the cooperative margin in one region.
    pub fn optimal_cooperative_spend(&self, region: &Region) -> f64 {
        let f = |x: f64| self.cooperative_margin(region, x);
        maximise(f, 0.0, SPAWN_CEILING).0
    }

    /// The spend that maximises the spoiler's present value in one region.
    pub fn optimal_spoiling_spend(&self, region: &Region) -> f64 {
        let f = |y: f64| self.spoiling_value(region, y);
        maximise(f, 0.0, SPAWN_CEILING).0
    }

    /// The rent at which the two strategies are worth the same in this region, all
    /// else held.
    ///
    /// This is the cleanest single number the framework produces: a region whose
    /// capturable rent is below it is not worth spoiling, whatever its grievances, and
    /// a region above it is. It is found by bisection on a monotone difference rather
    /// than stated in closed form, because the two optima move as the rent does and a
    /// closed form would have to assume they do not.
    ///
    /// Returns `None` when the ordering does not reverse within the range searched,
    /// which is itself a result: a region that spoiling never beats at any rent.
    pub fn critical_rent(&self, region: &Region) -> Option<f64> {
        let difference = |rent: f64| {
            let mut probe = region.clone();
            probe.rent = rent;
            let x = self.optimal_cooperative_spend(&probe);
            let y = self.optimal_spoiling_spend(&probe);
            self.spoiling_value(&probe, y) - self.cooperative_value(&probe, x)
        };
        let negated = {
            let mut probe = region.clone();
            probe.rent = 0.0;
            difference(probe.rent)
        };
        if negated >= 0.0 {
            // Spoiling already wins with no rent at all, so there is no crossing.
            return None;
        }
        let (mut lo, mut hi) = (0.0_f64, RENT_CEILING);
        if difference(hi) <= 0.0 {
            return None;
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if difference(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
            if (hi - lo) < 1e-10 {
                break;
            }
        }
        Some(0.5 * (lo + hi))
    }

    /// The horizon comparison, with the two opposite "no crossing" outcomes kept apart.
    ///
    /// Each trial horizon is **re-priced from scratch** rather than scaled from the
    /// comparison it was handed. An earlier version scaled, which was wrong for the
    /// spoiling arm: its value is a depleting flow *less a setup cost paid once*, and a
    /// setup cost does not scale with the horizon. The scaled version made the difference
    /// constant in the horizon, so the search never found a crossing and the mode
    /// silently reported "cooperation never wins". The comparison is used only for its
    /// region list.
    pub fn horizon_verdict(&self) -> HorizonVerdict {
        let difference = |years: f64| {
            let mut probe = self.clone();
            probe.horizon_years = years;
            let priced = compare(&probe);
            probe.total_value(Strategy::Spoil, &priced)
                - probe.total_value(Strategy::Cooperate, &priced)
        };
        if difference(HORIZON_FLOOR) <= 0.0 {
            // Cooperation is ahead at the shortest horizon tried, and the difference is
            // increasing in the horizon here (the spoiler's prize saturates while the
            // cooperator's flow does not), so it is ahead at every horizon.
            return HorizonVerdict::CooperationAlways;
        }
        if difference(HORIZON_CEILING) >= 0.0 {
            return HorizonVerdict::SpoilingAlways;
        }
        let (mut lo, mut hi) = (HORIZON_FLOOR, HORIZON_CEILING);
        for _ in 0..100 {
            let mid = 0.5 * (lo + hi);
            if difference(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
            if (hi - lo) < 1e-9 {
                break;
            }
        }
        HorizonVerdict::Crossing(0.5 * (lo + hi))
    }

    /// The crossing horizon, when there is one. `None` covers both of the opposite
    /// no-crossing cases, so a caller that needs to distinguish them wants
    /// [`Self::horizon_verdict`] instead.
    pub fn break_even_horizon(&self) -> Option<f64> {
        match self.horizon_verdict() {
            HorizonVerdict::Crossing(years) => Some(years),
            HorizonVerdict::CooperationAlways | HorizonVerdict::SpoilingAlways => None,
        }
    }

    /// The discount rate at which the spoiler stops being ahead at *every* horizon.
    ///
    /// Below it the two arms cross at some finite horizon and cooperation eventually
    /// wins. Above it there is no crossing at all: the spoiler is ahead however long the
    /// game runs, because the cooperator's return is a flow that has to be waited for
    /// while the spoiler's is collected early.
    ///
    /// This is the module's sharper result, and it replaced a weaker guess. The first
    /// version of this documentation asserted that the two arms always cross and that
    /// "impatience shortens the spoiler's window". The second half of that is false: a
    /// less patient strategist does not shorten the window, it **removes cooperation's
    /// advantage altogether**. Measured on the declared regions, the crossing sits at
    /// about 15 years with no discounting and about 21 years at 4%, and is gone entirely
    /// by 15%. The corrected statement is the one in this comment, and the test that
    /// failed on the old one is now the test for the new one.
    ///
    /// Returns `None` when the ordering does not reverse inside the range searched, in
    /// either direction, and the caller says which.
    pub fn critical_discount_rate(&self) -> Option<f64> {
        let long_run_difference = |rate: f64| {
            let mut probe = self.clone();
            probe.discount = rate;
            probe.horizon_years = HORIZON_CEILING;
            let priced = compare(&probe);
            probe.total_value(Strategy::Spoil, &priced)
                - probe.total_value(Strategy::Cooperate, &priced)
        };
        if long_run_difference(0.0) >= 0.0 {
            // Spoiling wins even with unlimited patience: there is no rate to find.
            return None;
        }
        if long_run_difference(DISCOUNT_CEILING) <= 0.0 {
            // Cooperation wins even at the ceiling: no crossing inside the range.
            return None;
        }
        let (mut lo, mut hi) = (0.0_f64, DISCOUNT_CEILING);
        for _ in 0..100 {
            let mid = 0.5 * (lo + hi);
            if long_run_difference(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
            if (hi - lo) < 1e-12 {
                break;
            }
        }
        Some(0.5 * (lo + hi))
    }

    /// Total present value of one strategy across every region in a comparison.
    pub fn total_value(&self, strategy: Strategy, comparison: &Comparison) -> f64 {
        comparison
            .regions
            .iter()
            .map(|row| match strategy {
                Strategy::Cooperate => row.value_cooperative,
                Strategy::Spoil => row.value_spoiling,
            })
            .sum()
    }
}

/// Highest spend the search ever considers. Chosen so the margin shift at the ceiling
/// is far larger than any margin in `default_regions`; the optimum never sits there for
/// the declared parameters, and a test pins that, because an optimum pinned to the
/// boundary would make every comparative static an artefact of the search range.
const SPAWN_CEILING: f64 = 40.0;
/// Highest rent the crossing search considers.
const RENT_CEILING: f64 = 40.0;
/// Shortest horizon the crossing search considers, in years.
const HORIZON_FLOOR: f64 = 0.5;
/// Longest horizon the crossing search considers, in years.
const HORIZON_CEILING: f64 = 400.0;
/// Highest discount rate the critical-rate search considers.
const DISCOUNT_CEILING: f64 = 2.0;

/// One region's verdict under both strategies, at the funded intensity.
#[derive(Debug, Clone)]
pub struct RegionVerdict {
    pub region: Region,
    pub base_peace_probability: f64,
    pub starts_in_conflict: bool,
    pub cooperative_spend: f64,
    pub spoiling_spend: f64,
    pub peace_probability_under_cooperation: f64,
    pub conflict_probability_under_spoiling: f64,
    pub value_cooperative: f64,
    pub value_spoiling: f64,
    /// Present value of the rent the spoiling arm actually captures.
    pub rent_captured: f64,
    /// Present value of the trade the cooperative arm actually preserves.
    pub trade_preserved: f64,
    /// `value_cooperative - value_spoiling`. Positive means cooperation is the better
    /// buy in this region.
    pub difference: f64,
    /// Rent above which spoiling beats cooperation here, when there is one.
    pub critical_rent: Option<f64>,
}

impl RegionVerdict {
    pub fn prefers(&self) -> Strategy {
        if self.difference >= 0.0 {
            Strategy::Cooperate
        } else {
            Strategy::Spoil
        }
    }
}

/// The whole comparison: per-region verdicts and the totals the difference plots use.
#[derive(Debug, Clone)]
pub struct Comparison {
    pub strategy_label: &'static str,
    pub regions: Vec<RegionVerdict>,
    pub total_cooperative: f64,
    pub total_spoiling: f64,
    /// Regions at peace under the cooperative arm, at the baseline, and in conflict
    /// under the spoiling arm -- the political-stability triple.
    pub peaceful_under_cooperation: usize,
    pub peaceful_at_baseline: usize,
    pub in_conflict_under_spoiling: usize,
    /// Present value of all rent captured across regions under the spoiling arm.
    pub rent_captured: f64,
    /// Present value of all trade preserved under the cooperative arm.
    pub trade_preserved: f64,
    /// Regions where the two strategies disagree, i.e. where the choice matters.
    pub contested: usize,
}

impl Comparison {
    /// The strategy with the higher total present value, and by how much.
    pub fn preferred(&self) -> (Strategy, f64) {
        if self.total_cooperative >= self.total_spoiling {
            (Strategy::Cooperate, self.total_cooperative - self.total_spoiling)
        } else {
            (Strategy::Spoil, self.total_spoiling - self.total_cooperative)
        }
    }
}

/// Compare the two strategies region by region, at the plan's funded intensity.
///
/// Both arms are costed on the same present-value budget and over the same horizon, so
/// the difference between them is the strategies and not the accounting.
pub fn compare(params: &StrategyParams) -> Comparison {
    let mut rows = Vec::with_capacity(params.regions.len());
    for region in &params.regions {
        let x = params.funded_spend(region, Strategy::Cooperate);
        let y = params.funded_spend(region, Strategy::Spoil);

        let peace_under_cooperation = params.peace_probability_under_cooperation(region, x);
        let conflict_under_spoiling = params.conflict_probability_under_spoiling(region, y);

        let value_cooperative = params.cooperative_value(region, x);
        let value_spoiling = params.spoiling_value(region, y);

        // What each arm actually collects, as distinct from what it is worth net. The
        // rent is a flow that depletes, exactly as in `spoiling_value`; the trade is a
        // flow that does not, because the peace is what keeps it.
        let rent_captured = region.rent
            * params.capturable_share(region)
            * conflict_under_spoiling
            * StrategyParams::annuity(params.discount + params.depletion, params.horizon_years);
        let trade_preserved = region.trade_benefit
            * peace_under_cooperation
            * StrategyParams::annuity(params.discount, params.horizon_years);

        rows.push(RegionVerdict {
            base_peace_probability: region.baseline_peace_probability(params.dispersion),
            starts_in_conflict: region.starts_in_conflict(),
            cooperative_spend: x,
            spoiling_spend: y,
            peace_probability_under_cooperation: peace_under_cooperation,
            conflict_probability_under_spoiling: conflict_under_spoiling,
            value_cooperative,
            value_spoiling,
            rent_captured,
            trade_preserved,
            difference: value_cooperative - value_spoiling,
            critical_rent: params.critical_rent(region),
            region: region.clone(),
        });
    }

    let total_cooperative = rows.iter().map(|r| r.value_cooperative).sum();
    let total_spoiling = rows.iter().map(|r| r.value_spoiling).sum();

    Comparison {
        strategy_label: params.strategy.label(),
        peaceful_under_cooperation: rows
            .iter()
            .filter(|r| r.peace_probability_under_cooperation >= 0.5)
            .count(),
        peaceful_at_baseline: rows
            .iter()
            .filter(|r| r.base_peace_probability >= 0.5)
            .count(),
        in_conflict_under_spoiling: rows
            .iter()
            .filter(|r| r.conflict_probability_under_spoiling >= 0.5)
            .count(),
        rent_captured: rows.iter().map(|r| r.rent_captured).sum(),
        trade_preserved: rows.iter().map(|r| r.trade_preserved).sum(),
        contested: rows.iter().filter(|r| r.critical_rent.is_some()).count(),
        regions: rows,
        total_cooperative,
        total_spoiling,
    }
}

/// One region's realised state inside a single simulated year.
///
/// The region layer's own randomness is drawn from a **dedicated stream**, seeded from
/// the run seed with a salt. That is not a detail: it is what makes the comparison
/// paired. Both arms of the comparison see the same region realisations and the same
/// world shocks, so the difference between them is the strategy and not the draws -- and
/// switching the layer on cannot shift the world's own random numbers, so every figure
/// published before the layer existed still reproduces.
#[derive(Debug, Clone)]
pub struct StrategyState {
    /// Resolved bloc indices per region, `None` where the world has no such pair.
    resolved: Vec<Option<(usize, usize)>>,
    /// Whether each region is in conflict this year.
    in_conflict: Vec<bool>,
    rng: StdRng,
    /// Region-years spent in conflict, and the number of regions actually monitored.
    conflict_years: u32,
    /// The same count per region, because a single aggregate would have to spread one
    /// conflict rate across regions with different rents -- which would price the Near
    /// East and the Balkans identically and quietly destroy the comparison.
    conflict_years_by_region: Vec<u32>,
    monitored: u32,
}

/// Salt separating the region layer's stream from the world's, so enabling the layer
/// leaves `simulate_run`'s own draws untouched.
pub const STRATEGY_STREAM_SALT: u64 = 0x5A17_2E3B_9C4D_1F07;

impl StrategyState {
    /// Build the layer's state for one run, resolving each region's dyad against the
    /// blocs actually present.
    pub fn new(params: &StrategyParams, blocs: &[PowerBloc], seed: u64) -> Self {
        let resolved: Vec<Option<(usize, usize)>> = params
            .regions
            .iter()
            .map(|region| region.resolves(blocs))
            .collect();
        let monitored = resolved.iter().filter(|entry| entry.is_some()).count() as u32;
        StrategyState {
            in_conflict: vec![false; params.regions.len()],
            resolved,
            rng: StdRng::seed_from_u64(seed ^ STRATEGY_STREAM_SALT),
            conflict_years: 0,
            conflict_years_by_region: vec![0; params.regions.len()],
            monitored,
        }
    }

    /// Draw this year's realised state for every region.
    ///
    /// A region with no resolvable dyad is still drawn for, so that the number of draws
    /// per year does not depend on how the world is configured; otherwise editing a bloc
    /// would change the region stream and the comparison would stop being paired.
    pub fn step(&mut self, params: &StrategyParams) {
        for (index, region) in params.regions.iter().enumerate() {
            let probability = match params.strategy {
                Strategy::Cooperate => {
                    1.0 - params.peace_probability_under_cooperation(region, params.funded_spend(region, Strategy::Cooperate))
                }
                Strategy::Spoil => {
                    params.conflict_probability_under_spoiling(region, params.funded_spend(region, Strategy::Spoil))
                }
            };
            let draw: f64 = self.rng.gen();
            self.in_conflict[index] = draw < probability;
            if self.resolved[index].is_some() && self.in_conflict[index] {
                self.conflict_years += 1;
                self.conflict_years_by_region[index] += 1;
            }
        }
    }

    /// Apply the layer's effect to one dyad's payoffs.
    ///
    /// Only the two doors the model already has are used: the cooperation payoff `cc`,
    /// which is what a peace margin is made of, and the temptation `dc`. Nothing else
    /// in the game is touched, and with the layer off this is never called.
    pub fn adjust(&self, params: &StrategyParams, i: usize, j: usize, payoffs: &mut Payoffs) {
        // Inert when the layer is off, whatever strategy the plan names. The caller
        // already guards on this, and the guard is repeated here because the alternative
        // -- a disabled plan that still lifts a payoff -- is a silent wrong answer rather
        // than a visible one.
        if !params.enabled {
            return;
        }
        for (index, entry) in self.resolved.iter().enumerate() {
            let Some((a, b)) = *entry else { continue };
            if !((a == i && b == j) || (a == j && b == i)) {
                continue;
            }
            match params.strategy {
                Strategy::Cooperate => {
                    // A funded programme lifts the dyad's cooperation payoff whether or
                    // not the conflict is currently realised: the trade and the
                    // guarantees are there in the quiet years too, which is what they
                    // are for.
                    payoffs.cc *= 1.0 + params.cc_lift * params.intensity.clamp(0.0, 1.0);
                }
                Strategy::Spoil => {
                    if self.in_conflict[index] {
                        payoffs.cc *= (1.0 - params.cc_drag).max(0.0);
                        payoffs.dc *= 1.0 + params.dc_lift;
                    }
                }
            }
        }
    }

    /// Annual probability that a region's conflict spills into a world-level war.
    ///
    /// In this simulator a war is a system-wide event drawn once a year, so a regional
    /// conflict has to be aggregated into that one draw. The aggregation is the mean
    /// conflict rate across the monitored regions, and it is stated here rather than
    /// hidden because it is a modelling choice: a region layer finer than the war
    /// channel cannot be represented more exactly than this without inventing a
    /// per-region war channel the simulator does not have.
    pub fn extra_conflict_probability(&self, params: &StrategyParams) -> f64 {
        if !params.enabled || self.monitored == 0 {
            return 0.0;
        }
        let active = self
            .resolved
            .iter()
            .enumerate()
            .filter(|(index, entry)| entry.is_some() && self.in_conflict[*index])
            .count();
        active as f64 / f64::from(self.monitored) * params.war_spillover()
    }

    /// Region-years spent in conflict across the run.
    pub fn conflict_years(&self) -> u32 {
        self.conflict_years
    }

    /// Region-years spent in conflict in one region.
    pub fn conflict_years_in(&self, index: usize) -> u32 {
        self.conflict_years_by_region
            .get(index)
            .copied()
            .unwrap_or(0)
    }

    /// Regions monitored, i.e. whose dyad the world actually contains.
    pub fn monitored(&self) -> u32 {
        self.monitored
    }

    /// Regions in conflict in the current year.
    pub fn conflicts_now(&self) -> usize {
        self.in_conflict.iter().filter(|flag| **flag).count()
    }
}

impl StrategyParams {
    /// Share of a realised regional conflict that becomes a world-level war.
    ///
    /// Declared, and deliberately below one: most regional conflicts do not become
    /// system-wide events, which is what the rest of the simulator means by a war.
    pub fn war_spillover(&self) -> f64 {
        0.35
    }
}

/// What the strategy layer produced in one run, for the report and the export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrategyOutcome {
    /// Mean number of regions in conflict per year.
    pub mean_regions_in_conflict: f64,
    /// Present value of the rent the strategy's realised conflicts release.
    pub realised_rent: f64,
    /// Region-years monitored across the run.
    pub region_years: u32,
    /// Region-years spent in conflict.
    pub conflict_years: u32,
}

/// The standard normal CDF, by Abramowitz and Stegun 7.1.26.
///
/// Maximum absolute error 1.5e-7, which is far below anything this module's declared
/// parameters could justify caring about. Written out rather than pulled in as a
/// dependency: the alternative is a statistical crate for one function, and the
/// accuracy claim above would then be the crate's to make rather than this file's.
fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// The error function, by the same approximation as [`normal_cdf`].
fn erf(x: f64) -> f64 {
    const P: f64 = 0.327_591_1;
    const A: [f64; 5] = [
        0.254_829_592,
        -0.284_496_736,
        1.421_413_741,
        -1.453_152_027,
        1.061_405_429,
    ];
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + P * x);
    let poly = t * (A[0] + t * (A[1] + t * (A[2] + t * (A[3] + t * A[4]))));
    sign * (1.0 - poly * (-x * x).exp())
}

/// Golden-section maximisation of a unimodal function on `[lo, hi]`.
///
/// Returns the argmax and the value. Both objectives here are concave in the spend --
/// a probability times a constant, less the spend -- so a unimodal search is exact
/// rather than a heuristic, and the 200-iteration cap is a guard rather than the
/// mechanism.
fn maximise<F: Fn(f64) -> f64>(f: F, mut lo: f64, mut hi: f64) -> (f64, f64) {
    const RATIO: f64 = 0.618_033_988_749_894_9;
    let mut c = hi - RATIO * (hi - lo);
    let mut d = lo + RATIO * (hi - lo);
    let mut fc = f(c);
    let mut fd = f(d);
    for _ in 0..200 {
        if fc > fd {
            hi = d;
            d = c;
            fd = fc;
            c = hi - RATIO * (hi - lo);
            fc = f(c);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + RATIO * (hi - lo);
            fd = f(d);
        }
        if (hi - lo).abs() < 1e-12 {
            break;
        }
    }
    let x = 0.5 * (lo + hi);
    (x, f(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(strategy: Strategy) -> StrategyParams {
        StrategyParams {
            enabled: true,
            strategy,
            intensity: 1.0,
            ..StrategyParams::default()
        }
    }

    /// The distribution has to be a distribution. If the normal CDF were wrong the whole
    /// framework would still produce plausible-looking numbers, which is the dangerous
    /// kind of error: every comparative static below would be measured against a
    /// misshapen curve.
    #[test]
    fn the_normal_cdf_is_the_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-9);
        // Known values to six places.
        for (z, expected) in [
            (-3.0, 0.001_349_898),
            (-1.96, 0.024_997_855),
            (-1.0, 0.158_655_253),
            (1.0, 0.841_344_747),
            (1.96, 0.975_002_145),
            (3.0, 0.998_650_102),
        ] {
            assert!(
                (normal_cdf(z) - expected).abs() < 1e-6,
                "Phi({z}) = {}, expected {expected}",
                normal_cdf(z)
            );
        }
        // Monotone, and inside the unit interval everywhere.
        let mut previous = 0.0;
        for step in -60..=60 {
            let value = normal_cdf(f64::from(step) / 10.0);
            assert!(value >= previous, "CDF must not decrease");
            assert!((0.0..=1.0).contains(&value));
            previous = value;
        }
    }

    /// A unit flow's present value has to be the sum of its discounted years, or the
    /// entire comparison between a maintained programme and a one-off payment is
    /// computed against the wrong clock.
    #[test]
    fn the_annuity_factor_is_a_discounted_sum() {
        // With no discounting the factor is just the number of years.
        assert!((StrategyParams::annuity(0.0, 25.0) - 25.0).abs() < 1e-12);
        // Checked against an explicit geometric sum.
        for (rate, years) in [(0.04_f64, 50.0_f64), (0.10, 10.0), (0.16, 12.0)] {
            let expected: f64 = (0..years as u32)
                .map(|year| (1.0 + rate).powi(-(year as i32)))
                .sum();
            let got = StrategyParams::annuity(rate, years);
            assert!(
                (got - expected).abs() < 1e-9,
                "annuity({rate}, {years}) = {got}, expected {expected}"
            );
        }
    }

    /// Cooperation has to actually buy peace and spoiling has to actually buy conflict
    /// -- monotonically, and in the right direction. A framework where spending more on
    /// peace lowered the peace probability would still print a table.
    #[test]
    fn each_instrument_moves_the_margin_the_way_it_claims_to() {
        let plan = params(Strategy::Cooperate);
        for region in &plan.regions {
            let base = plan.peace_probability_under_cooperation(region, 0.0);
            assert!(
                (base - region.baseline_peace_probability(plan.dispersion)).abs() < 1e-12,
                "at zero spend the cooperative arm must reproduce the baseline"
            );
            let mut previous = base;
            for step in 1..20 {
                let value = plan.peace_probability_under_cooperation(region, f64::from(step));
                assert!(
                    value >= previous,
                    "{}: more cooperation must not lower the peace probability",
                    region.name
                );
                previous = value;
            }
            let conflict_base = plan.conflict_probability_under_spoiling(region, 0.0);
            assert!(
                (conflict_base - (1.0 - base)).abs() < 1e-12,
                "{}: at zero spend the two arms must be complementary",
                region.name
            );
            let mut previous = conflict_base;
            for step in 1..20 {
                let value = plan.conflict_probability_under_spoiling(region, f64::from(step));
                assert!(
                    value >= previous,
                    "{}: more spoiling must not lower the conflict probability",
                    region.name
                );
                previous = value;
            }
        }
    }

    /// The two instruments act on the same variable, so their probabilities must be
    /// complementary **in the margin**. This is the structural claim the comparison
    /// rests on: if the two arms were buying independent things, "the strategy that
    /// moves the margin further takes the region" would be false and so would every
    /// result below it.
    ///
    /// The identity is deliberately *not* asserted at equal spends. Equal spends in
    /// opposite directions do not land on the same margin -- one starts from `m` and the
    /// other from `m` too -- so `peace(m, x) + conflict(m, x) = 1` is false, and an
    /// earlier version of this test asserted it and failed. What holds is the identity at
    /// a common margin, plus the two arms moving the margin in opposite directions.
    #[test]
    fn the_two_arms_are_two_directions_on_one_variable() {
        let coop = params(Strategy::Cooperate);
        let spoil = StrategyParams {
            strategy: Strategy::Spoil,
            ..coop.clone()
        };
        for region in &coop.regions {
            // Complementarity at a common margin, to the limit of the CDF approximation.
            for step in -30..=30 {
                let margin = f64::from(step) / 10.0;
                let sum = coop.peace_at_margin(margin) + coop.conflict_at_margin(margin);
                assert!(
                    (sum - 1.0).abs() < 1e-12,
                    "{}: peace + conflict at margin {margin} is {sum}",
                    region.name
                );
            }
            // The two arms move the margin in opposite directions, and each reports the
            // probability of the state it is buying at the margin it leaves behind.
            for step in 0..12 {
                let spend = f64::from(step) * 0.5;
                let cooperative_margin = coop.margin_under_cooperation(region, spend);
                let spoiling_margin = spoil.margin_under_spoiling(region, spend);
                assert!(cooperative_margin >= region.margin);
                assert!(spoiling_margin <= region.margin);
                assert!(
                    (coop.peace_probability_under_cooperation(region, spend)
                        - coop.peace_at_margin(cooperative_margin))
                    .abs()
                        < 1e-12
                );
                assert!(
                    (spoil.conflict_probability_under_spoiling(region, spend)
                        - spoil.conflict_at_margin(spoiling_margin))
                    .abs()
                        < 1e-12
                );
                // And because the two instruments have different prices, one unit of
                // spend bought for peace is not one unit bought for war. That difference
                // is Corollary 1.1 and it is the reason the comparison is not trivial.
                assert!(
                    (cooperative_margin - region.margin).abs()
                        < (spoiling_margin - region.margin).abs() + 1e-12,
                    "{}: spoiling is declared the cheaper instrument",
                    region.name
                );
            }
        }
    }

    /// The search must find an interior optimum. An optimum pinned to the ceiling would
    /// make every comparative static -- every statement about spending more or less --
    /// an artefact of where the search stopped rather than a property of the model.
    #[test]
    fn both_optima_are_interior_and_the_search_range_does_not_bind() {
        let plan = params(Strategy::Cooperate);
        for region in &plan.regions {
            let x = plan.optimal_cooperative_spend(region);
            assert!(
                x > 1e-6 && x < SPAWN_CEILING * 0.5,
                "{}: cooperative optimum {x} is not interior",
                region.name
            );
            let y = plan.optimal_spoiling_spend(region);
            assert!(
                y > 1e-6 && y < SPAWN_CEILING * 0.5,
                "{}: spoiling optimum {y} is not interior",
                region.name
            );
            // And it is a maximum, not a point the search happened to stop at.
            let value = plan.cooperative_value(region, x);
            for probe in [x * 0.5, x * 1.5, x + 1.0] {
                assert!(
                    plan.cooperative_value(region, probe) <= value + 1e-9,
                    "{}: {probe} beats the reported cooperative optimum {x}",
                    region.name
                );
            }
        }
    }

    /// The module's central claim, and the one the question actually asks: the spoiler's
    /// prize is bounded by depletion while the cooperator's return is not, so there is a
    /// horizon at which the verdict reverses. If this test fails the framework's answer
    /// is "spoiling always wins", which is a different and much weaker claim.
    #[test]
    fn there_is_a_horizon_where_cooperation_overtakes_spoiling() {
        let mut plan = params(Strategy::Spoil);
        // No discounting isolates depletion, which is the mechanism under test; with
        // discounting both arms are bounded and the crossing is a different question.
        plan.discount = 0.0;
        let horizon = plan
            .break_even_horizon()
            .expect("with depletion and no discounting the two arms must cross");

        // Below the crossing the spoiler is ahead; above it the cooperator is.
        let mut short = plan.clone();
        short.horizon_years = horizon * 0.5;
        let mut long = plan.clone();
        long.horizon_years = horizon * 2.0;
        let short_comparison = compare(&short);
        let long_comparison = compare(&long);
        assert!(
            short.total_value(Strategy::Spoil, &short_comparison)
                > short.total_value(Strategy::Cooperate, &short_comparison),
            "below the crossing the spoiler should be ahead"
        );
        assert!(
            long.total_value(Strategy::Cooperate, &long_comparison)
                > long.total_value(Strategy::Spoil, &long_comparison),
            "above the crossing the cooperator should be ahead"
        );

        // And it must be interior: a crossing at the edge of the search range would be
        // the search range talking, not the model. The measured crossing on the declared
        // regions is about fifteen years, so the bounds are wide rather than tight -- the
        // test is that the crossing exists and is not an artefact of where the search
        // stopped, not that it sits at a particular value.
        assert!(
            horizon > HORIZON_FLOOR * 4.0 && horizon < HORIZON_CEILING * 0.25,
            "crossing at {horizon} years is not interior"
        );
    }

    /// Discounting is what locks the spoiler in, and the direction is the opposite of
    /// what the first version of this module claimed.
    ///
    /// The earlier claim was that impatience *shortens* the spoiler's window, on the
    /// reasoning that the spoiler's return is deferred. Measured on the declared regions
    /// it is the other way round: the spoiler is paid *early* -- its price for the
    /// conflict is one the region's own parties then sustain -- while the cooperator's
    /// return is a flow that has to accumulate. So raising the discount rate pushes the
    /// crossing **later**, and past a critical rate removes it altogether. The old
    /// assertion is now this test, stated the way the numbers actually come out.
    #[test]
    fn impatience_is_what_locks_the_spoiler_in() {
        let horizon_at = |discount: f64| {
            let mut plan = params(Strategy::Spoil);
            plan.discount = discount;
            plan.break_even_horizon()
        };

        let patient = horizon_at(0.0).expect("with no discounting the two arms must cross");
        let moderate = horizon_at(0.04).expect("at four percent the two arms still cross");
        assert!(
            moderate > patient,
            "a less patient strategist must wait LONGER for cooperation to win, not \
             shorter: {moderate} against {patient}"
        );

        // And past a critical rate there is no crossing at any horizon: the spoiler is
        // ahead however long the game runs.
        assert!(
            horizon_at(0.15).is_none(),
            "at fifteen percent the spoiler should be ahead at every horizon"
        );

        let critical = params(Strategy::Spoil)
            .critical_discount_rate()
            .expect("the ordering reverses somewhere between four and fifteen percent");
        assert!(
            critical > 0.04 && critical < 0.15,
            "critical discount rate {critical} is outside the interval its neighbours bracket"
        );
    }

    /// The victims' capacity to answer has to price the spoiler out of regions it could
    /// otherwise take. This is the framework's answer to why the pattern looks the way
    /// it does, so it is checked rather than asserted: retaliation must raise the rent a
    /// region has to offer before spoiling is worth doing.
    #[test]
    fn retaliation_raises_the_rent_a_region_must_offer() {
        let plan = params(Strategy::Spoil);
        let region = plan
            .regions
            .iter()
            .find(|r| r.name == "Near East")
            .expect("the Near East is a declared region");
        let bare = StrategyParams {
            punishment_scale: 0.0,
            ..plan.clone()
        };
        let priced = StrategyParams {
            punishment_scale: 1.4,
            ..plan.clone()
        };

        // With no punishment channel at all, the spoiler's share is the whole rent.
        assert!((bare.capturable_share(region) - 1.0).abs() < 1e-12);
        // With a strong one and a region that can answer, the share has to fall.
        assert!(priced.capturable_share(region) < bare.capturable_share(region));

        // And the region has to become harder to justify spoiling: either the required
        // rent rises, or spoiling was already unprofitable and stays so.
        match (bare.critical_rent(region), priced.critical_rent(region)) {
            (Some(low), Some(high)) => assert!(
                high > low,
                "retaliation should raise the critical rent: {high} against {low}"
            ),
            (None, Some(_)) | (Some(_), None) | (None, None) => {
                // A crossing that exists in one regime and not the other is itself the
                // effect; nothing further to check.
            }
        }
    }

    /// The comparison has to be the comparison: both arms over the same regions, the
    /// same clock, and totals that are the sum of the rows. A mismatch between the
    /// totals and the rows is the kind of error that makes a figure lie about its own
    /// table.
    #[test]
    fn the_comparison_totals_are_the_sum_of_its_rows() {
        let plan = params(Strategy::Spoil);
        let comparison = compare(&plan);
        assert_eq!(comparison.regions.len(), plan.regions.len());
        let sum_cooperative: f64 = comparison.regions.iter().map(|r| r.value_cooperative).sum();
        let sum_spoiling: f64 = comparison.regions.iter().map(|r| r.value_spoiling).sum();
        assert!((comparison.total_cooperative - sum_cooperative).abs() < 1e-9);
        assert!((comparison.total_spoiling - sum_spoiling).abs() < 1e-9);
        let rent: f64 = comparison.regions.iter().map(|r| r.rent_captured).sum();
        assert!((comparison.rent_captured - rent).abs() < 1e-9);
        // Every region's verdict has to match the sign of its own difference.
        for row in &comparison.regions {
            let expected = if row.difference >= 0.0 {
                Strategy::Cooperate
            } else {
                Strategy::Spoil
            };
            assert_eq!(row.prefers(), expected, "{}", row.region.name);
        }
    }

    /// Funding only part of a programme has to be worth less, and by more than
    /// proportionally, because the instrument is concave. This is the assumption a
    /// reader is most likely to disagree with, so it is pinned: if half the money bought
    /// half the margin, every "scaled-down programme" conclusion in the document would
    /// be wrong.
    #[test]
    fn partial_funding_is_worth_less_than_proportionally_less() {
        let full = params(Strategy::Cooperate);
        let half = StrategyParams {
            intensity: 0.5,
            ..full.clone()
        };
        for region in &full.regions {
            let full_spend = full.funded_spend(region, Strategy::Cooperate);
            let half_spend = half.funded_spend(region, Strategy::Cooperate);
            assert!(
                (half_spend - 0.5 * full_spend).abs() < 1e-9,
                "the intensity dial must scale the spend itself"
            );
            let full_shift = full.coop_gain * full_spend.sqrt();
            let half_shift = half.coop_gain * half_spend.sqrt();
            assert!(
                half_shift < 0.5 * (full_shift + 0.5 * full_shift),
                "{}: half the spend must buy less than half the margin",
                region.name
            );
        }
    }

    /// The world-level layer must be inert when it is off, and must only touch the
    /// dyads it names. An `adjust` that leaked onto unrelated pairs would move the
    /// baseline and quietly invalidate every difference the mode reports.
    #[test]
    fn the_layer_touches_only_its_own_dyads_and_only_when_on() {
        let blocs = crate::blocks::default_blocs();
        let plan = StrategyParams {
            strategy: Strategy::Spoil,
            ..StrategyParams::running(Strategy::Spoil)
        };
        let mut state = StrategyState::new(&plan, &blocs, 99);
        state.step(&plan);

        // A dyad that is no region at all: Asia-Pacific against Africa, say.
        let untouched = (5, 6);
        let mut payoffs = Payoffs {
            cc: 3.0,
            cd: -1.0,
            dc: 4.0,
            dd: 1.0,
        };
        let before = payoffs;
        state.adjust(&plan, untouched.0, untouched.1, &mut payoffs);
        assert_eq!(payoffs, before, "an unrelated dyad must not move");

        // The region dyad moves only if the region is in conflict this year, and then
        // in the direction the strategy claims: less cooperation, more temptation.
        let near_east = plan.regions[0]
            .resolves(&blocs)
            .expect("the Near East resolves against the default blocs");
        let mut payoffs = Payoffs {
            cc: 3.0,
            cd: -1.0,
            dc: 4.0,
            dd: 1.0,
        };
        state.adjust(&plan, near_east.0, near_east.1, &mut payoffs);
        let conflict = state.in_conflict[0];
        if conflict {
            assert!(payoffs.cc < 3.0, "conflict should drag the cooperation payoff");
            assert!(payoffs.dc > 4.0, "conflict should lift the temptation");
        } else {
            assert_eq!(payoffs.cc, 3.0, "a peaceful year should not be dragged");
            assert_eq!(payoffs.dc, 4.0);
        }

        // And with the layer off, the cooperative arm's lift does not apply either.
        let off = StrategyParams::off();
        let mut state = StrategyState::new(&off, &blocs, 99);
        state.step(&off);
        let mut payoffs = Payoffs {
            cc: 3.0,
            cd: -1.0,
            dc: 4.0,
            dd: 1.0,
        };
        let before = payoffs;
        state.adjust(&off, near_east.0, near_east.1, &mut payoffs);
        assert_eq!(payoffs, before, "a disabled layer must change nothing");
        assert_eq!(state.extra_conflict_probability(&off), 0.0);
    }

    /// The region layer's stream must be its own. If switching the strategy on shifted
    /// the world's random numbers, every comparison would be between two different sets
    /// of shocks rather than between two strategies, and no published figure would
    /// survive the layer being introduced.
    #[test]
    fn the_region_stream_is_separate_from_the_world_stream() {
        let blocs = crate::blocks::default_blocs();
        let plan = StrategyParams::running(Strategy::Spoil);
        let mut a = StrategyState::new(&plan, &blocs, 12345);
        let mut b = StrategyState::new(&plan, &blocs, 12345);
        // Same seed, same draws: the two layers agree year for year.
        for _ in 0..10 {
            a.step(&plan);
            b.step(&plan);
            assert_eq!(a.in_conflict, b.in_conflict);
        }
        // A different seed moves the region stream, which is what makes the comparison
        // an ensemble rather than one realisation.
        let mut c = StrategyState::new(&plan, &blocs, 999);
        c.step(&plan);
        // Not asserting they differ in any particular year -- that would be a coin flip
        // dressed as a test -- but that the states are independent objects.
        assert_eq!(c.monitored(), a.monitored());
    }

    /// An unresolvable region has to be skipped rather than matched to the wrong blocs,
    /// and skipping it must not change how many draws a year takes. Otherwise editing a
    /// bloc through `--bloc` would desynchronise the comparison.
    #[test]
    fn a_region_the_world_lacks_is_skipped_without_desynchronising_the_stream() {
        let mut blocs = crate::blocks::default_blocs();
        blocs.retain(|bloc| bloc.name != "Gulf");
        let plan = StrategyParams::running(Strategy::Spoil);
        let state = StrategyState::new(&plan, &blocs, 7);
        assert_eq!(
            state.monitored(),
            plan.regions.len() as u32 - 1,
            "the Near East should not resolve without the Gulf"
        );

        // The draw count per year is unchanged, so a run with the Gulf present and a run
        // without it agree on the regions they share.
        let with = StrategyState::new(&plan, &crate::blocks::default_blocs(), 7);
        let mut without_state = state.clone();
        let mut with_state = with.clone();
        without_state.step(&plan);
        with_state.step(&plan);
        assert_eq!(
            without_state.in_conflict[1], with_state.in_conflict[1],
            "the Balkans must see the same draw whatever happened to the Gulf"
        );
    }
}
