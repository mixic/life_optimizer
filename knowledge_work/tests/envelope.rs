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

//! The envelope's contract: the share is a share, the boundary brackets its predicate, and a
//! tighter commitment has to bind earlier.

use human_capacity as capacity;
use knowledge_work::envelope::*;
use knowledge_work::workflow::{self, Policy, WorkflowParameters};

fn params() -> capacity::ModelParameters {
    capacity::ModelParameters::default()
}

/// The absorbable share must be a fraction of generated work, in the unit interval, and it must
/// be one when the workflow can take everything.
#[test]
fn the_absorbable_share_is_a_share() {
    let wf = WorkflowParameters::default();
    let params = params();
    let full = envelope_at(&wf, 1.0, &params);
    assert!(
        (full.absorbable_share.unwrap_or(0.0) - 1.0).abs() < 1e-9,
        "at full capacity the workflow should absorb everything, got {:?}",
        full.absorbable_share
    );
    assert!(full.stable && full.meets_deadline);

    for point in sweep(&wf, &params, 20) {
        let share = point.absorbable_share.expect("a deadline of five days is reachable");
        assert!(
            (0.0..=1.0).contains(&share),
            "absorbable share {share} at capacity {}",
            point.review_capacity
        );
    }
}

/// Losing the capacity to judge must reduce what the workflow can take: the share must be
/// monotone in review capacity, or the coupling has no operational content.
#[test]
fn the_absorbable_share_falls_as_review_capacity_falls() {
    let wf = WorkflowParameters::default();
    let params = params();
    let mut previous = f64::INFINITY;
    for step in (0..=20).rev() {
        let h = f64::from(step) / 20.0;
        let share = envelope_at(&wf, h, &params)
            .absorbable_share
            .expect("reachable at every capacity on this grid");
        assert!(
            share <= previous + 1e-9,
            "the absorbable share rose as capacity fell, at h {h}: {share} after {previous}"
        );
        previous = share;
    }
    // And at zero capacity it is materially below one: the workflow cannot take its own output.
    assert!(envelope_at(&wf, 0.0, &params).absorbable_share.unwrap_or(1.0) < 0.95);
}

/// The critical absorption capacity must bracket its own predicate, and it must sit **above**
/// the stability threshold: a commitment binds earlier than a mathematical impossibility.
#[test]
fn the_commitment_binds_before_stability_does() {
    let wf = WorkflowParameters::default();
    let params = params();
    let absorption = critical_absorption_capacity(&wf, &params)
        .expect("the defaults absorb their own output at full capacity");
    let stability = workflow::critical_review_capacity(&wf, Policy::Push, &params)
        .expect("the defaults have a stability crossing");
    println!("capacity needed for the commitment: {absorption:.4}; for stability: {stability:.4}");
    assert!(
        absorption > stability,
        "the commitment binds at {absorption} but stability only fails at {stability}, so the \
         commitment is doing no work"
    );
    // Bracketed from both sides.
    let just_above = envelope_at(&wf, absorption + 0.02, &params);
    let just_below = envelope_at(&wf, absorption - 0.02, &params);
    assert!((just_above.absorbable_share.unwrap_or(0.0) - 1.0).abs() < 1e-9);
    assert!(just_below.absorbable_share.unwrap_or(1.0) < 1.0);
}

/// A tighter commitment has to bind at a higher capacity, and its cost is what decides whether
/// the deadline is a policy lever or a rounding error.
///
/// The second half of this test is the finding. Beyond a certain tightness the workflow cannot
/// meet the commitment **even at full review capacity**, and the function returns `None` rather
/// than a capacity outside its own range: that is a design failure, and reporting it as a
/// threshold would dress it up as a trade-off.
#[test]
fn a_tighter_commitment_costs_capacity_and_eventually_becomes_impossible() {
    let wf = WorkflowParameters::default();
    let params = params();
    let curve = deadline_sensitivity(&wf, &params, &[7.0, 5.0, 3.0, 2.0, 1.0, 0.5]);
    println!(
        "critical capacity by deadline: {}",
        curve
            .iter()
            .map(|(d, c)| format!("{d:.0}d -> {:.3}", c.unwrap_or(f64::NAN)))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let reachable: Vec<(f64, f64)> =
        curve.iter().filter_map(|(deadline, critical)| critical.map(|c| (*deadline, c))).collect();
    assert!(
        reachable.len() >= 3,
        "the fixture should contain several reachable deadlines, got {reachable:?}"
    );
    let mut previous = 0.0;
    for (deadline, critical) in &reachable {
        assert!(
            *critical >= previous - 1e-12,
            "deadline {deadline} days needs only {critical} capacity, less than the looser \
             deadline before it ({previous})"
        );
        assert!(
            *critical <= 1.0,
            "a critical capacity of {critical} is outside the admissible range"
        );
        previous = *critical;
    }

    // The commitments that cannot be met at all must be refused, and they must be the tight
    // ones: a one-day commitment on a workflow whose review alone takes 0.125 days per item
    // cannot be met, and saying so is the answer.
    let impossible: Vec<f64> = curve
        .iter()
        .filter(|(_, critical)| critical.is_none())
        .map(|(deadline, _)| *deadline)
        .collect();
    assert!(
        !impossible.is_empty(),
        "the fixture should contain a deadline the workflow cannot meet"
    );
    for deadline in &impossible {
        assert!(*deadline <= 1.0, "deadline {deadline} was refused but should be reachable");
    }

    // A deadline shorter than one service time is refused for a more basic reason.
    let mut impossible = wf.clone();
    impossible.deadline_days = 0.01;
    let point = envelope_at(&impossible, 1.0, &params);
    assert!(point.rho_max_for_deadline.is_none());
    assert!(point.absorbable_share.is_none());
    assert!(!point.meets_deadline);
}
/// The margin that §6.4's breach consumes comes from the *generated volume*, not from the
/// utilisation target. This is the claim §4.4 makes when it points a sceptic at the generation
/// multiplier, and it is worth a test because it is the sentence that tells a reader where to
/// attack the argument.
#[test]
fn the_margin_comes_from_generated_volume_not_the_utilisation_target() {
    let params = params();
    let mut person = params.clone();
    person.expectation_ceiling = 1.7;
    let states = capacity::simulate(0.95, 0.40, &person, capacity::Strategy::Tracking, 0.55);

    let declared = WorkflowParameters::default();
    let path = knowledge_work::workflow::trajectory(
        &states,
        &declared,
        knowledge_work::workflow::Policy::Push,
        &params,
    );
    assert!(
        path.iter().any(|point| !point.stable),
        "the declared multiplier should drive the workflow out of its envelope"
    );

    let mut restrained = declared.clone();
    restrained.generation_multiplier = 1.5;
    let calm = knowledge_work::workflow::trajectory(
        &states,
        &restrained,
        knowledge_work::workflow::Policy::Push,
        &params,
    );
    assert!(
        calm.iter().all(|point| point.stable),
        "at a multiplier of 1.5 the workflow should stay inside its envelope throughout"
    );
    // And the utilisation target is not what did it: the target policy never appears here.
    let opening = calm.first().expect("a non-empty trajectory").rho;
    assert!(
        (0.60..0.80).contains(&opening),
        "the restrained workflow opens at {opening}, not the comfortable margin the document \
         claims"
    );
}
