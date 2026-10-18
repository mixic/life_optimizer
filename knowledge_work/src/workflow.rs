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

//! The AI-assisted workflow: a review stage whose load depends on how many defects escape it.
//!
//! # The load identity
//!
//! One generated item consumes review time in three ways: it is reviewed, and then either its
//! defect is caught and reworked, or its defect escapes and is discovered late and reworked at
//! greater cost. So the **review time consumed per generated item**, in units of the base
//! review time, is
//!
//! ```text
//! L(e) = 1 + d·[ (1 − e)·c_caught + e·c_escape ]
//! ```
//!
//! with `d` the defect share, `e` the share of defects that escape review, and the two `c`
//! terms the review cost of fixing a caught or an escaped defect. The whole model turns on the
//! fact that **`e` is not a constant**: it is the residual defect rate of the person doing the
//! reviewing, which [`human_capacity`] models as falling with the capacity that review itself
//! erodes.
//!
//! # Why this is the JIT link rather than an analogy
//!
//! `L` is an *effective process time*, which is the same object as a lean system's total
//! processing time including rework. Utilisation is then `λ·L·S/c`, and because `L > 1` the
//! utilisation of a system that only counts first-pass review systematically understates its
//! load. That is not a metaphor about factories: it is the same arithmetic, applied to service
//! times.
//!
//! The escape cost is where lean's own logic bites. A buffer absorbs variance; without one, a
//! defect that escapes is discovered by a customer, in production, or in an incident review,
//! and the rework is larger *and arrives outside the plan* — which is why `c_escape > c_caught`
//! is the empirically ordinary case and the one the model defaults to.

use human_capacity as capacity;

/// How work is admitted to the review stage.
///
/// This is the model's normative fork: the three variants are three answers to "what does the
/// firm do with the slack the machine created". They are not optimisations, they are policies,
/// and the framework's job is to price them rather than to pick one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Policy {
    /// Everything generated is pushed into review. The machine sets the arrival rate, and
    /// utilisation is an outcome rather than a choice. This is the no-buffer case.
    Push,
    /// Admission is whatever holds effective utilisation at the target, whatever arrives.
    /// The lean norm, transplanted: utilisation is preserved and the slack becomes backlog.
    UtilisationTarget,
    /// Admission is bounded by a work-in-process cap, by Little's law. The buffer exists, and
    /// what grows instead is the queue of work that has not been admitted.
    WipCap,
}

/// The workflow's parameters. Every one is declared; see [`WorkflowParameters::provenance_table`].
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowParameters {
    /// Items the organisation wants per day, before any AI-driven inflation.
    pub demand_per_day: f64,
    /// How much generated volume the machine produces relative to demand, at full assistance.
    /// The declared measure of "the machine makes it cheap to make more work".
    pub generation_multiplier: f64,
    /// Reviewers.
    pub servers: usize,
    /// Items one reviewer can process per day, at full capacity, before rework.
    pub review_rate: f64,
    /// Share of generated items that carry a defect before review.
    pub defect_share: f64,
    /// Review cost of fixing a caught defect, in multiples of the base review time.
    pub caught_rework_cost: f64,
    /// Review cost of an escaped defect discovered late, in multiples of the base review time.
    /// Greater than the caught cost because late discovery is the thing a buffer exists to
    /// prevent.
    pub escape_rework_cost: f64,
    /// Coefficient of variation of the arrival process. Raised by machines that make
    /// generation cheap and bursty.
    pub arrival_cv: f64,
    /// Coefficient of variation of review time, *including* the rework mix. Declared rather
    /// than derived: the compound arithmetic gives a term that is non-monotone in the escape
    /// share for reasons that belong to the algebra and not to the workflow, and a non-monotone
    /// variance channel would be an artefact dressed as a finding.
    pub service_cv: f64,
    /// The utilisation the target policy holds.
    pub utilisation_target: f64,
    /// The work-in-process cap, in items, for the buffered policy.
    pub wip_cap: f64,
    /// The cycle-time commitment the organisation makes, in days.
    pub deadline_days: f64,
}

impl Default for WorkflowParameters {
    fn default() -> Self {
        WorkflowParameters {
            demand_per_day: 12.0,
            generation_multiplier: 2.0,
            servers: 4,
            review_rate: 8.0,
            defect_share: 0.20,
            caught_rework_cost: 0.5,
            escape_rework_cost: 3.0,
            arrival_cv: 1.0,
            service_cv: 1.5,
            utilisation_target: 0.90,
            wip_cap: 40.0,
            deadline_days: 5.0,
        }
    }
}

impl WorkflowParameters {
    /// Every parameter with its provenance, in the shape the rest of the repository uses.
    pub fn provenance_table(&self) -> Vec<(&'static str, capacity::Provenance)> {
        const DECLARED: capacity::Provenance = capacity::Provenance::Declared {
            rationale: "no published estimate exists for this quantity in knowledge work; the \
                        value is chosen to be plausible in sign and magnitude and is not \
                        calibrated",
        };
        vec![
            ("demand per day", DECLARED),
            ("generation multiplier", DECLARED),
            ("reviewers", DECLARED),
            ("review rate", DECLARED),
            ("defect share", DECLARED),
            ("caught rework cost", DECLARED),
            ("escape rework cost", DECLARED),
            ("arrival coefficient of variation", DECLARED),
            ("service coefficient of variation", DECLARED),
            ("utilisation target", DECLARED),
            ("work-in-process cap", DECLARED),
            ("cycle-time deadline", DECLARED),
        ]
    }

    /// Review capacity in items per day, before rework.
    pub fn review_capacity(&self) -> f64 {
        self.servers as f64 * self.review_rate
    }

    /// Mean base review time, in days.
    pub fn base_service(&self) -> f64 {
        if self.review_rate > 0.0 {
            1.0 / self.review_rate
        } else {
            f64::INFINITY
        }
    }

    /// Items generated per day, before admission control.
    pub fn generated_per_day(&self) -> f64 {
        (self.demand_per_day * self.generation_multiplier).max(0.0)
    }
}

/// The share of defects that escape review at a given review capacity.
///
/// This is [`human_capacity::residual_defect_rate`], not a second model of it. The point of the
/// crate extraction was that the workflow and the person must not disagree about how many
/// defects get through, and this is the line that makes the disagreement impossible.
pub fn escape_share(review_capacity: f64, params: &capacity::ModelParameters) -> f64 {
    capacity::residual_defect_rate(review_capacity, params)
}

/// Review time consumed per generated item, in units of the base review time.
pub fn load_per_item(escape: f64, wf: &WorkflowParameters) -> f64 {
    let e = escape.clamp(0.0, 1.0);
    let defect = wf.defect_share.clamp(0.0, 1.0);
    1.0 + defect * ((1.0 - e) * wf.caught_rework_cost.max(0.0) + e * wf.escape_rework_cost.max(0.0))
}

/// Effective utilisation of the review stage at an admitted arrival rate.
///
/// `λ·L` is work in *base-service units* per day, and the capacity is `servers × review_rate`
/// items per day, so the ratio needs no service time in it. Writing `λ·L·S/c` as well —
/// which the first version of this function did — divides by the review rate twice and reports
/// a utilisation eight times too small for the defaults. It is worth saying where that showed
/// up: every closed-form test still agreed with itself, and it was the discrete-event simulation
/// realising a different utilisation from the one the model claimed that found it.
///
/// `None` when capacity is zero, which is a real answer: there is no utilisation of nothing.
pub fn effective_utilisation(
    admitted_per_day: f64,
    escape: f64,
    wf: &WorkflowParameters,
) -> Option<f64> {
    let capacity_per_day = wf.review_capacity();
    if capacity_per_day <= 0.0 {
        return None;
    }
    Some(admitted_per_day.max(0.0) * load_per_item(escape, wf) / capacity_per_day)
}

/// One year's — or one moment's — verdict on the workflow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    /// The escape share used, which comes from the review capacity.
    pub escape: f64,
    /// Review time per item, in base units.
    pub load: f64,
    pub generated_per_day: f64,
    pub admitted_per_day: f64,
    /// Effective utilisation after rework, which is what the queue actually experiences.
    pub rho: f64,
    /// Mean cycle time in days, or infinity when there is no steady state.
    pub cycle_time_days: f64,
    /// Work that arrives but is never admitted, per day. Grows without bound; it is the cost
    /// of admission control, and the reason a WIP cap is not a solution to overload.
    pub unadmitted_per_day: f64,
    /// Whether the stage has a steady state at all (`rho < 1`).
    pub stable: bool,
    /// Whether the cycle-time commitment is still met.
    pub meets_deadline: bool,
}

/// Evaluate the workflow at one review capacity under one policy.
pub fn evaluate(
    wf: &WorkflowParameters,
    policy: Policy,
    review_capacity: f64,
    capacity_params: &capacity::ModelParameters,
) -> Envelope {
    let escape = escape_share(review_capacity, capacity_params);
    let load = load_per_item(escape, wf);
    let generated = wf.generated_per_day();

    // Admission under the target policy: the arrival rate that holds effective utilisation at
    // the target, given the load this escape share implies.
    let admitted = match policy {
        Policy::Push => generated,
        Policy::UtilisationTarget => {
            // λ such that λ·L / capacity = target, so λ = target·capacity / L. The same
            // double-count fixed in `effective_utilisation` was present here, and it made the
            // target policy admit far more than its own target allowed.
            let capacity_for_target =
                wf.utilisation_target.clamp(0.0, 1.0) * wf.review_capacity() / load;
            if capacity_for_target.is_finite() {
                capacity_for_target.min(generated.max(0.0))
            } else {
                0.0
            }
        }
        Policy::WipCap => solve_wip_cap(wf, escape, generated),
    };

    let rho = effective_utilisation(admitted, escape, wf).unwrap_or(f64::INFINITY);
    let stable = rho < 1.0 && rho.is_finite();
    let cycle_time_days = if stable {
        crate::queue::cycle_time(
            rho,
            wf.servers,
            wf.arrival_cv,
            wf.service_cv,
            wf.base_service(),
        )
    } else {
        f64::INFINITY
    };

    Envelope {
        escape,
        load,
        generated_per_day: generated,
        admitted_per_day: admitted,
        rho,
        cycle_time_days,
        unadmitted_per_day: (generated - admitted).max(0.0),
        stable,
        meets_deadline: stable && cycle_time_days <= wf.deadline_days,
    }
}

/// The admitted rate that fills a work-in-process cap, by Little's law.
///
/// The cap makes admission and cycle time mutually determined: admitting more lengthens the
/// cycle, which by `WIP = λW` admits more, so the answer is a fixed point. Bisection on
/// `λ·W(λ) − cap`, which is monotone because both factors are increasing in `λ`.
fn solve_wip_cap(wf: &WorkflowParameters, escape: f64, generated: f64) -> f64 {
    let cap = wf.wip_cap.max(0.0);
    if cap <= 0.0 {
        return 0.0;
    }
    let wip_at = |admitted: f64| {
        let rho = effective_utilisation(admitted, escape, wf).unwrap_or(f64::INFINITY);
        if rho >= 1.0 || !rho.is_finite() {
            return f64::INFINITY;
        }
        let cycle = crate::queue::cycle_time(
            rho,
            wf.servers,
            wf.arrival_cv,
            wf.service_cv,
            wf.base_service(),
        );
        admitted * cycle
    };
    if wip_at(generated) <= cap {
        return generated;
    }
    let (mut low, mut high) = (0.0_f64, generated);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if wip_at(mid) <= cap {
            low = mid;
        } else {
            high = mid;
        }
        if (high - low) < 1e-12 {
            break;
        }
    }
    0.5 * (low + high)
}

/// The review capacity below which the workflow has **no steady state** under `policy`.
///
/// This is the envelope's edge, and it is the crate's central number: it says how much of the
/// capacity to judge the workflow can lose before the queue stops converging. Below it the
/// cycle time does not lengthen, it grows without bound.
///
/// Returns `None` when no crossing exists in the admissible range, which has two very different
/// meanings and the caller must distinguish them with [`is_stable_at`]: either the workflow is
/// already unstable with perfect review capacity, or it remains stable even at zero. Neither is
/// a threshold, and returning a number for either would be inventing one.
pub fn critical_review_capacity(
    wf: &WorkflowParameters,
    policy: Policy,
    capacity_params: &capacity::ModelParameters,
) -> Option<f64> {
    let rho_at = |review_capacity: f64| {
        effective_utilisation(
            admitted_for(wf, policy, review_capacity, capacity_params),
            escape_share(review_capacity, capacity_params),
            wf,
        )
        .unwrap_or(f64::INFINITY)
    };
    // Utilisation falls as review capacity rises: more capacity catches more defects, so fewer
    // escape and the rework load per item falls.
    if rho_at(1.0) >= 1.0 || rho_at(0.0) < 1.0 {
        return None;
    }
    let (mut low, mut high) = (0.0_f64, 1.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if rho_at(mid) >= 1.0 {
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

/// Whether the workflow has a steady state at a given review capacity.
pub fn is_stable_at(
    wf: &WorkflowParameters,
    policy: Policy,
    review_capacity: f64,
    capacity_params: &capacity::ModelParameters,
) -> bool {
    evaluate(wf, policy, review_capacity, capacity_params).stable
}

/// The admitted rate under a policy, extracted so the critical-capacity search and the
/// envelope evaluation cannot disagree about it.
fn admitted_for(
    wf: &WorkflowParameters,
    policy: Policy,
    review_capacity: f64,
    capacity_params: &capacity::ModelParameters,
) -> f64 {
    evaluate(wf, policy, review_capacity, capacity_params).admitted_per_day
}

/// Evaluate the workflow year by year against a capacity trajectory from `human_capacity`.
///
/// The coupling in one function: the person's capacity to judge falls as assistance is solved
/// for, the escape share rises with it, and the review stage carries more load at the same
/// nominal utilisation. Nothing about the workflow has to change for it to leave its envelope.
pub fn trajectory(
    states: &[capacity::State],
    wf: &WorkflowParameters,
    policy: Policy,
    capacity_params: &capacity::ModelParameters,
) -> Vec<Envelope> {
    states
        .iter()
        .map(|state| evaluate(wf, policy, state.capacity, capacity_params))
        .collect()
}

/// The first year in which the workflow leaves its envelope, if it ever does.
///
/// Two ways to leave it, and they are not the same finding: losing the steady state (`rho ≥ 1`,
/// which is unbounded lateness) or merely missing the commitment (`cycle time > deadline`).
/// The predicate selects one, so a caller cannot silently get the weaker claim.
pub fn first_year_outside(
    trajectory: &[Envelope],
    requirement: Requirement,
) -> Option<u32> {
    trajectory
        .iter()
        .position(|envelope| !requirement.holds(envelope))
        .map(|index| index as u32)
}

/// Which commitment the question is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    /// The queue must have a steady state at all.
    Stability,
    /// The cycle-time commitment must be met.
    Deadline,
}

impl Requirement {
    fn holds(self, envelope: &Envelope) -> bool {
        match self {
            Requirement::Stability => envelope.stable,
            Requirement::Deadline => envelope.meets_deadline,
        }
    }
}
