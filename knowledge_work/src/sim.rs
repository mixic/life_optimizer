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

//! A discrete-event check on the analytical envelope.
//!
//! # Why a simulator when the model has closed forms
//!
//! Because the model's most important claim is not a closed form. It is that the review stage
//! has **no steady state** once effective utilisation reaches one, and a closed-form threshold
//! that agrees with itself proves nothing. Here the queue is simulated event by event, with the
//! rework drawn per item and no use of the VUT approximation at all, so agreement between the
//! two is evidence and disagreement is a bug in one of them.
//!
//! # What is simulated, exactly
//!
//! An FCFS queue with `servers` identical servers, Poisson arrivals at the admitted rate, and a
//! per-item service time of
//!
//! ```text
//! S · (1 + X),  X = 0                     with probability 1 − d
//!                    c_caught             with probability d·(1 − e)
//!                    c_escape             with probability d·e
//! ```
//!
//! which is the load model of [`crate::workflow`] with the rework drawn rather than averaged.
//! The base review time is deterministic on purpose: the question is whether the *rework*
//! arithmetic is right, and adding distributional noise to the base service would obscure a
//! wrong answer rather than test it.
//!
//! Arrivals are generated over the horizon and then all of them are processed, including those
//! that finish after it. Truncating at the horizon would drop exactly the long waits that
//! distinguish a stable queue from an unstable one, which is the distinction being tested.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp};

use crate::workflow::WorkflowParameters;

/// What one simulated horizon came to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outcome {
    /// Mean time from arrival to completion, in days, over every item generated.
    pub mean_cycle_time: f64,
    /// Mean work in process over the horizon, by Little's law from the two quantities above.
    pub mean_wip: f64,
    /// Items generated and completed.
    pub items: usize,
    /// Realised arrival rate, which differs from the requested one when the horizon is short.
    pub arrival_rate: f64,
    /// Effective utilisation implied by the realised mean service time.
    pub rho: f64,
}

impl Outcome {
    /// `NaN` rather than a misleading zero when nothing was simulated.
    fn empty() -> Self {
        Outcome {
            mean_cycle_time: f64::NAN,
            mean_wip: f64::NAN,
            items: 0,
            arrival_rate: 0.0,
            rho: f64::NAN,
        }
    }
}

/// Simulate the review stage for `horizon_days` and report what the queue did.
///
/// `escape` is supplied rather than derived from a capacity, so the simulator can be pointed at
/// any escape share — including ones the analytical model would not produce — which is what
/// makes it an independent check rather than a second implementation of the same idea.
pub fn simulate(
    wf: &WorkflowParameters,
    escape: f64,
    arrival_rate: f64,
    horizon_days: f64,
    seed: u64,
) -> Outcome {
    if wf.servers == 0
        || arrival_rate <= 0.0
        || !arrival_rate.is_finite()
        || horizon_days <= 0.0
        || !horizon_days.is_finite()
        || !wf.base_service().is_finite()
    {
        return Outcome::empty();
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let exp = match Exp::new(arrival_rate) {
        Ok(exp) => exp,
        Err(_) => return Outcome::empty(),
    };

    let base = wf.base_service();
    let defect = wf.defect_share.clamp(0.0, 1.0);
    let escape = escape.clamp(0.0, 1.0);

    // Generate the arrivals and their service requirements first. Generating and processing
    // together would let the arrival process depend on how far behind the servers are, which
    // would quietly turn an open queue into a closed one.
    let mut arrivals: Vec<f64> = Vec::new();
    let mut services: Vec<f64> = Vec::new();
    let mut now = 0.0_f64;
    loop {
        now += exp.sample(&mut rng);
        if now > horizon_days {
            break;
        }
        let draw: f64 = rng.gen();
        let rework = if draw < defect {
            // Among defective items, the escape share is the share review fails to catch.
            let inner: f64 = rng.gen();
            if inner < escape {
                wf.escape_rework_cost.max(0.0)
            } else {
                wf.caught_rework_cost.max(0.0)
            }
        } else {
            0.0
        };
        arrivals.push(now);
        services.push(base * (1.0 + rework));
    }

    if arrivals.is_empty() {
        return Outcome::empty();
    }

    // FCFS across identical servers: each arrival starts at the earlier of its own arrival and
    // the first server to become free.
    let mut free_times = vec![0.0_f64; wf.servers];
    let mut total_cycle = 0.0_f64;
    let mut total_service = 0.0_f64;
    for (arrival, service) in arrivals.iter().zip(services.iter()) {
        let mut earliest = 0;
        for index in 1..free_times.len() {
            if free_times[index] < free_times[earliest] {
                earliest = index;
            }
        }
        let start = arrival.max(free_times[earliest]);
        let done = start + service;
        free_times[earliest] = done;
        total_cycle += done - arrival;
        total_service += service;
    }

    let items = arrivals.len();
    let mean_cycle_time = total_cycle / items as f64;
    let mean_service = total_service / items as f64;
    let rho = mean_service * arrival_rate / wf.servers as f64;
    Outcome { mean_cycle_time, mean_wip: arrival_rate * mean_cycle_time, items, arrival_rate, rho }
}

/// The arrival rate at which the simulation's own effective utilisation reaches `target`.
///
/// Used by the tests and the report to place a simulation on a chosen point of the utilisation
/// axis without having to invert the load model by hand.
pub fn arrival_rate_for_utilisation(wf: &WorkflowParameters, escape: f64, target: f64) -> f64 {
    let mean_service = wf.base_service() * crate::workflow::load_per_item(escape, wf);
    if mean_service <= 0.0 || !mean_service.is_finite() {
        return 0.0;
    }
    target * wf.servers as f64 / mean_service
}
