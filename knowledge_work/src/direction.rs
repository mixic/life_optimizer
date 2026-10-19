// knowledge_work -- just-in-time logic in AI-assisted knowledge work
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

//! The direction question, operationalised as far as it can be without being answered.
//!
//! # What this module refuses to do
//!
//! It does not produce a single score, a ranking, or a recommendation. Anyone who wants one
//! has to supply the weights, and the weights are a moral position rather than a fact — which
//! is the whole content of the question "productivity toward what?".
//!
//! # What it does instead
//!
//! It takes the options a workflow model produces — a lean configuration, a buffered one, one
//! that holds utilisation down — scores each under **every** weighting in a grid over the
//! weight simplex, and reports for each option the share of weightings under which it comes
//! first. That is the honest form of a multi-criteria claim:
//!
//! > *If you weight autonomy above throughput, this option wins; if you weight throughput above
//! > autonomy, that one does; here is the surface on which the disagreement lives.*
//!
//! A framework that reported a winner would be smuggling a normative commitment in as
//! arithmetic. This one makes the commitment visible, and then hands it back.
//!
//! # Why the weights are still declared
//!
//! Because a grid over the simplex is only meaningful if its axes are the criteria that matter
//! to someone. The five here are declared in [`Weights::provenance_table`], along with the
//! admission that their *relative* importance is exactly what is under dispute. A reader who
//! thinks a sixth criterion belongs — say, the distribution of the queue's waiting time rather
//! than its mean — can say so, and the ranking will move.

/// One option's measured performance, in the units the models produce.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outcome {
    /// Throughput in items per day.
    pub throughput: f64,
    /// Mean cycle time in days.
    pub cycle_time: f64,
    /// Defects that escape review per day.
    pub escaped_defects: f64,
    /// Years a person spends above the autonomy floor: the psychological model's own output.
    pub autonomy_years: f64,
    /// Work generated but never admitted, per day.
    pub backlog: f64,
}

impl Outcome {
    /// The value on criterion `index`, signed so that **higher is always better**.
    ///
    /// Three of the five are goods and two are bads, and signing the bads here means the
    /// scoring code has one direction to get right instead of five.
    fn signed(&self, index: usize) -> f64 {
        match index {
            0 => self.throughput,
            1 => -self.cycle_time,
            2 => -self.escaped_defects,
            3 => self.autonomy_years,
            4 => -self.backlog,
            _ => f64::NAN,
        }
    }
}

/// How many criteria the framework scores on.
pub const CRITERIA: usize = 5;

/// The name of a criterion, for reports.
pub fn criterion_name(index: usize) -> &'static str {
    match index {
        0 => "throughput",
        1 => "cycle time",
        2 => "escaped defects",
        3 => "autonomy years",
        4 => "backlog",
        _ => "unknown",
    }
}

/// A point on the weight simplex: how much each criterion counts, summing to one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weights {
    pub throughput: f64,
    pub cycle_time: f64,
    pub escaped_defects: f64,
    pub autonomy_years: f64,
    pub backlog: f64,
}

impl Weights {
    /// A weight vector, normalised so the components sum to one.
    pub fn new(values: [f64; CRITERIA]) -> Self {
        let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
        let scale = if total > 0.0 { 1.0 / total } else { 0.0 };
        Weights {
            throughput: values[0].max(0.0) * scale,
            cycle_time: values[1].max(0.0) * scale,
            escaped_defects: values[2].max(0.0) * scale,
            autonomy_years: values[3].max(0.0) * scale,
            backlog: values[4].max(0.0) * scale,
        }
    }

    /// The weight on criterion `index`.
    pub fn get(&self, index: usize) -> f64 {
        match index {
            0 => self.throughput,
            1 => self.cycle_time,
            2 => self.escaped_defects,
            3 => self.autonomy_years,
            4 => self.backlog,
            _ => 0.0,
        }
    }

    /// The weights this framework reports its headline result under.
    ///
    /// Declared, and deliberately **not** anonymous: this vector says throughput and escaped
    /// defects matter most, which is the ordinary managerial view, and reporting it makes the
    /// ordinary manager's answer visible instead of pretending the framework has none. It is one
    /// point on the simplex, not a privileged one, and the report prints the ranking under it
    /// beside the ranking under every other.
    pub fn declared() -> Self {
        Weights::new([0.30, 0.15, 0.25, 0.20, 0.10])
    }

    /// Where each weight comes from.
    pub fn provenance_table() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "throughput",
                "declared: items per day is the quantity every productivity metric already \
                 reports, which is precisely why the framework has to show what ignoring the \
                 others would choose",
            ),
            ("cycle time", "declared: the commitment made to whoever receives the work"),
            (
                "escaped defects",
                "declared: the cost that lands on someone outside the team, which is why no \
                 team-level metric counts it",
            ),
            (
                "autonomy years",
                "declared: years spent above the autonomy floor, the psychological model's \
                 output; weighting it at all is the normative claim this framework exists to \
                 examine",
            ),
            (
                "backlog",
                "declared: the work generated and never admitted, which is what a utilisation \
                 target buys itself",
            ),
        ]
    }
}

/// Score one option under one weighting, on a `0..1` scale within the option set.
///
/// Min-max normalised **per criterion across the options compared**: the question is which
/// option is better, not whether any of them is good. That is a real limitation and it is
/// stated rather than hidden — a framework that normalised against an absolute standard would
/// be able to say an option is unacceptable, and this one cannot.
fn score(options: &[Outcome], weights: &Weights, index: usize) -> f64 {
    let mut total = 0.0;
    for criterion in 0..CRITERIA {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for option in options {
            let value = option.signed(criterion);
            low = low.min(value);
            high = high.max(value);
        }
        let span = high - low;
        let normalised = if span > 0.0 && span.is_finite() {
            (options[index].signed(criterion) - low) / span
        } else {
            // A criterion on which every option is identical carries no information and must
            // not reward or punish anyone; half is the neutral contribution.
            0.5
        };
        total += weights.get(criterion) * normalised;
    }
    total
}

/// Order the options best-first under a weighting. Higher score wins; ties by original order.
pub fn rank(options: &[Outcome], weights: &Weights) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..options.len()).collect();
    let scores: Vec<f64> = (0..options.len()).map(|index| score(options, weights, index)).collect();
    indices.sort_by(|a, b| {
        scores[*b].partial_cmp(&scores[*a]).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(b))
    });
    indices
}

/// An evenly spaced grid over the weight simplex.
///
/// Every way of splitting `steps` units of importance among the five criteria, which is the
/// simplex lattice. With `steps = 8` there are 495 weightings — enough that a ranking surviving
/// all of them is a real finding, few enough that the report can print the map.
pub fn weight_simplex(steps: usize) -> Vec<Weights> {
    let n = steps.max(1);
    let mut out = Vec::new();
    for a in 0..=n {
        for b in 0..=(n - a) {
            for c in 0..=(n - a - b) {
                for d in 0..=(n - a - b - c) {
                    let e = n - a - b - c - d;
                    out.push(Weights::new([a as f64, b as f64, c as f64, d as f64, e as f64]));
                }
            }
        }
    }
    out
}

/// The result of asking every weighting in a grid the same question.
#[derive(Debug, Clone, PartialEq)]
pub struct Acceptance {
    /// For each option, the share of weightings under which it ranks first.
    pub shares: Vec<f64>,
    /// How many weightings were tried.
    pub weightings: usize,
    /// The ranking under the declared weights.
    pub declared_ranking: Vec<usize>,
    /// Options that never come first under any weighting in the grid: dominated in the sense
    /// that matters, which is that no defensible set of priorities rescues them.
    pub never_first: Vec<usize>,
}

/// Ask every weighting which option it prefers.
pub fn acceptance(options: &[Outcome], steps: usize) -> Acceptance {
    let grid = weight_simplex(steps);
    let mut shares = vec![0.0; options.len()];
    for weights in &grid {
        if let Some(winner) = rank(options, weights).first() {
            shares[*winner] += 1.0;
        }
    }
    let total = grid.len().max(1) as f64;
    for share in shares.iter_mut() {
        *share /= total;
    }
    let never_first = (0..options.len()).filter(|index| shares[*index] <= 0.0).collect();
    Acceptance {
        shares,
        weightings: grid.len(),
        declared_ranking: rank(options, &Weights::declared()),
        never_first,
    }
}
