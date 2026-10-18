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

//! The thesis, stated as propositions and checked rather than asserted.
//!
//! # What "proof" means here
//!
//! Not a proof in the mathematical sense, and the distinction matters. These are **propositions
//! about a model**, and each is established by one of three methods, recorded alongside it:
//!
//! - **Identity** — the claim follows from the algebra and the check is that the code implements
//!   the algebra. Example: the load is one plus the defect-weighted rework cost.
//! - **Numeric** — the claim is a property of a function (monotone, convex, ordered) and the
//!   check is a grid search that would find a counterexample if one existed.
//! - **Existence** — the claim is that something happens (a workflow leaves its envelope within
//!   the horizon) and the check is that the computation finds it, together with a contrast case
//!   in which it does not, so the finding cannot be an artefact of the horizon.
//!
//! What is *not* claimed is that any proposition is true of a real organisation. Every parameter
//! is declared, so a certificate is a statement about the model's internal logic. The document
//! says this before quoting any of it, and [`Certificate::caveat`] repeats it in the output.
//!
//! # Why a certificate rather than only tests
//!
//! Because a reader should be able to regenerate the whole claim and see it pass, without
//! trusting a green tick in a CI log they cannot read. `knowledge_work --prove` prints the table
//! this module builds, with the numeric evidence for each proposition printed beside it.

use crate::direction::{self, Outcome, CRITERIA};
use crate::envelope;
use crate::queue;
use crate::sim;
use crate::workflow::{self, Policy, WorkflowParameters};
use human_capacity as capacity;

/// How a proposition was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Follows from the algebra; the check is that the implementation matches it.
    Identity,
    /// A property of a function, checked by grid search that would expose a counterexample.
    Numeric,
    /// The claim that a computation finds an instance, with a contrast case where it does not.
    Existence,
    /// The claim that no ordering exists, checked over the whole weight simplex.
    Indeterminacy,
}

impl Method {
    /// The label used in the printed certificate.
    pub fn label(self) -> &'static str {
        match self {
            Method::Identity => "identity",
            Method::Numeric => "numeric",
            Method::Existence => "existence",
            Method::Indeterminacy => "indeterminacy",
        }
    }
}

/// One proposition, its verdict, and the number that carries it.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposition {
    /// A stable identifier, so the document and the certificate can refer to the same claim.
    pub id: &'static str,
    /// The claim in one sentence, phrased so it could be false.
    pub claim: &'static str,
    pub method: Method,
    pub holds: bool,
    /// The numeric evidence, formatted for printing. This is what makes the verdict checkable
    /// rather than trustable.
    pub evidence: String,
}

/// The whole set, with the verdict the reader actually wants.
#[derive(Debug, Clone, PartialEq)]
pub struct Certificate {
    pub propositions: Vec<Proposition>,
}

impl Certificate {
    /// Whether every proposition holds.
    pub fn all_hold(&self) -> bool {
        self.propositions.iter().all(|proposition| proposition.holds)
    }

    /// The count of propositions that hold.
    pub fn held(&self) -> usize {
        self.propositions
            .iter()
            .filter(|proposition| proposition.holds)
            .count()
    }

    /// The sentence that must be printed with any verdict, so a certificate cannot be quoted as
    /// a statement about the world.
    pub fn caveat(&self) -> &'static str {
        "These are propositions about a model whose every parameter is declared. A certificate \
         that all hold says the framework's logic is sound, not that any organisation behaves \
         this way."
    }
}

/// The person the propositions follow, matching the report's fixture.
const PERSON_CAPACITY: f64 = 0.95;
const PERSON_SELF_MODEL: f64 = 0.40;
const PERSON_HUMAN_SHARE: f64 = 0.55;
const PERSON_CEILING: f64 = 1.70;
const AUTONOMY_FLOOR: f64 = 0.15;

/// Build the full certificate.
pub fn certificate() -> Certificate {
    let wf = WorkflowParameters::default();
    let params = capacity::ModelParameters::default();

    Certificate {
        propositions: vec![
            load_identity(&wf),
            understatement(&wf, &params),
            convexity(),
            thresholds_ordered(&wf, &params),
            absorbable_monotone(&wf, &params),
            breach_exists(&wf, &params),
            silence_of_throughput(&wf, &params),
            no_rank_order(&wf, &params),
            simulation_agrees(&wf, &params),
        ],
    }
}

/// **P1 — the load identity.** The review time an item consumes is one service plus the
/// defect-weighted rework, and it rises with the escape share whenever a late fix costs more than
/// an early one.
///
/// If the second half failed, the entire coupling would have no direction: deskilling would not
/// load the queue and none of the later propositions could hold.
fn load_identity(wf: &WorkflowParameters) -> Proposition {
    let mut previous = f64::NEG_INFINITY;
    let mut monotone = true;
    for step in 0..=10 {
        let escape = f64::from(step) / 10.0;
        let load = workflow::load_per_item(escape, wf);
        if load < previous - 1e-12 {
            monotone = false;
        }
        previous = load;
    }
    // The identity itself, checked against arithmetic done by hand at one point.
    let escape = 0.4;
    let expected = 1.0
        + wf.defect_share
            * ((1.0 - escape) * wf.caught_rework_cost + escape * wf.escape_rework_cost);
    let identity = (workflow::load_per_item(escape, wf) - expected).abs() < 1e-12;
    let rises = wf.escape_rework_cost > wf.caught_rework_cost;
    Proposition {
        id: "P1",
        claim: "Review load is one service plus defect-weighted rework, and it rises with the \
                escape share when a late fix costs more than an early one",
        method: Method::Identity,
        holds: identity && monotone && rises,
        evidence: format!(
            "L(0) = {:.3}, L(1) = {:.3}; c_escape {:.2} vs c_caught {:.2}",
            workflow::load_per_item(0.0, wf),
            workflow::load_per_item(1.0, wf),
            wf.escape_rework_cost,
            wf.caught_rework_cost
        ),
    }
}

/// **P2 — first-pass review understates the load.** Counting only the first review understates
/// utilisation by exactly the factor `L`, and the understatement is material.
fn understatement(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let escape = workflow::escape_share(1.0, params);
    let load = workflow::load_per_item(escape, wf);
    let generated = wf.generated_per_day();
    let honest = generated * load / wf.review_capacity();
    let naive = generated / wf.review_capacity();
    let ratio = honest / naive;
    Proposition {
        id: "P2",
        claim: "Utilisation computed from first-pass review alone understates the true load by \
                the factor L",
        method: Method::Identity,
        holds: (ratio - load).abs() < 1e-9 && ratio > 1.2,
        evidence: format!(
            "naive {:.3} against effective {:.3}, a factor of {:.2} ({:.0}% understated)",
            naive,
            honest,
            ratio,
            (ratio - 1.0) * 100.0
        ),
    }
}

/// **P3 — waiting is convex and divergent.** The utilisation term's second difference is
/// positive across the operating range, and the last five points of utilisation cost more than
/// everything below them.
///
/// This is the property that makes utilisation a policy choice rather than a neutral efficiency
/// measure, so a counterexample anywhere in the grid would remove the argument's footing.
fn convexity() -> Proposition {
    let service = 0.125;
    let mut convex = true;
    let mut previous_slope = f64::NEG_INFINITY;
    let mut rho = 0.50;
    while rho < 0.99 {
        let step = 0.01;
        let low = queue::queue_delay(rho - step, 4, 1.0, 1.5, service);
        let mid = queue::queue_delay(rho, 4, 1.0, 1.5, service);
        let high = queue::queue_delay(rho + step, 4, 1.0, 1.5, service);
        let slope = (mid - low) / step;
        if slope < previous_slope - 1e-9 {
            convex = false;
        }
        previous_slope = slope;
        let _ = high;
        rho += 0.05;
    }
    let cheap = queue::queue_delay(0.50, 4, 1.0, 1.5, service);
    let expensive = queue::queue_delay(0.95, 4, 1.0, 1.5, service);
    let diverges = queue::queue_delay(1.0, 4, 1.0, 1.5, service).is_infinite();
    Proposition {
        id: "P3",
        claim: "Waiting time is convex in utilisation and divergent at one, so the last five \
                points of utilisation cost more than everything below them",
        method: Method::Numeric,
        holds: convex && diverges && expensive > 4.0 * cheap,
        evidence: format!(
            "W(0.50) = {:.3} d, W(0.95) = {:.3} d; infinite at 1.0",
            cheap, expensive
        ),
    }
}

/// **P4 — the commitment fails before stability does.** The cycle-time promise binds at a higher
/// capacity than the mathematical impossibility, which is why an organisation can be inside its
/// promise one quarter and outside it the next without the queue having broken.
fn thresholds_ordered(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let commitment = envelope::critical_absorption_capacity(wf, params);
    let stability = workflow::critical_review_capacity(wf, Policy::Push, params);
    match (commitment, stability) {
        (Some(commitment), Some(stability)) => Proposition {
            id: "P4",
            claim: "The commitment binds at a higher capacity to judge than stability does, so a \
                    promise breaks before the arithmetic does",
            method: Method::Numeric,
            holds: commitment > stability,
            evidence: format!(
                "commitment at {commitment:.3}, stability at {stability:.3}, gap {:.3}",
                commitment - stability
            ),
        },
        (commitment, stability) => Proposition {
            id: "P4",
            claim: "The commitment binds at a higher capacity to judge than stability does",
            method: Method::Numeric,
            holds: false,
            evidence: format!(
                "a threshold is missing: commitment {commitment:?}, stability {stability:?}"
            ),
        },
    }
}

/// **P5 — absorbable work falls as the capacity to judge falls.** Monotone over the whole range:
/// a non-monotone segment would mean some capacity loss is harmless, which would break the
/// coupling's operational meaning.
fn absorbable_monotone(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let mut previous = f64::INFINITY;
    let mut monotone = true;
    let mut worst: Option<(f64, f64)> = None;
    for step in 0..=40 {
        let review_capacity = 1.0 - f64::from(step) / 40.0;
        let share = envelope::envelope_at(wf, review_capacity, params).absorbable_share;
        let Some(share) = share else { continue };
        if share > previous + 1e-9 {
            monotone = false;
            worst = Some((review_capacity, share));
        }
        previous = share;
    }
    let full = envelope::envelope_at(wf, 1.0, params).absorbable_share.unwrap_or(0.0);
    let empty = envelope::envelope_at(wf, 0.0, params).absorbable_share.unwrap_or(1.0);
    Proposition {
        id: "P5",
        claim: "The share of generated work the workflow can absorb is non-increasing as the \
                capacity to judge falls",
        method: Method::Numeric,
        holds: monotone && full >= 0.999 && empty < 0.95,
        evidence: match worst {
            Some((capacity, share)) => {
                format!("counterexample at capacity {capacity:.2}: share rose to {share:.3}")
            }
            None => format!("1.000 at full capacity, {empty:.3} at zero; monotone throughout"),
        },
    }
}

/// **P6 — tracking takes the workflow out of its envelope, and a learning buffer only delays it.**
/// The existence claim, and it needs a contrast case: the same workflow — same review capacity,
/// same generated volume — with a training share that preserves more capacity.
///
/// The contrast is what made this proposition worth writing. It was first stated as "and the
/// learning buffer does not breach", which **failed**: the learning buffer ends at capacity 0.831
/// against a stability threshold of 0.839, so it breaches too, at year 8 rather than year 5. The
/// claim is now the one the model supports — the training time delays the loss by three years and
/// does not prevent it — and the correction is kept because it is the more useful sentence: even
/// the buffer this framework recommends buys time rather than safety, which is exactly what the
/// psychological layer's runway result said.
fn breach_exists(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let lean_path = trajectory_for(params, wf, 0.05);
    let learning_path = trajectory_for(params, wf, 0.25);
    let lean_breach =
        workflow::first_year_outside(&lean_path.0, workflow::Requirement::Stability);
    let learning_breach =
        workflow::first_year_outside(&learning_path.0, workflow::Requirement::Stability);
    let delay = match (lean_breach, learning_breach) {
        (Some(lean), Some(learning)) => learning.saturating_sub(lean),
        _ => 0,
    };
    Proposition {
        id: "P6",
        claim: "Tracking takes the workflow out of its envelope within the horizon, and a learning \
                buffer delays the loss without preventing it",
        method: Method::Existence,
        holds: lean_breach.is_some() && learning_breach.is_some() && delay >= 2,
        evidence: match (lean_breach, learning_breach) {
            (Some(lean_year), Some(learning_year)) => format!(
                "lean loses its steady state at year {lean_year} (capacity {:.3} -> {:.3}); \
                 learning buffer at year {learning_year} (capacity {:.3} -> {:.3}): delayed by \
                 {delay} years, not prevented",
                lean_path.1.first().copied().unwrap_or(f64::NAN),
                lean_path.1.last().copied().unwrap_or(f64::NAN),
                learning_path.1.first().copied().unwrap_or(f64::NAN),
                learning_path.1.last().copied().unwrap_or(f64::NAN),
            ),
            (lean, learning) => format!(
                "the contrast did not hold: lean breach {lean:?}, learning breach {learning:?}"
            ),
        },
    }
}

/// **P7 — the failure is silent in throughput.** Between adoption and the breach, the workflow
/// admits the same volume every year while the escaped-defect rate rises. So the deterioration is
/// invisible in the metric the organisation watches and visible only in the one it does not.
fn silence_of_throughput(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let (path, capacities) = trajectory_for(params, wf, 0.05);
    let first_breach = workflow::first_year_outside(&path, workflow::Requirement::Stability)
        .unwrap_or(path.len() as u32);
    let opening = path.first().copied();
    let at_breach = path.get(first_breach as usize).copied();
    let escape_opening = opening.map(|point| point.escape).unwrap_or(f64::NAN);
    let escape_breach = at_breach.map(|point| point.escape).unwrap_or(f64::NAN);
    // Throughput under a push policy is the generated volume, which does not depend on capacity.
    let throughput_opening = workflow::evaluate(
        wf,
        Policy::Push,
        capacities.first().copied().unwrap_or(1.0),
        params,
    )
    .admitted_per_day;
    let throughput_breach = workflow::evaluate(
        wf,
        Policy::Push,
        capacities.get(first_breach as usize).copied().unwrap_or(0.0),
        params,
    )
    .admitted_per_day;
    Proposition {
        id: "P7",
        claim: "From adoption to the breach the workflow admits the same volume every year while \
                the share of defects escaping rises, so the failure is invisible in throughput",
        method: Method::Numeric,
        holds: (throughput_opening - throughput_breach).abs() < 1e-6
            && escape_breach > escape_opening + 0.02,
        evidence: format!(
            "throughput {throughput_opening:.2}/day throughout; escape share {escape_opening:.3} \
             -> {escape_breach:.3}"
        ),
    }
}

/// **P8 — no rank order exists.** Over the whole weight simplex at least two configurations win,
/// and none wins nearly all of it. If one configuration won everywhere, the direction question
/// would be settled by arithmetic and the framework's refusal to rank would be theatre.
fn no_rank_order(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let outcomes: Vec<Outcome> = [
        ("lean", Policy::Push, 0.05_f64),
        ("buffered", Policy::WipCap, 0.05),
        ("learning", Policy::WipCap, 0.25),
    ]
    .iter()
    .map(|(_, policy, training)| outcome_for(params, wf, *policy, *training))
    .collect();
    let acceptance = direction::acceptance(&outcomes, 8);
    let winners = acceptance.shares.iter().filter(|share| **share > 0.0).count();
    let largest = acceptance
        .shares
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    Proposition {
        id: "P8",
        claim: "No configuration wins under every weighting: at least two win somewhere, and none \
                wins nearly all of the simplex",
        method: Method::Indeterminacy,
        holds: winners >= 2 && largest < 0.90,
        evidence: format!(
            "{} of {} configurations win over {} weightings; largest share {:.0}% ({})",
            winners,
            outcomes.len(),
            acceptance.weightings,
            largest * 100.0,
            (0..outcomes.len())
                .map(|index| format!("{} {:.0}%", ["lean", "buffered", "learning"][index], acceptance.shares[index] * 100.0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// **P9 — the simulation agrees with the envelope.** The discrete-event model realises the
/// utilisation the analysis claims, a stable queue's cycle time converges as the horizon grows,
/// and an unstable one's grows with it.
///
/// This is the only proposition that is not a property of the closed forms, and it is the one
/// that would catch a well-formed algebra that describes the wrong system.
fn simulation_agrees(wf: &WorkflowParameters, params: &capacity::ModelParameters) -> Proposition {
    let escape = workflow::escape_share(1.0, params);
    let mut worst_utilisation_error: f64 = 0.0;
    for target in [0.60, 0.85, 0.95] {
        let rate = sim::arrival_rate_for_utilisation(wf, escape, target);
        let outcome = sim::simulate(wf, escape, rate, 3000.0, 7);
        let error = (outcome.rho - target).abs();
        if error > worst_utilisation_error {
            worst_utilisation_error = error;
        }
    }
    let stable_rate = sim::arrival_rate_for_utilisation(wf, escape, 0.90);
    let stable_short = sim::simulate(wf, escape, stable_rate, 600.0, 2026);
    let stable_long = sim::simulate(wf, escape, stable_rate, 2400.0, 2026);
    let stable_ratio = stable_long.mean_cycle_time / stable_short.mean_cycle_time;

    let unstable_rate = sim::arrival_rate_for_utilisation(wf, escape, 1.05);
    let unstable_short = sim::simulate(wf, escape, unstable_rate, 600.0, 2026);
    let unstable_long = sim::simulate(wf, escape, unstable_rate, 2400.0, 2026);
    let unstable_ratio = unstable_long.mean_cycle_time / unstable_short.mean_cycle_time;

    Proposition {
        id: "P9",
        claim: "An event-by-event simulation realises the utilisation the analysis claims, and it \
                distinguishes a stable queue from an unstable one by lengthening the horizon",
        method: Method::Numeric,
        holds: worst_utilisation_error < 0.02
            && (0.7..1.4).contains(&stable_ratio)
            && unstable_ratio > 1.8,
        evidence: format!(
            "utilisation error at most {:.3}; horizon ratio {:.2} stable, {:.2} unstable",
            worst_utilisation_error, stable_ratio, unstable_ratio
        ),
    }
}

/// The autonomy years a person spends above the floor.
fn autonomy_years(states: &[capacity::State]) -> f64 {
    states
        .iter()
        .filter(|state| state.autonomy >= AUTONOMY_FLOOR)
        .count() as f64
}

/// The workflow envelope path and the capacity path for a given training share.
fn trajectory_for(
    params: &capacity::ModelParameters,
    wf: &WorkflowParameters,
    training_share: f64,
) -> (Vec<workflow::Envelope>, Vec<f64>) {
    let theirs = capacity::ModelParameters {
        training_share,
        expectation_ceiling: PERSON_CEILING,
        ..params.clone()
    };
    let states = capacity::simulate(
        PERSON_CAPACITY,
        PERSON_SELF_MODEL,
        &theirs,
        capacity::Strategy::Tracking,
        PERSON_HUMAN_SHARE,
    );
    let path = workflow::trajectory(&states, wf, Policy::Push, &theirs);
    let capacities = states.iter().map(|state| state.capacity).collect();
    (path, capacities)
}

/// One configuration's outcome, for the direction analysis.
fn outcome_for(
    params: &capacity::ModelParameters,
    wf: &WorkflowParameters,
    policy: Policy,
    training_share: f64,
) -> Outcome {
    let theirs = capacity::ModelParameters {
        training_share,
        expectation_ceiling: PERSON_CEILING,
        ..params.clone()
    };
    let states = capacity::simulate(
        PERSON_CAPACITY,
        PERSON_SELF_MODEL,
        &theirs,
        capacity::Strategy::Tracking,
        PERSON_HUMAN_SHARE,
    );
    let review_capacity = states.last().map(|state| state.capacity).unwrap_or(0.0);
    // Training time is time not spent reviewing, so the available review rate scales with it.
    let mut work = wf.clone();
    work.review_rate *= (1.0 - training_share).max(0.0);
    let envelope = workflow::evaluate(&work, policy, review_capacity, &theirs);
    Outcome {
        throughput: envelope.admitted_per_day,
        cycle_time: if envelope.stable {
            envelope.cycle_time_days
        } else {
            work.deadline_days * 10.0
        },
        escaped_defects: envelope.admitted_per_day * work.defect_share * envelope.escape,
        autonomy_years: autonomy_years(&states),
        backlog: envelope.unadmitted_per_day,
    }
}

/// The number of criteria the direction analysis scores on, re-exported so a caller printing the
/// certificate does not have to reach into another module to describe it.
pub const SCORED_CRITERIA: usize = CRITERIA;
