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

//! The sustainable operating envelope: how much of the machine's output a workflow can actually
//! take before something breaks, and what it costs to keep taking all of it.
//!
//! # The question the envelope answers
//!
//! Not "what is the optimum" — the framework does not have one — but the question a manager
//! actually faces when a tool multiplies what the team can generate: **how much of that output
//! can be admitted without breaking the commitment, and what has to give when capacity falls?**
//!
//! [`EnvelopePoint::absorbable_share`] is that answer. It is the fraction of generated work that
//! can be admitted while still meeting the cycle-time commitment, computed by inverting the
//! queueing approximation at the deadline rather than by declaring a policy. When it reaches one,
//! the workflow can take everything; when it falls below one, the organisation is choosing
//! between cutting what it accepts, missing the commitment, or buying capacity.
//!
//! # Two constraints, and they are not the same
//!
//! A workflow can fail in two ways that the envelope keeps separate:
//!
//! - **stability** — effective utilisation reaching one, beyond which the cycle time does not
//!   lengthen but grows without bound;
//! - **the commitment** — the deadline, which bites earlier and is a choice rather than a law.
//!
//! With a generous deadline the stability bound is what binds; with a tight one the commitment
//! does, and it binds much earlier. Which of the two is active in a given organisation is an
//! empirical question, so [`deadline_sensitivity`] sweeps it rather than assuming an answer.

use human_capacity as capacity;

use crate::queue;
use crate::workflow::{self, WorkflowParameters};

/// One point on the envelope: the workflow evaluated against the bar its own output sets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopePoint {
    /// The review capacity the person brings, on the psychological model's `0..1` scale.
    pub review_capacity: f64,
    /// Share of defects that escape this capacity, from `human_capacity`.
    pub escape: f64,
    /// Review time consumed per generated item, in base units.
    pub load: f64,
    /// Effective utilisation if every generated item were admitted.
    pub rho_at_generated: f64,
    /// The highest effective utilisation that still meets the cycle-time commitment, when any
    /// utilisation can.
    pub rho_max_for_deadline: Option<f64>,
    /// The fraction of generated work that can be admitted while meeting the commitment.
    /// `None` when the commitment cannot be met at any utilisation.
    pub absorbable_share: Option<f64>,
    /// Whether the review stage has a steady state at all if everything is admitted.
    pub stable: bool,
    /// Whether admitting everything still meets the commitment.
    pub meets_deadline: bool,
}

/// Evaluate the envelope at one review capacity, admitting everything the machine generates.
pub fn envelope_at(
    wf: &WorkflowParameters,
    review_capacity: f64,
    capacity_params: &capacity::ModelParameters,
) -> EnvelopePoint {
    let workflow_envelope =
        workflow::evaluate(wf, workflow::Policy::Push, review_capacity, capacity_params);
    let escape = workflow::escape_share(review_capacity, capacity_params);
    let load = workflow::load_per_item(escape, wf);

    // The commitment is on cycle time, and the queueing inversion is in *waiting* time, so the
    // item's own service time has to come off the deadline first. A deadline shorter than one
    // service time is not a tight target, it is an impossible one, and saying so is the answer.
    let base = wf.base_service();
    let target_wait = wf.deadline_days - base;
    let rho_max_for_deadline = if target_wait > 0.0 {
        queue::utilisation_for_delay(target_wait, wf.servers, wf.arrival_cv, wf.service_cv, base)
    } else {
        None
    };

    let rho_at_generated = workflow_envelope.rho;
    let absorbable_share = rho_max_for_deadline.map(|rho_max| {
        if rho_at_generated <= 0.0 || !rho_at_generated.is_finite() {
            // Nothing is generated or the load is unbounded; there is no share to speak of.
            0.0
        } else {
            (rho_max / rho_at_generated).clamp(0.0, 1.0)
        }
    });

    EnvelopePoint {
        review_capacity,
        escape,
        load,
        rho_at_generated,
        rho_max_for_deadline,
        absorbable_share,
        stable: workflow_envelope.stable,
        meets_deadline: workflow_envelope.meets_deadline,
    }
}

/// The envelope across a grid of review capacities: the curve the figures are drawn from.
pub fn sweep(
    wf: &WorkflowParameters,
    capacity_params: &capacity::ModelParameters,
    steps: usize,
) -> Vec<EnvelopePoint> {
    let steps = steps.max(1);
    (0..=steps)
        .map(|step| {
            let review_capacity = step as f64 / steps as f64;
            envelope_at(wf, review_capacity, capacity_params)
        })
        .collect()
}

/// The review capacity below which the workflow can no longer absorb everything it generates.
///
/// This is the operational answer to "how much capacity can this team lose before the
/// commitment breaks", and it is computed from the commitment rather than from the stability
/// bound, because the commitment is what an organisation has actually promised.
///
/// Returns `None` when the workflow cannot absorb its own output even at full capacity, which is
/// not a threshold but a design failure and must be reported as one.
pub fn critical_absorption_capacity(
    wf: &WorkflowParameters,
    capacity_params: &capacity::ModelParameters,
) -> Option<f64> {
    let at = |review_capacity: f64| envelope_at(wf, review_capacity, capacity_params);
    let full = at(1.0);
    if full.absorbable_share.map(|share| share < 1.0).unwrap_or(true) {
        return None;
    }
    let empty = at(0.0);
    if empty.absorbable_share.map(|share| share >= 1.0).unwrap_or(false) {
        return None;
    }
    // The absorbable share falls monotonically as capacity falls, so a bisection on the crossing
    // is exact to the tolerance and cannot get the direction wrong.
    let (mut low, mut high) = (0.0_f64, 1.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        let absorbable = at(mid).absorbable_share.unwrap_or(0.0);
        if absorbable >= 1.0 {
            high = mid;
        } else {
            low = mid;
        }
        if (high - low) < 1e-12 {
            break;
        }
    }
    Some(0.5 * (low + high))
}

/// How the critical capacity moves with the cycle-time commitment.
///
/// A tighter commitment has to bind earlier, and the question worth asking is *how much*
/// earlier: if a two-day commitment instead of five costs very little capacity, the commitment
/// is nearly free and arguing about it is arguing about nothing. If it costs most of the margin,
/// then the commitment is doing the work and the deadline is the policy lever.
pub fn deadline_sensitivity(
    wf: &WorkflowParameters,
    capacity_params: &capacity::ModelParameters,
    deadlines: &[f64],
) -> Vec<(f64, Option<f64>)> {
    deadlines
        .iter()
        .map(|deadline| {
            let mut probe = wf.clone();
            probe.deadline_days = *deadline;
            (*deadline, critical_absorption_capacity(&probe, capacity_params))
        })
        .collect()
}
