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

//! The psychological layer: how assistance removes the practice that builds the capacity to
//! judge, and how the tempo that forces assistance then closes the loop.
//!
//! # The claim, in one paragraph
//!
//! The stated risk of AI at work is laziness. The real one is quieter and structural.
//! Assistance substitutes for the person's *output*, so the person receives the output
//! without the **practice** that produced it — and the artistic, emotional, social and
//! psychological capacities are built by practice and decay without it. Meanwhile the
//! machine sets the tempo, and the tempo sets what counts as fast enough. So the person is
//! not choosing to lean on the machine; they are *tracking a reference they did not set*,
//! and the tracking is what removes the practice. That is the machine-to-human inversion:
//! not a machine that feels, but a person who adapts to a rhythm and then finds the capacity
//! to disagree has gone.
//!
//! # The one equation that carries the argument
//!
//! `autonomy = self_model × capacity × (1 − assistance)`
//!
//! Each factor is *necessary*: any of them at zero gives zero autonomy, whatever the others
//! are. Capacity without self-knowledge is competence that cannot say what it wants.
//! Self-knowledge without capacity is an opinion with nothing behind it. Both, without doing
//! the work, is a person who has outsourced the judgement while retaining the vocabulary for
//! it. This is `autonomy()`, and it is the model's formalisation of the argument that people
//! who have not investigated who they are can be led.
//!
//! # The loop, and why it is invisible from inside a single period
//!
//! Assistance buys throughput *now* and costs capacity *later*, and capacity is what catches
//! errors. So a myopic objective is maximised by maximal assistance at every instant, and the
//! cost never appears in the period where the choice is made. `ModelParameters` therefore
//! has no utility function to optimise in the ordinary sense: what it has is a threshold, and
//! the threshold is about the machine's tempo rather than about anyone's preference.
//!
//! # What every number in this module is
//!
//! **Declared.** There is no published measurement of a "substitution coefficient", an
//! "emotional coherence" stock, or the rate at which a person's judgement tracks a machine's
//! register. These are not quantities that have been estimated and then rounded; they are
//! quantities for which no estimate exists, and `ModelParameters::provenance_table` says so
//! alongside every one of them. The model's contribution is therefore its **structure** and
//! its **thresholds**, never its levels. A reader who wants to refute it should refute the
//! structure — in particular A1, A3 and A5 below — because the arithmetic will follow.

/// Where a number came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Published and citable, at the named source and vintage.
    Sourced {
        source: &'static str,
        vintage: &'static str,
    },
    /// Invented: plausible in sign and rough magnitude, not calibrated.
    Declared { rationale: &'static str },
}

impl Provenance {
    pub fn label(self) -> &'static str {
        match self {
            Provenance::Sourced { .. } => "sourced",
            Provenance::Declared { .. } => "declared",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Provenance::Sourced { source, .. } => source,
            Provenance::Declared { rationale } => rationale,
        }
    }
}

/// The two channels through which assistance acts on capacity.
///
/// They are **not** two terms in one sum, and conflating them was the model's first
/// structural error. Assistance does not make practice both better and worse; it removes reps
/// from *producing* and adds efficiency to *learning*. Those are different activities, chosen
/// by how the period is spent, and the whole trade-off lives in that choice — which is why the
/// runway, meaning time available to learn before the bar rises, is the model's policy lever.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Channels {
    /// Reps removed from a producing period per unit of assistance, at full intensity. A1.
    pub substitution: f64,
    /// Extra learning efficiency per unit of assistance, in a learning period. A2.
    pub scaffolding: f64,
}

impl Channels {
    /// Reps generated by a period spent **producing**, at assistance `a`.
    ///
    /// Only the substitution channel acts here: the machine produces, so the person does not
    /// rehearse the thing being produced.
    pub fn production_practice(&self, assistance: f64) -> f64 {
        (1.0 - self.substitution * assistance.clamp(0.0, 1.0)).max(0.0)
    }

    /// Reps generated by a period spent **learning**, at assistance `a`.
    ///
    /// Only the scaffolding channel acts here: a competent tutor makes study more productive,
    /// whatever it does to the student's rehearsal of production.
    pub fn learning_practice(&self, assistance: f64) -> f64 {
        1.0 + self.scaffolding * assistance.clamp(0.0, 1.0)
    }

    /// The share of a period spent learning at which the two channels balance, at full
    /// assistance.
    ///
    /// `(1 − t)(1 − σ) + t(1 + λ) = 1` gives `t* = σ / (σ + λ)`. Below it, assistance is net
    /// harmful to capacity; above it, net beneficial. This is the quantity the runway is
    /// trying to move, and it is the model's answer to the question of how much a person has
    /// to be allowed to learn in order to keep the tool from consuming them.
    pub fn break_even_learning_share(&self) -> Option<f64> {
        let total = self.substitution + self.scaffolding;
        if total <= 0.0 {
            None
        } else {
            Some((self.substitution / total).clamp(0.0, 1.0))
        }
    }
}

/// The whole parameter set. Every field is declared; see the module documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelParameters {
    pub horizon_years: u32,
    /// Assistance intensity at the start, before the tempo forces it up. Assumption A6.
    pub initial_assistance: f64,
    /// Throughput multiplier gained per unit of assistance: output is `1 + gain * a`.
    pub throughput_gain: f64,
    pub channels: Channels,
    /// Capacity built per unit of practice. Assumption A3.
    pub capacity_build: f64,
    /// Capacity lost per year from disuse. Assumption A3.
    pub capacity_decay: f64,
    /// Quality floor with no capacity left: what an unskilled person still catches. A4.
    pub quality_floor: f64,
    /// Growth rate of the machine's tempo, which is also the growth rate of what counts as
    /// fast enough. Assumption A5, and the model's single most consequential parameter.
    pub tempo_growth: f64,
    /// How many years the employer waits before raising expectations to the machine's tempo.
    /// The **runway**.
    pub expectation_lag_years: u32,
    /// The bar expectations rise *to* and then stop at, as a multiple of the pre-AI baseline.
    /// Assumption A10.
    ///
    /// Expectations cannot compound without bound. The bar is set by what the work is worth to
    /// whoever commissions it, not by how fast the machine can go, so it saturates — eventually
    /// at the level the output is worth paying for. Leaving it unbounded is not a stronger
    /// assumption but an unexamined one, and it is where the first version of this model broke:
    /// with a 10% tempo over thirty years the bar reached fourteen times baseline, every person in
    /// the cohort delegated everything, autonomy was identically zero, and the cohort collapsed to
    /// a single point because the fully-delegated state is the same for everyone. A population
    /// model that has stopped describing a population is a bug, not a finding, and the unbounded
    /// exponential was the defect rather than the saturation being the result.
    pub expectation_ceiling: f64,
    /// Relative dispersion of the settled bar across people. Assumption A11.
    ///
    /// The ceiling on its own is not enough to make this a population model, and finding that out
    /// cost a second collapse. With one bar for everyone the recursion has a single stable fixed
    /// point, so every person converges to the same capacity however differently they started:
    /// the cohort reported `p10 = median = mean` and the summary's dispersion statistics measured
    /// nothing. A role's bar settles where the work is worth paying for, and roles differ, so the
    /// settled bar is drawn per person as a relative spread around the model's value. This is what
    /// makes "who is deskilled" a question the model can answer rather than one it averages away.
    pub expectation_ceiling_spread: f64,
    /// Share of a period spent on deliberate learning rather than on output.
    pub training_share: f64,
    /// Rate at which self-knowledge accumulates from that deliberate work. Assumption A7.
    pub self_model_build: f64,
    /// Share of a person's interaction that is with other people rather than with a machine.
    pub human_share: f64,
    /// Emotional coherence built by human co-regulation. Assumption A8.
    pub co_regulation: f64,
    /// Emotional coherence eroded by entrainment to a machine register. Assumption A8.
    pub entrainment: f64,
    /// Share of defects that are *characteristic* of the machine and therefore invisible to
    /// a test the same machine wrote. Assumption A9.
    pub shared_blind_spot: f64,
    /// Detection rate of an independent human reviewer at full capacity.
    pub human_detection: f64,
    /// Detection rate of the machine on its own characteristic defects.
    pub machine_detection: f64,
}

impl Default for ModelParameters {
    fn default() -> Self {
        ModelParameters {
            horizon_years: 30,
            initial_assistance: 0.2,
            throughput_gain: 1.5,
            channels: Channels {
                substitution: 0.8,
                scaffolding: 1.4,
            },
            capacity_build: 0.35,
            capacity_decay: 0.06,
            quality_floor: 0.25,
            tempo_growth: 0.10,
            expectation_lag_years: 2,
            expectation_ceiling: 1.5,
            expectation_ceiling_spread: 0.25,
            training_share: 0.05,
            self_model_build: 0.30,
            human_share: 0.55,
            co_regulation: 0.60,
            entrainment: 0.35,
            shared_blind_spot: 0.35,
            human_detection: 0.85,
            machine_detection: 0.20,
        }
    }
}

impl ModelParameters {
    /// Every parameter with its provenance, in the same shape the rest of the repository uses.
    pub fn provenance_table(&self) -> Vec<(&'static str, Provenance)> {
        const DECLARED: Provenance = Provenance::Declared {
            rationale: "no published estimate exists for this quantity; the value is chosen \
                        to be plausible in sign and magnitude and is not calibrated",
        };
        vec![
            ("substitution coefficient", DECLARED),
            ("scaffolding coefficient", DECLARED),
            ("capacity build rate", DECLARED),
            ("capacity decay rate", DECLARED),
            ("quality floor", DECLARED),
            ("machine tempo growth", DECLARED),
            ("expectation lag (runway)", DECLARED),
            ("expectation ceiling", DECLARED),
            ("expectation ceiling spread", DECLARED),
            ("self-model build rate", DECLARED),
            ("co-regulation coefficient", DECLARED),
            ("entrainment coefficient", DECLARED),
            ("shared blind spot share", DECLARED),
            ("human detection rate", DECLARED),
            ("machine detection rate", DECLARED),
        ]
    }
}

/// One person's state at one point in time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub year: u32,
    /// Assistance actually used, which is the larger of what was chosen and what the tempo
    /// forced. This is the inversion made numeric.
    pub assistance: f64,
    pub capacity: f64,
    pub self_model: f64,
    /// `self_model × capacity × (1 − assistance)`. Zero if any factor is zero.
    pub autonomy: f64,
    /// Quality of what the person *catches*, on a 0..1 scale.
    pub quality: f64,
    /// Output volume relative to the pre-AI baseline.
    pub throughput: f64,
    /// Throughput × quality: the figure that actually counts, and the one that does not
    /// monotonically rise with assistance.
    pub effective_output: f64,
    /// Share of defects that survive, given who wrote the tests.
    pub residual_defect_rate: f64,
    /// Emotional coherence, in equilibrium for the prevailing interaction mix.
    pub emotional_coherence: f64,
}

/// The autonomy equation, isolated so it can be tested on its own.
///
/// Three factors, each necessary. The multiplication is the whole claim: autonomy is not the
/// sum of competence and self-knowledge and not the better of them, because each is useless
/// for this purpose without the others.
pub fn autonomy(self_model: f64, capacity: f64, assistance: f64) -> f64 {
    self_model.clamp(0.0, 1.0) * capacity.clamp(0.0, 1.0) * (1.0 - assistance.clamp(0.0, 1.0))
}

/// Quality: what the person still catches, given their capacity.
///
/// Linear between the floor and one. Linear is a modelling choice rather than a finding, but
/// it is the weakest shape that preserves the two endpoints the argument needs: no capacity
/// still catches something, and full capacity catches nearly everything.
pub fn quality(capacity: f64, quality_floor: f64) -> f64 {
    quality_floor + (1.0 - quality_floor) * capacity.clamp(0.0, 1.0)
}

/// The assistance a person **must** use to meet an expectation, given their own quality.
///
/// This is the model's central mechanism and the reason the argument is not about laziness.
/// Assistance is not chosen freely: output has to clear a bar, the bar rises with the
/// machine's tempo, and the only way to raise output within a period is more assistance. So a
/// person who would prefer to keep the practice cannot, without falling behind a standard they
/// did not set.
///
/// Returns zero when the person already clears the bar unaided.
pub fn required_assistance(expectation: f64, capacity: f64, params: &ModelParameters) -> f64 {
    let q = quality(capacity, params.quality_floor);
    if q <= 0.0 || params.throughput_gain <= 0.0 {
        return 1.0;
    }
    let needed = expectation / q;
    ((needed - 1.0) / params.throughput_gain).clamp(0.0, 1.0)
}

/// The expectation a person faces in a given year: what counts as fast enough.
///
/// Flat at the pre-AI baseline until the runway elapses, then rising at the machine's tempo, and
/// finally flat again at the declared ceiling (A10). The two flat segments are not padding: the
/// first is the runway, which is the only thing in the model anyone can grant, and the second is
/// the reason a person has a steady state at all. Without it the bar outruns any assistance a
/// person can give, everyone ends fully delegated, and the cohort becomes one point.
pub fn expectation_at(year: u32, params: &ModelParameters) -> f64 {
    let ceiling = params.expectation_ceiling.max(1.0);
    if year < params.expectation_lag_years {
        return 1.0;
    }
    let elapsed = f64::from(year - params.expectation_lag_years);
    (1.0 + params.tempo_growth).powf(elapsed).min(ceiling)
}

/// The residual defect rate when the same machine writes the artefact and its tests.
///
/// # Why this has a floor
///
/// A share `ω` of defects are characteristic of the machine's own distribution — the failure
/// modes it does not represent, and therefore does not think to test for. A test suite written
/// by the same process shares that blind spot, so its detection rate on those defects is
/// `machine_detection` rather than the human reviewer's. The other share is caught by whatever
/// independent review remains, and that review weakens as capacity falls.
///
/// Therefore the residual rate is `ω(1 − ρ_m) + (1 − ω)(1 − ρ_h(h))`, and as `h → 0` it tends
/// to `ω(1 − ρ_m) + (1 − ω)` — bounded below by `ω(1 − ρ_m)`, **which no amount of assistance
/// removes**. Perfection is not purchasable, and the floor is set by the correlation, not by
/// the effort.
pub fn residual_defect_rate(capacity: f64, params: &ModelParameters) -> f64 {
    let omega = params.shared_blind_spot.clamp(0.0, 1.0);
    let q = quality(capacity, 0.0);
    let independent = params.human_detection * q;
    omega * (1.0 - params.machine_detection) + (1.0 - omega) * (1.0 - independent)
}

/// The floor the residual defect rate cannot fall below, however good the assistance.
pub fn residual_defect_floor(params: &ModelParameters) -> f64 {
    let omega = params.shared_blind_spot.clamp(0.0, 1.0);
    omega * (1.0 - params.machine_detection)
}

/// Emotional coherence in equilibrium for a given interaction mix.
///
/// Coherence is built by interaction with other people and worn by entrainment to a machine
/// register. At human share `H` the equilibrium is `αH / (αH + β(1−H))`, which is monotone in
/// `H` and gives a clean threshold: coherence falls below a half once the machine's share
/// exceeds the point where `β(1−H) > αH`.
pub fn emotional_coherence(human_share: f64, params: &ModelParameters) -> f64 {
    let h = human_share.clamp(0.0, 1.0);
    let build = params.co_regulation * h;
    let wear = params.entrainment * (1.0 - h);
    let total = build + wear;
    if total <= 0.0 {
        0.0
    } else {
        build / total
    }
}

/// The human share at which emotional coherence falls to `target`.
///
/// The inversion condition for the emotional channel: below this mix, the machine's register
/// dominates the person's equilibrium regardless of anything else.
pub fn critical_human_share(target: f64, params: &ModelParameters) -> Option<f64> {
    let target = target.clamp(0.0, 1.0);
    if params.co_regulation <= 0.0 || params.entrainment <= 0.0 {
        return None;
    }
    // alpha H = target (alpha H + beta (1-H))  =>  H (alpha(1-target)) = target beta (1-H)
    // => H = target beta / (alpha (1 - target) + target beta)
    let numerator = target * params.entrainment;
    let denominator = params.co_regulation * (1.0 - target) + target * params.entrainment;
    if denominator <= 0.0 {
        None
    } else {
        Some((numerator / denominator).clamp(0.0, 1.0))
    }
}

/// The capacity change over one period.
///
/// A period is split between producing and learning. Producing builds capacity out of reps,
/// and assistance removes reps from producing. Learning builds capacity more directly, and
/// assistance makes learning more efficient. The build is concave in `h` through `(1 − h)`, so
/// capacity approaches its ceiling without passing it; disuse erodes it proportionally through
/// `δ h`.
///
/// **This split is the model.** A person who spends the period producing under high assistance
/// loses; a person who spends it learning under high assistance gains; and which of those
/// happens is not a property of the tool but of whether anyone was given the time.
pub fn capacity_change(
    assistance: f64,
    capacity: f64,
    training_share: f64,
    params: &ModelParameters,
) -> f64 {
    let h = capacity.clamp(0.0, 1.0);
    let t = training_share.clamp(0.0, 1.0);
    let producing = (1.0 - t) * params.channels.production_practice(assistance);
    let learning = t * params.channels.learning_practice(assistance);
    let build = params.capacity_build * (producing + learning) * (1.0 - h);
    let decay = params.capacity_decay * h;
    build - decay
}

/// The **substitution coefficient above which full assistance deskills** at a given capacity
/// and training share.
///
/// Solve `build < decay` at `a = 1`:
///
/// ```text
/// κ[(1 − t)(1 − σ) + t(1 + λ)](1 − h) < δ h
/// (1 − t)(1 − σ) < δ h / (κ(1 − h)) − t(1 + λ)
/// σ > 1 − [δ h / (κ(1 − h)) − t(1 + λ)] / (1 − t)
/// ```
///
/// The `(1 − t)` divides the whole bracket, not just the `κ` term. Getting that wrong is easy
/// and silent: the wrong inversion still returns a number in `[0, 1]` for ordinary parameters,
/// so it looks like a threshold and the dynamics quietly disagree with it. The test below
/// probes the dynamics on both sides, which is the only thing that catches this.
///
/// Returns `None` when the inversion lands outside the admissible range of the coefficient —
/// above one, where no substitution is strong enough to deskill at this capacity and training
/// share, or at `h = 1`, where there is no capacity left to build. Substitution is bounded by
/// one because a period cannot lose more reps than it has ([`Channels::production_practice`]
/// clamps there), so a threshold above one is not a threshold but a statement that the region
/// has none. Returning the number anyway was a bug worth recording: the plotting check in
/// `tools/plot_psyche.py` refused to draw it, because a "substitution coefficient" of 1.13 is
/// not something the model can be asked about.
pub fn deskilling_threshold(
    capacity: f64,
    training_share: f64,
    params: &ModelParameters,
) -> Option<f64> {
    let h = capacity.clamp(0.0, 1.0);
    let t = training_share.clamp(0.0, 1.0);
    if h >= 1.0 || params.capacity_build <= 0.0 || t >= 1.0 {
        return None;
    }
    let producing_share = 1.0 - t;
    let slack = 1.0 - h;
    // Invert: (1 − t)(1 − σ) + t(1 + λ) < δ h / (κ (1 − h))
    let demand = params.capacity_decay * h / (params.capacity_build * slack);
    let threshold = 1.0 - (demand - t * (1.0 + params.channels.scaffolding)) / producing_share;
    if threshold > 1.0 {
        None
    } else {
        Some(threshold.max(0.0))
    }
}

/// The equilibrium capacity for a fixed assistance level, or `None` when assistance is high
/// enough that the only equilibrium is collapse.
///
/// Found by bisection on a monotone difference rather than in closed form: the build is
/// quadratic in `h` and a closed form would be worth less than the monotonicity it rests on.
pub fn equilibrium_capacity(
    assistance: f64,
    training_share: f64,
    params: &ModelParameters,
) -> Option<f64> {
    let growth = |h: f64| capacity_change(assistance, h, training_share, params);
    // At h = 0 growth is non-negative; if the function is already non-positive there, the
    // only fixed point is the collapsed one.
    if growth(1e-9) <= 0.0 {
        return None;
    }
    // If it is still positive at h -> 1 there is no interior crossing: capacity is pinned at
    // its ceiling by assistance that adds practice.
    if growth(1.0 - 1e-9) >= 0.0 {
        return Some(1.0);
    }
    let (mut low, mut high) = (0.0_f64, 1.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if growth(mid) > 0.0 {
            low = mid;
        } else {
            high = mid;
        }
        if (high - low) < 1e-12 {
            break;
        }
    }
    Some(0.5 * (low + high))
}

/// How one person's run is driven.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Strategy {
    /// A fixed assistance level, whatever the tempo does.
    Fixed(f64),
    /// Assistance is whatever the expectation forces, which is how the loop actually runs.
    Tracking,
}

/// The assistance level that maximises mean effective output over the horizon.
///
/// Grid search rather than an optimum: effective output is a product of a linear throughput and
/// a capacity path that is itself the solution of a recurrence, so the exact optimum has no
/// closed form worth trusting. A grid is also the honest object for the claim being made — the
/// question is whether the best available level is below one, which a grid answers stably,
/// whereas a first-order condition would have to assume a shape the model does not have.
///
/// Returns `(level, mean effective output)`. The level is on the same 0.01 grid the figures use.
pub fn best_fixed_assistance(
    capacity_0: f64,
    self_model_0: f64,
    params: &ModelParameters,
    human_share: f64,
) -> (f64, f64) {
    let mut best = (0.0_f64, f64::NEG_INFINITY);
    for step in 0..=100 {
        let level = f64::from(step) / 100.0;
        let path = simulate(
            capacity_0,
            self_model_0,
            params,
            Strategy::Fixed(level),
            human_share,
        );
        if path.is_empty() {
            continue;
        }
        let mean = path.iter().map(|s| s.effective_output).sum::<f64>() / path.len() as f64;
        if mean > best.1 {
            best = (level, mean);
        }
    }
    best
}

/// Run one person's trajectory.
///
/// A person with `Strategy::Tracking` is not making a choice: at each period the expectation
/// has risen with the machine's tempo, and assistance is set to whatever clears it
/// ([`required_assistance`]). Their `Strategy::Fixed` counterpart is the counterfactual — the
/// same person under the same tempo who somehow holds assistance constant — and the gap
/// between the two trajectories is the model's measure of what the tempo costs.
pub fn simulate(
    capacity_0: f64,
    self_model_0: f64,
    params: &ModelParameters,
    strategy: Strategy,
    human_share: f64,
) -> Vec<State> {
    let mut states = Vec::with_capacity(params.horizon_years as usize);
    let mut capacity = capacity_0.clamp(0.0, 1.0);
    let mut self_model = self_model_0.clamp(0.0, 1.0);
    let coherence = emotional_coherence(human_share, params);

    for year in 0..params.horizon_years {
        let expectation = expectation_at(year, params);

        let assistance = match strategy {
            Strategy::Fixed(level) => level.clamp(0.0, 1.0),
            Strategy::Tracking => {
                required_assistance(expectation, capacity, params).max(params.initial_assistance)
            }
        };

        // The period's learning. Producing and learning draw on the same finite period, and
        // the training share is the only thing that decides which of the two channels reaches
        // the person's capacity. The self-model is built by the learning part alone, which is
        // why an employer who buys a longer runway is buying inner work as well as skill.
        capacity = (capacity
            + capacity_change(assistance, capacity, params.training_share, params))
        .clamp(0.0, 1.0);
        self_model = (self_model
            + params.self_model_build * params.training_share * (1.0 - self_model))
            .clamp(0.0, 1.0);

        let q = quality(capacity, params.quality_floor);
        let throughput = 1.0 + params.throughput_gain * assistance;
        states.push(State {
            year,
            assistance,
            capacity,
            self_model,
            autonomy: autonomy(self_model, capacity, assistance),
            quality: q,
            throughput,
            effective_output: throughput * q,
            residual_defect_rate: residual_defect_rate(capacity, params),
            emotional_coherence: coherence,
        });
    }
    states
}

/// What a whole cohort's run comes to, in the figures worth reporting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortSummary {
    pub persons: usize,
    /// Mean capacity at the horizon.
    pub mean_capacity: f64,
    pub p10_capacity: f64,
    pub median_capacity: f64,
    /// Mean autonomy at the horizon.
    pub mean_autonomy: f64,
    /// Share of people whose autonomy has fallen below the declared threshold.
    pub share_led: f64,
    /// Mean effective output at the horizon, and its peak across the run.
    pub mean_effective_output: f64,
    pub peak_effective_output: f64,
    /// Mean residual defect rate at the horizon.
    pub mean_residual_defect_rate: f64,
    pub mean_emotional_coherence: f64,
    /// Mean year at which autonomy first fell below the threshold, when it did.
    pub mean_year_autonomy_lost: f64,
    /// Mean number of years spent above the autonomy floor, across the whole run.
    ///
    /// This is the measure the runway acts on. A person can dip below the floor and climb back
    /// above it — autonomy rises again as the self-model builds — so a terminal snapshot and a
    /// first-crossing year both miss what the runway buys, which is *time spent judging* rather
    /// than a different destination.
    pub mean_autonomous_years: f64,
}

/// One person's heterogeneity, drawn or supplied.
#[derive(Debug, Clone, Copy)]
pub struct Person {
    pub capacity_0: f64,
    pub self_model_0: f64,
    pub human_share: f64,
    /// The bar this person's role settles at. Drawn per person because it is the one asymmetry
    /// that survives to the horizon: with a common bar every person converges to the same state
    /// and the cohort stops being a cohort.
    pub expectation_ceiling: f64,
}

/// Summarise a cohort of trajectories.
///
/// `autonomy_floor` is a **declared** threshold rather than a derived one: there is no
/// measurement of the point at which a person has stopped judging for themselves, so the
/// figure is a parameter of the analysis and the caller has to say what it is.
pub fn summarise(trajectories: &[Vec<State>], autonomy_floor: f64) -> CohortSummary {
    let persons = trajectories.len();
    if persons == 0 {
        return CohortSummary {
            persons: 0,
            mean_capacity: f64::NAN,
            p10_capacity: f64::NAN,
            median_capacity: f64::NAN,
            mean_autonomy: f64::NAN,
            share_led: f64::NAN,
            mean_effective_output: f64::NAN,
            peak_effective_output: f64::NAN,
            mean_residual_defect_rate: f64::NAN,
            mean_emotional_coherence: f64::NAN,
            mean_year_autonomy_lost: f64::NAN,
            mean_autonomous_years: f64::NAN,
        };
    }
    let last = |path: &Vec<State>| path.last().copied();
    let mut finals: Vec<State> = trajectories.iter().filter_map(last).collect();
    finals.sort_by(|a, b| {
        a.capacity
            .partial_cmp(&b.capacity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let n = finals.len().max(1);
    let mean = |f: &dyn Fn(&State) -> f64| finals.iter().map(f).sum::<f64>() / n as f64;

    let led = finals
        .iter()
        .filter(|s| s.autonomy < autonomy_floor)
        .count();
    let lost_years: Vec<f64> = trajectories
        .iter()
        .filter_map(|path| {
            path.iter()
                .find(|s| s.autonomy < autonomy_floor)
                .map(|s| f64::from(s.year))
        })
        .collect();

    CohortSummary {
        persons,
        mean_capacity: mean(&|s| s.capacity),
        p10_capacity: finals[finals.len() / 10].capacity,
        median_capacity: finals[finals.len() / 2].capacity,
        mean_autonomy: mean(&|s| s.autonomy),
        share_led: led as f64 / n as f64,
        mean_effective_output: mean(&|s| s.effective_output),
        peak_effective_output: trajectories
            .iter()
            .filter_map(|path| {
                path.iter()
                    .map(|s| s.effective_output)
                    .fold(None, |acc: Option<f64>, v| {
                        Some(acc.map_or(v, |a: f64| a.max(v)))
                    })
            })
            .fold(0.0_f64, f64::max),
        mean_residual_defect_rate: mean(&|s| s.residual_defect_rate),
        mean_emotional_coherence: mean(&|s| s.emotional_coherence),
        mean_year_autonomy_lost: if lost_years.is_empty() {
            f64::NAN
        } else {
            lost_years.iter().sum::<f64>() / lost_years.len() as f64
        },
        mean_autonomous_years: trajectories
            .iter()
            .map(|path| {
                path.iter()
                    .filter(|state| state.autonomy >= autonomy_floor)
                    .count() as f64
            })
            .sum::<f64>()
            / n as f64,
    }
}

/// Draw a heterogeneous cohort.
///
/// The spread is declared, and the seed makes two policies comparable on the *same* people —
/// which is the whole reason a Monte Carlo is used here rather than a single trajectory with
/// three different parameters.
///
/// Three axes are drawn independently and one is drawn around the model's own value: the settled
/// bar. That last one is not decoration. The other three converge — capacity and self-knowledge
/// are transients of a recursion with a single attractor, and the interaction mix is a level, not
/// a state — so without a per-person bar every person in the cohort ends in the same place and
/// the only thing the Monte Carlo measures is how long it took them to get there.
pub fn draw_cohort(persons: usize, params: &ModelParameters, seed: u64) -> Vec<Person> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);
    let spread = params.expectation_ceiling_spread.clamp(0.0, 0.95);
    (0..persons)
        .map(|_| {
            // Capability and self-knowledge are drawn independently: a capable person may
            // have done no inner work and vice versa, and that independence is the point of
            // having two factors in the autonomy equation rather than one.
            let capacity_0 = rng.gen_range(0.35_f64..0.95);
            let self_model_0 = rng.gen_range(0.05_f64..0.75);
            let human_share = (params.human_share + rng.gen_range(-0.15_f64..0.15)).clamp(0.0, 1.0);
            // A role whose bar settles high demands more delegation for the same person, which
            // is the mechanism by which two people with identical capability end up differently
            // autonomous. Drawn multiplicatively so the spread stays proportional at any level.
            let expectation_ceiling =
                (params.expectation_ceiling * (1.0 + rng.gen_range(-spread..spread))).max(1.0);
            Person {
                capacity_0,
                self_model_0,
                human_share,
                expectation_ceiling,
            }
        })
        .collect()
}

/// Run a cohort under one strategy, pairing each person with their own draws.
pub fn run_cohort(
    cohort: &[Person],
    params: &ModelParameters,
    strategy: Strategy,
) -> Vec<Vec<State>> {
    cohort
        .iter()
        .map(|person| {
            let mut theirs = params.clone();
            theirs.expectation_ceiling = person.expectation_ceiling;
            simulate(
                person.capacity_0,
                person.self_model_0,
                &theirs,
                strategy,
                person.human_share,
            )
        })
        .collect()
}

/// CSV export, so the figures are drawn from the model rather than beside it.
///
/// # Why export at all
///
/// The claim defended in `THE_PSYCHOLOGICAL_LAYER.md` is a *shape*, not a level: capacity falls
/// under tracked assistance and holds under a fixed one, effective output has an interior
/// optimum for some parameters and not others, and the residual defect rate has a floor no
/// assistance removes. Shapes are hard to see in a table of thirty numbers and easy to see in a
/// line, and a reader who distrusts the prose can look at the curve that carries it.
///
/// # Why the model writes the files
///
/// Every column here comes out of the same functions the tests check. A plotting script that
/// reimplemented `deskilling_threshold` in Python would let the figure and the code disagree,
/// and the disagreement would be invisible: two plausible-looking curves, one of them wrong.
/// Writing the numbers from the model means a wrong threshold is wrong in the same way in the
/// picture as in the code, where it can be seen.
///
/// # What the export deliberately does not contain
///
/// No calibration, and no column that could be mistaken for a measurement. `psyche_provenance.txt`
/// sits beside the data and repeats, in the place where a figure legend would otherwise float
/// free, that every parameter is declared. `export.rs` does the same for the geopolitical layer.
pub mod export {
    use std::fmt::Write as _;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::{
        best_fixed_assistance, critical_human_share, deskilling_threshold, emotional_coherence,
        run_cohort, summarise, ModelParameters, Person, State, Strategy,
    };

    /// Capacity grid for the threshold panel: `0.00 ..= 1.00` in steps of 0.025.
    const CAPACITY_STEPS: usize = 41;
    /// Training shares drawn on the threshold panel. The second is the default, so the figure
    /// always contains the line the text quotes.
    const TRAINING_SHARES: [f64; 4] = [0.0, 0.05, 0.15, 0.30];
    /// Machine tempo growth rates swept.
    const TEMPO_VALUES: [f64; 7] = [0.00, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30];
    /// Longest runway swept, in years between the machine arriving and expectations following.
    const RUNWAY_YEARS: u32 = 8;
    /// Coherence targets marked on the emotional panel.
    const COHERENCE_TARGETS: [f64; 4] = [0.75, 0.60, 0.50, 0.40];
    /// Throughput gains swept for the phase panel.
    const GAIN_VALUES: [f64; 9] = [0.0, 0.25, 0.50, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0];
    /// Substitution values swept for the phase panel.
    const SUBSTITUTION_STEPS: usize = 11;

    /// Write the whole bundle into `dir`, creating it if needed.
    ///
    /// `cohort` is passed in rather than drawn here so the figures describe the *same* people
    /// the CLI reported on: a figure drawn from a second draw of the same seed would be a
    /// different experiment wearing the same name.
    pub fn write_bundle(
        dir: &Path,
        params: &ModelParameters,
        cohort: &[Person],
        autonomy_floor: f64,
        seed: u64,
    ) -> io::Result<Vec<PathBuf>> {
        fs::create_dir_all(dir)?;
        let mut files = Vec::new();
        for (name, body) in [
            ("psyche_trajectories.csv", trajectories_csv(params, cohort)),
            (
                "psyche_cohort.csv",
                cohort_csv(params, cohort, autonomy_floor),
            ),
            (
                "psyche_runway.csv",
                runway_csv(params, cohort, autonomy_floor),
            ),
            (
                "psyche_tempo.csv",
                tempo_csv(params, cohort, autonomy_floor),
            ),
            ("psyche_thresholds.csv", thresholds_csv(params)),
            ("psyche_defects.csv", defects_csv(params)),
            ("psyche_coherence.csv", coherence_csv(params)),
            (
                "psyche_meta.csv",
                meta_csv(params, cohort.len(), autonomy_floor, seed),
            ),
            ("psyche_phase.csv", phase_csv(params)),
            (
                "psyche_provenance.txt",
                provenance_txt(params, cohort.len(), autonomy_floor, seed),
            ),
        ] {
            let path = dir.join(name);
            fs::write(&path, body)?;
            files.push(path);
        }
        Ok(files)
    }

    /// Mean of one field at one year across the cohort, or `NaN` if the year is out of range.
    fn mean_at(paths: &[Vec<State>], year: usize, project: impl Fn(&State) -> f64) -> f64 {
        let mut total = 0.0;
        let mut count = 0_i32;
        for path in paths {
            if let Some(state) = path.get(year) {
                total += project(state);
                count += 1;
            }
        }
        if count == 0 {
            f64::NAN
        } else {
            total / f64::from(count)
        }
    }

    /// The inversion itself: mean tracked trajectory against mean held trajectory, year by year.
    ///
    /// Both arms are the same people: the cohort is drawn once and run twice, so the gap between
    /// the columns is what the tempo costs and not a difference between two groups.
    fn trajectories_csv(params: &ModelParameters, cohort: &[Person]) -> String {
        let tracking = run_cohort(cohort, params, Strategy::Tracking);
        let held = run_cohort(cohort, params, Strategy::Fixed(params.initial_assistance));
        let mut out = String::from(
            "year,track_assistance,hold_assistance,track_capacity,hold_capacity,\
             track_self_model,hold_self_model,track_autonomy,hold_autonomy,\
             track_effective_output,hold_effective_output,track_defect,hold_defect\n",
        );
        for year in 0..params.horizon_years as usize {
            let _ = writeln!(
                out,
                "{year},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},\
                 {:.6},{:.6}",
                mean_at(&tracking, year, |s| s.assistance),
                mean_at(&held, year, |s| s.assistance),
                mean_at(&tracking, year, |s| s.capacity),
                mean_at(&held, year, |s| s.capacity),
                mean_at(&tracking, year, |s| s.self_model),
                mean_at(&held, year, |s| s.self_model),
                mean_at(&tracking, year, |s| s.autonomy),
                mean_at(&held, year, |s| s.autonomy),
                mean_at(&tracking, year, |s| s.effective_output),
                mean_at(&held, year, |s| s.effective_output),
                mean_at(&tracking, year, |s| s.residual_defect_rate),
                mean_at(&held, year, |s| s.residual_defect_rate),
            );
        }
        out
    }

    /// Residual defects against capacity, at three shares of characteristic defects.
    ///
    /// The floor is the panel's point: each curve is flat at the bottom because the correlated
    /// share is caught by neither reviewer, and the flat part is what no assistance removes.
    fn defects_csv(params: &ModelParameters) -> String {
        let mut out = String::from("capacity,shared_blind_spot,residual_defect_rate,floor\n");
        for step in 0..CAPACITY_STEPS {
            let capacity = f64::from(step as u32) / (CAPACITY_STEPS as f64 - 1.0);
            for omega in [0.10, 0.35, 0.60] {
                let mut probe = params.clone();
                probe.shared_blind_spot = omega;
                let _ = writeln!(
                    out,
                    "{capacity:.6},{omega:.4},{:.6},{:.6}",
                    super::residual_defect_rate(capacity, &probe),
                    super::residual_defect_floor(&probe),
                );
            }
        }
        out
    }

    /// One row per person per arm, for the distribution figures.
    ///
    /// The summary can only report a mean; the figure needs the spread, because "mean capacity
    /// fell" and "half the cohort lost most of their capacity while the other half held" are
    /// different findings and a mean cannot tell them apart.
    fn cohort_csv(params: &ModelParameters, cohort: &[Person], autonomy_floor: f64) -> String {
        let mut out = String::from(
            "person,arm,final_capacity,final_autonomy,final_effective_output,\
             final_defect_rate,led,expectation_ceiling\n",
        );
        for (arm, strategy) in [
            ("tracking", Strategy::Tracking),
            ("held", Strategy::Fixed(params.initial_assistance)),
        ] {
            for (index, path) in run_cohort(cohort, params, strategy).iter().enumerate() {
                if let Some(state) = path.last() {
                    let _ = writeln!(
                        out,
                        "{index},{arm},{:.6},{:.6},{:.6},{:.6},{},{:.6}",
                        state.capacity,
                        state.autonomy,
                        state.effective_output,
                        state.residual_defect_rate,
                        u8::from(state.autonomy < autonomy_floor),
                        cohort
                            .get(index)
                            .map(|person| person.expectation_ceiling)
                            .unwrap_or(f64::NAN),
                    );
                }
            }
        }
        out
    }

    /// Mean outcomes against the length of the runway.
    ///
    /// The runway is the one variable in the model an employer or a funder controls directly, so
    /// it is the one curve that is a policy instrument rather than a forecast.
    fn runway_csv(params: &ModelParameters, cohort: &[Person], autonomy_floor: f64) -> String {
        let mut out = String::from(
            "runway_years,mean_capacity,p10_capacity,median_capacity,mean_autonomy,\
             share_led,mean_effective_output,mean_defect,mean_autonomous_years\n",
        );
        for lag in 0..=RUNWAY_YEARS {
            let mut probe = params.clone();
            probe.expectation_lag_years = lag;
            let summary = summarise(
                &run_cohort(cohort, &probe, Strategy::Tracking),
                autonomy_floor,
            );
            let _ = writeln!(
                out,
                "{lag},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                summary.mean_capacity,
                summary.p10_capacity,
                summary.median_capacity,
                summary.mean_autonomy,
                summary.share_led,
                summary.mean_effective_output,
                summary.mean_residual_defect_rate,
                summary.mean_autonomous_years,
            );
        }
        out
    }

    /// Mean outcomes against the tempo the machine sets.
    fn tempo_csv(params: &ModelParameters, cohort: &[Person], autonomy_floor: f64) -> String {
        let mut out = String::from(
            "tempo_growth,mean_capacity,p10_capacity,share_led,mean_autonomy,\
             mean_effective_output,mean_defect,mean_autonomous_years\n",
        );
        for tempo in TEMPO_VALUES {
            let mut probe = params.clone();
            probe.tempo_growth = tempo;
            let summary = summarise(
                &run_cohort(cohort, &probe, Strategy::Tracking),
                autonomy_floor,
            );
            let _ = writeln!(
                out,
                "{tempo:.4},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                summary.mean_capacity,
                summary.p10_capacity,
                summary.share_led,
                summary.mean_autonomy,
                summary.mean_effective_output,
                summary.mean_residual_defect_rate,
                summary.mean_autonomous_years,
            );
        }
        out
    }

    /// The deskilling threshold as a curve, one per training share.
    ///
    /// This is the panel that carries the argument's conditionality: for each capacity level it
    /// gives the substitution coefficient above which full assistance deskills. The gap between
    /// the `0.00` and `0.30` lines is the runway's worth, in the same units as the tool's own
    /// substitutability.
    fn thresholds_csv(params: &ModelParameters) -> String {
        let mut out = String::from("capacity,training_share,sigma_star\n");
        for step in 0..CAPACITY_STEPS {
            let capacity = f64::from(step as u32) / (CAPACITY_STEPS as f64 - 1.0);
            for share in TRAINING_SHARES {
                match deskilling_threshold(capacity, share, params) {
                    Some(value) => {
                        let _ = writeln!(out, "{capacity:.6},{share:.4},{value:.6}");
                    }
                    None => {
                        // Undefined at the ceiling and in a pure-learning period. Written as
                        // NaN rather than dropped, so a reader can tell "undefined here" from
                        // "this row does not exist".
                        let _ = writeln!(out, "{capacity:.6},{share:.4},NaN");
                    }
                }
            }
        }
        out
    }

    /// The emotional channel: coherence against the human share, plus its critical points.
    fn coherence_csv(params: &ModelParameters) -> String {
        let mut out = String::from("section,human_share,value\n");
        for step in 0..=100 {
            let share = f64::from(step) / 100.0;
            let _ = writeln!(
                out,
                "curve,{share:.4},{:.6}",
                emotional_coherence(share, params)
            );
        }
        for target in COHERENCE_TARGETS {
            if let Some(share) = critical_human_share(target, params) {
                let _ = writeln!(out, "target_{target:.2},{share:.6},{target:.6}");
            }
        }
        out
    }

    /// The phase panel: for a representative person, the optimal assistance across the
    /// (throughput gain, substitution) plane.
    ///
    /// This is the figure that keeps the paper honest. It shows *where* maximum assistance is
    /// optimal and where it is not, instead of asserting one answer for all parameters. A
    /// representative person is used because a phase diagram needs a fixed point in the other
    /// dimensions to be readable at all; the values are declared and are the same ones the
    /// tests use, so the diagram and the tests cannot drift apart.
    fn phase_csv(params: &ModelParameters) -> String {
        let capacity = 0.7;
        let self_model = 0.4;
        let mut out = String::from("throughput_gain,substitution,optimal_assistance,best_output\n");
        for gain in GAIN_VALUES {
            for step in 0..SUBSTITUTION_STEPS {
                let substitution = f64::from(step as u32) / (SUBSTITUTION_STEPS as f64 - 1.0);
                let mut probe = params.clone();
                probe.throughput_gain = gain;
                probe.channels.substitution = substitution;
                let (level, output) =
                    best_fixed_assistance(capacity, self_model, &probe, params.human_share);
                let _ = writeln!(out, "{gain:.4},{substitution:.6},{level:.4},{output:.6}");
            }
        }
        out
    }

    /// The scalars a figure needs but that do not belong in any one table.
    ///
    /// The autonomy floor especially: it is drawn on the autonomy panel as a line, and a plotting
    /// script that hard-coded it would keep drawing 0.15 after someone moved it, which is the
    /// quiet kind of disagreement between figure and model that this export exists to prevent.
    fn meta_csv(
        params: &ModelParameters,
        persons: usize,
        autonomy_floor: f64,
        seed: u64,
    ) -> String {
        let mut out = String::from("key,value\n");
        let _ = writeln!(out, "autonomy_floor,{autonomy_floor:.6}");
        let _ = writeln!(out, "horizon_years,{}", params.horizon_years);
        let _ = writeln!(out, "persons,{persons}");
        let _ = writeln!(out, "seed,{seed}");
        let _ = writeln!(out, "tempo_growth,{:.6}", params.tempo_growth);
        let _ = writeln!(
            out,
            "expectation_lag_years,{}",
            params.expectation_lag_years
        );
        let _ = writeln!(out, "expectation_ceiling,{:.6}", params.expectation_ceiling);
        out
    }

    /// The header a figure legend should be read against.
    fn provenance_txt(
        params: &ModelParameters,
        persons: usize,
        autonomy_floor: f64,
        seed: u64,
    ) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "PSYCHOLOGICAL LAYER -- EXPORT PROVENANCE");
        let _ = writeln!(out, "{}", "-".repeat(70));
        let _ = writeln!(
            out,
            "Every parameter below is DECLARED. None is calibrated against data, and no"
        );
        let _ = writeln!(
            out,
            "quantity in these files is an estimate of anything in the world. What the"
        );
        let _ = writeln!(
            out,
            "files support is the SHAPE of the relationships and how they move with the"
        );
        let _ = writeln!(out, "parameters -- not the levels.");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "Simulation: {} people, {} years, seed {seed}, autonomy floor {autonomy_floor:.2}.",
            persons, params.horizon_years
        );
        let _ = writeln!(
            out,
            "The autonomy floor is itself declared: no measurement fixes the point at"
        );
        let _ = writeln!(out, "which a person has stopped judging for themselves.");
        let _ = writeln!(out);
        let _ = writeln!(out, "DECLARED PARAMETERS");
        for (name, provenance) in params.provenance_table() {
            let _ = writeln!(
                out,
                "  {name:<28} {} ({})",
                value_of(name, params),
                provenance.label()
            );
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "The values are printed beside the label so a figure can be traced back to the"
        );
        let _ = writeln!(out, "parameterisation that produced it.");
        out
    }

    /// The current value of a named parameter, for the provenance header.
    ///
    /// Matched by name rather than stored alongside, because `provenance_table` is the public
    /// list and duplicating the values there would let the two disagree.
    fn value_of(name: &str, params: &ModelParameters) -> String {
        let value = match name {
            "substitution coefficient" => params.channels.substitution,
            "scaffolding coefficient" => params.channels.scaffolding,
            "capacity build rate" => params.capacity_build,
            "capacity decay rate" => params.capacity_decay,
            "quality floor" => params.quality_floor,
            "machine tempo growth" => params.tempo_growth,
            "expectation lag (runway)" => f64::from(params.expectation_lag_years),
            "expectation ceiling" => params.expectation_ceiling,
            "expectation ceiling spread" => params.expectation_ceiling_spread,
            "self-model build rate" => params.self_model_build,
            "co-regulation coefficient" => params.co_regulation,
            "entrainment coefficient" => params.entrainment,
            "shared blind spot share" => params.shared_blind_spot,
            "human detection rate" => params.human_detection,
            "machine detection rate" => params.machine_detection,
            _ => f64::NAN,
        };
        format!("{value:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Autonomy must be zero if *any* factor is zero. This is the model's whole claim, so it
    /// is the first thing checked: the multiplication is not decoration, and a sum would let
    /// a person with total self-knowledge and no capacity register as autonomous.
    #[test]
    fn autonomy_is_zero_if_any_factor_is_zero() {
        assert_eq!(autonomy(0.0, 1.0, 0.0), 0.0);
        assert_eq!(autonomy(1.0, 0.0, 0.0), 0.0);
        assert_eq!(autonomy(1.0, 1.0, 1.0), 0.0);
        assert!(close(autonomy(1.0, 1.0, 0.0), 1.0));
        // And it is strictly increasing in each factor.
        assert!(autonomy(0.5, 0.5, 0.5) < autonomy(0.6, 0.5, 0.5));
        assert!(autonomy(0.5, 0.5, 0.5) < autonomy(0.5, 0.6, 0.5));
        assert!(autonomy(0.5, 0.5, 0.5) > autonomy(0.5, 0.5, 0.6));
    }

    /// The two channels must move in opposite directions, and the learning share at which they
    /// balance must actually balance them.
    ///
    /// This replaces a test that encoded the model's first structural error — it had
    /// substitution and scaffolding fighting inside a *single* practice term, so "does
    /// assistance help?" had no answer independent of capacity. The honest structure is that
    /// producing and learning are different activities: assistance removes reps from one and
    /// adds efficiency to the other, and the training share decides which reaches the person.
    #[test]
    fn assistance_removes_production_reps_and_adds_learning_efficiency() {
        let channels = Channels {
            substitution: 0.8,
            scaffolding: 1.4,
        };
        // Producing with assistance yields fewer reps, monotonically.
        assert!(channels.production_practice(0.0) > channels.production_practice(0.5));
        assert!(channels.production_practice(0.5) > channels.production_practice(1.0));
        assert!((channels.production_practice(1.0) - 0.2).abs() < 1e-12);
        // Learning with assistance yields more, monotonically.
        assert!(channels.learning_practice(1.0) > channels.learning_practice(0.0));
        assert!((channels.learning_practice(0.0) - 1.0).abs() < 1e-12);

        // The break-even share must be where the two exactly cancel at full assistance.
        let t = channels
            .break_even_learning_share()
            .expect("channels are nonzero");
        assert!(
            (t - 0.8 / (0.8 + 1.4)).abs() < 1e-12,
            "break-even share is {t}"
        );
        let blended =
            (1.0 - t) * channels.production_practice(1.0) + t * channels.learning_practice(1.0);
        assert!(
            (blended - 1.0).abs() < 1e-12,
            "at the break-even share the blend should be 1.0, got {blended}"
        );
        // Below the break-even share, assistance builds *less* capacity than the same
        // training split without it; above, more. This is the correct comparison and the
        // obvious wrong one is tempting: the break-even share does not mean build equals
        // decay, only that assisted practice equals unassisted practice. Whether the build
        // clears decay at all is a separate question, answered by `deskilling_threshold`.
        let params = ModelParameters::default();
        let low = capacity_change(1.0, 0.5, t * 0.5, &params)
            - capacity_change(0.0, 0.5, t * 0.5, &params);
        assert!(low < 0.0, "assistance built more capacity below break-even");
        let high_share = (t + 0.4).min(1.0);
        let high = capacity_change(1.0, 0.5, high_share, &params)
            - capacity_change(0.0, 0.5, high_share, &params);
        assert!(
            high > 0.0,
            "assistance built less capacity above break-even"
        );
        // And the sign must flip exactly at the break-even share, not near it.
        let at = capacity_change(1.0, 0.5, t, &params) - capacity_change(0.0, 0.5, t, &params);
        assert!(
            at.abs() < 1e-12,
            "the blend is not flat at break-even: {at}"
        );
    }

    /// The deskilling threshold must agree with the dynamics it was derived from.
    ///
    /// This is the check that matters: the closed form is an inequality inversion, and an
    /// inversion is exactly where a sign error hides. The test asserts that setting sigma just
    /// above the threshold drives capacity down and just below it drives capacity up.
    #[test]
    fn the_deskilling_threshold_agrees_with_the_dynamics() {
        let params = ModelParameters::default();
        let h = 0.7;
        let t = params.training_share;
        let threshold = deskilling_threshold(h, t, &params).expect("defined below the ceiling");
        for (delta, expect_fall) in [(-0.05, false), (0.05, true)] {
            let mut probe = params.clone();
            probe.channels.substitution = threshold + delta;
            let change = capacity_change(1.0, h, t, &probe);
            assert_eq!(
                change < 0.0,
                expect_fall,
                "sigma {} : change {change} at capacity {h}",
                probe.channels.substitution
            );
        }
    }

    /// Expectations must rise, then stop.
    ///
    /// The second half of this test is a regression test for a real defect: the first version of
    /// the model compounded the bar for the whole horizon, reached fourteen times baseline in
    /// thirty years, drove every person to full delegation, and left the cohort with one distinct
    /// outcome. The unbounded exponential was the mistake, so the ceiling is asserted rather than
    /// assumed, and the assertion is followed by the population check that it protects.
    #[test]
    fn expectations_rise_to_a_ceiling_and_then_stop() {
        let params = ModelParameters::default();
        assert_eq!(expectation_at(0, &params), 1.0, "the runway must be flat");
        assert_eq!(
            expectation_at(params.expectation_lag_years - 1, &params),
            1.0,
            "the bar moved before the runway elapsed"
        );
        let mut previous = 0.0_f64;
        for year in 0..60 {
            let value = expectation_at(year, &params);
            assert!(
                value >= previous - 1e-12,
                "expectations fell at year {year}"
            );
            assert!(
                value <= params.expectation_ceiling + 1e-12,
                "expectations passed the ceiling at year {year}: {value}"
            );
            previous = value;
        }
        // Long enough and it must actually reach the ceiling, or the ceiling is decoration.
        assert!(
            (expectation_at(200, &params) - params.expectation_ceiling).abs() < 1e-9,
            "the ceiling is never reached, so it cannot be what bounds the model"
        );
        // A machine that sets no tempo leaves the bar where it was.
        let mut still = params.clone();
        still.tempo_growth = 0.0;
        for year in [0, 5, 20, 200] {
            assert_eq!(expectation_at(year, &still), 1.0);
        }
    }

    /// The cohort must remain a cohort: a population model whose every member ends in the same
    /// state is not describing a population.
    ///
    /// This is the test the saturated version failed. `p10 == median == mean` at the horizon is
    /// the signature of a single attractor reached by everyone, and it means any dispersion
    /// statistic in the summary is measuring nothing.
    #[test]
    fn the_cohort_keeps_dispersion_at_the_horizon() {
        let params = ModelParameters::default();
        let cohort = draw_cohort(500, &params, 20_260_101);
        let summary = summarise(&run_cohort(&cohort, &params, Strategy::Tracking), 0.15);
        assert!(
            summary.p10_capacity < summary.median_capacity - 1e-6,
            "the cohort collapsed to one capacity: p10 {} and median {}",
            summary.p10_capacity,
            summary.median_capacity
        );
        assert!(
            summary.mean_autonomy > 0.0,
            "mean autonomy is zero, so every person is fully delegated and the autonomy \
             channel carries no information"
        );
        assert!(
            summary.share_led > 0.0 && summary.share_led < 1.0,
            "the autonomy floor classifies everyone the same way ({}), which makes it a \
             constant rather than a threshold",
            summary.share_led
        );
    }

    /// The runway must buy *time*, and this test pins down that it does not buy a different
    /// destination.
    ///
    /// Both halves matter. The first is the claim the document makes; the second is the model's
    /// honest limitation, and writing it as an assertion means that anyone who changes the
    /// dynamics so that the runway does change the endpoint will be told, loudly, that the
    /// document's central sentence about the runway no longer describes the code.
    #[test]
    fn the_runway_buys_years_of_autonomy_rather_than_a_different_endpoint() {
        let params = ModelParameters::default();
        let cohort = draw_cohort(400, &params, 7_700_042);
        let mut none = params.clone();
        none.expectation_lag_years = 0;
        let mut six = params.clone();
        six.expectation_lag_years = 6;
        let bare = summarise(&run_cohort(&cohort, &none, Strategy::Tracking), 0.15);
        let long = summarise(&run_cohort(&cohort, &six, Strategy::Tracking), 0.15);

        assert!(
            long.mean_autonomous_years > bare.mean_autonomous_years,
            "a longer runway bought no additional years of autonomy: {} then {}",
            bare.mean_autonomous_years,
            long.mean_autonomous_years
        );
        assert!(
            long.mean_autonomous_years - bare.mean_autonomous_years > 0.5,
            "the runway's effect is too small to be the edge the argument claims: {} then {}",
            bare.mean_autonomous_years,
            long.mean_autonomous_years
        );
        assert!(
            (long.mean_capacity - bare.mean_capacity).abs() < 0.05,
            "the runway changed the endpoint, so the model is no longer one in which the bar \
             rather than the path sets the destination: {} then {}",
            bare.mean_capacity,
            long.mean_capacity
        );
    }

    /// The residual defect rate must have a floor and the floor must be the correlated share,
    /// not zero. A model in which enough assistance bought perfection would make the argument
    /// against maximal assistance rest on preference; it rests on arithmetic instead.
    #[test]
    fn the_residual_defect_rate_has_a_floor_that_effort_cannot_remove() {
        let params = ModelParameters::default();
        let floor = residual_defect_floor(&params);
        assert!(close(
            floor,
            params.shared_blind_spot * (1.0 - params.machine_detection)
        ));
        assert!(
            floor > 0.0,
            "the floor should be positive for a correlated tester"
        );
        // As capacity vanishes, the rate approaches the floor plus the undetected remainder of
        // the independent share — never below the floor itself.
        let at_zero = residual_defect_rate(0.0, &params);
        assert!(at_zero >= floor - 1e-12);
        // And more capacity strictly reduces it.
        assert!(residual_defect_rate(0.9, &params) < residual_defect_rate(0.1, &params));
        // If there were no blind spot, assistance could in principle reach perfection.
        let mut none = params.clone();
        none.shared_blind_spot = 0.0;
        assert_eq!(residual_defect_floor(&none), 0.0);
    }

    /// Emotional coherence must rise with the human share of interaction, not fall.
    ///
    /// The first version of this test asserted the wrong direction, and the wrong direction is
    /// plausible enough to be worth naming: the model's risk is that coherence *falls*, so a
    /// test that "checks it moves" in either direction passes while proving nothing. Coherence
    /// is `αH / (αH + β(1−H))`, which is zero when nobody is human and one when everybody is.
    #[test]
    fn emotional_coherence_and_its_critical_share_are_consistent() {
        let params = ModelParameters::default();
        let mut previous = f64::NEG_INFINITY;
        for step in 0..=100 {
            let share = f64::from(step) / 100.0;
            let value = emotional_coherence(share, &params);
            assert!(value >= previous - 1e-12, "coherence fell at share {share}");
            assert!((0.0..=1.0).contains(&value));
            previous = value;
        }
        assert_eq!(emotional_coherence(1.0, &params), 1.0);
        assert_eq!(emotional_coherence(0.0, &params), 0.0);

        let target = 0.6;
        let critical = critical_human_share(target, &params).expect("both coefficients nonzero");
        assert!(
            (emotional_coherence(critical, &params) - target).abs() < 1e-9,
            "coherence at the critical share is {} not {target}",
            emotional_coherence(critical, &params)
        );
        // The critical share is a real threshold: crossing it takes coherence below target,
        // and the direction is the one the argument needs.
        assert!(emotional_coherence(critical - 0.02, &params) < target);
        assert!(emotional_coherence(critical + 0.02, &params) > target);
    }

    /// The required assistance must rise with the expectation and fall with capacity, and it
    /// must be clamped — an expectation nobody can meet cannot demand more than all of it.
    #[test]
    fn required_assistance_rises_with_the_bar_and_falls_with_capacity() {
        let params = ModelParameters::default();
        let a_low = required_assistance(1.0, 0.8, &params);
        let a_high = required_assistance(2.0, 0.8, &params);
        assert!(a_high > a_low, "a higher bar must demand more assistance");
        let a_capable = required_assistance(2.0, 0.9, &params);
        let a_weak = required_assistance(2.0, 0.2, &params);
        assert!(
            a_capable < a_weak,
            "a more capable person should need less: {a_capable} against {a_weak}"
        );
        // Clamped at both ends.
        assert_eq!(required_assistance(1.0, 1.0, &params), 0.0);
        assert_eq!(required_assistance(1e6, 0.0, &params), 1.0);
    }

    /// The equilibrium capacity must be the fixed point of the dynamics it is solved from.
    /// Checked by evolving the system and asserting it stays put, which is what "equilibrium"
    /// means and what a bisection on the wrong sign would not satisfy.
    #[test]
    fn the_equilibrium_capacity_is_a_fixed_point() {
        let params = ModelParameters::default();
        let t = params.training_share;
        for assistance in [0.0, 0.2, 0.5] {
            let Some(h) = equilibrium_capacity(assistance, t, &params) else {
                continue;
            };
            let change = capacity_change(assistance, h, t, &params);
            assert!(
                change.abs() < 1e-6,
                "assistance {assistance}: capacity {h} is not a fixed point (change {change})"
            );
            // And it is stable: a perturbation decays back toward it.
            let up = capacity_change(assistance, (h + 0.02).min(1.0), t, &params);
            let down = capacity_change(assistance, (h - 0.02).max(0.0), t, &params);
            assert!(
                up < 0.0 || h + 0.02 >= 1.0,
                "the fixed point should be stable from above"
            );
            assert!(
                down > 0.0 || h - 0.02 <= 0.0,
                "the fixed point should be stable from below"
            );
        }
    }

    /// Tracking must not leave capacity above the fixed-assistance counterfactual at the same
    /// final assistance level: the tempo is what removes the practice, and a model in which
    /// tracking preserved capacity would have no mechanism in it.
    #[test]
    fn tracking_the_tempo_costs_more_capacity_than_holding_assistance_still() {
        let params = ModelParameters::default();
        let tracked = simulate(0.8, 0.5, &params, Strategy::Tracking, params.human_share);
        let held = simulate(0.8, 0.5, &params, Strategy::Fixed(0.2), params.human_share);
        let tracked_end = tracked.last().expect("non-empty horizon").capacity;
        let held_end = held.last().expect("non-empty horizon").capacity;
        assert!(
            tracked_end < held_end,
            "tracking ended at capacity {tracked_end}, holding at {held_end}"
        );
        // And the trajectory's assistance must actually have risen, or the comparison is empty.
        let first = tracked.first().expect("non-empty").assistance;
        let last = tracked.last().expect("non-empty").assistance;
        assert!(
            last > first,
            "tracked assistance did not rise: {first} to {last}"
        );
    }

    /// The runway must matter, and in the direction claimed: a longer lag before expectations
    /// rise must leave more capacity at the horizon, because it buys years in which the person
    /// practises at low assistance.
    #[test]
    fn a_longer_runway_preserves_capacity() {
        let base = ModelParameters::default();
        let mut previous: Option<(u32, f64)> = None;
        for lag in [0u32, 1, 2, 4, 8, 12] {
            let mut params = base.clone();
            params.expectation_lag_years = lag;
            let path = simulate(0.8, 0.5, &params, Strategy::Tracking, params.human_share);
            let end = path.last().expect("non-empty").capacity;
            if let Some((earlier_lag, earlier_end)) = previous {
                assert!(
                    end >= earlier_end - 1e-9,
                    "capacity fell when the runway lengthened from {earlier_lag} to {lag}: \
                     {earlier_end} then {end}"
                );
            }
            previous = Some((lag, end));
        }
    }

    /// Whether maximum assistance is optimal is a **parameter question, not a finding** — and
    /// saying so is the honest form of the claim.
    ///
    /// The first version of this test asserted that the default parameters produce an interior
    /// optimum. They do not: at `throughput_gain = 1.5` the speed is worth more than the
    /// capacity costs, and full assistance wins. The temptation is to retune the defaults until
    /// the "right" answer appears, which would make the model an illustration of its author's
    /// conclusion rather than a test of it.
    ///
    /// Instead the test maps the boundary. There must exist a speed-up small enough that
    /// assistance is not worth its capacity cost — the regime in which the argument bites — and
    /// a speed-up large enough that it is, which is the regime the argument is *not* needed for.
    /// A model that could not produce both regimes would be a fixed conclusion wearing a
    /// parameter.
    #[test]
    fn the_optimal_assistance_trades_speed_against_capacity() {
        let base = ModelParameters::default();
        let mut regimes = Vec::new();
        for gain in [0.05, 0.1, 0.2, 0.4, 0.8, 1.5, 3.0] {
            let mut params = base.clone();
            params.throughput_gain = gain;
            let mut best = (0.0_f64, f64::NEG_INFINITY);
            let mut means = Vec::new();
            for step in 0..=10 {
                let level = f64::from(step) / 10.0;
                let path = simulate(
                    0.7,
                    0.4,
                    &params,
                    Strategy::Fixed(level),
                    params.human_share,
                );
                let mean: f64 =
                    path.iter().map(|s| s.effective_output).sum::<f64>() / path.len() as f64;
                means.push(mean);
                if mean > best.1 {
                    best = (level, mean);
                }
            }
            // Effective output must rise then fall, or rise throughout. What it must never do
            // is fall then rise: capacity costs are monotone in assistance.
            for window in means.windows(3) {
                assert!(
                    !(window[1] < window[0] && window[2] > window[1]),
                    "effective output fell then rose at gain {gain}: {means:?}"
                );
            }
            regimes.push((gain, best.0));
        }

        let interior = regimes.iter().filter(|(_, level)| *level < 1.0).count();
        assert!(
            interior > 0,
            "no speed-up is small enough for assistance to cost more than it earns, so the \
             model cannot express the argument's premise: {regimes:?}"
        );
        assert!(
            interior < regimes.len(),
            "no speed-up is large enough for full assistance to win, so the model has no \
             opposite regime to test the argument against: {regimes:?}"
        );

        // And the optimal assistance must be non-decreasing in the speed-up: a faster tool is
        // never worth *less* use. This is the comparative static that gives the boundary its
        // meaning.
        for pair in regimes.windows(2) {
            assert!(
                pair[1].1 >= pair[0].1 - 1e-12,
                "optimal assistance fell from {} at gain {} to {} at gain {}",
                pair[0].1,
                pair[0].0,
                pair[1].1,
                pair[1].0
            );
        }
    }

    /// Every parameter must have a stated origin, and the origin must say *declared* — because
    /// nothing in this module is calibrated against data.
    #[test]
    fn every_parameter_is_declared_and_says_so() {
        let params = ModelParameters::default();
        let table = params.provenance_table();
        assert!(table.len() >= 13, "the table has only {} rows", table.len());
        for (name, provenance) in &table {
            assert!(!name.is_empty());
            assert_eq!(
                provenance.label(),
                "declared",
                "{name} claims to be something other than declared"
            );
            assert!(!provenance.detail().is_empty());
        }
    }

    /// The cohort summary must be the summary of its own trajectories, and the seed must make
    /// two policies comparable on the same people.
    #[test]
    fn the_cohort_summary_describes_the_trajectories_it_was_given() {
        let params = ModelParameters::default();
        let cohort = draw_cohort(200, &params, 12345);
        assert_eq!(cohort.len(), 200);
        let again = draw_cohort(200, &params, 12345);
        for (a, b) in cohort.iter().zip(again.iter()) {
            assert_eq!(a.capacity_0, b.capacity_0);
            assert_eq!(a.self_model_0, b.self_model_0);
        }
        let paths = run_cohort(&cohort, &params, Strategy::Tracking);
        let summary = summarise(&paths, 0.25);
        assert_eq!(summary.persons, 200);
        assert!((0.0..=1.0).contains(&summary.mean_capacity));
        assert!((0.0..=1.0).contains(&summary.mean_autonomy));
        assert!((0.0..=1.0).contains(&summary.share_led));
        assert!(summary.p10_capacity <= summary.median_capacity);
        assert!(summary.peak_effective_output >= summary.mean_effective_output);
    }
}
