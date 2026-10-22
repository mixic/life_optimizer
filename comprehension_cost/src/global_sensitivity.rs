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

//! Variance-based global sensitivity analysis (Sobol / Saltelli indices).
//!
//! # Why this replaces the one-at-a-time sweep as the headline attribution
//!
//! A one-at-a-time (OAT) sweep varies one parameter, holds the other fifteen at their
//! central values, and reports the resulting change. That answers "what if this parameter
//! moved while everything else stayed put", which is not the question. In this model the
//! parameters interact by construction: the review catch probability is a function of
//! comprehension, comprehension is a function of retention and of how often the module is
//! touched, and the touch rate is a function of the defect multiplier. An OAT attribution
//! therefore assigns a parameter only the variance it contributes *along its own axis*, and
//! silently drops the variance it contributes through combinations.
//!
//! Variance-based decomposition answers the other question: how much of the variance of the
//! outcome is attributable to each input over the whole joint space, including interactions.
//! The implementation here is the Saltelli (2010) estimator of the first-order index `S_i`
//! and the total-order index `ST_i`:
//!
//! - `S_i` is the share of outcome variance explained by `i` alone.
//! - `ST_i` is the share that would remain if everything *except* `i` were fixed, so it
//!   includes every interaction `i` participates in.
//! - `ST_i - S_i` is therefore the interaction share, and `Σ ST_i - Σ S_i` is the total
//!   interaction variance in the model.
//!
//! # What the inputs are, and what that costs
//!
//! The inputs are uniform over each parameter's range — the same plus-or-minus-fifty-percent
//! band the OAT sweep used, so the two are directly comparable. A uniform prior is not a
//! belief about the parameters; it is a statement that we do not know their distribution,
//! which is the honest position for parameters with no measurements behind them. The indices
//! are conditional on that choice and are reported as such.

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::analysis::benefit_at_horizon;
use crate::params::Params;

/// One input parameter: how to read it, how to write it, and the range it varies over.
#[derive(Clone, Copy)]
pub struct Knob {
    pub name: &'static str,
    pub get: fn(&Params) -> f64,
    pub set: fn(&mut Params, f64),
    /// Multiplier applied to the central value to form the range. 0.5 means the range is
    /// half to one-and-a-half times the declared value, matching the OAT sweep.
    pub spread: f64,
    /// Hard limits the sampled value is clamped into, for quantities that cannot be negative
    /// or that are probabilities.
    pub floor: f64,
    pub ceiling: f64,
}

const PROBABILITY: (f64, f64) = (0.0, 1.0);

/// The canonical parameter list. The OAT sweep and the Sobol indices both use this, so an
/// attribution computed one way cannot describe a different parameter set from the other.
pub fn knobs() -> Vec<Knob> {
    let knob =
        |name, get, set, floor, ceiling| Knob { name, get, set, spread: 0.5, floor, ceiling };
    vec![
        knob(
            "c_ai (initial comprehension)",
            |p| p.c_ai,
            |p, v| p.c_ai = v,
            PROBABILITY.0,
            PROBABILITY.1,
        ),
        knob(
            "c_human (initial comprehension)",
            |p| p.c_human,
            |p, v| p.c_human = v,
            PROBABILITY.0,
            PROBABILITY.1,
        ),
        knob("catch_threshold", |p| p.catch_threshold, |p, v| p.catch_threshold = v, 0.01, 1.0),
        knob("leak_hazard", |p| p.leak_hazard, |p, v| p.leak_hazard = v, 0.0, f64::INFINITY),
        knob(
            "half_life_ai (months)",
            |p| p.half_life_ai,
            |p, v| p.half_life_ai = v,
            0.5,
            f64::INFINITY,
        ),
        knob(
            "half_life_human (months)",
            |p| p.half_life_human,
            |p, v| p.half_life_human = v,
            0.5,
            f64::INFINITY,
        ),
        knob(
            "ai_defect_multiplier",
            |p| p.ai_defect_multiplier,
            |p, v| p.ai_defect_multiplier = v,
            0.0,
            f64::INFINITY,
        ),
        knob("diag_slope", |p| p.diag_slope, |p, v| p.diag_slope = v, 0.0, f64::INFINITY),
        knob("review_slope", |p| p.review_slope, |p, v| p.review_slope = v, 0.0, f64::INFINITY),
        knob(
            "review_gain",
            |p| p.review_gain,
            |p, v| p.review_gain = v,
            PROBABILITY.0,
            PROBABILITY.1,
        ),
        knob("hazard", |p| p.hazard, |p, v| p.hazard = v, 0.0, f64::INFINITY),
        knob(
            "review_hours_per_unit",
            |p| p.review_hours_per_unit,
            |p, v| p.review_hours_per_unit = v,
            0.0,
            f64::INFINITY,
        ),
        knob(
            "diag_hours_per_unit",
            |p| p.diag_hours_per_unit,
            |p, v| p.diag_hours_per_unit = v,
            0.0,
            f64::INFINITY,
        ),
        knob(
            "fix_hours_per_unit",
            |p| p.fix_hours_per_unit,
            |p, v| p.fix_hours_per_unit = v,
            0.0,
            f64::INFINITY,
        ),
        knob("alpha (authoring h/unit)", |p| p.alpha, |p, v| p.alpha = v, 0.0, f64::INFINITY),
        knob("savings (authoring fraction)", |p| p.savings, |p, v| p.savings = v, 0.0, 1.0),
    ]
}

/// The range a knob varies over, given a base parameter set.
pub fn range(knob: &Knob, base: &Params) -> (f64, f64) {
    let central = (knob.get)(base);
    let low = (central * (1.0 - knob.spread)).max(knob.floor);
    let high = (central * (1.0 + knob.spread)).min(knob.ceiling);
    // Degenerate ranges cannot be sampled, and a knob with no range has no variance to
    // attribute; keep them non-empty so the caller does not special-case them.
    if high - low < 1e-9 {
        (low, low + 1e-6)
    } else {
        (low, high)
    }
}

/// Variance-based indices for one input.
#[derive(Debug, Clone, PartialEq)]
pub struct SobolIndex {
    pub name: &'static str,
    /// Share of outcome variance explained by this input alone.
    pub first: f64,
    /// Share that would remain if every other input were fixed: first order plus every
    /// interaction this input takes part in.
    pub total: f64,
}

impl SobolIndex {
    /// The part of this input's influence that only appears through interactions.
    pub fn interaction(&self) -> f64 {
        (self.total - self.first).max(0.0)
    }
}

/// Aggregate of a full run, including the checks that make the indices interpretable.
#[derive(Debug, Clone, PartialEq)]
pub struct SobolReport {
    pub indices: Vec<SobolIndex>,
    pub variance: f64,
    pub samples: usize,
    /// `Σ S_i`, which cannot exceed 1 and falls short of it by the interaction variance.
    pub sum_first: f64,
    /// `Σ ST_i`, which cannot fall below 1 in an additive model and exceeds it with
    /// interactions. A value far below 1 means the sample size is too small to resolve the
    /// indices, not that the model is additive.
    pub sum_total: f64,
}

impl SobolReport {
    /// Total share of variance attributable to interactions between inputs.
    pub fn interaction_share(&self) -> f64 {
        (self.sum_total - self.sum_first).max(0.0)
    }

    /// The ranked attribution by total-order effect.
    pub fn ranked(&self) -> Vec<&SobolIndex> {
        let mut out: Vec<&SobolIndex> = self.indices.iter().collect();
        out.sort_by(|a, b| b.total.partial_cmp(&a.total).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn by_name(&self, name: &str) -> Option<&SobolIndex> {
        self.indices.iter().find(|index| index.name == name)
    }
}

/// Saltelli's first-order and total-order estimators over a generic model function.
///
/// Exposed generically so it can be validated against functions with analytically known
/// indices — an implementation of a variance decomposition that has only ever been run on the
/// model it was written for is an implementation nobody has checked.
pub fn sobol<F>(
    model: F,
    bounds: &[(f64, f64)],
    names: &[&'static str],
    samples: usize,
    seed: u64,
) -> SobolReport
where
    F: Fn(&[f64]) -> f64,
{
    let dimension = bounds.len();
    assert_eq!(dimension, names.len(), "one name per dimension");
    assert!(samples > 0, "need at least one sample");
    let mut rng = StdRng::seed_from_u64(seed);

    let draw = |rng: &mut StdRng, matrix: &mut Vec<Vec<f64>>| {
        for _ in 0..samples {
            let row: Vec<f64> =
                bounds.iter().map(|(low, high)| low + (high - low) * rng.gen::<f64>()).collect();
            matrix.push(row);
        }
    };

    let mut a: Vec<Vec<f64>> = Vec::with_capacity(samples);
    let mut b: Vec<Vec<f64>> = Vec::with_capacity(samples);
    draw(&mut rng, &mut a);
    draw(&mut rng, &mut b);

    let ya: Vec<f64> = a.iter().map(|row| model(row)).collect();
    let yb: Vec<f64> = b.iter().map(|row| model(row)).collect();

    let mean_a = ya.iter().sum::<f64>() / samples as f64;
    let mean_b = yb.iter().sum::<f64>() / samples as f64;
    // Pooled second moment: the variance of the outcome over the joint input space.
    let mean_square =
        ya.iter().chain(yb.iter()).map(|value| value * value).sum::<f64>() / (2 * samples) as f64;
    let variance = mean_square - ((mean_a + mean_b) / 2.0).powi(2);
    if variance <= 0.0 {
        return SobolReport {
            indices: names.iter().map(|name| SobolIndex { name, first: 0.0, total: 0.0 }).collect(),
            variance: 0.0,
            samples,
            sum_first: 0.0,
            sum_total: 0.0,
        };
    }

    let mut indices = Vec::with_capacity(dimension);
    for i in 0..dimension {
        // AB_i: A with column i taken from B.
        let yab: Vec<f64> = (0..samples)
            .map(|j| {
                let mut row = a[j].clone();
                row[i] = b[j][i];
                model(&row)
            })
            .collect();

        // Saltelli et al. (2010), eqs. for the first and total effect.
        let first = yb
            .iter()
            .zip(yab.iter())
            .zip(ya.iter())
            .map(|((b_j, ab_j), a_j)| b_j * (ab_j - a_j))
            .sum::<f64>()
            / samples as f64
            / variance;

        let total = ya.iter().zip(yab.iter()).map(|(a_j, ab_j)| (a_j - ab_j).powi(2)).sum::<f64>()
            / (2.0 * samples as f64)
            / variance;

        indices.push(SobolIndex { name: names[i], first, total });
    }

    let sum_first = indices.iter().map(|index| index.first).sum();
    let sum_total = indices.iter().map(|index| index.total).sum();
    SobolReport { indices, variance, samples, sum_first, sum_total }
}

/// The global sensitivity analysis of the model's outcome at the horizon.
///
/// The outcome is cumulative benefit at the horizon on the default parameterisation, with all
/// sixteen inputs varying jointly over their ranges.
pub fn model_indices(base: &Params, samples: usize, seed: u64) -> SobolReport {
    let knobs = knobs();
    let bounds: Vec<(f64, f64)> = knobs.iter().map(|knob| range(knob, base)).collect();
    let names: Vec<&'static str> = knobs.iter().map(|knob| knob.name).collect();

    sobol(
        |values| {
            let mut params = base.clone();
            for (knob, value) in knobs.iter().zip(values.iter()) {
                (knob.set)(&mut params, *value);
            }
            benefit_at_horizon(&params)
        },
        &bounds,
        &names,
        samples,
        seed,
    )
}
