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
    /// Percentile bootstrap interval for `first`, when a bootstrap was requested.
    pub first_ci: Option<(f64, f64)>,
    /// Percentile bootstrap interval for `total`.
    pub total_ci: Option<(f64, f64)>,
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
fn evaluate<F>(model: &F, matrix: &[Vec<f64>]) -> Vec<f64>
where
    F: Fn(&[f64]) -> f64,
{
    matrix.iter().map(|row| model(row)).collect()
}

/// The estimators, evaluated on a chosen set of sample rows.
///
/// Kept separate from the sampling so a bootstrap can call it on resampled rows without
/// re-running the model. Returns the per-input `(first, total)` pairs and the variance.
fn estimate(ya: &[f64], yb: &[f64], yabs: &[Vec<f64>], rows: &[usize]) -> (Vec<(f64, f64)>, f64) {
    let n = rows.len() as f64;
    let mean_a = rows.iter().map(|r| ya[*r]).sum::<f64>() / n;
    let mean_b = rows.iter().map(|r| yb[*r]).sum::<f64>() / n;
    let mean_square =
        rows.iter().map(|r| ya[*r] * ya[*r] + yb[*r] * yb[*r]).sum::<f64>() / (2.0 * n);
    let variance = mean_square - ((mean_a + mean_b) / 2.0).powi(2);
    if variance <= 0.0 {
        return (vec![(0.0, 0.0); yabs.len()], 0.0);
    }

    let mut out = Vec::with_capacity(yabs.len());
    for series in yabs {
        let first = rows.iter().map(|r| yb[*r] * (series[*r] - ya[*r])).sum::<f64>() / n / variance;
        let total =
            rows.iter().map(|r| (ya[*r] - series[*r]).powi(2)).sum::<f64>() / (2.0 * n) / variance;
        out.push((first, total));
    }
    (out, variance)
}

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    sorted[index]
}

/// Saltelli estimators with optional percentile bootstrap intervals.
///
/// The interval is required to report a ranking. Two point estimates 16 points apart look
/// ordered; if their intervals overlap, the ordering is not established, and a variance-based
/// attribution whose whole purpose is to rank contributors would be overstating itself.
pub fn sobol_with_ci<F>(
    model: F,
    bounds: &[(f64, f64)],
    names: &[&'static str],
    samples: usize,
    seed: u64,
    bootstrap: usize,
    bootstrap_seed: u64,
) -> SobolReport
where
    F: Fn(&[f64]) -> f64,
{
    let dimension = bounds.len();
    assert_eq!(dimension, names.len(), "one name per dimension");
    assert!(samples > 0, "need at least one sample");
    let mut rng = StdRng::seed_from_u64(seed);

    let draw = |rng: &mut StdRng| -> Vec<Vec<f64>> {
        (0..samples)
            .map(|_| {
                bounds.iter().map(|(low, high)| low + (high - low) * rng.gen::<f64>()).collect()
            })
            .collect()
    };
    let a = draw(&mut rng);
    let b = draw(&mut rng);

    let ya = evaluate(&model, &a);
    let yb = evaluate(&model, &b);
    let yabs: Vec<Vec<f64>> = (0..dimension)
        .map(|i| {
            (0..samples)
                .map(|j| {
                    let mut row = a[j].clone();
                    row[i] = b[j][i];
                    model(&row)
                })
                .collect()
        })
        .collect();

    let all: Vec<usize> = (0..samples).collect();
    let (point, variance) = estimate(&ya, &yb, &yabs, &all);

    // Percentile bootstrap over the sample rows.
    let mut first_draws = vec![Vec::with_capacity(bootstrap); dimension];
    let mut total_draws = vec![Vec::with_capacity(bootstrap); dimension];
    if bootstrap > 0 && variance > 0.0 {
        let mut boot_rng = StdRng::seed_from_u64(bootstrap_seed);
        for _ in 0..bootstrap {
            let rows: Vec<usize> = (0..samples).map(|_| boot_rng.gen_range(0..samples)).collect();
            let (draw, _) = estimate(&ya, &yb, &yabs, &rows);
            for i in 0..dimension {
                first_draws[i].push(draw[i].0);
                total_draws[i].push(draw[i].1);
            }
        }
    }

    let mut indices = Vec::with_capacity(dimension);
    for i in 0..dimension {
        let (first_ci, total_ci) = if first_draws[i].is_empty() {
            (None, None)
        } else {
            let mut f = first_draws[i].clone();
            f.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let mut s = total_draws[i].clone();
            s.sort_by(|x, y| x.partial_cmp(y).unwrap());
            (
                Some((percentile(&f, 0.025), percentile(&f, 0.975))),
                Some((percentile(&s, 0.025), percentile(&s, 0.975))),
            )
        };
        indices.push(SobolIndex {
            name: names[i],
            first: point[i].0,
            total: point[i].1,
            first_ci,
            total_ci,
        });
    }

    SobolReport {
        sum_first: indices.iter().map(|index| index.first).sum(),
        sum_total: indices.iter().map(|index| index.total).sum(),
        indices,
        variance,
        samples,
    }
}

/// Point estimates only, for callers that do not need intervals.
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
    sobol_with_ci(model, bounds, names, samples, seed, 0, 0)
}

/// The global sensitivity analysis of the model's outcome at the horizon.
///
/// The outcome is cumulative benefit at the horizon on the default parameterisation, with all
/// sixteen inputs varying jointly over their ranges.
pub fn model_indices(base: &Params, samples: usize, seed: u64) -> SobolReport {
    model_indices_with_ci(base, samples, seed, 0)
}

/// As [`model_indices`], with a percentile bootstrap over the sample rows.
pub fn model_indices_with_ci(
    base: &Params,
    samples: usize,
    seed: u64,
    bootstrap: usize,
) -> SobolReport {
    let knobs = knobs();
    let bounds: Vec<(f64, f64)> = knobs.iter().map(|knob| range(knob, base)).collect();
    let names: Vec<&'static str> = knobs.iter().map(|knob| knob.name).collect();

    sobol_with_ci(
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
        bootstrap,
        seed ^ 0x5eed,
    )
}

/// The share of the plausible parameter space in which the net effect is negative.
///
/// A single central cell is one point in a space of Declared values, so "the net sign is
/// negative" is either a statement about that point or a statement about the space. This
/// measures the space: draw uniformly over every input's range, and count how often the
/// cumulative benefit at the horizon is below zero.
#[derive(Debug, Clone, PartialEq)]
pub struct SignAudit {
    pub samples: usize,
    pub negative: usize,
    /// Proportion negative, with a Wilson score interval — a fraction without an interval is
    /// the same error as an index without one.
    pub fraction: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    /// The extremes of the outcome over the sampled space.
    pub worst: f64,
    pub best: f64,
}

impl SignAudit {
    /// True when the interval excludes zero, so the sign is a property of the space rather
    /// than of the sample.
    pub fn is_unanimous(&self) -> bool {
        self.negative == self.samples
    }
}

/// Draw over the joint space and count the sign, on the default band.
pub fn sign_audit(base: &Params, samples: usize, seed: u64) -> SignAudit {
    sign_audit_with_spread(base, samples, seed, 0.5)
}

/// As [`sign_audit`], with the width of the band as a parameter.
///
/// A robustness claim is only as good as the band it was computed on, and the default ±50%
/// band already excludes the one authoring-saving value that a measured study supports
/// (`σ = 0`, METR's null for experienced developers). Sweeping the band is therefore not
/// decoration: it tests whether the conclusion survives widening the prior, which is the
/// question the identification argument actually turns on.
pub fn sign_audit_with_spread(base: &Params, samples: usize, seed: u64, spread: f64) -> SignAudit {
    let mut knobs = knobs();
    for knob in knobs.iter_mut() {
        knob.spread = spread;
    }
    let bounds: Vec<(f64, f64)> = knobs.iter().map(|knob| range(knob, base)).collect();
    let mut rng = StdRng::seed_from_u64(seed);

    let mut negative = 0usize;
    let mut worst = f64::INFINITY;
    let mut best = f64::NEG_INFINITY;
    for _ in 0..samples {
        let mut params = base.clone();
        for (knob, (low, high)) in knobs.iter().zip(bounds.iter()) {
            let value = low + (high - low) * rng.gen::<f64>();
            (knob.set)(&mut params, value);
        }
        let benefit = benefit_at_horizon(&params);
        if benefit < 0.0 {
            negative += 1;
        }
        worst = worst.min(benefit);
        best = best.max(benefit);
    }

    let n = samples as f64;
    let p = negative as f64 / n;
    let z = 1.96;
    let denominator = 1.0 + z * z / n;
    let centre = (p + z * z / (2.0 * n)) / denominator;
    let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / denominator;

    SignAudit {
        samples,
        negative,
        fraction: p,
        ci_low: (centre - half).max(0.0),
        ci_high: (centre + half).min(1.0),
        worst,
        best,
    }
}
