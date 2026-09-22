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

//! The third-party spoiler: divide and conquer as a strategy, and why the profiteer is
//! so rarely made to pay.
//!
//! This module is the executable form of `THIRD_PARTY_SPOILER.md`. That document proves
//! six results; this module implements the objects they are about and its test module
//! asserts the results rather than restating them in prose.
//!
//! # The one idea
//!
//! A conflict is not the crossing of interests -- almost every interaction has that -- and
//! it is not an attack, which is only the symptom. It is **the failure of the cooperative
//! outcome to be a Nash equilibrium**, measured by the peace margin
//! `sigma = cc - dc`: positive at peace, negative in conflict. Under that definition a
//! third party can be blamed for something precise. It moved a dyad from `sigma >= 0` to
//! `sigma < 0`, or it made an already-negative margin pay.
//!
//! Everything below follows from taking the resulting inefficiency seriously as a
//! **product with a price**.
//!
//! # Where the numbers come from
//!
//! The dyad's payoffs are `game.rs`'s, unchanged, and the harvest base is that module's
//! `efficiency_loss`, unchanged. Every parameter belonging to the spoiler -- the capture
//! rate `g`, the coordination charge `kappa`, the abuse hazard -- is **illustrative**, and
//! section 7 of the document names the two that have no measured counterpart at all.

use crate::game::{solve, Payoffs};

/// The peace margin `sigma = cc - dc`.
///
/// `sigma >= 0` means the cooperative cell is a Nash equilibrium: given the other side
/// cooperates, neither wants to defect. `sigma < 0` means the dyad is in conflict, and
/// that is Definition 1 of `THIRD_PARTY_SPOILER.md` section 1.
pub fn peace_margin(payoffs: &Payoffs) -> f64 {
    payoffs.cc - payoffs.dc
}

/// The three readings of "conflict" do not coincide. This says which one applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictState {
    /// `sigma >= 0`: the cooperative cell is an equilibrium. The dyad may still have
    /// crossed interests, which is why Theorem 0(i) matters.
    Peace,
    /// `sigma < 0`: the cooperative cell is not an equilibrium. This is conflict.
    Conflict,
}

/// Classify a dyad by Definition 1, not by whether anyone has attacked.
pub fn classify(payoffs: &Payoffs) -> ConflictState {
    if peace_margin(payoffs) >= 0.0 {
        ConflictState::Peace
    } else {
        ConflictState::Conflict
    }
}

/// The three readings of Theorem 0, as a computable check.
///
/// Theorem 0 says crossed interests, violence and the failure of equilibrium are pairwise
/// independent. Rather than assert that in prose, this reports which readings are true of
/// a given game, so the independence can be seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readings {
    /// There is a joint surplus available: `cc + cc > dd + dd`.
    pub crossed_interests: bool,
    /// The cooperative cell is not an equilibrium: `sigma < 0`.
    pub failure_of_equilibrium: bool,
}

impl Readings {
    /// A conflict may be "materialised" -- the equilibrium is the inferior cell -- which
    /// is the reading an observer with only the outcome can see.
    pub fn materialised(payoffs: &Payoffs) -> bool {
        matches!(
            solve(payoffs).equilibrium,
            crate::game::Equilibrium::Dilemma
        )
    }
}

pub fn readings(payoffs: &Payoffs) -> Readings {
    Readings {
        crossed_interests: 2.0 * payoffs.cc > 2.0 * payoffs.dd,
        failure_of_equilibrium: peace_margin(payoffs) < 0.0,
    }
}

/// The spoiler's leverage: harvest over the price of the margin.
///
/// `THIRD_PARTY_SPOILER.md` section 3.1. The result that matters is that the ratio
/// diverges as the margin goes to zero, which is the formal content of "cultivating
/// grievance".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leverage {
    /// `g`: the share of the victims' efficiency loss the spoiler captures, in [0, 1].
    pub capture: f64,
    /// `Lambda`: the efficiency loss the induced conflict would produce, in [0, 1].
    pub loss: f64,
    /// `epsilon`: the fixed cost of operating at all.
    pub operating_cost: f64,
}

impl Default for Leverage {
    fn default() -> Self {
        // `capture` is the model's most important unknown and has no measured
        // counterpart for any conflict; 0.35 is a placeholder, not an estimate.
        Leverage {
            capture: 0.35,
            loss: 0.45,
            operating_cost: 0.02,
        }
    }
}

impl Leverage {
    /// The harvest at zero exposure: `g*Lambda`.
    pub fn harvest(&self) -> f64 {
        self.capture * self.loss
    }

    /// `t*(sigma) = max(0, sigma) + epsilon`.
    ///
    /// A dyad already in conflict needs nothing but to be left alone.
    pub fn transfer_needed(&self, margin: f64) -> f64 {
        margin.max(0.0) + self.operating_cost
    }

    /// `R(sigma) = g*Lambda / (max(0, sigma) + epsilon)`.
    pub fn ratio(&self, margin: f64) -> f64 {
        let denominator = self.transfer_needed(margin);
        if denominator <= 0.0 {
            return f64::INFINITY;
        }
        self.harvest() / denominator
    }

    /// Whether the dyad is worth spoiling at all at this margin.
    pub fn profitable(&self, margin: f64) -> bool {
        self.harvest() > self.transfer_needed(margin)
    }
}

/// Manufacturing grievance, as opposed to subsidising defection.
///
/// `THIRD_PARTY_SPOILER.md` Corollary 1.1: by assumption A3 the two instruments are
/// perfect substitutes in their effect on the margin, so the choice between them is purely
/// one of relative cost. This is the object that makes the choice computable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrievanceCost {
    /// The dyad's natural margin before the spoiler works on it.
    pub natural_margin: f64,
    /// The curvature of the cost of moving the margin down.
    pub curvature: f64,
}

impl Default for GrievanceCost {
    fn default() -> Self {
        GrievanceCost {
            natural_margin: 0.60,
            curvature: 1.60,
        }
    }
}

impl GrievanceCost {
    /// `c_m(sigma) = curvature/2 * (natural - sigma)^2` for `sigma <= natural`.
    ///
    /// Quadratic in the *reduction* achieved, so moving a margin is cheap at first and
    /// expensive as the dyad runs out of patience. Zero reduction costs nothing.
    pub fn cost(&self, margin: f64) -> f64 {
        let reduction = (self.natural_margin - margin).max(0.0);
        0.5 * self.curvature * reduction * reduction
    }

    /// `c_m'(sigma) = -curvature * (natural - sigma)`, negative for any reduction.
    pub fn marginal(&self, margin: f64) -> f64 {
        let reduction = (self.natural_margin - margin).max(0.0);
        -self.curvature * reduction
    }

    /// `Pi(sigma) = g*Lambda/(sigma + epsilon) - c_m(sigma)`, the spoiler's objective.
    pub fn objective(&self, leverage: &Leverage, margin: f64) -> f64 {
        leverage.harvest() / (margin.max(0.0) + leverage.operating_cost)
            - self.cost(margin)
    }
}

/// Where the spoiler stops, and whether it stopped voluntarily.
///
/// `THIRD_PARTY_SPOILER.md` Theorem 1(iii) is a **bifurcation**, and the first draft of the
/// theorem got it wrong by asserting that the optimum is always interior. It is not. There
/// is a critical operating cost above which the spoiler stops short of the edge, and below
/// which it drives the margin to zero -- cheap operations are brazen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrievanceOptimum {
    /// The margin the spoiler chooses.
    pub margin: f64,
    /// True when the optimum is the corner `sigma = 0`: the spoiler pushes the dyad all
    /// the way to the edge rather than stopping short.
    pub at_edge: bool,
}

/// The critical operating cost `epsilon_c = sqrt(g*Lambda / (a*sigma_0))`.
///
/// Below it the spoiler's optimum is the corner; above it there is an interior optimum.
pub fn critical_operating_cost(cost: &GrievanceCost, leverage: &Leverage) -> f64 {
    let numerator = leverage.harvest();
    let denominator = cost.curvature * cost.natural_margin;
    if denominator <= 0.0 {
        return f64::INFINITY;
    }
    (numerator / denominator).sqrt()
}

/// The margin the spoiler chooses.
///
/// `THIRD_PARTY_SPOILER.md` Theorem 1(iii):
///
/// * `epsilon <= epsilon_c` -- the objective is decreasing in the margin throughout, so the
///   optimum is the corner `sigma = 0`;
/// * `epsilon > epsilon_c` -- the objective rises at zero and falls by `sigma_0`, so there
///   is an interior optimum satisfying `a(sigma_0 - sigma) = g*Lambda/(sigma + epsilon)^2`,
///   found here by bisection on that first-order condition.
///
/// Returns `None` when the harvest does not cover the cheapest possible operation, which is
/// the case where the dyad is simply not worth touching.
pub fn grievance_optimum(cost: &GrievanceCost, leverage: &Leverage) -> Option<GrievanceOptimum> {
    // Nothing to gain anywhere on the interval means leave the dyad alone.
    let best_possible = cost.objective(leverage, 0.0).max(cost.objective(leverage, cost.natural_margin));
    if best_possible <= 0.0 {
        return None;
    }

    let epsilon = leverage.operating_cost;
    let critical = critical_operating_cost(cost, leverage);
    if epsilon <= critical {
        return Some(GrievanceOptimum {
            margin: 0.0,
            at_edge: true,
        });
    }

    // Interior: bisect the first-order condition, whose left side rises and right side
    // falls in the margin, so the root is unique.
    let condition = |margin: f64| {
        cost.curvature * (cost.natural_margin - margin)
            - leverage.harvest() / (margin + epsilon).powi(2)
    };
    let (mut lo, mut hi) = (0.0_f64, cost.natural_margin);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if condition(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(GrievanceOptimum {
        margin: 0.5 * (lo + hi),
        at_edge: false,
    })
}

/// The victims' deterrent, charged on the links between them.
///
/// `THIRD_PARTY_SPOILER.md` section 2.3 and Corollary 2.1: `D(n, P) = P - kappa*n(n-1)/2`.
/// A split leaves `P` untouched and adds links, which is the coordination-cost channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deterrent {
    /// The victims' total weight, unchanged by a split.
    pub power: f64,
    /// How many parties must coordinate.
    pub parties: usize,
    /// The charge on each pairwise link.
    pub kappa: f64,
}

impl Deterrent {
    pub fn links(&self) -> f64 {
        let n = self.parties as f64;
        n * (n - 1.0) / 2.0
    }

    /// `D(n, P)`.
    pub fn value(&self) -> f64 {
        (self.power - self.kappa * self.links()).max(0.0)
    }

    /// The same coalition with one member of weight `w` divided into `parts` pieces.
    ///
    /// Total power is preserved exactly, which is the point: divide and conquer is not a
    /// claim about material weight.
    pub fn divided(&self, parts: usize) -> Deterrent {
        Deterrent {
            power: self.power,
            parties: self.parties + parts.saturating_sub(1),
            kappa: self.kappa,
        }
    }
}

/// Divide and conquer, in the quota form of Theorem 2.
///
/// The mechanism the theorem identifies is that division destroys **correlation**, not
/// weight: a divided member's two parts never both support, so the maximum contribution
/// its weight can make to any coalition falls from `w` to `max(w', w'')`.
#[derive(Debug, Clone, PartialEq)]
pub struct Coalition {
    /// Members' weights.
    pub weights: Vec<f64>,
    /// Members' independent support probabilities.
    pub support: Vec<f64>,
    /// The quota the coalition must reach to act.
    pub quota: f64,
}

impl Coalition {
    /// Probability the coalition reaches the quota, by the Poisson-binomial recursion.
    pub fn action_probability(&self) -> f64 {
        let distribution = self.weight_distribution();
        distribution
            .iter()
            .enumerate()
            .filter(|(index, _)| *index as f64 >= self.quota)
            .map(|(_, p)| *p)
            .sum()
    }

    /// Distribution of the total supporting weight, discretised to integer units.
    ///
    /// Weights are rounded to integers so that the recursion is exact rather than
    /// approximate; the theorem is about monotonicity in weights, which rounding
    /// preserves as long as the split is also expressed in the same units.
    fn weight_distribution(&self) -> Vec<f64> {
        let capacity: usize = self.weights.iter().map(|w| w.round().max(0.0) as usize).sum();
        let mut distribution = vec![0.0; capacity + 1];
        distribution[0] = 1.0;
        let mut reach = 0usize;
        for (weight, support) in self.weights.iter().zip(&self.support) {
            let w = weight.round().max(0.0) as usize;
            let p = support.clamp(0.0, 1.0);
            for total in (0..=reach + w).rev() {
                let with_support = if total >= w { distribution[total - w] * p } else { 0.0 };
                distribution[total] = distribution[total] * (1.0 - p) + with_support;
            }
            reach += w;
        }
        distribution
    }

    /// Split member `index` into `parts` pieces that decide **independently**.
    ///
    /// This is fragmentation on its own: the member becomes several actors, each deciding
    /// for itself. Expected weight is preserved and the variance collapses -- which,
    /// `THIRD_PARTY_SPOILER.md` Theorem 2(i), means the effect on collective action has
    /// **no fixed sign**. Against a coalition whose expected weight sits *above* the quota,
    /// removing spread makes it *more* reliable, and the split arms the target.
    pub fn split_independent(&self, index: usize, parts: usize) -> Coalition {
        if parts < 2 || index >= self.weights.len() {
            return self.clone();
        }
        let piece = self.weights[index] / parts as f64;
        let p = self.support[index].clamp(0.0, 1.0);
        let mut weights = self.weights.clone();
        let mut support = self.support.clone();
        weights[index] = piece;
        support[index] = p;
        for _ in 1..parts {
            weights.push(piece);
            support.push(p);
        }
        Coalition {
            weights,
            support,
            quota: self.quota,
        }
    }

    /// Split member `index` into `parts` that **can never both support**.
    ///
    /// This is divide and conquer proper, and `THIRD_PARTY_SPOILER.md` Theorem 2(ii) says
    /// it is what the strategy actually consists of. In any state at most one part is in
    /// favour, so the most the member's weight can ever contribute is its largest part --
    /// `w/parts` for an equal split -- and the realised contribution is weakly lower in
    /// every state. The weapon is the **enmity**, not the division: `split_independent`
    /// alone can raise the coalition's reliability, and only this can lower it.
    pub fn split_estranged(&self, index: usize, parts: usize) -> Coalition {
        if parts < 2 || index >= self.weights.len() {
            return self.clone();
        }
        // The cap: at most one part supports, so the member's effective weight becomes the
        // largest part. Modelling the survivor as a single member of that weight is exact
        // for the bound the theorem uses, and keeps the distribution a product form.
        let capped = self.weights[index] / parts as f64;
        let p = self.support[index].clamp(0.0, 1.0);
        let mut weights = self.weights.clone();
        weights[index] = capped;
        Coalition {
            weights,
            support: {
                let mut s = self.support.clone();
                s[index] = p;
                s
            },
            quota: self.quota,
        }
    }
}

/// The punishment game of Theorem 3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Punishment {
    /// `phi`: the probability the victims can attribute the operation to the spoiler.
    pub attribution: f64,
    /// `K`: the cost each punisher bears.
    pub cost_per_punisher: f64,
    /// `B`: the deterrent benefit each punisher receives.
    pub benefit: f64,
    /// How many victims there are.
    pub victims: usize,
    /// `k`: punishment capacity per unit of available surplus.
    pub capacity_per_surplus: f64,
    /// The victims' available surplus before the conflict.
    pub surplus: f64,
    /// `Lambda`: the loss the induced conflict inflicts on them.
    pub loss: f64,
    /// The spoiler's harvest, which is what has to be deterred.
    pub harvest: f64,
}

impl Punishment {
    /// `phi* = K/B`: the attribution probability at which a victim is indifferent.
    ///
    /// Theorem 3(i): each victim uses its *own* benefit, so this threshold is private and
    /// the collective deterrent is under-provided.
    pub fn individual_threshold(&self) -> f64 {
        if self.benefit <= 0.0 {
            return f64::INFINITY;
        }
        self.cost_per_punisher / self.benefit
    }

    /// The maximum punishment the victims can inflict, **after** their own conflict has
    /// depleted the surplus it is funded from.
    pub fn max_punishment(&self) -> f64 {
        self.capacity_per_surplus * (self.surplus - self.loss).max(0.0)
    }

    /// The same capacity had the conflict not occurred, for comparison.
    pub fn max_punishment_without_conflict(&self) -> f64 {
        self.capacity_per_surplus * self.surplus
    }

    /// Theorem 3(iii): the attribution below which **no punishment occurs although the
    /// spoiler is guilty**.
    ///
    /// `phi-dagger = g*Lambda / (n * K_max)`, and the deterrent is absent below it.
    pub fn attribution_threshold(&self) -> f64 {
        let capacity = self.max_punishment() * self.victims as f64;
        if capacity <= 0.0 {
            return f64::INFINITY;
        }
        self.harvest / capacity
    }

    /// Whether the spoiler is deterred at the current attribution.
    pub fn deterred(&self) -> bool {
        let expected = self.attribution * self.max_punishment() * self.victims as f64;
        expected > self.harvest
    }

    /// Theorem 3(iv): the spoiler's optimal deniability **investment**, where the target is
    /// to hold attribution below the threshold and no further.
    ///
    /// Returns what the spoiler needs to spend, as a multiple of the threshold it must
    /// reach: zero when it is already below `phi-dagger`.
    pub fn deniability_required(&self) -> f64 {
        if self.attribution < self.attribution_threshold() {
            0.0
        } else {
            self.attribution - self.attribution_threshold()
        }
    }
}

/// Blowback: the proxy that turns on its sponsor. Theorem 4.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blowback {
    /// The scale of what the proxy delivers while it is pointed outward.
    pub benefit_scale: f64,
    /// `H`: the damage if it turns.
    pub damage: f64,
    /// `r`: the sponsor's discount rate.
    pub discount: f64,
    /// Convexity of the abuse hazard.
    pub hazard_curvature: f64,
    /// Convexity of the cost of support.
    pub support_curvature: f64,
}

impl Default for Blowback {
    fn default() -> Self {
        Blowback {
            benefit_scale: 1.0,
            damage: 4.0,
            discount: 0.10,
            hazard_curvature: 1.20,
            support_curvature: 0.80,
        }
    }
}

impl Blowback {
    /// `B(s) = benefit_scale * sqrt(s)`: concave.
    pub fn benefit(&self, support: f64) -> f64 {
        self.benefit_scale * support.max(0.0).sqrt()
    }

    /// `h(s) = hazard_curvature * s^2 / 2`: increasing and convex.
    pub fn hazard(&self, support: f64) -> f64 {
        0.5 * self.hazard_curvature * support.max(0.0).powi(2)
    }

    /// `c(s) = support_curvature * s^2 / 2`: convex.
    pub fn support_cost(&self, support: f64) -> f64 {
        0.5 * self.support_curvature * support.max(0.0).powi(2)
    }

    /// `Pi(s) = B(s) - h(s)*H/(1+r) - c(s)`.
    pub fn payoff(&self, support: f64) -> f64 {
        self.payoff_with_damage(support, self.damage)
    }

    /// The same payoff at an arbitrary damage, which is what makes Theorem 4(iv)
    /// checkable: an ex-post-negative damage always exists.
    pub fn payoff_with_damage(&self, support: f64, damage: f64) -> f64 {
        self.benefit(support)
            - self.hazard(support) * damage / (1.0 + self.discount)
            - self.support_cost(support)
    }

    /// The optimal support, from `B'(s) = h'(s)H/(1+r) + c'(s)`.
    ///
    /// Bisection on the derivative, which is strictly decreasing under the curvatures in
    /// `Default`, so the root is the unique maximum.
    pub fn optimal_support(&self) -> f64 {
        let derivative = |s: f64| {
            let marginal_benefit = self.benefit_scale / (2.0 * s.max(1e-12).sqrt());
            let marginal_hazard = self.hazard_curvature * s;
            let marginal_cost = self.support_curvature * s;
            marginal_benefit - marginal_hazard * self.damage / (1.0 + self.discount) - marginal_cost
        };
        let (mut lo, mut hi) = (1e-9_f64, 1.0_f64);
        // The derivative is positive at zero (the benefit term dominates) and eventually
        // negative, so expand the bracket until it contains the sign change.
        while derivative(hi) > 0.0 && hi < 1e9 {
            hi *= 2.0;
        }
        if derivative(hi) > 0.0 {
            return hi;
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if derivative(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// The damage at which the strategy is ex post negative, for a given support.
    ///
    /// Theorem 4(iv): such an `H` always exists, because `H` is unbounded.
    pub fn break_even_damage(&self, support: f64) -> f64 {
        let flow = self.benefit(support) - self.support_cost(support);
        if self.hazard(support) <= 0.0 {
            return f64::INFINITY;
        }
        // (B - c) - h*H/(1+r) = 0
        flow * (1.0 + self.discount) / self.hazard(support)
    }
}

/// Theorem 6: the two conditions that decide whether a spoiler profits, and whether it
/// is punished.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Exposure {
    /// `g_0`: the capture rate at zero exposure.
    pub capture_at_zero: f64,
    /// `Lambda`: the harvest base.
    pub loss: f64,
    /// `T`: the spoiler's total cost of operating.
    pub total_cost: f64,
    /// `phi * n * k * S`: the victims' attribution-weighted punishment capacity.
    pub punishment_capacity: f64,
}

impl Exposure {
    /// The maximum profit, at zero exposure: `X = H - T`.
    pub fn max_profit(&self) -> f64 {
        self.harvest() - self.total_cost
    }

    /// `H = g_0 * Lambda`.
    pub fn harvest(&self) -> f64 {
        self.capture_at_zero * self.loss
    }

    /// Theorem 6(i): the exposure above which the strategy stops being worth doing.
    pub fn profitable_until(&self) -> f64 {
        let h = self.harvest();
        if h <= 0.0 {
            return 0.0;
        }
        1.0 - self.total_cost / h
    }

    /// Theorem 6(i): the exposure above which the victims' punishment deters.
    pub fn deterred_from(&self) -> f64 {
        let h = self.harvest();
        let denominator = self.punishment_capacity + h;
        if denominator <= 0.0 {
            return 0.0;
        }
        h / denominator
    }

    /// Profit at a given exposure, `g_0 (1-e) Lambda - T`.
    pub fn profit(&self, exposure: f64) -> f64 {
        self.capture_at_zero * (1.0 - exposure.clamp(0.0, 1.0)) * self.loss - self.total_cost
    }

    /// Theorem 6(iii): whether the profitable region is contained in the region where
    /// punishment is too weak to stop it.
    ///
    /// The condition is `X/T <= H/(phi n k S)`. The first draft of the theorem asserted
    /// this unconditionally, on the reasoning that low entanglement helps the harvest and
    /// hurts the victims. The reasoning is sound and the conclusion does not follow: two
    /// monotone functions of the same variable are not automatically nested.
    pub fn impunity_contains_profit(&self) -> bool {
        if self.total_cost <= 0.0 {
            return false;
        }
        let left = self.max_profit() / self.total_cost;
        let right = if self.punishment_capacity > 0.0 {
            self.harvest() / self.punishment_capacity
        } else {
            f64::INFINITY
        };
        left <= right
    }

    /// Theorem 6(iv): the band of exposures in which the spoiler **would profit but is
    /// deterred** -- `(e-dagger, e-bar)`.
    ///
    /// This is the mirror of impunity and it is what the model has to say about the
    /// effectiveness of punishment. It is empty exactly when
    /// [`Exposure::impunity_contains_profit`] holds, so the two are complements.
    ///
    /// The substantive reading: strengthening the victims' capacity does not touch the
    /// *most* profitable spoilers at all -- those sit near zero exposure, where there is
    /// nothing to punish -- and only ever closes this high-exposure tail.
    pub fn deterred_band(&self) -> Option<(f64, f64)> {
        let (low, high) = (self.deterred_from(), self.profitable_until());
        if high > low && low >= 0.0 {
            Some((low, high))
        } else {
            None
        }
    }
}

/// Theorem 5: the bystander's proportional gain from a war between others.
///
/// Since shares are relative, destroying `destroyed` units of a rival's weight raises
/// every other share by the same factor, whatever the bystander's size.
pub fn share_gain_factor(total_weight: f64, destroyed: f64) -> f64 {
    let after = total_weight - destroyed;
    if after <= 0.0 {
        return f64::INFINITY;
    }
    total_weight / after
}

/// Whether a bystander's power is better spent growing or breaking, given its outside
/// option. Theorem 5(v): the rate of return is common, so the choice is made by the
/// alternative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutsideOption {
    /// What the bloc can achieve by its own growth over the same horizon.
    pub growth_return: f64,
    /// What spoiling returns, common to every spoiler by Theorem 5(v).
    pub spoiling_return: f64,
}

impl OutsideOption {
    /// Theorem 5(v): a bloc spoils when its own growth is the worse option.
    pub fn prefers_spoiling(&self) -> bool {
        self.spoiling_return > self.growth_return
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    /// Theorem 0: the three readings of "conflict" are independent, and the equilibrium
    /// reading is the one that carries the others as special cases. Checked by exhibiting
    /// a game of each kind rather than by asserting the taxonomy.
    #[test]
    fn crossed_interests_violence_and_equilibrium_failure_are_independent() {
        // (i) Crossed interests, no conflict: a coordination game.
        let coordination = Payoffs {
            cc: 2.0,
            cd: 0.0,
            dc: 1.0,
            dd: 0.0,
        };
        let r = readings(&coordination);
        assert!(r.crossed_interests, "there is a joint surplus here");
        assert!(!r.failure_of_equilibrium, "but cooperation IS an equilibrium");
        assert_eq!(classify(&coordination), ConflictState::Peace);

        // (ii) Conflict, no crossed interests: nothing to fight over.
        let positional = Payoffs {
            cc: 0.0,
            cd: -1.0,
            dc: 1.0,
            dd: 0.0,
        };
        let r = readings(&positional);
        assert!(!r.crossed_interests, "mutual cooperation is no better jointly");
        assert!(r.failure_of_equilibrium, "yet the cooperative cell is not an equilibrium");
        assert_eq!(classify(&positional), ConflictState::Conflict);

        // (iii) Violence in a game where the cooperative cell IS an equilibrium. `(D,D)` is
        // also an equilibrium here, so a war can happen with no failure of equilibrium to
        // explain it -- pure mistrust, which is neither of the other two readings.
        let mistrust = Payoffs {
            cc: 2.0,
            cd: 0.0,
            dc: 1.0,
            dd: 1.0,
        };
        assert_eq!(
            classify(&mistrust),
            ConflictState::Peace,
            "the cooperative cell is an equilibrium: sigma = {} >= 0",
            peace_margin(&mistrust)
        );
        assert!(
            mistrust.dd >= mistrust.cd && mistrust.dd >= mistrust.dc,
            "and so is mutual defection, so violence is reachable without any dilemma"
        );
    }

    /// Theorem 1(i)-(ii): the transfer needed is the margin plus the operating cost, and
    /// the leverage ratio falls in the margin and diverges as the margin reaches zero.
    #[test]
    fn leverage_diverges_as_the_peace_margin_closes() {
        let leverage = Leverage::default();

        // A dyad already in conflict needs nothing but to be left alone.
        assert!(close(leverage.transfer_needed(-0.5), leverage.operating_cost, 1e-12));
        // A dyad at peace needs its margin closed.
        assert!(close(leverage.transfer_needed(0.4), 0.4 + leverage.operating_cost, 1e-12));

        // The ratio is strictly INCREASING as the margin closes -- that is the whole point,
        // and the first draft of this test asserted the wrong direction.
        let mut previous = 0.0;
        for step in 0..20 {
            let margin = 1.0 - step as f64 * 0.05;
            let ratio = leverage.ratio(margin);
            assert!(
                ratio > previous,
                "margin {margin}: ratio {ratio} should rise above {previous}"
            );
            previous = ratio;
        }

        // And it diverges at the margin: the harvest divided by the operating cost alone.
        let at_the_edge = leverage.ratio(0.0);
        assert!(
            at_the_edge > 10.0 * leverage.ratio(1.0),
            "the whole point is that the return is unbounded as the margin closes: \
             {at_the_edge} vs {}",
            leverage.ratio(1.0)
        );
        assert!(close(at_the_edge, leverage.harvest() / leverage.operating_cost, 1e-9));
    }

    /// Theorem 1(iii): the optimum **bifurcates** on the operating cost. Below the critical
    /// value the spoiler drives the margin to zero; above it, it stops short. The first
    /// draft asserted the interior case unconditionally, which is false -- the leverage
    /// term's derivative blows up as `1/sigma^2` and beats a quadratic grievance cost.
    #[test]
    fn the_spoiler_is_brazen_when_cheap_and_subtle_when_dear() {
        let cost = GrievanceCost::default();
        let cheap = Leverage {
            operating_cost: 0.02,
            ..Leverage::default()
        };
        let dear = Leverage {
            operating_cost: 1.50,
            ..Leverage::default()
        };
        let critical = critical_operating_cost(&cost, &cheap);
        assert!(
            cheap.operating_cost < critical && dear.operating_cost > critical,
            "the two cases must straddle the critical cost {critical}"
        );

        // Cheap: the corner. The spoiler pushes the dyad to the edge of conflict.
        let brazen = grievance_optimum(&cost, &cheap).expect("worth doing");
        assert!(brazen.at_edge, "a cheap operation is brazen");
        assert!(close(brazen.margin, 0.0, 1e-12));

        // Dear: interior, and the first-order condition holds there.
        let subtle = grievance_optimum(&cost, &dear).expect("worth doing");
        assert!(!subtle.at_edge, "a dear operation stops short of the edge");
        assert!(
            subtle.margin > 0.0 && subtle.margin < cost.natural_margin,
            "the optimum must be interior: {}",
            subtle.margin
        );
        let lhs = cost.curvature * (cost.natural_margin - subtle.margin);
        let rhs = dear.harvest() / (subtle.margin + dear.operating_cost).powi(2);
        assert!(
            close(lhs, rhs, 1e-6),
            "the first-order condition must hold: {lhs} vs {rhs}"
        );

        // And the comparative static: a dearer operation is a more subtle one.
        let mut previous = 0.0;
        for epsilon in [0.60, 0.80, 1.00, 1.50, 2.00] {
            let model = Leverage {
                operating_cost: epsilon,
                ..Leverage::default()
            };
            let optimum = grievance_optimum(&cost, &model).expect("worth doing");
            assert!(
                !optimum.at_edge && optimum.margin > previous,
                "epsilon {epsilon}: the optimal margin must rise, got {}",
                optimum.margin
            );
            previous = optimum.margin;
        }
    }

    /// Corollary 1.1: the two instruments are perfect substitutes in their effect on the
    /// margin, so only relative cost decides -- which is what makes propaganda and arms
    /// comparable at all.
    #[test]
    fn the_two_instruments_are_substitutes_and_only_price_decides() {
        let leverage = Leverage::default();
        let margin = 0.5;

        // Reducing the margin by delta and subsidising defection by delta leave the dyad
        // identically placed: same sigma, hence same ratio.
        let by_grievance = margin - 0.2;
        let by_subsidy = margin - 0.2; // dc raised by 0.2 lowers sigma by 0.2
        assert!(close(
            leverage.ratio(by_grievance),
            leverage.ratio(by_subsidy),
            1e-12
        ));

        // Therefore the choice is a pure price comparison, and the cheaper instrument wins.
        let narrative_unit_cost = 0.30;
        let subsidy_unit_cost = 1.00;
        assert!(
            narrative_unit_cost < subsidy_unit_cost,
            "where claims are cheap, the narrative instrument is the one used"
        );
    }

    /// Theorem 2: fragmentation alone has **no fixed sign** -- it can arm the target -- and
    /// only the manufactured enmity reliably disarms it. This is the correction the first
    /// draft of the theorem needed, and the test exhibits both directions.
    #[test]
    fn fragmentation_alone_can_arm_the_target_and_only_enmity_disarms_it() {
        // A coalition whose expected supporting weight sits comfortably above the quota, so
        // its unreliability is the only thing stopping it.
        let coalition = Coalition {
            weights: vec![6.0, 3.0, 3.0, 2.0],
            support: vec![0.8, 0.8, 0.8, 0.8],
            quota: 9.0,
        };
        let before = coalition.action_probability();
        assert!(before > 0.0, "the coalition must be able to act");

        // (i) Independent fragmentation REMOVES spread around a mean above the quota, so it
        // makes the coalition MORE reliable. Dividing a rival is not automatically good.
        let independent = coalition.split_independent(0, 2).action_probability();
        assert!(
            independent > before,
            "fragmentation alone can arm the target: {independent} vs {before}"
        );

        // (ii) The enmity is what does the work. Making the two parts unable to support
        // together caps the member's contribution and strictly lowers the probability.
        let estranged = coalition.split_estranged(0, 2).action_probability();
        assert!(
            estranged < before,
            "the estrangement must reduce the coalition's ability to act: \
             {estranged} vs {before}"
        );

        // (iii) The largest member is the one worth dividing: splitting a small member
        // leaves the coalition better placed than splitting the large one.
        let split_small = coalition.split_estranged(3, 2).action_probability();
        assert!(
            split_small > estranged,
            "splitting the small member leaves {split_small}; splitting the large one \
             leaves {estranged}"
        );

        // Aggregate *nominal* weight is untouched by independent fragmentation -- which is
        // why divide and conquer is not a claim about material power. The estrangement, by
        // contrast, does reduce the weight the member can ever bring to bear: that is the
        // cap, and it is precisely why it works where fragmentation alone does not.
        let total_before: f64 = coalition.weights.iter().sum();
        let independent_total: f64 = coalition.split_independent(0, 2).weights.iter().sum();
        assert!(
            close(total_before, independent_total, 1e-9),
            "an independent split preserves aggregate weight: {independent_total} vs \
             {total_before}"
        );

        let estranged_total: f64 = coalition.split_estranged(0, 2).weights.iter().sum();
        let cap = coalition.weights[0] / 2.0;
        assert!(
            close(total_before - estranged_total, coalition.weights[0] - cap, 1e-9),
            "the estrangement must reduce effective weight by w - w/parts = {}",
            coalition.weights[0] - cap
        );
        assert!(estranged_total < independent_total);
    }

    /// Corollary 2.1: with deterrence charged on the links, division costs exactly
    /// `kappa * n` while leaving aggregate power alone.
    #[test]
    fn the_coordination_charge_is_the_whole_gain_from_division() {
        let deterrent = Deterrent {
            power: 10.0,
            parties: 4,
            kappa: 0.20,
        };
        let before = deterrent.value();
        let divided = deterrent.divided(2);
        let after = divided.value();

        assert!(after < before, "division must weaken the deterrent");
        let expected = deterrent.kappa * deterrent.parties as f64;
        assert!(
            close(before - after, expected, 1e-9),
            "the loss must be exactly kappa*n = {expected}, got {}",
            before - after
        );
        assert!(close(divided.power, deterrent.power, 1e-12));
        // With no coordination cost there is nothing to gain from dividing.
        let free = Deterrent { kappa: 0.0, ..deterrent };
        assert!(close(free.value(), free.divided(2).value(), 1e-12));
    }

    /// Theorem 3(i)-(iii): impunity. The individual punishment threshold is above the
    /// collective one, the induced conflict depletes the capacity to punish, and below a
    /// positive attribution threshold NO punishment occurs although the spoiler is guilty.
    #[test]
    fn impunity_is_derived_from_the_spoilers_own_success() {
        let punishment = Punishment {
            attribution: 0.20,
            cost_per_punisher: 0.30,
            benefit: 0.60,
            victims: 5,
            capacity_per_surplus: 0.50,
            surplus: 4.0,
            loss: 1.5,
            harvest: 0.50,
        };

        // (i) The private threshold is what each victim uses; the collective one is lower.
        assert!(close(punishment.individual_threshold(), 0.50, 1e-12));

        // (ii) The induced conflict disarms the response to itself.
        let without = punishment.max_punishment_without_conflict();
        let with = punishment.max_punishment();
        assert!(with < without, "the conflict must deplete the capacity to punish");
        assert!(
            close(without - with, punishment.capacity_per_surplus * punishment.loss, 1e-12),
            "the loss must be exactly k*Lambda"
        );

        // (iii) There is a positive attribution threshold below which nobody acts.
        let threshold = punishment.attribution_threshold();
        assert!(threshold > 0.0, "the threshold must be positive");
        let guilty_but_free = Punishment {
            attribution: threshold * 0.5,
            ..punishment
        };
        assert!(
            !guilty_but_free.deterred(),
            "below the threshold the spoiler is guilty and nobody acts"
        );
        let caught = Punishment {
            attribution: threshold * 3.0,
            ..punishment
        };
        assert!(caught.deterred(), "well above it, the spoiler is deterred");
    }

    /// Theorem 3(iv): deniability has a TARGET, not a maximum -- a spoiler already below
    /// the threshold spends nothing on it.
    #[test]
    fn deniability_has_a_target_and_can_be_free() {
        let base = Punishment {
            attribution: 0.20,
            cost_per_punisher: 0.30,
            benefit: 0.60,
            victims: 5,
            capacity_per_surplus: 0.50,
            surplus: 4.0,
            loss: 1.5,
            harvest: 0.50,
        };
        let threshold = base.attribution_threshold();

        let already_hidden = Punishment {
            attribution: threshold * 0.5,
            ..base
        };
        assert_eq!(
            already_hidden.deniability_required(),
            0.0,
            "a spoiler facing victims who cannot attribute the operation buys nothing"
        );

        let exposed = Punishment {
            attribution: threshold * 2.0,
            ..base
        };
        assert!(
            close(exposed.deniability_required(), threshold, 1e-12),
            "an exposed spoiler buys exactly enough deniability to reach the threshold"
        );
    }

    /// Theorem 4(ii): a more patient sponsor supports its proxy less, so impatience is
    /// what produces blowback. And (iv): a bankruptcy-level damage always exists.
    #[test]
    fn impatience_funds_the_proxy_harder_and_the_tail_always_exists() {
        let base = Blowback::default();
        let patient = Blowback { discount: 0.02, ..base };
        let impatient = Blowback { discount: 0.50, ..base };

        let s_patient = patient.optimal_support();
        let s_impatient = impatient.optimal_support();
        assert!(
            s_impatient > s_patient,
            "the impatient sponsor must support harder: {s_impatient} vs {s_patient}"
        );

        // The first-order condition holds at each optimum.
        for model in [patient, impatient] {
            let s = model.optimal_support();
            let h = 1e-6;
            let slope = (model.payoff(s + h) - model.payoff(s - h)) / (2.0 * h);
            assert!(slope.abs() < 1e-4, "support {s} must be stationary, slope {slope}");
        }

        // (iv) An ex-post-negative damage exists for the support actually chosen.
        let s = impatient.optimal_support();
        let break_even = impatient.break_even_damage(s);
        assert!(break_even.is_finite(), "a finite break-even damage must exist");
        assert!(
            impatient.payoff_with_damage(s, break_even * 1.01) < 0.0,
            "just past the break-even, the strategy is ex post negative"
        );
        assert!(impatient.payoff(s) > 0.0, "and at the optimum it is ex ante positive");
    }

    /// Theorem 5: destruction is a substitute for growth, and the proportional gain is
    /// common to every bystander.
    #[test]
    fn destruction_substitutes_for_growth_and_the_gain_is_common() {
        let total = 10.0;
        let factor = share_gain_factor(total, 2.0);
        assert!(close(factor, 10.0 / 8.0, 1e-12));

        // The same factor applies whatever the bystander's size: a small bloc and a large
        // one gain proportionally the same from someone else's destruction.
        for own in [0.5, 2.0, 5.0] {
            let before = own / total;
            let after = own / (total - 2.0);
            assert!(
                close(after / before, factor, 1e-12),
                "bloc of {own} should gain by the common factor"
            );
        }

        // And the strategy needs no capability of its own, which is why the outside
        // option decides who uses it.
        let stalled = OutsideOption { growth_return: 0.01, spoiling_return: 0.05 };
        let growing = OutsideOption { growth_return: 0.08, spoiling_return: 0.05 };
        assert!(stalled.prefers_spoiling(), "the stagnant bloc spoils");
        assert!(!growing.prefers_spoiling(), "the growing bloc has a better use for the money");
    }

    /// Theorem 6: impunity attaches to **successful** spoiling, not to spoiling. The strong
    /// containment claim needs the condition `X/T <= H/(phi n k S)`, and both regimes are
    /// exhibited because asserting only the first is the error the document records.
    #[test]
    fn impunity_attaches_to_successful_spoiling_not_to_spoiling() {
        // Modest stakes, strong victims: the containment holds and no profitable spoiler is
        // deterred. `H = 0.20`, `T = 0.10`, so `X/T = 1.0` and `H/(phi n k S) = 1.33`.
        let contained = Exposure {
            capture_at_zero: 0.40,
            loss: 0.50,
            total_cost: 0.10,
            punishment_capacity: 0.15,
        };
        assert!(contained.max_profit() > 0.0, "the strategy must be worth considering");
        assert!(
            contained.impunity_contains_profit(),
            "X/T = {:.3} should be within H/(phi n k S) = {:.3}",
            contained.max_profit() / contained.total_cost,
            contained.harvest() / contained.punishment_capacity
        );
        assert!(
            contained.deterred_band().is_none(),
            "when the containment holds, no profitable spoiler is deterred"
        );

        // The same prize with victims who can actually punish: the containment fails and a
        // band of profitable-but-deterred spoilers opens.
        let strong_victims = Exposure {
            punishment_capacity: 3.00,
            ..contained
        };
        assert!(!strong_victims.impunity_contains_profit());
        let band = strong_victims
            .deterred_band()
            .expect("the complement must give a nonempty band");
        assert!(band.0 < band.1, "the band must be an interval: {band:?}");
        // Inside the band the spoiler would profit, and does not act.
        let inside = 0.5 * (band.0 + band.1);
        assert!(strong_victims.profit(inside) > 0.0, "profitable at {inside}");
        assert!(inside > strong_victims.deterred_from());

        // The substantive reading: punishment only ever closes the HIGH-exposure tail. The
        // most profitable spoilers sit near zero exposure, where there is nothing to punish.
        assert!(
            band.0 > 0.0,
            "the band must not reach zero exposure: the best spoilers are untouched"
        );
        assert!(contained.profit(0.0) > contained.profit(0.5));
    }
}
