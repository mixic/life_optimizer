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

//! The information layer: legitimation, credibility as a depletable stock, and the
//! public-good failure in verification.
//!
//! This module is the executable form of `INFORMATION_WARFARE.md`. That document
//! proves six results; this module implements the objects they are about, and its
//! test module asserts the results rather than restating them in prose. A theorem
//! that only exists in a markdown file drifts from the code the moment either
//! changes; a theorem encoded as an invariant fails a build instead.
//!
//! # What the layer does to the existing simulator
//!
//! Exactly one thing, and it does it only when switched on: it **replaces the
//! exogenous war draw** (`ShockParams::war_probability`) with an endogenous one.
//! Force is used when a bloc's payoff from using it crosses zero, and believed
//! culpability is what moves that payoff across zero. Everything else in the Monte
//! Carlo -- growth, the payoff matrices, tension, the pension channel -- is
//! untouched, and a test pins the byte-identical baseline that makes that claim
//! checkable rather than hopeful.
//!
//! # Read the conditions, not the constants
//!
//! Every parameter here is illustrative in the sense `blocks.rs` states: chosen for
//! plausible sign and rough magnitude, calibrated to nothing. What the module is
//! for is the *conditions* the theorems name -- the band where no lie can work, the
//! threshold `lambda = rho(Sigma)` where belief stops being stable, the time-cost
//! boundary where dribbling forever becomes optimal. Those are structural; the
//! numbers attached to them are not.

/// Detection, damage and recovery of the credibility stock. `INFORMATION_WARFARE.md`
/// section 2.4.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CredibilityParams {
    /// `mu`: detection hazard per unit of sustained assertion mass. Verification
    /// capacity lives here -- inspections, courts, a free press, forensic capacity.
    /// It is a property of the *audience*, not of any bloc, which is why section 3.5
    /// finds it under-provided rather than merely present.
    pub hazard: f64,
    /// `chi`: credibility destroyed per unit of *exposed* falsehood. Damage is charged
    /// on the size of the lie, not on the number of times it was told; section 3.7
    /// states what depends on that and what happens if it is wrong.
    pub damage: f64,
    /// `r`: annual regeneration of standing -- institutions rebuilding themselves.
    pub regeneration: f64,
    /// `kappa`: the ceiling on credibility, and the scale of the whole stock.
    pub ceiling: f64,
}

impl Default for CredibilityParams {
    fn default() -> Self {
        CredibilityParams {
            // Illustrative throughout. The orderings are what the theorems use:
            // a larger hazard means a smaller affordable lie, a larger regeneration
            // rate a larger one.
            hazard: 0.70,
            damage: 1.20,
            regeneration: 0.25,
            ceiling: 1.00,
        }
    }
}

impl CredibilityParams {
    /// `pi(x) = 1 - exp(-mu x)`: the probability that sustained effort `x` is exposed.
    pub fn detection_probability(&self, effort: f64) -> f64 {
        if effort <= 0.0 || self.hazard <= 0.0 {
            return 0.0;
        }
        1.0 - (-self.hazard * effort).exp()
    }

    /// `beta(x) = chi * x * pi(x)`: expected credibility damage per year.
    ///
    /// Strictly increasing on `[0, inf)` with `beta(0) = 0`, which is what makes the
    /// break-even lie unique. See `break_even_lie`.
    pub fn detection_burden(&self, effort: f64) -> f64 {
        self.damage * effort * self.detection_probability(effort)
    }

    /// `k_{t+1}` from `k_t`, for a given realisation of the detection draw.
    ///
    /// The clip is not decoration: credibility is bounded above by what an audience
    /// can extend and below by zero, and the floor is what makes the reference
    /// process a bounded chain rather than a process that goes negative forever.
    pub fn step(&self, k: f64, effort: f64, detected: bool) -> f64 {
        let drift = self.regeneration * (1.0 - k / self.ceiling);
        let hit = if detected { self.damage * effort } else { 0.0 };
        (k + drift - hit).clamp(0.0, self.ceiling)
    }

    /// `x-dagger`: the effort at which expected damage exactly equals regeneration.
    ///
    /// Below it the stock has a positive reversion target; above it the stock is
    /// self-consuming. Returns `None` when `mu = 0`, i.e. when nothing is ever
    /// detected and every lie is therefore free -- the degenerate case the model
    /// should report rather than silently absorb.
    pub fn break_even_lie(&self) -> Option<f64> {
        if self.hazard <= 0.0 || self.damage <= 0.0 || self.regeneration <= 0.0 {
            return None;
        }
        // beta is a strictly increasing bijection [0, inf) -> [0, inf), so bisection
        // on a bracket that is doubled until it contains the root always terminates.
        let target = self.regeneration;
        let mut hi = 1.0;
        while self.detection_burden(hi) < target {
            hi *= 2.0;
            if hi > 1e12 {
                return None;
            }
        }
        let mut lo = 0.0;
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if self.detection_burden(mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(0.5 * (lo + hi))
    }

    /// The credibility the stock reverts toward at sustained effort `x`, from
    /// `r(1 - kappa_bar/kappa) = beta(x)`. May be negative, which is the model's way
    /// of saying that at that effort the stock is being spent faster than it refills.
    pub fn stationary_credibility(&self, effort: f64) -> f64 {
        if self.regeneration <= 0.0 {
            return 0.0;
        }
        self.ceiling * (1.0 - self.detection_burden(effort) / self.regeneration)
    }
}

/// The legitimation function `Lambda`: the fraction of the cost of force that
/// believed culpability removes. `INFORMATION_WARFARE.md` section 2.5.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegitimationParams {
    /// `Lambda_max`: how much of the cost can *ever* be removed. Below 1 there is an
    /// irreducible price on force, which is what makes the deterred band of Theorem 1
    /// non-empty even for a maximally justified war.
    pub max: f64,
    /// Steepness of the response. `0.0` gives the linear case `Lambda = max * b`.
    pub steepness: f64,
}

impl Default for LegitimationParams {
    fn default() -> Self {
        LegitimationParams {
            max: 0.85,
            steepness: 3.0,
        }
    }
}

impl LegitimationParams {
    /// `Lambda(b)`, normalised so that `Lambda(1) = max` exactly.
    ///
    /// Without the normalisation the ceiling would depend on the steepness, and two
    /// parameters that are supposed to mean different things -- how much can be
    /// removed, and how fast -- would be entangled.
    pub fn legitimation(&self, belief: f64) -> f64 {
        let b = belief.clamp(0.0, 1.0);
        if self.steepness <= 1e-12 {
            return (self.max * b).clamp(0.0, self.max);
        }
        let denom = 1.0 - (-self.steepness).exp();
        (self.max * (1.0 - (-self.steepness * b).exp()) / denom).clamp(0.0, self.max)
    }
}

/// Whether force is used against a given target. The three cases of Theorem 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttackThreshold {
    /// `V(0) >= 0`: the operation pays for itself and needs no justification.
    /// Section 3.2(iv). Narrative here is decoration, not cause.
    NoneNeeded,
    /// `V(1) < 0`: no achievable belief covers the cost. Section 3.2(iii).
    /// **No quantity of disinformation can produce an attack in this dyad.**
    Deterred,
    /// The belief at which the attack becomes worthwhile. Section 3.2(i).
    At(f64),
}

impl AttackThreshold {
    /// Whether a war in this dyad is *available to be justified* -- Corollary 1.1's
    /// band `B`. True exactly when disinformation can change the outcome.
    pub fn is_contestable(&self) -> bool {
        matches!(self, AttackThreshold::At(_))
    }
}

/// The attack payoff `V = G + gamma*b - C*(1 - Lambda(b))`.
///
/// The derivative in `b` is what Theorem 1 needs, and it is strictly positive:
/// `gamma + C*Lambda'(b) > 0` whenever `C > 0` or `gamma > 0`.
pub fn attack_payoff(
    spoils: f64,
    cost: f64,
    coalition_channel: f64,
    legitimation: &LegitimationParams,
    belief: f64,
) -> f64 {
    spoils + coalition_channel * belief.clamp(0.0, 1.0)
        - cost * (1.0 - legitimation.legitimation(belief))
}

/// Locate the threshold of Theorem 1 by bisection on `[0, 1]`.
///
/// Bisection rather than a closed form because the closed form exists only for the
/// linear case, and hard-coding it would mean the checked model and the plotted model
/// were two different models. The closed form is asserted against this one in the
/// tests instead.
pub fn attack_threshold(
    spoils: f64,
    cost: f64,
    coalition_channel: f64,
    legitimation: &LegitimationParams,
) -> AttackThreshold {
    let at = |b: f64| attack_payoff(spoils, cost, coalition_channel, legitimation, b);
    if at(0.0) >= 0.0 {
        return AttackThreshold::NoneNeeded;
    }
    if at(1.0) < 0.0 {
        return AttackThreshold::Deterred;
    }
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if at(mid) >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    AttackThreshold::At(0.5 * (lo + hi))
}

/// Influence structure: how suspicion about one bloc transfers to belief about another.
/// `INFORMATION_WARFARE.md` section 2.3.
#[derive(Debug, Clone, PartialEq)]
pub struct Influence {
    /// `Sigma`, zero diagonal, nonnegative.
    pub contamination: Vec<Vec<f64>>,
}

impl Influence {
    /// A ring of mutual reinforcement of the given strength. Not a measurement of any
    /// real media system: a shape whose spectral radius can be dialled, so that the
    /// threshold of Theorem 3 can be approached from both sides.
    pub fn ring(n: usize, strength: f64) -> Self {
        let mut contamination = vec![vec![0.0; n]; n];
        for i in 0..n {
            contamination[i][(i + 1) % n] = strength;
            contamination[(i + 1) % n][i] = strength;
        }
        Influence { contamination }
    }

    /// Spectral radius by power iteration. Converges for any nonnegative matrix with a
    /// positive Perron root.
    ///
    /// The Rayleigh quotient is taken from the **raw** matrix-vector product, before
    /// normalisation. Normalising first and then forming `v . next` gives the cosine of
    /// the angle between successive iterates, which is 1 at convergence for *every*
    /// matrix -- an error that reports a spectral radius of 1 and silently destroys the
    /// threshold of Corollary 3.1.
    pub fn spectral_radius(&self) -> f64 {
        let n = self.contamination.len();
        if n == 0 {
            return 0.0;
        }
        let mut v = vec![1.0 / (n as f64).sqrt(); n];
        let mut eigenvalue = 0.0_f64;
        for _ in 0..2000 {
            let mut next = vec![0.0; n];
            for (slot, row) in next.iter_mut().zip(&self.contamination) {
                *slot = row.iter().zip(&v).map(|(a, b)| a * b).sum();
            }
            let norm = next.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm <= 1e-15 {
                return 0.0;
            }
            let lambda: f64 = v.iter().zip(&next).map(|(a, b)| a * b).sum();
            for x in next.iter_mut() {
                *x /= norm;
            }
            let converged = (lambda - eigenvalue).abs() < 1e-14;
            eigenvalue = lambda;
            v = next;
            if converged {
                break;
            }
        }
        eigenvalue.max(0.0)
    }

    /// The critical verification rate `lambda* = rho(Sigma)` of Corollary 3.1.
    pub fn verification_threshold(&self) -> f64 {
        self.spectral_radius()
    }

    /// The long-run multiplier on a sustained injection, `1 / (1 - rho(A))` with
    /// `A = (1 - lambda) I + Sigma`. `None` when the system is unstable, because the
    /// honest answer there is that there is no multiplier -- belief does not settle.
    pub fn amplification(&self, verification: f64) -> Option<f64> {
        let rho = (1.0 - verification) + self.spectral_radius();
        if rho >= 1.0 {
            None
        } else {
            Some(1.0 / (1.0 - rho))
        }
    }

    /// `A = (1 - lambda) I + Sigma`.
    pub fn transition(&self, verification: f64) -> Vec<Vec<f64>> {
        let mut a = self.contamination.clone();
        for (i, row) in a.iter_mut().enumerate() {
            row[i] += 1.0 - verification;
        }
        a
    }

    /// Solve `(I - A) b = rhs` by Gaussian elimination with partial pivoting.
    ///
    /// Returns `None` when the system is unstable, i.e. when
    /// `(1 - lambda) + rho(Sigma) >= 1`. Theorem 3(iv) says belief then has no steady
    /// state, and the refusal is made on that ground rather than on a pivot happening to
    /// be numerically zero: just below the threshold the matrix is *nearly* singular and
    /// elimination cheerfully returns a number larger than the effect that produced it,
    /// which is exactly the kind of invented answer a model should decline to give.
    pub fn steady_belief(
        &self,
        verification: f64,
        theta: &[f64],
        injection: &[f64],
    ) -> Option<Vec<f64>> {
        if (1.0 - verification) + self.spectral_radius() >= 1.0 {
            return None;
        }
        let a = self.transition(verification);
        let n = a.len();
        if theta.len() != n || injection.len() != n {
            return None;
        }
        // (I - A) b = lambda*theta + injection
        let mut m = vec![vec![0.0; n + 1]; n];
        for i in 0..n {
            for j in 0..n {
                m[i][j] = if i == j { 1.0 } else { 0.0 } - a[i][j];
            }
            m[i][n] = verification * theta[i] + injection[i];
        }
        for col in 0..n {
            let mut pivot = col;
            for row in (col + 1)..n {
                if m[row][col].abs() > m[pivot][col].abs() {
                    pivot = row;
                }
            }
            if m[pivot][col].abs() < 1e-12 {
                return None;
            }
            m.swap(col, pivot);
            let scale = m[col][col];
            for value in m[col][col..=n].iter_mut() {
                *value /= scale;
            }
            // The pivot row is cloned so the elimination below can borrow the matrix
            // mutably without holding an immutable borrow of one of its own rows.
            let pivot_row = m[col].clone();
            for (row, values) in m.iter_mut().enumerate() {
                if row == col || values[col].abs() == 0.0 {
                    continue;
                }
                let factor = values[col];
                for (value, pivot_value) in
                    values[col..=n].iter_mut().zip(&pivot_row[col..=n])
                {
                    *value -= factor * pivot_value;
                }
            }
        }
        Some((0..n).map(|i| m[i][n]).collect())
    }
}

/// The instalment problem of Theorem 6.
///
/// Total assertion mass fixed, delivered as `T` equal instalments, with a time cost
/// `delta` per instalment. Spread thin is cheaper in credibility; spreading takes
/// time. Theorem 6 shows the trade-off has an interior optimum below a critical time
/// cost and degenerates to "never stop" above it.
pub fn instalment_damage(params: &CredibilityParams, mass: f64, instalments: f64) -> f64 {
    if instalments < 1.0 || mass <= 0.0 {
        return 0.0;
    }
    // T * chi * (M/T) * (1 - exp(-mu * M/T))
    params.damage * mass * (1.0 - (-params.hazard * mass / instalments).exp())
}

/// The optimum number of instalments, or `None` when the optimum is unbounded --
/// `delta * mu >= 4 * chi * exp(-2)`, above which the bloc dribbles forever.
pub fn instalment_optimum(
    params: &CredibilityParams,
    mass: f64,
    time_cost: f64,
) -> Option<f64> {
    if mass <= 0.0 || time_cost <= 0.0 || params.hazard <= 0.0 || params.damage <= 0.0 {
        return None;
    }
    let critical = 4.0 * params.damage * (-2.0_f64).exp();
    if time_cost * params.hazard >= critical {
        return None;
    }
    // Solve chi * z^2 * exp(-z) = delta * mu on the increasing branch z in (0, 2).
    let target = time_cost * params.hazard;
    let h = |z: f64| params.damage * z * z * (-z).exp();
    let (mut lo, mut hi) = (0.0_f64, 2.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if h(mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let z = 0.5 * (lo + hi);
    if z <= 1e-12 {
        return None;
    }
    // The continuous optimum can fall below one instalment, in which case the discrete
    // optimum is the corner T = 1: there is no way to spread a claim over less than one
    // claim.
    Some((params.hazard * mass / z).max(1.0))
}

/// The verification game of Theorem 4.
///
/// Each bloc `i` has a marginal valuation `marginal_valuation[i] = Phi_i' - Psi_i'` of a
/// higher verification rate: positive for a net target, negative for a net aggressor.
/// Investment is chosen simultaneously at convex cost.
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationGame {
    /// `u_i'(lambda)`, one entry per bloc. Sign is what makes a bloc an aggressor.
    pub marginal_valuation: Vec<f64>,
    /// Quadratic cost coefficients, one per bloc.
    pub cost: Vec<f64>,
    /// `Lambda`: how much aggregate investment moves the verification rate.
    pub productivity: f64,
    /// `lambda_0`: the verification rate with no investment at all.
    pub baseline: f64,
}

impl VerificationGame {
    /// Nash investment: each bloc equates its **own** marginal benefit to its marginal
    /// cost. A net aggressor invests nothing.
    pub fn nash(&self) -> Vec<f64> {
        self.marginal_valuation
            .iter()
            .zip(&self.cost)
            .map(|(u, c)| {
                if *u <= 0.0 || *c <= 0.0 {
                    0.0
                } else {
                    self.productivity * u / c
                }
            })
            .collect()
    }

    /// Indices of the net targets: the blocs with `u_i' > 0`, which are the ones for
    /// which verification is a good rather than a cost.
    pub fn targets(&self) -> Vec<usize> {
        self.marginal_valuation
            .iter()
            .enumerate()
            .filter(|(_, u)| **u > 0.0)
            .map(|(i, _)| i)
            .collect()
    }

    /// The benchmark that matters for the public-good claim: the **targets acting
    /// jointly**, each equating the sum of the *targets'* marginal valuations to its own
    /// marginal cost. `Theorem 4(ii)`.
    ///
    /// This is deliberately not the world-welfare planner. Verification is a good for a
    /// bloc that is lied about and a bad for a bloc that wants to lie, so a planner that
    /// counts both is not measuring under-provision of a public good -- it is measuring a
    /// tug of war, and its sign is not fixed. `world_welfare_gap` reports that comparison
    /// separately, because the model's honesty depends on not conflating them.
    pub fn target_coalition(&self) -> Vec<f64> {
        let total: f64 = self
            .targets()
            .iter()
            .map(|i| self.marginal_valuation[*i])
            .sum();
        self.cost
            .iter()
            .map(|c| {
                if total <= 0.0 || *c <= 0.0 {
                    0.0
                } else {
                    self.productivity * total / c
                }
            })
            .collect()
    }

    /// The world-welfare planner: the sum over **all** blocs, aggressors included.
    pub fn social(&self) -> Vec<f64> {
        let total: f64 = self.marginal_valuation.iter().sum();
        self.cost
            .iter()
            .map(|c| {
                if total <= 0.0 || *c <= 0.0 {
                    0.0
                } else {
                    self.productivity * total / c
                }
            })
            .collect()
    }

    /// The verification rate each regime produces.
    pub fn rate(&self, investment: &[f64]) -> f64 {
        self.baseline + self.productivity * investment.iter().sum::<f64>()
    }

    /// Theorem 4(ii)'s gap: how far short of what the exposed blocs would jointly choose
    /// the equilibrium falls. This is the under-provision result, and its sign is fixed.
    pub fn under_provision_gap(&self) -> f64 {
        self.rate(&self.target_coalition()) - self.rate(&self.nash())
    }

    /// Theorem 4(iii): the same comparison against *world* welfare, aggressors counted.
    /// **The sign is not fixed.** It is positive iff the targets' total exposure exceeds
    /// the aggressors' stake in being believed, and negative otherwise -- which is to say
    /// that a system dominated by a net aggressor can present the appearance of
    /// over-verification.
    pub fn world_welfare_gap(&self) -> f64 {
        self.rate(&self.social()) - self.rate(&self.nash())
    }
}

/// Distribute the *return* to disinformation across Council members.
///
/// Theorem 0(iii): the marginal return to effort aimed at `v` is the product of
/// pivotality, persuadability and the injector's credibility. This takes the first two
/// as inputs and returns the normalised allocation that maximises the probability of
/// authorisation for a fixed effort budget -- which is where the "target the swing veto
/// holder, not the strongest member" claim becomes arithmetic.
pub fn pivotality_weights(pivotality: &[f64], persuadability: &[f64]) -> Vec<f64> {
    let raw: Vec<f64> = pivotality
        .iter()
        .zip(persuadability)
        .map(|(p, q)| (p * q).max(0.0))
        .collect();
    let total: f64 = raw.iter().sum();
    if total <= 0.0 {
        return vec![0.0; raw.len()];
    }
    raw.iter().map(|x| x / total).collect()
}

/// The veto structure of the body whose authorisation legitimates force.
///
/// The default is the UN Security Council: fifteen members, nine votes required, and
/// five of them able to block alone. It is here as an institutional fact, and it is the
/// reason Theorem 0(iii)'s containment is strict rather than an artefact of the numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Council {
    pub elected: usize,
    pub permanent: usize,
    pub quota: usize,
}

impl Default for Council {
    fn default() -> Self {
        Council {
            elected: 10,
            permanent: 5,
            quota: 9,
        }
    }
}

impl Council {
    pub fn size(&self) -> usize {
        self.elected + self.permanent
    }

    /// Probability the resolution passes, given each member's own support probability.
    ///
    /// A "no" from any permanent member vetoes, so passage requires **every** permanent
    /// member in favour and at least `quota` votes in total. That is the structure that
    /// makes Theorem 0(iii) strict rather than incidental.
    pub fn pass_probability(&self, support: &[f64]) -> f64 {
        if support.len() != self.size() {
            return 0.0;
        }
        let permanent = yes_distribution(&support[..self.permanent]);
        let elected = yes_distribution(&support[self.permanent..]);
        let mut total = 0.0;
        for (yes_p, p_weight) in permanent.iter().enumerate() {
            if p_weight == &0.0 {
                continue;
            }
            // Every permanent member must be in favour.
            if yes_p != self.permanent {
                continue;
            }
            for (yes_e, e_weight) in elected.iter().enumerate() {
                if yes_p + yes_e >= self.quota {
                    total += p_weight * e_weight;
                }
            }
        }
        total
    }

    /// The same probability with every member at `q_all` except one member of the
    /// chosen kind, which supports with probability `q_member`. This is the object whose
    /// derivative Theorem 0(ii) identifies with pivotality.
    pub fn pass_probability_with_one(
        &self,
        q_all: f64,
        permanent_member: bool,
        q_member: f64,
    ) -> f64 {
        let mut support = vec![q_all; self.size()];
        let index = if permanent_member { 0 } else { self.permanent };
        support[index] = q_member;
        self.pass_probability(&support)
    }

    /// Probability that a member is pivotal, by exact enumeration over the other
    /// members' votes, under A6 and equal support probability `q` for everyone.
    ///
    /// `permanent_member` selects which kind is being evaluated. Enumeration rather
    /// than a closed form because the point of Theorem 0(iii) is a *comparison*
    /// between two structures, and a shared enumeration cannot favour either.
    pub fn pivotality(&self, q: f64, permanent_member: bool) -> f64 {
        let others_permanent = if permanent_member {
            self.permanent - 1
        } else {
            self.permanent
        };
        let others_elected = if permanent_member {
            self.elected
        } else {
            self.elected - 1
        };
        let mut probability = 0.0;
        for yes_p in 0..=others_permanent {
            // A no vote from any other permanent member vetoes, so their support is
            // required in every pivotal configuration.
            if yes_p != others_permanent {
                continue;
            }
            let p_p = binomial(others_permanent, yes_p, q);
            for yes_e in 0..=others_elected {
                let p_e = binomial(others_elected, yes_e, q);
                let weight = p_p * p_e;
                if weight == 0.0 {
                    continue;
                }
                let yes = yes_p + yes_e;
                let pivotal = if permanent_member {
                    // This member's no is a veto; it is pivotal iff its yes would pass,
                    // i.e. the others already supply quota - 1 votes.
                    yes >= self.quota - 1
                } else {
                    // Its no is not a veto, so it is pivotal iff its yes reaches the
                    // quota and its no falls short: exactly quota - 1 others in favour.
                    yes == self.quota - 1
                };
                if pivotal {
                    probability += weight;
                }
            }
        }
        probability
    }
}

/// Distribution of the number of "yes" votes among members with the given support
/// probabilities, by the Poisson-binomial recursion. Index is the number of yes votes.
fn yes_distribution(probabilities: &[f64]) -> Vec<f64> {
    let n = probabilities.len();
    let mut distribution = vec![0.0; n + 1];
    distribution[0] = 1.0;
    for (i, p) in probabilities.iter().enumerate() {
        let p = p.clamp(0.0, 1.0);
        // Walk downwards so the update uses the previous stage's values.
        for k in (1..=i + 1).rev() {
            distribution[k] = distribution[k] * (1.0 - p) + distribution[k - 1] * p;
        }
        distribution[0] *= 1.0 - p;
    }
    distribution
}

/// `C(n, k) q^k (1-q)^(n-k)`.
fn binomial(n: usize, k: usize, q: f64) -> f64 {
    if k > n {
        return 0.0;
    }
    let mut coefficient = 1.0_f64;
    for i in 0..k {
        coefficient *= (n - i) as f64 / (i + 1) as f64;
    }
    coefficient * q.powi(k as i32) * (1.0 - q).powi((n - k) as i32)
}

/// World-economy loss from a war, `INFORMATION_WARFARE.md` section 2.6 and Theorem 5.
///
/// Split into the three channels the theorem treats separately, so that the
/// comparative statics can be asserted per channel rather than on a sum in which an
/// error in one could hide behind another.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LossParams {
    /// Direct destruction per unit of combined belligerent power.
    pub destruction: f64,
    /// Coefficient on the convex severance term.
    pub severance: f64,
    /// Severance is charged on interdependence to this power.
    pub severance_curvature: f64,
    /// Deadweight per unit of tension the war adds.
    pub tension_deadweight: f64,
}

impl Default for LossParams {
    fn default() -> Self {
        LossParams {
            destruction: 0.06,
            severance: 0.25,
            severance_curvature: 1.5,
            tension_deadweight: 0.10,
        }
    }
}

impl LossParams {
    /// `L = l_d + l_t(E) + l_tau(dtau)`.
    pub fn loss(&self, combined_power: f64, interdependence: f64, tension_increment: f64) -> f64 {
        let e = interdependence.clamp(0.0, 1.0);
        self.destruction * combined_power.max(0.0)
            + self.severance * e.powf(self.severance_curvature)
            + self.tension_deadweight * tension_increment.max(0.0)
    }
}

/// Parameters for the information layer. `Default` is **off**, and that is load-bearing:
/// with the layer disabled the simulator must reproduce its published figures exactly.
#[derive(Debug, Clone, PartialEq)]
pub struct InformationParams {
    pub enabled: bool,
    pub credibility: CredibilityParams,
    pub legitimation: LegitimationParams,
    pub influence: Influence,
    /// Per-bloc sustained assertion mass, indexed by bloc position.
    pub effort: Vec<f64>,
    /// `gamma`: the channel by which believed culpability raises the spoils directly.
    pub coalition_channel: f64,
    /// `lambda`: the verification rate of the audience.
    pub verification: f64,
    /// `theta`: true culpability, indexed by bloc position.
    pub truth: Vec<f64>,
    /// The fixed cost of force in the simulator's own payoff units -- the part of `C`
    /// that no amount of justification removes. It is the calibration dial for the
    /// derived war rate: with the layer on, the war rate is an *output*, and this is the
    /// input that decides what it comes out as.
    pub force_cost: f64,
    /// Theorem 5(i) inside the simulation: how much more a war costs the belligerents
    /// when the pair is deeply interdependent. Zero reproduces a world where severance is
    /// free, which is the layer-off behaviour.
    pub severance_scale: f64,
}

impl Default for InformationParams {
    fn default() -> Self {
        InformationParams {
            enabled: false,
            credibility: CredibilityParams::default(),
            legitimation: LegitimationParams::default(),
            influence: Influence { contamination: Vec::new() },
            effort: Vec::new(),
            coalition_channel: 0.15,
            verification: 0.30,
            truth: Vec::new(),
            force_cost: 1.60,
            severance_scale: 0.50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    /// Theorem 0(ii): the derivative of the authorisation probability with respect to a
    /// member's support probability *is* that member's pivotality. Checked numerically
    /// against the exact enumeration rather than restated.
    #[test]
    fn the_return_to_persuading_a_member_is_its_pivotality() {
        let council = Council::default();
        let h = 1e-6;
        for q in [0.2, 0.4, 0.5, 0.6, 0.8] {
            for permanent in [true, false] {
                let numeric = (council.pass_probability_with_one(q, permanent, q + h)
                    - council.pass_probability_with_one(q, permanent, q - h))
                    / (2.0 * h);
                let analytic = council.pivotality(q, permanent);
                assert!(
                    close(numeric, analytic, 1e-5),
                    "q={q}, permanent={permanent}: dP/dq = {numeric}, pivotality = {analytic}"
                );
            }
        }
    }

    /// Theorem 0(iii): in the veto structure a permanent member is pivotal in strictly
    /// more configurations than an elected member. This is the containment claim, and
    /// it is the formal reason disinformation is aimed at the swing veto holder.
    #[test]
    fn the_veto_concentrates_pivotality_on_the_permanent_members() {
        let council = Council::default();
        for q in [0.2, 0.35, 0.5, 0.65, 0.8] {
            let permanent = council.pivotality(q, true);
            let elected = council.pivotality(q, false);
            assert!(
                permanent > elected,
                "q={q}: permanent {permanent} should exceed elected {elected}"
            );
        }
    }

    /// Theorem 1(i)-(ii): the attack set is an upper interval, and the threshold falls
    /// in the spoils and rises in the cost.
    #[test]
    fn the_attack_threshold_falls_in_spoils_and_rises_in_cost() {
        let legit = LegitimationParams::default();
        let threshold = |g: f64, c: f64| match attack_threshold(g, c, 0.0, &legit) {
            AttackThreshold::At(b) => Some(b),
            _ => None,
        };

        let base = threshold(0.35, 1.0).expect("interior case");
        let richer = threshold(0.50, 1.0).expect("interior case");
        let dearer = threshold(0.35, 1.4).expect("interior case");
        assert!(richer < base, "more spoils must need less belief: {richer} vs {base}");
        assert!(dearer > base, "a dearer war must need more belief: {dearer} vs {base}");
    }

    /// Theorem 1(iii)-(iv) and Corollary 1.1: outside the band, disinformation is either
    /// unnecessary or useless. This is the paper's most testable claim, so it is pinned.
    #[test]
    fn outside_the_band_disinformation_changes_nothing() {
        let legit = LegitimationParams::default();
        // Profitable on its own: no justification required.
        assert_eq!(
            attack_threshold(1.5, 1.0, 0.0, &legit),
            AttackThreshold::NoneNeeded
        );
        // Too expensive for any belief to cover: deterred, and no lie can help.
        assert_eq!(
            attack_threshold(0.05, 5.0, 0.0, &legit),
            AttackThreshold::Deterred
        );
        // ...and only in between is the dyad contestable.
        assert!(attack_threshold(0.35, 1.0, 0.0, &legit).is_contestable());
    }

    /// Theorem 2a, as a pathwise coupling: two sustained lie masses driven by the *same*
    /// uniform sequence must order the credibility paths everywhere, not merely on
    /// average. This is stronger than a mean comparison and is what the proof asserts.
    #[test]
    fn credibility_is_monotone_in_sustained_lying_pathwise() {
        let params = CredibilityParams::default();
        let small = 0.10;
        let large = 0.35;

        // A deterministic, reproducible uniform sequence stands in for the draws.
        let mut state = 0x1234_5678_9abc_def0_u64;
        let mut next_uniform = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };

        let (mut k_small, mut k_large) = (params.ceiling, params.ceiling);
        for step in 0..500 {
            let u = next_uniform();
            let d_small = u < params.detection_probability(small);
            let d_large = u < params.detection_probability(large);
            // The coupling relies on this: the same draw detects the larger lie whenever
            // it detects the smaller one.
            assert!(!(d_small && !d_large), "detection must be monotone in the lie");
            k_small = params.step(k_small, small, d_small);
            k_large = params.step(k_large, large, d_large);
            assert!(
                k_small >= k_large - 1e-12,
                "step {step}: k({small}) = {k_small} fell below k({large}) = {k_large}"
            );
        }
    }

    /// Theorem 2b: the break-even lie mass exists, is unique, and moves the way the
    /// comparative statics say. Verification capacity (`hazard`) lowers it; institutional
    /// regeneration raises it.
    #[test]
    fn the_break_even_lie_moves_with_verification_and_regeneration() {
        let base = CredibilityParams::default();
        let x = base.break_even_lie().expect("a break-even lie exists");

        // It is a break-even: expected damage equals regeneration there.
        assert!(close(base.detection_burden(x), base.regeneration, 1e-6));

        let better_verification = CredibilityParams {
            hazard: base.hazard * 2.0,
            ..base
        };
        let more_regeneration = CredibilityParams {
            regeneration: base.regeneration * 2.0,
            ..base
        };
        let cheaper_scandal = CredibilityParams {
            damage: base.damage / 2.0,
            ..base
        };

        assert!(
            better_verification.break_even_lie().unwrap() < x,
            "better verification must shrink the affordable lie"
        );
        assert!(
            more_regeneration.break_even_lie().unwrap() > x,
            "faster institutional recovery must enlarge it"
        );
        assert!(
            cheaper_scandal.break_even_lie().unwrap() > x,
            "a cheaper scandal must enlarge it"
        );

        // And below the break-even the stock has a positive reversion target.
        assert!(base.stationary_credibility(0.5 * x) > 0.0);
        assert!(base.stationary_credibility(1.5 * x) < 0.0);
    }

    /// Theorem 3(i): the stability threshold is exactly `lambda = rho(Sigma)`. Exact,
    /// not approximate: `rho((1-lambda)I + Sigma) = (1-lambda) + rho(Sigma)`.
    #[test]
    fn the_spectral_identity_holds_and_names_the_verification_threshold() {
        for strength in [0.05, 0.10, 0.20, 0.30] {
            let influence = Influence::ring(8, strength);
            let rho = influence.spectral_radius();
            // A symmetric ring has spectral radius equal to its row sum.
            assert!(
                close(rho, 2.0 * strength, 1e-9),
                "strength {strength}: rho = {rho}"
            );
            for verification in [0.1, 0.3, 0.6] {
                let a = influence.transition(verification);
                // The identity is what makes Corollary 3.1 exact.
                let rho_a = spectral_radius_dense(&a);
                assert!(
                    close(rho_a, (1.0 - verification) + rho, 1e-9),
                    "lambda {verification}: rho(A) = {rho_a}, expected {}",
                    (1.0 - verification) + rho
                );
            }
        }
    }

    /// Theorem 3(iii): the amplification is exactly `1/(1-rho)` in the Perron
    /// direction, and the Leontief inverse is nonnegative.
    #[test]
    fn the_perron_amplification_is_exactly_one_over_one_minus_rho() {
        let influence = Influence::ring(6, 0.12);
        let verification = 0.4;
        let rho = (1.0 - verification) + influence.spectral_radius();
        let multiplier = influence.amplification(verification).expect("stable");

        // Feed the Perron direction and read the steady state back: the ratio must be
        // the multiplier, not merely close to it.
        let n = 6;
        let theta = vec![0.0; n];
        let injection = vec![1.0; n]; // constant, and the ring's Perron vector is constant
        let steady = influence
            .steady_belief(verification, &theta, &injection)
            .expect("an interior steady state exists above the threshold");
        for value in &steady {
            assert!(
                close(*value, multiplier, 1e-6),
                "steady belief {value} should equal 1/(1-rho) = {multiplier}"
            );
            assert!(*value >= 0.0, "the Leontief inverse must be nonnegative");
        }
        let _ = rho;
    }

    /// Theorem 3(iv) and Corollary 3.1: below the threshold there is no steady state, and
    /// the model must say so rather than return a number. This is the phase transition.
    #[test]
    fn below_the_verification_threshold_belief_has_no_equilibrium() {
        let influence = Influence::ring(8, 0.20);
        let threshold = influence.verification_threshold();
        assert!(close(threshold, 0.40, 1e-9));

        let theta = vec![0.5; 8];
        let injection = vec![0.1; 8];

        assert!(
            influence.amplification(threshold - 0.01).is_none(),
            "just below the threshold the multiplier must not exist"
        );
        assert!(
            influence.steady_belief(threshold - 0.01, &theta, &injection).is_none(),
            "just below the threshold the steady state must not exist"
        );
        assert!(
            influence.amplification(threshold + 0.01).is_some(),
            "just above the threshold it must exist"
        );
        assert!(
            influence.amplification(threshold + 0.01).unwrap() > 1.0,
            "and it must amplify rather than damp"
        );
    }

    /// Theorem 4: verification is under-provided *among the blocs it protects*, a net
    /// aggressor invests nothing, and the world-welfare comparison has no fixed sign.
    ///
    /// The third part is the correction that writing this test forced: the first version
    /// of the theorem claimed the equilibrium was below the world-welfare planner for
    /// every bloc, which is false whenever an aggressor's stake in being believed exceeds
    /// the other targets' combined exposure. The theorem was wrong, not the arithmetic.
    #[test]
    fn verification_is_under_provided_among_targets_and_aggressors_free_ride() {
        let game = VerificationGame {
            // One net aggressor, three net targets. The aggressor is the largest bloc.
            marginal_valuation: vec![-0.9, 0.6, 0.4, 0.3],
            cost: vec![1.0, 1.0, 1.0, 1.0],
            productivity: 0.5,
            baseline: 0.2,
        };

        let nash = game.nash();
        let coalition = game.target_coalition();

        assert_eq!(nash[0], 0.0, "a net aggressor must invest nothing");
        assert!(
            game.under_provision_gap() > 0.0,
            "against the exposed blocs' own joint benchmark, the equilibrium falls short"
        );
        for i in game.targets() {
            assert!(
                nash[i] <= coalition[i] + 1e-12,
                "target {i}: Nash {} exceeded the coalition benchmark {}",
                nash[i],
                coalition[i]
            );
            assert!(
                nash[i] < coalition[i] - 1e-12,
                "target {i}: with three targets each must strictly under-invest"
            );
        }

        // Bloc 1's own valuation (0.6) exceeds the sum over everyone (0.4), because the
        // aggressor's -0.9 drags the total below it. So against *world* welfare this
        // target invests too much, not too little -- the sign of that comparison is a
        // tug of war between exposures and aggressors' stakes, and it is not fixed.
        let social = game.social();
        assert!(
            nash[1] > social[1] + 1e-12,
            "against world welfare a target can over-invest: Nash {} vs planner {}",
            nash[1],
            social[1]
        );
        assert!(
            game.world_welfare_gap() > 0.0,
            "here the targets' total exposure (1.3) still outweighs the aggressor (0.9)"
        );

        // Make the aggressor decisive and the world-welfare comparison flips outright:
        // the planner wants *less* verification than the exposed blocs already fund.
        let dominant_aggressor = VerificationGame {
            marginal_valuation: vec![-2.0, 0.6, 0.4, 0.3],
            ..game.clone()
        };
        assert!(
            dominant_aggressor.world_welfare_gap() < 0.0,
            "with a decisive aggressor the world-welfare comparison must flip, got {}",
            dominant_aggressor.world_welfare_gap()
        );
        assert!(
            dominant_aggressor.under_provision_gap() > 0.0,
            "while the exposed blocs are still under-provided relative to their own benchmark"
        );

        // With everyone a net target the two benchmarks coincide and the gap is still
        // positive: free-riding is about the public good, not about the aggressor.
        let all_targets = VerificationGame {
            marginal_valuation: vec![0.5, 0.5, 0.5],
            ..game.clone()
        };
        assert!(all_targets.under_provision_gap() > 0.0);
        assert!(
            close(all_targets.under_provision_gap(), all_targets.world_welfare_gap(), 1e-12),
            "with no aggressor the two benchmarks must agree"
        );

        // With no targets at all there is nothing to under-provide.
        let all_aggressors = VerificationGame {
            marginal_valuation: vec![-0.4, -0.2],
            cost: vec![1.0, 1.0],
            productivity: 0.5,
            baseline: 0.2,
        };
        assert_eq!(all_aggressors.nash(), vec![0.0, 0.0]);
        assert_eq!(all_aggressors.under_provision_gap(), 0.0);
    }

    /// Theorem 5(i)-(ii): the world's exposure rises in integration without exception,
    /// while the aggressor's private incentive moves with the balance of the two
    /// elasticities and can go either way. The wedge between them is the result.
    #[test]
    fn integration_raises_world_exposure_and_can_cut_either_way_privately() {
        let loss = LossParams::default();
        let legit = LegitimationParams::default();

        // World exposure: strictly increasing, and convex.
        let mut previous = loss.loss(0.3, 0.0, 0.5);
        for step in 1..=10 {
            let e = step as f64 / 10.0;
            let value = loss.loss(0.3, e, 0.5);
            assert!(value > previous, "world loss must rise in E at E = {e}");
            previous = value;
        }
        let increments: Vec<f64> = (1..=10)
            .map(|step| {
                let e = step as f64 / 10.0;
                loss.loss(0.3, e, 0.5) - loss.loss(0.3, e - 0.1, 0.5)
            })
            .collect();
        for pair in increments.windows(2) {
            assert!(pair[1] > pair[0], "severance must be convex in E");
        }

        // Private incentive: the spoils channel can dominate or be dominated.
        let payoff = |e: f64, spoils_elasticity: f64, own_exposure: f64| {
            let g = 0.30 * (1.0 + spoils_elasticity * e);
            let c = 1.00 + own_exposure * e;
            attack_payoff(g, c, 0.0, &legit, 0.5)
        };
        let pacified = payoff(1.0, 0.10, 1.20);
        let destabilised = payoff(1.0, 1.20, 0.05);
        assert!(
            pacified < payoff(0.0, 0.10, 1.20),
            "when the aggressor's own exposure dominates, integration must pacify"
        );
        assert!(
            destabilised > payoff(0.0, 1.20, 0.05),
            "when the spoils channel dominates, integration must destabilise"
        );
    }

    /// Theorem 5(iii)-(iv): the aggressor's private optimum exceeds the planner's, and
    /// the *marginal* wedge between them grows with integration. This is the link from
    /// lies to the world economy, and it is stated marginally because that is the part
    /// the proof establishes -- the ordering of the maximisers follows from it only
    /// under the concavity in A8.
    #[test]
    fn the_private_return_to_lying_exceeds_the_social_one_and_the_gap_grows_with_integration() {
        let loss = LossParams::default();
        let params = CredibilityParams::default();
        let legit = LegitimationParams::default();

        // Belief bought by injection, through the credibility stock: more injection buys
        // more belief, with diminishing returns because it burns the stock it is spent
        // from. This is Theorem 2's mechanism inside Theorem 5's decision.
        let belief = |s: f64| (s * params.stationary_credibility(s).max(0.0)).min(1.0);

        // The war needs justification at b = 0 and is feasible at the belief the bloc
        // can actually buy, so this dyad sits inside Corollary 1.1's band.
        let (spoils, cost, gamma) = (0.75, 1.0, 0.15);
        assert!(
            attack_payoff(spoils, cost, gamma, &legit, 0.0) < 0.0,
            "the example must need justification, or the band is trivial"
        );
        assert!(
            attack_payoff(spoils, cost, gamma, &legit, 0.2) > 0.0,
            "and the band must be reachable, or nothing is being tested"
        );

        // The objective, with the aggressor bearing `share` of the world loss.
        let objective = |s: f64, share: f64, integration: f64| -> f64 {
            let b = belief(s);
            let war = attack_payoff(spoils, cost, gamma, &legit, b);
            let total_loss = loss.loss(0.3, integration, 0.5);
            b * (war - share * total_loss) - 0.05 * s * s
        };
        let best = |share: f64, integration: f64| -> f64 {
            let mut best_s = 0.0;
            let mut best_value = f64::NEG_INFINITY;
            // A fine grid, because the shift between the two optima at low integration is
            // real but small, and a coarse grid would report a spurious equality.
            let mut s = 0.0;
            while s <= 1.5 {
                let value = objective(s, share, integration);
                if value > best_value {
                    best_value = value;
                    best_s = s;
                }
                s += 0.0002;
            }
            best_s
        };

        for integration in [0.2, 0.6, 1.0] {
            let private = best(0.25, integration);
            let social = best(1.00, integration);
            assert!(
                private > social,
                "E = {integration}: private injection {private} should exceed the planner's {social}"
            );
        }

        // The marginal wedge is (1 - share) * L(E) * b'(s); it is positive wherever more
        // injection still buys belief, and it is what grows with integration.
        let wedge = |s: f64, integration: f64| -> f64 {
            let h = 1e-5;
            let d_belief = (belief(s + h) - belief(s - h)) / (2.0 * h);
            0.75 * loss.loss(0.3, integration, 0.5) * d_belief
        };
        let s = 0.30;
        assert!(wedge(s, 0.0) > 0.0, "the wedge must be positive where belief is rising");
        let mut previous = wedge(s, 0.0);
        for step in 1..=10 {
            let integration = step as f64 / 10.0;
            let value = wedge(s, integration);
            assert!(
                value > previous,
                "the wedge must grow with integration: {value} at E = {integration} vs {previous}"
            );
            previous = value;
        }
    }

    /// Theorem 6: the same mass costs less in more instalments, there is an interior
    /// optimum below a critical time cost, and above it the bloc never stops.
    #[test]
    fn the_instalment_trade_off_has_an_interior_optimum_below_a_critical_time_cost() {
        let params = CredibilityParams::default();
        let mass = 1.0;

        // (ii) same mass, more instalments, less damage.
        let mut previous = f64::INFINITY;
        for t in [1.0, 2.0, 4.0, 8.0, 16.0, 64.0] {
            let value = instalment_damage(&params, mass, t);
            assert!(
                value < previous,
                "T = {t}: damage {value} should be below the previous {previous}"
            );
            previous = value;
        }
        assert!(instalment_damage(&params, mass, 1e6) < 1e-3);

        // (i) the critical time cost is 4*chi*exp(-2)/mu; below it there is an optimum,
        // above it the optimum runs away.
        let critical = 4.0 * params.damage * (-2.0_f64).exp() / params.hazard;
        // A mass large enough that the interior optimum exceeds one instalment; with
        // M = 1 the continuous optimum sits below T = 1 and the answer is the corner.
        let mass = 4.0;
        let below = instalment_optimum(&params, mass, 0.5 * critical)
            .expect("an interior optimum below the critical time cost");
        assert!(below.is_finite() && below > 1.0, "expected an interior T, got {below}");
        assert!(
            instalment_optimum(&params, mass, 1.5 * critical).is_none(),
            "above the critical time cost the optimum must be unbounded"
        );

        // The first-order condition is actually satisfied at the reported optimum.
        let h = 1e-3;
        let total = |t: f64| instalment_damage(&params, mass, t) + 0.5 * critical * t;
        let slope = (total(below + h) - total(below - h)) / (2.0 * h);
        assert!(slope.abs() < 1e-2, "the optimum must be stationary, slope {slope}");
    }

    /// The layer must be inert when switched off. Published figures depend on it, and
    /// the repository's rule is that a new knob defaults neutral.
    #[test]
    fn the_layer_is_off_by_default() {
        let params = InformationParams::default();
        assert!(!params.enabled);
        assert!(params.effort.is_empty());
        assert!(params.influence.contamination.is_empty());
    }

    /// The default Council is the one the paper reasons about, so the numbers it is
    /// built from are pinned rather than left to drift with an edit.
    #[test]
    fn the_default_council_has_the_structure_the_theorem_uses() {
        let council = Council::default();
        assert_eq!(council.size(), 15);
        assert_eq!(council.quota, 9);
        assert_eq!(council.permanent, 5);
    }

    /// Spectral radius of a dense matrix, by the same power iteration, for tests that
    /// need to check the identity rather than assume it.
    fn spectral_radius_dense(m: &[Vec<f64>]) -> f64 {
        let n = m.len();
        if n == 0 {
            return 0.0;
        }
        let mut v = vec![1.0 / (n as f64).sqrt(); n];
        let mut eigenvalue = 0.0;
        for _ in 0..5000 {
            let mut next = vec![0.0; n];
            for (slot, row) in next.iter_mut().zip(m) {
                *slot = row.iter().zip(&v).map(|(a, b)| a * b).sum();
            }
            let norm = next.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm <= 1e-15 {
                return 0.0;
            }
            // Rayleigh quotient from the raw product, as in `spectral_radius`.
            let lambda: f64 = v.iter().zip(&next).map(|(a, b)| a * b).sum();
            for x in next.iter_mut() {
                *x /= norm;
            }
            eigenvalue = lambda;
            v = next;
        }
        eigenvalue.abs()
    }
}
