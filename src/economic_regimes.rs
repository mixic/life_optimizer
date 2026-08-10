// Economic regime-switching model (Markov chain) for macro scenario simulation.
// Models transitions between Boom / Normal / Recession / Stagflation regimes,
// each with distinct return, volatility, and inflation characteristics.
// This captures fat-tail / clustered risk that a single log-normal distribution misses.
#![allow(dead_code)]

use rand::Rng;
use rand_distr::{Distribution, Normal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Regime {
    Boom,
    Normal,
    Recession,
    Stagflation,
}

impl Regime {
    pub fn index(&self) -> usize {
        match self {
            Regime::Boom => 0,
            Regime::Normal => 1,
            Regime::Recession => 2,
            Regime::Stagflation => 3,
        }
    }

    pub fn from_index(i: usize) -> Regime {
        match i {
            0 => Regime::Boom,
            1 => Regime::Normal,
            2 => Regime::Recession,
            _ => Regime::Stagflation,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Regime::Boom => "Boom",
            Regime::Normal => "Normal",
            Regime::Recession => "Recession",
            Regime::Stagflation => "Stagflation",
        }
    }

    /// (real_return, volatility, inflation) for this regime
    pub fn parameters(&self) -> (f64, f64, f64) {
        match self {
            Regime::Boom        => (0.080, 0.120, 0.010),
            Regime::Normal      => (0.032, 0.080, 0.015),
            Regime::Recession   => (-0.080, 0.180, 0.005),
            Regime::Stagflation => (-0.030, 0.140, 0.050),
        }
    }
}

/// Annual transition matrix: row = current regime, col = next regime probability
/// Calibrated loosely to post-WWII Swiss/developed-market business cycle frequency:
/// - Recessions ~once every 8-10 years, lasting 1-2 years
/// - Stagflation rarer (~once every 20-25 years), e.g. 1970s style shock
/// - Booms follow recoveries
pub struct TransitionMatrix {
    pub matrix: [[f64; 4]; 4], // [from][to]
}

impl TransitionMatrix {
    pub fn calibrated() -> Self {
        Self {
            matrix: [
                // From Boom:        Boom  Normal Recession Stagflation
                [0.50, 0.35, 0.10, 0.05],
                // From Normal:
                [0.15, 0.70, 0.12, 0.03],
                // From Recession:
                [0.05, 0.45, 0.40, 0.10],
                // From Stagflation:
                [0.05, 0.40, 0.20, 0.35],
            ],
        }
    }

    pub fn next_regime<R: Rng>(&self, current: Regime, rng: &mut R) -> Regime {
        let probs = self.matrix[current.index()];
        let r: f64 = rng.gen();
        let mut cumulative = 0.0;
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if r <= cumulative {
                return Regime::from_index(i);
            }
        }
        Regime::from_index(3) // fallback
    }

    /// Long-run stationary probability of each regime (steady-state)
    pub fn stationary_distribution(&self) -> [f64; 4] {
        // Power-iterate the Markov chain to approximate stationary distribution
        let mut dist = [0.25f64; 4];
        for _ in 0..1000 {
            let mut next = [0.0f64; 4];
            for from in 0..4 {
                for to in 0..4 {
                    next[to] += dist[from] * self.matrix[from][to];
                }
            }
            dist = next;
        }
        dist
    }
}

/// Draw one year's nominal return and inflation from a given regime
pub fn sample_year<R: Rng>(regime: Regime, rng: &mut R) -> (f64, f64) {
    let (real_return, vol, inflation) = regime.parameters();
    let mu_nominal = real_return + inflation;
    let mu_ln = mu_nominal - 0.5 * vol * vol;
    let normal = Normal::new(mu_ln, vol).unwrap();
    let nominal_return = normal.sample(rng).exp() - 1.0;
    (nominal_return, inflation)
}

/// Simulate a full path of regimes across `n_years`, starting from a given regime
/// (or drawn from stationary distribution if None). Returns (returns, inflations, regimes).
pub fn simulate_regime_path<R: Rng>(
    n_years: usize,
    start_regime: Option<Regime>,
    transitions: &TransitionMatrix,
    rng: &mut R,
) -> (Vec<f64>, Vec<f64>, Vec<Regime>) {
    let mut regime = start_regime.unwrap_or(Regime::Normal);
    let mut returns = Vec::with_capacity(n_years);
    let mut inflations = Vec::with_capacity(n_years);
    let mut regimes = Vec::with_capacity(n_years);

    for _ in 0..n_years {
        let (r, infl) = sample_year(regime, rng);
        returns.push(r);
        inflations.push(infl);
        regimes.push(regime);
        regime = transitions.next_regime(regime, rng);
    }

    (returns, inflations, regimes)
}

/// Force a recession/stagflation shock to start at a specific year index (for stress testing
/// sequence-of-returns risk, e.g. a downturn hitting right before/at retirement).
pub fn simulate_stress_path<R: Rng>(
    n_years: usize,
    shock_start_year: usize,
    shock_regime: Regime,
    shock_duration: usize,
    transitions: &TransitionMatrix,
    rng: &mut R,
) -> (Vec<f64>, Vec<f64>, Vec<Regime>) {
    let mut regime = Regime::Normal;
    let mut returns = Vec::with_capacity(n_years);
    let mut inflations = Vec::with_capacity(n_years);
    let mut regimes = Vec::with_capacity(n_years);

    for year in 0..n_years {
        if year >= shock_start_year && year < shock_start_year + shock_duration {
            regime = shock_regime;
        }

        let (r, infl) = sample_year(regime, rng);
        returns.push(r);
        inflations.push(infl);
        regimes.push(regime);

        if year < shock_start_year || year >= shock_start_year + shock_duration {
            regime = transitions.next_regime(regime, rng);
        } else {
            // stay in shock regime for its duration, then let it transition after
            regime = shock_regime;
        }
    }

    (returns, inflations, regimes)
}
