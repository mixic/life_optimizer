// Life Optimizer
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

//! The perception gap between believed and true competence, simulated with known ground truth.
//!
//! # What the simulation is for
//!
//! Not to produce a number. To answer two questions that a paper about "estimating the AI–human
//! discrepancy" has to answer before its estimate means anything:
//!
//! 1. **Is the gap ever large?** It is a difference of two latents that both saturate, so the
//!    interesting quantity is not whether it grows but when, and how fast.
//! 2. **Is it estimable at all from the signals available?** A state estimator returns an estimate
//!    and a covariance whether or not the state is observable. The observability rank is computed
//!    here as a first-class output, before any filtering.
//!
//! # The design decision that keeps the filter exact
//!
//! The adoption share `a` is a **known input**, so the transition is affine in the state and a
//! standard Kalman filter is exact. The nonlinearity sits in the input, not in the dynamics. If a
//! later version moves it into the covariance or makes the transition depend on the state
//! multiplicatively, the extended/unscented filter objection returns and must be argued then.

pub mod analysis;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

/// The five observation channels. Indices are used throughout.
pub const REVIEW_TIME: usize = 0;
pub const DEFECT_ESCAPE: usize = 1;
pub const MEASURED_SAVING: usize = 2;
pub const REPORTED_SAVING: usize = 3;
pub const REPORTED_CONFIDENCE: usize = 4;
pub const CHANNELS: usize = 5;

/// Which channels an observation design uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Design {
    /// All five channels: behaviour plus what the person says about themselves.
    Full,
    /// Behaviour only — review time and defect escape. No self-report.
    BehaviouralOnly,
}

impl Design {
    pub fn permits(self, channel: usize) -> bool {
        match self {
            Design::Full => true,
            Design::BehaviouralOnly => matches!(channel, REVIEW_TIME | DEFECT_ESCAPE),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Design::Full => "all five channels",
            Design::BehaviouralOnly => "behavioural only",
        }
    }
}

/// Everything the model needs. Every value is `Declared` — invented, plausible in sign.
#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    /// True competence at the start.
    pub tau0: f64,
    /// Believed competence at the start.
    pub beta0: f64,
    /// Monthly pull of true competence toward its floor, at full adoption.
    pub kappa_a: f64,
    /// Monthly pull of believed competence toward its ceiling, at full adoption.
    pub kappa_b: f64,
    /// Where true competence lands under sustained assistance.
    pub tau_floor: f64,
    /// Where believed competence lands under sustained assistance.
    pub beta_ceiling: f64,
    /// Process noise on true competence.
    pub sigma_tau: f64,
    /// Process noise on belief.
    pub sigma_beta: f64,
    /// Measurement noise per channel.
    pub sigma_obs: [f64; CHANNELS],
    /// How strongly confidence moves review time *independently of competence*.
    ///
    /// This is the identifiability parameter. At zero the two latents enter the review-time channel
    /// in strictly different proportions; as it rises they become harder to separate.
    pub gamma: f64,
    /// Months to simulate.
    pub horizon: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            tau0: 0.80,
            beta0: 0.75,
            kappa_a: 0.05,
            kappa_b: 0.08,
            tau_floor: 0.35,
            beta_ceiling: 0.95,
            sigma_tau: 0.03,
            sigma_beta: 0.02,
            sigma_obs: [0.10; CHANNELS],
            gamma: 0.0,
            horizon: 36,
        }
    }
}

/// The observation matrix, two columns `[τ, β]`, one row per channel.
///
/// Signs carry the model's claims: less competence means a **longer** review and a **higher** defect
/// escape rate, so those coefficients are negative on `τ`; belief enters review time through `gamma`
/// with the same sign, which is the confound.
pub fn observation_matrix(params: &Params, design: Design) -> Vec<[f64; 2]> {
    let base = [[-2.0, -2.0 * params.gamma], [-1.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 1.0]];
    base.iter()
        .enumerate()
        .filter(|(channel, _)| design.permits(*channel))
        .map(|(_, row)| *row)
        .collect()
}

/// Transition matrix and offset for a given adoption share.
///
/// `x' = F x + u + w`, with `F = diag(1 − κ a)` and `u = [κ_a a τ_floor, κ_b a β_ceiling]`.
pub fn transition(params: &Params, adoption: f64) -> ([f64; 2], [f64; 2]) {
    let a = adoption.clamp(0.0, 1.0);
    let f = [1.0 - params.kappa_a * a, 1.0 - params.kappa_b * a];
    let u = [params.kappa_a * a * params.tau_floor, params.kappa_b * a * params.beta_ceiling];
    (f, u)
}

/// The asymptotic gap under sustained adoption, and how long it takes to get most of the way there.
///
/// Both latents converge geometrically, so the gap has a closed form and a half-life. This is what
/// makes "when is the gap largest" answerable without a simulation, and the simulation is then a
/// check on the algebra.
pub fn steady_state(params: &Params, adoption: f64) -> (f64, f64) {
    let (f, _) = transition(params, adoption);
    let gap = params.beta_ceiling - params.tau_floor;
    // The slower of the two modes governs the gap's approach to its limit.
    let slowest = f[0].max(f[1]).clamp(1e-9, 1.0 - 1e-9);
    let half_life = (0.5_f64).ln() / slowest.ln();
    (gap, half_life)
}

/// One simulated run: the truth, and the observations the designs get to see.
#[derive(Debug, Clone)]
pub struct Trace {
    pub tau: Vec<f64>,
    pub beta: Vec<f64>,
    pub observations: Vec<[f64; CHANNELS]>,
}

impl Trace {
    /// The true gap at each month.
    pub fn gap(&self) -> Vec<f64> {
        self.beta.iter().zip(self.tau.iter()).map(|(b, t)| b - t).collect()
    }
}

/// Simulate the latents and the observations, with a fixed seed so a figure can be reproduced.
pub fn simulate(params: &Params, adoption: f64, seed: u64) -> Trace {
    let mut rng = StdRng::seed_from_u64(seed);
    let process_tau = Normal::new(0.0, params.sigma_tau).expect("positive sigma");
    let process_beta = Normal::new(0.0, params.sigma_beta).expect("positive sigma");
    let obs_noise: Vec<Normal<f64>> = params
        .sigma_obs
        .iter()
        .map(|sigma| Normal::new(0.0, *sigma).expect("positive sigma"))
        .collect();

    let (f, u) = transition(params, adoption);
    let mut tau = params.tau0;
    let mut beta = params.beta0;
    let mut trace = Trace {
        tau: Vec::with_capacity(params.horizon),
        beta: Vec::with_capacity(params.horizon),
        observations: Vec::with_capacity(params.horizon),
    };

    for _ in 0..params.horizon {
        tau = (f[0] * tau + u[0] + process_tau.sample(&mut rng)).clamp(0.0, 1.0);
        beta = (f[1] * beta + u[1] + process_beta.sample(&mut rng)).clamp(0.0, 1.0);

        let rows = observation_matrix(params, Design::Full);
        let mut z = [0.0; CHANNELS];
        for (channel, row) in rows.iter().enumerate() {
            z[channel] = row[0] * tau + row[1] * beta + obs_noise[channel].sample(&mut rng);
        }
        trace.tau.push(tau);
        trace.beta.push(beta);
        trace.observations.push(z);
    }
    trace
}

/// A 2×2 matrix, stored row-major, with the handful of operations this model needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2(pub [[f64; 2]; 2]);

impl Mat2 {
    pub fn identity() -> Self {
        Mat2([[1.0, 0.0], [0.0, 1.0]])
    }

    pub fn scale(&self, k: f64) -> Self {
        Mat2([[self.0[0][0] * k, self.0[0][1] * k], [self.0[1][0] * k, self.0[1][1] * k]])
    }

    pub fn add(&self, other: &Mat2) -> Self {
        Mat2([
            [self.0[0][0] + other.0[0][0], self.0[0][1] + other.0[0][1]],
            [self.0[1][0] + other.0[1][0], self.0[1][1] + other.0[1][1]],
        ])
    }

    pub fn sub(&self, other: &Mat2) -> Self {
        Mat2([
            [self.0[0][0] - other.0[0][0], self.0[0][1] - other.0[0][1]],
            [self.0[1][0] - other.0[1][0], self.0[1][1] - other.0[1][1]],
        ])
    }

    pub fn mul(&self, other: &Mat2) -> Self {
        Mat2([
            [
                self.0[0][0] * other.0[0][0] + self.0[0][1] * other.0[1][0],
                self.0[0][0] * other.0[0][1] + self.0[0][1] * other.0[1][1],
            ],
            [
                self.0[1][0] * other.0[0][0] + self.0[1][1] * other.0[1][0],
                self.0[1][0] * other.0[0][1] + self.0[1][1] * other.0[1][1],
            ],
        ])
    }

    pub fn transpose(&self) -> Self {
        Mat2([[self.0[0][0], self.0[1][0]], [self.0[0][1], self.0[1][1]]])
    }

    pub fn det(&self) -> f64 {
        self.0[0][0] * self.0[1][1] - self.0[0][1] * self.0[1][0]
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.det();
        if det.abs() < 1e-12 {
            return None;
        }
        Some(Mat2([
            [self.0[1][1] / det, -self.0[0][1] / det],
            [-self.0[1][0] / det, self.0[0][0] / det],
        ]))
    }

    pub fn trace(&self) -> f64 {
        self.0[0][0] + self.0[1][1]
    }

    /// Eigenvalues of a symmetric 2×2 matrix, larger first.
    pub fn eigen(&self) -> (f64, f64) {
        let a = self.0[0][0];
        let d = self.0[1][1];
        let b = self.0[0][1];
        let mean = (a + d) / 2.0;
        let diff = ((a - d) / 2.0).powi(2) + b * b;
        let root = diff.max(0.0).sqrt();
        (mean + root, mean - root)
    }
}

/// What a state estimator produces for one run.
#[derive(Debug, Clone)]
pub struct Estimate {
    pub gap: Vec<f64>,
    /// Width of the gap's one-sigma band at each month.
    pub sigma: Vec<f64>,
}

impl Estimate {
    pub fn rmse(&self, truth: &[f64]) -> f64 {
        let n = truth.len().min(self.gap.len()) as f64;
        let sse: f64 = truth.iter().zip(self.gap.iter()).map(|(t, e)| (t - e).powi(2)).sum();
        (sse / n).sqrt()
    }

    /// Share of months where the truth sits inside the one-sigma band.
    pub fn coverage(&self, truth: &[f64]) -> f64 {
        let n = truth.len().min(self.gap.len());
        if n == 0 {
            return 0.0;
        }
        let inside = truth
            .iter()
            .zip(self.gap.iter())
            .zip(self.sigma.iter())
            .filter(|((t, e), s)| (*t - *e).abs() <= **s)
            .count();
        inside as f64 / n as f64
    }
}

/// A Kalman filter over `[τ, β]`, observing the gap indirectly.
pub struct Kalman {
    x: [f64; 2],
    p: Mat2,
    params: Params,
    design: Design,
    adoption: f64,
}

impl Kalman {
    pub fn new(params: &Params, design: Design, adoption: f64) -> Self {
        Self {
            x: [params.tau0, params.beta0],
            // Not zero: an unknown initial state whose covariance is tiny would make the filter
            // ignore the data for the first few months and look artificially confident.
            p: Mat2([[1e-3, 0.0], [0.0, 1e-3]]),
            params: params.clone(),
            design,
            adoption,
        }
    }

    fn rows(&self) -> Vec<[f64; 2]> {
        observation_matrix(&self.params, self.design)
    }

    fn channels(&self) -> Vec<usize> {
        (0..CHANNELS).filter(|c| self.design.permits(*c)).collect()
    }

    /// The predict step alone: propagate the state and covariance with no observation.
    ///
    /// Exposed so the report can ask whether a working filter is working, or whether its process
    /// model is simply right. In a simulation the truth is generated by the same transition the
    /// filter assumes, so a filter can look excellent while reading nothing from the data. The
    /// prior-only baseline is how that is detected rather than believed.
    pub fn predict(&mut self) -> f64 {
        let (f, u) = transition(&self.params, self.adoption);
        let fm = Mat2([[f[0], 0.0], [0.0, f[1]]]);
        let q = Mat2([[self.params.sigma_tau.powi(2), 0.0], [0.0, self.params.sigma_beta.powi(2)]]);
        self.x[0] = f[0] * self.x[0] + u[0];
        self.x[1] = f[1] * self.x[1] + u[1];
        self.p = fm.mul(&self.p).mul(&fm.transpose()).add(&q);
        let variance = self.p.0[0][0] + self.p.0[1][1] - 2.0 * self.p.0[0][1];
        variance.max(0.0).sqrt()
    }

    /// One predict–update cycle. Returns the posterior one-sigma band on the gap.
    pub fn step(&mut self, z: &[f64; CHANNELS]) -> f64 {
        let (f, u) = transition(&self.params, self.adoption);
        let _ = (f, u);
        // Predict, then update — the same propagation the prior-only baseline uses.
        self.predict();

        // Update, one channel at a time — equivalent to the joint update for independent noise and
        // it keeps the code to 2×2 inverses.
        let rows = self.rows();
        let channels = self.channels();
        for (row, channel) in rows.iter().zip(channels.iter()) {
            let r = self.params.sigma_obs[*channel].powi(2);
            // Innovation: predicted observation minus the observation, no offset terms.
            let predicted = row[0] * self.x[0] + row[1] * self.x[1];
            let innovation = z[*channel] - predicted;

            // S = H P Hᵀ + R, a scalar for one channel.
            let hph = row[0] * (self.p.0[0][0] * row[0] + self.p.0[1][0] * row[1])
                + row[1] * (self.p.0[0][1] * row[0] + self.p.0[1][1] * row[1]);
            let s = hph + r;
            if s <= 0.0 {
                continue;
            }

            // K = P Hᵀ / S.
            let ph_t = [
                self.p.0[0][0] * row[0] + self.p.0[0][1] * row[1],
                self.p.0[1][0] * row[0] + self.p.0[1][1] * row[1],
            ];
            let k = [ph_t[0] / s, ph_t[1] / s];

            self.x[0] += k[0] * innovation;
            self.x[1] += k[1] * innovation;

            // P = P − K H P.
            let hp = [
                row[0] * self.p.0[0][0] + row[1] * self.p.0[1][0],
                row[0] * self.p.0[0][1] + row[1] * self.p.0[1][1],
            ];
            let update = Mat2([[k[0] * hp[0], k[0] * hp[1]], [k[1] * hp[0], k[1] * hp[1]]]);
            self.p = self.p.sub(&update);
        }

        // Symmetrise, which the sequential update can drift away from.
        let sym = (self.p.0[0][1] + self.p.0[1][0]) / 2.0;
        self.p.0[0][1] = sym;
        self.p.0[1][0] = sym;

        // One-sigma band on the gap g = β − τ.
        let variance = self.p.0[0][0] + self.p.0[1][1] - 2.0 * self.p.0[0][1];
        variance.max(0.0).sqrt()
    }

    pub fn gap(&self) -> f64 {
        self.x[1] - self.x[0]
    }

    /// Run with no observations at all: the filter's own model, unchecked.
    pub fn run_prior_only(&mut self, months: usize) -> Estimate {
        let mut gap = Vec::with_capacity(months);
        let mut sigma = Vec::with_capacity(months);
        for _ in 0..months {
            sigma.push(self.predict());
            gap.push(self.gap());
        }
        Estimate { gap, sigma }
    }

    pub fn run(&mut self, trace: &Trace) -> Estimate {
        let mut gap = Vec::with_capacity(trace.observations.len());
        let mut sigma = Vec::with_capacity(trace.observations.len());
        for z in &trace.observations {
            sigma.push(self.step(z));
            gap.push(self.gap());
        }
        Estimate { gap, sigma }
    }
}

/// The baseline that needs no filter: subtract the reported estimate from the measured one.
///
/// Two of the channels measure one latent each with a known unit coefficient, so this is the
/// comparison the filter has to beat. If it cannot, the dynamics are decoration.
pub fn subtract(trace: &Trace, params: &Params) -> Estimate {
    let mut gap = Vec::with_capacity(trace.observations.len());
    let mut sigma = Vec::with_capacity(trace.observations.len());
    for z in &trace.observations {
        gap.push(z[REPORTED_CONFIDENCE] - z[MEASURED_SAVING]);
        let variance = params.sigma_obs[REPORTED_CONFIDENCE].powi(2)
            + params.sigma_obs[MEASURED_SAVING].powi(2);
        sigma.push(variance.sqrt());
    }
    Estimate { gap, sigma }
}

/// Static weighted least squares: all channels, no dynamics.
pub fn static_fit(trace: &Trace, params: &Params, design: Design) -> Estimate {
    let rows = observation_matrix(params, design);
    let channels: Vec<usize> = (0..CHANNELS).filter(|c| design.permits(*c)).collect();

    // Information matrix Hᵀ R⁻¹ H, accumulated once — the design does not change over time.
    let mut info = Mat2([[0.0, 0.0], [0.0, 0.0]]);
    for (row, channel) in rows.iter().zip(channels.iter()) {
        let w = 1.0 / params.sigma_obs[*channel].powi(2);
        info.0[0][0] += w * row[0] * row[0];
        info.0[0][1] += w * row[0] * row[1];
        info.0[1][0] += w * row[1] * row[0];
        info.0[1][1] += w * row[1] * row[1];
    }

    let mut gap = Vec::with_capacity(trace.observations.len());
    let mut sigma = Vec::with_capacity(trace.observations.len());

    let Some(info_inv) = info.inverse() else {
        // Rank-deficient: no estimate exists, and pretending otherwise would be the failure this
        // whole module is built to name.
        return Estimate {
            gap: vec![f64::NAN; trace.observations.len()],
            sigma: vec![f64::NAN; trace.observations.len()],
        };
    };

    for z in &trace.observations {
        let mut rhtz = [0.0, 0.0];
        for (row, channel) in rows.iter().zip(channels.iter()) {
            let w = 1.0 / params.sigma_obs[*channel].powi(2);
            rhtz[0] += w * row[0] * z[*channel];
            rhtz[1] += w * row[1] * z[*channel];
        }
        let tau = info_inv.0[0][0] * rhtz[0] + info_inv.0[0][1] * rhtz[1];
        let beta = info_inv.0[1][0] * rhtz[0] + info_inv.0[1][1] * rhtz[1];
        gap.push(beta - tau);
        let variance = info_inv.0[0][0] + info_inv.0[1][1] - 2.0 * info_inv.0[0][1];
        sigma.push(variance.max(0.0).sqrt());
    }
    Estimate { gap, sigma }
}

/// Result of the observability test on one design.
#[derive(Debug, Clone, PartialEq)]
pub struct Observability {
    /// Rank of `[H; H F]`, which is 2 when both latents are identifiable.
    pub rank: usize,
    /// Condition number of the Gram matrix; large means nominally identifiable but fragile.
    pub condition: f64,
    /// The null-space direction, when there is one.
    pub null_direction: Option<[f64; 2]>,
}

/// Test whether a design can in principle recover both latents.
///
/// Computed before any filtering, because a filter will happily return a confident estimate from an
/// unobservable state and the covariance will not reveal it.
pub fn observability(params: &Params, design: Design, adoption: f64) -> Observability {
    let rows = observation_matrix(params, design);
    let (f, _) = transition(params, adoption);

    // O = [H ; H F], stacked.
    let mut stacked: Vec<[f64; 2]> = rows.clone();
    for row in &rows {
        stacked.push([f[0] * row[0], f[1] * row[1]]);
    }

    // Gram matrix of the stack, whose rank equals the stack's rank.
    let mut gram = Mat2([[0.0, 0.0], [0.0, 0.0]]);
    for row in &stacked {
        gram.0[0][0] += row[0] * row[0];
        gram.0[0][1] += row[0] * row[1];
        gram.0[1][0] += row[1] * row[0];
        gram.0[1][1] += row[1] * row[1];
    }

    let (largest, smallest) = gram.eigen();
    let scale = gram.0[0][0].abs().max(gram.0[1][1].abs()).max(1e-12);
    let rank = if smallest.abs() <= 1e-9 * scale {
        1
    } else if largest > 0.0 {
        2
    } else {
        0
    };
    let condition =
        if smallest.abs() <= 1e-12 { f64::INFINITY } else { (largest / smallest).sqrt() };

    // Null direction for the rank-1 case: orthogonal to every row.
    let null_direction = if rank <= 1 {
        let anchor =
            stacked.iter().find(|row| row[0].abs() > 1e-12 || row[1].abs() > 1e-12).copied();
        anchor.map(|row| [-row[1], row[0]])
    } else {
        None
    };

    Observability { rank, condition, null_direction }
}

/// A bootstrap particle filter over the same state space.
///
/// # Why this exists
///
/// A referee objected that the linear-Gaussian assumption is doing unacknowledged work, and that a
/// standard Kalman filter's "well conditioned" result is a property of the linearisation rather than
/// of the problem. The objection is correct in general and testable here.
///
/// In *this* model every map is affine — the transition is `F x + u` with a known input, and the
/// observation is `H x` — so the extended and unscented filters reduce to the Kalman filter exactly,
/// and a particle filter should agree with it up to Monte Carlo error. That is a claim about this
/// specification, not about state estimation in general, and it is checked empirically by
/// `the_particle_filter_agrees_with_the_kalman_filter` rather than asserted.
///
/// If a later version makes the transition depend on the state multiplicatively, or puts the
/// adoption share inside the covariance, this equality breaks and the choice of filter must be
/// argued rather than inherited.
pub fn particle_filter(
    trace: &Trace,
    params: &Params,
    design: Design,
    adoption: f64,
    particles: usize,
    seed: u64,
) -> Estimate {
    let mut rng = StdRng::seed_from_u64(seed);
    let init = Normal::new(0.0, 0.03).expect("positive sigma");
    let (f, u) = transition(params, adoption);
    let rows = observation_matrix(params, design);
    let channels: Vec<usize> = (0..CHANNELS).filter(|c| design.permits(*c)).collect();

    let mut xs: Vec<[f64; 2]> = (0..particles)
        .map(|_| {
            [
                (params.tau0 + init.sample(&mut rng)).clamp(0.0, 1.0),
                (params.beta0 + init.sample(&mut rng)).clamp(0.0, 1.0),
            ]
        })
        .collect();
    let mut weights = vec![1.0 / particles as f64; particles];
    let mut gap = Vec::with_capacity(trace.observations.len());
    let mut sigma = Vec::with_capacity(trace.observations.len());

    for z in &trace.observations {
        // Propagate.
        for x in xs.iter_mut() {
            x[0] =
                (f[0] * x[0] + u[0] + Normal::new(0.0, params.sigma_tau).unwrap().sample(&mut rng))
                    .clamp(0.0, 1.0);
            x[1] = (f[1] * x[1]
                + u[1]
                + Normal::new(0.0, params.sigma_beta).unwrap().sample(&mut rng))
            .clamp(0.0, 1.0);
        }

        // Weight by the likelihood of the observations this design can see.
        let mut total = 0.0;
        for (x, w) in xs.iter().zip(weights.iter_mut()) {
            let mut log_likelihood = 0.0;
            for (row, channel) in rows.iter().zip(channels.iter()) {
                let variance = params.sigma_obs[*channel].powi(2);
                let predicted = row[0] * x[0] + row[1] * x[1];
                log_likelihood -= 0.5 * (z[*channel] - predicted).powi(2) / variance;
            }
            *w *= log_likelihood.exp();
            total += *w;
        }
        if total <= 0.0 || !total.is_finite() {
            // Every particle is implausible; fall back to uniform rather than emitting NaN.
            weights.iter_mut().for_each(|w| *w = 1.0 / particles as f64);
        } else {
            weights.iter_mut().for_each(|w| *w /= total);
        }

        // Weighted moments of the gap.
        let mean_gap: f64 = xs.iter().zip(weights.iter()).map(|(x, w)| (x[1] - x[0]) * w).sum();
        let var_gap: f64 = xs
            .iter()
            .zip(weights.iter())
            .map(|(x, w)| ((x[1] - x[0]) - mean_gap).powi(2) * w)
            .sum();
        gap.push(mean_gap);
        sigma.push(var_gap.max(0.0).sqrt());

        // Resample when the effective sample size collapses.
        let ess = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();
        if ess < particles as f64 / 2.0 {
            let mut cumulative = Vec::with_capacity(particles);
            let mut running = 0.0;
            for w in &weights {
                running += *w;
                cumulative.push(running);
            }
            // Systematic resampling: ONE uniform offset, then evenly spaced targets. Drawing a
            // fresh random target per particle is the classic wrong version — it lets the index
            // advance monotonically while the targets do not, which biases the sample and was
            // producing a 25-35% error against the Kalman filter on a model where the two filters
            // are supposed to coincide.
            let mut rebuilt = Vec::with_capacity(particles);
            let offset = rng.gen::<f64>() / particles as f64;
            let mut index = 0;
            let mut running = cumulative[0];
            for i in 0..particles {
                let target = offset + i as f64 / particles as f64;
                while target > running && index < particles - 1 {
                    index += 1;
                    running = cumulative[index];
                }
                rebuilt.push(xs[index]);
            }
            xs = rebuilt;
            weights.iter_mut().for_each(|w| *w = 1.0 / particles as f64);
        }
    }

    Estimate { gap, sigma }
}
