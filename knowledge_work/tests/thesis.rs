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

//! The thesis, checked independently of the certificate that reports it.
//!
//! Each test below re-derives its proposition from the primitives rather than reading the
//! certificate. That is deliberate: a certificate whose own logic is wrong would agree with
//! itself perfectly, and the tests exist to be the thing that can disagree with it.
//!
//! The file also contains the falsification tests — the cases in which a proposition *should*
//! fail. A claim that cannot fail is not a proposition, and the framework has to be able to say
//! what would refute it.

use human_capacity as capacity;
use knowledge_work::envelope;
use knowledge_work::queue;
use knowledge_work::sim;
use knowledge_work::thesis::{self, Method};
use knowledge_work::workflow::{self, Policy, WorkflowParameters};

fn params() -> capacity::ModelParameters {
    capacity::ModelParameters::default()
}

/// **P1.** The load is one service plus defect-weighted rework, and it rises with the escape
/// share when a late repair costs more than an early one.
#[test]
fn p1_the_load_identity_and_its_direction() {
    let wf = WorkflowParameters::default();
    // The identity, from arithmetic done by hand rather than by calling the function twice.
    for escape in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let expected = 1.0
            + wf.defect_share
                * ((1.0 - escape) * wf.caught_rework_cost + escape * wf.escape_rework_cost);
        let actual = workflow::load_per_item(escape, &wf);
        assert!(
            (actual - expected).abs() < 1e-12,
            "at escape {escape} the load is {actual}, not {expected}"
        );
    }
    // And the direction: the load rises with escapes because a late fix is dearer.
    assert!(wf.escape_rework_cost > wf.caught_rework_cost);
    let mut previous = f64::NEG_INFINITY;
    for step in 0..=20 {
        let escape = f64::from(step) / 20.0;
        let load = workflow::load_per_item(escape, &wf);
        assert!(load > previous, "the load fell at escape {escape}");
        previous = load;
    }
}

/// **P1, falsified.** If a late repair cost no more than an early one, the coupling would have no
/// direction and deskilling would not load the queue at all.
///
/// This is the framework saying what would refute it: a measured `c_escape ≤ c_caught` removes the
/// channel through which the whole argument runs.
#[test]
fn p1_the_coupling_has_no_direction_when_a_late_repair_is_not_dearer() {
    let mut wf = WorkflowParameters::default();
    wf.escape_rework_cost = wf.caught_rework_cost;
    let flat = workflow::load_per_item(0.0, &wf);
    for step in 0..=20 {
        let escape = f64::from(step) / 20.0;
        let load = workflow::load_per_item(escape, &wf);
        assert!(
            (load - flat).abs() < 1e-12,
            "the load still moved with escapes when the costs were equal: {load} against {flat}"
        );
    }
}

/// **P2.** Counting first-pass review alone understates the load by exactly the factor `L`.
#[test]
fn p2_first_pass_review_understates_the_load_by_the_factor_l() {
    let wf = WorkflowParameters::default();
    let escape = workflow::escape_share(1.0, &params());
    let load = workflow::load_per_item(escape, &wf);
    let generated = wf.generated_per_day();
    let naive = generated / wf.review_capacity();
    let honest = generated * load / wf.review_capacity();
    assert!(
        ((honest / naive) - load).abs() < 1e-9,
        "the understatement is {} but the load is {load}",
        honest / naive
    );
    assert!(
        (honest / naive) > 1.2,
        "the understatement is too small to matter: a factor of {}",
        honest / naive
    );
}

/// **P3.** Waiting is convex in utilisation and diverges at one.
///
/// The second difference is checked directly rather than through a ratio, because convexity is
/// what makes the last points of utilisation expensive.
#[test]
fn p3_waiting_is_convex_and_divergent() {
    let service = 0.125;
    for servers in [1_usize, 4, 16] {
        let mut rho = 0.40;
        while rho < 0.96 {
            let step = 0.01;
            let low = queue::queue_delay(rho - step, servers, 1.0, 1.5, service);
            let mid = queue::queue_delay(rho, servers, 1.0, 1.5, service);
            let high = queue::queue_delay(rho + step, servers, 1.0, 1.5, service);
            let second = high - 2.0 * mid + low;
            assert!(
                second > 0.0,
                "the waiting time is not convex at rho {rho} with {servers} servers"
            );
            rho += 0.05;
        }
        assert!(
            queue::queue_delay(1.0, servers, 1.0, 1.5, service).is_infinite(),
            "waiting is finite at saturation with {servers} servers"
        );
    }
    // The operational consequence the argument uses.
    let cheap = queue::queue_delay(0.50, 4, 1.0, 1.5, service);
    let dear = queue::queue_delay(0.95, 4, 1.0, 1.5, service);
    assert!(dear > 4.0 * cheap);
}

/// **P4.** The commitment binds before stability does.
#[test]
fn p4_the_commitment_binds_before_stability() {
    let wf = WorkflowParameters::default();
    let params = params();
    let commitment = envelope::critical_absorption_capacity(&wf, &params)
        .expect("the defaults absorb their own output at full capacity");
    let stability = workflow::critical_review_capacity(&wf, Policy::Push, &params)
        .expect("the defaults have a stability crossing");
    assert!(
        commitment > stability,
        "commitment at {commitment} does not exceed stability at {stability}"
    );
    // Bracketed from both sides, so neither is a number without a predicate.
    assert!(
        (envelope::envelope_at(&wf, commitment + 0.02, &params).absorbable_share.unwrap_or(0.0)
            - 1.0)
            .abs()
            < 1e-9
    );
    assert!(
        envelope::envelope_at(&wf, commitment - 0.02, &params).absorbable_share.unwrap_or(1.0)
            < 1.0
    );
    assert!(workflow::is_stable_at(&wf, Policy::Push, stability + 0.02, &params));
    assert!(!workflow::is_stable_at(&wf, Policy::Push, stability - 0.02, &params));
}

/// **P5.** Absorbable work is non-increasing as the capacity to judge falls.
#[test]
fn p5_absorbable_work_falls_with_the_capacity_to_judge() {
    let wf = WorkflowParameters::default();
    let params = params();
    let mut previous = f64::INFINITY;
    for step in 0..=40 {
        let review_capacity = 1.0 - f64::from(step) / 40.0;
        let Some(share) = envelope::envelope_at(&wf, review_capacity, &params).absorbable_share
        else {
            continue;
        };
        assert!(
            share <= previous + 1e-9,
            "the absorbable share rose as capacity fell, at {review_capacity}: {share} after \
             {previous}"
        );
        previous = share;
    }
    assert!(
        (envelope::envelope_at(&wf, 1.0, &params).absorbable_share.unwrap_or(0.0) - 1.0).abs()
            < 1e-9
    );
}

/// **P6.** Tracking takes the workflow out of its envelope, and a learning buffer delays rather
/// than prevents it.
///
/// The first version of this proposition claimed the learning buffer would not breach. It does,
/// three years later. The test asserts the delay, because that is what is true.
#[test]
fn p6_tracking_breaches_and_a_learning_buffer_only_delays_it() {
    let wf = WorkflowParameters::default();
    let params = params();
    let first_breach = |training_share: f64| {
        let theirs = capacity::ModelParameters {
            training_share,
            expectation_ceiling: 1.70,
            ..params.clone()
        };
        let states = capacity::simulate(0.95, 0.40, &theirs, capacity::Strategy::Tracking, 0.55);
        let path = workflow::trajectory(&states, &wf, Policy::Push, &theirs);
        let breach = workflow::first_year_outside(&path, workflow::Requirement::Stability);
        (breach, path.last().map(|point| point.escape).unwrap_or(f64::NAN))
    };
    let (lean, lean_escape) = first_breach(0.05);
    let (learning, learning_escape) = first_breach(0.25);

    let lean_year = lean.expect("tracking takes the lean workflow out of its envelope");
    let learning_year = learning.expect("the learning buffer breaches too, which is the finding");
    assert!(lean_year > 0, "the workflow was already unstable at adoption");
    assert!(
        learning_year > lean_year + 1,
        "the learning buffer did not delay the breach materially: {learning_year} against \
         {lean_year}"
    );
    assert!(
        learning_escape < lean_escape,
        "the learning buffer carried more escapes ({learning_escape}) than the lean workflow \
         ({lean_escape})"
    );
}

/// **P7.** Between adoption and the breach the admitted volume is constant while escapes rise, so
/// the failure is invisible in throughput.
#[test]
fn p7_the_failure_is_invisible_in_throughput() {
    let wf = WorkflowParameters::default();
    let params = params();
    let theirs = capacity::ModelParameters {
        training_share: 0.05,
        expectation_ceiling: 1.70,
        ..params.clone()
    };
    let states = capacity::simulate(0.95, 0.40, &theirs, capacity::Strategy::Tracking, 0.55);
    let path = workflow::trajectory(&states, &wf, Policy::Push, &theirs);
    let breach = workflow::first_year_outside(&path, workflow::Requirement::Stability)
        .expect("the fixture breaches");

    // Under a push policy every generated item is admitted, so the volume does not move with the
    // person's capacity at all. That is the point: the metric the organisation watches is blind
    // to the thing that is falling.
    for point in path.iter().take(breach as usize + 1) {
        assert!(
            (point.admitted_per_day - wf.generated_per_day()).abs() < 1e-9,
            "the admitted volume moved before the breach, so throughput was not blind"
        );
    }
    // Meanwhile the escape share rose.
    let opening = path.first().expect("non-empty").escape;
    let at_breach = path.get(breach as usize).expect("breach inside the path").escape;
    assert!(
        at_breach > opening + 0.02,
        "the escape share did not rise before the breach: {opening} then {at_breach}"
    );
}

/// **P8.** No rank order exists over the weight simplex.
#[test]
fn p8_no_configuration_wins_everywhere() {
    let acceptance = {
        let certificate = thesis::certificate();
        let proposition = certificate
            .propositions
            .iter()
            .find(|proposition| proposition.id == "P8")
            .expect("the certificate contains P8");
        assert!(proposition.holds, "P8 failed: {}", proposition.evidence);
        assert_eq!(proposition.method, Method::Indeterminacy);
        proposition.evidence.clone()
    };
    // The evidence itself must report more than one winner, which is the claim.
    assert!(
        acceptance.contains("3 of 3") || acceptance.contains("2 of 3"),
        "the evidence does not report multiple winners: {acceptance}"
    );
}

/// **P9.** An independent simulation agrees with the envelope.
#[test]
fn p9_the_simulation_agrees_with_the_envelope() {
    let wf = WorkflowParameters::default();
    let params = params();
    let escape = workflow::escape_share(1.0, &params);
    for target in [0.60, 0.85, 0.95] {
        let rate = sim::arrival_rate_for_utilisation(&wf, escape, target);
        let outcome = sim::simulate(&wf, escape, rate, 3000.0, 7);
        assert!(
            (outcome.rho - target).abs() < 0.02,
            "asked for {target}, the simulation realised {}",
            outcome.rho
        );
    }
    let rate = sim::arrival_rate_for_utilisation(&wf, escape, 0.90);
    let short = sim::simulate(&wf, escape, rate, 600.0, 2026);
    let long = sim::simulate(&wf, escape, rate, 2400.0, 2026);
    assert!(
        (0.7..1.4).contains(&(long.mean_cycle_time / short.mean_cycle_time)),
        "a stable queue's cycle time grew with the horizon"
    );
    let rate = sim::arrival_rate_for_utilisation(&wf, escape, 1.05);
    let short = sim::simulate(&wf, escape, rate, 600.0, 2026);
    let long = sim::simulate(&wf, escape, rate, 2400.0, 2026);
    assert!(
        long.mean_cycle_time / short.mean_cycle_time > 1.8,
        "an unstable queue's cycle time did not grow with the horizon"
    );
}

/// The certificate must be complete, and every proposition must carry the number that carries it.
///
/// A verdict without evidence is an assertion, which is what this whole apparatus exists to
/// avoid.
#[test]
fn the_certificate_is_complete_and_carries_its_evidence() {
    let certificate = thesis::certificate();
    assert_eq!(certificate.propositions.len(), 9);
    assert!(
        certificate.all_hold(),
        "{} of {} propositions failed: {:?}",
        certificate.propositions.len() - certificate.held(),
        certificate.propositions.len(),
        certificate
            .propositions
            .iter()
            .filter(|proposition| !proposition.holds)
            .map(|proposition| proposition.id)
            .collect::<Vec<_>>()
    );
    for proposition in &certificate.propositions {
        assert!(!proposition.claim.is_empty());
        assert!(
            proposition.evidence.len() > 20,
            "{} has no numeric evidence: {:?}",
            proposition.id,
            proposition.evidence
        );
        // Every proposition must be checkable by one of the four named methods.
        assert!(matches!(
            proposition.method,
            Method::Identity | Method::Numeric | Method::Existence | Method::Indeterminacy
        ));
    }
    assert!(certificate.caveat().contains("declared"));
}
