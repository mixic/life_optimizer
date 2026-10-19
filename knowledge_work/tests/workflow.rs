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

//! The workflow's contract: the load identity, the envelope, and the year it is breached.

use human_capacity as capacity;
use knowledge_work::sim;
use knowledge_work::workflow::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

fn capacity_params() -> capacity::ModelParameters {
    capacity::ModelParameters::default()
}

/// The load identity is the model, so it is checked against arithmetic done by hand rather than
/// against the function's own output.
#[test]
fn the_load_identity_is_the_arithmetic_it_claims_to_be() {
    let wf = WorkflowParameters::default();
    let escape = 0.40;
    // L = 1 + d[(1 − e)c_caught + e·c_escape]
    let expected = 1.0
        + wf.defect_share
            * ((1.0 - escape) * wf.caught_rework_cost + escape * wf.escape_rework_cost);
    assert!(close(load_per_item(escape, &wf), expected));

    // With no defects there is no rework and the load is exactly one service.
    let mut clean = wf.clone();
    clean.defect_share = 0.0;
    assert!(close(load_per_item(escape, &clean), 1.0));

    // With every defect escaping, the load is the escape cost, which must exceed the caught
    // cost — otherwise the model has no reason to care whether review works.
    let mut all_escape = wf.clone();
    all_escape.defect_share = 1.0;
    let escaped = all_escape.escape_rework_cost;
    assert!(close(load_per_item(1.0, &all_escape), 1.0 + escaped));
    assert!(all_escape.escape_rework_cost > all_escape.caught_rework_cost);
}

/// The escape share comes from `human_capacity` and must fall as review capacity rises: that
/// monotonicity is what makes the capacity trajectory matter to the queue at all.
#[test]
fn the_escape_share_falls_as_review_capacity_rises() {
    let params = capacity_params();
    let mut previous = f64::INFINITY;
    for step in 0..=20 {
        let h = f64::from(step) / 20.0;
        let escape = escape_share(h, &params);
        assert!((0.0..=1.0).contains(&escape), "escape {escape} outside the unit interval");
        assert!(escape < previous + 1e-12, "escape rose with capacity at h {h}");
        previous = escape;
    }
    // The floor of the psychological model is the ceiling of this one: even a perfect reviewer
    // cannot catch the defects the machine and the tests share.
    let floor = capacity::residual_defect_floor(&params);
    assert!(
        escape_share(1.0, &params) >= floor,
        "the escape share at full capacity is below the correlated floor"
    );
    assert!(escape_share(0.0, &params) > escape_share(1.0, &params));
}

/// The declared defaults must sit **inside** the envelope at full review capacity, or the
/// model's baseline is an unstable workflow and every other result is about a system that never
/// worked.
///
/// They sit inside it narrowly, and that is itself worth an assertion: at the defaults the
/// workflow runs at roughly 97% of capacity, so the margin that a capacity loss has to eat
/// through is three per cent.
#[test]
fn the_declared_defaults_start_inside_the_envelope() {
    let wf = WorkflowParameters::default();
    let envelope = evaluate(&wf, Policy::Push, 1.0, &capacity_params());
    assert!(
        envelope.stable,
        "the defaults are already unstable at full capacity: rho {}",
        envelope.rho
    );
    assert!(
        (0.90..1.0).contains(&envelope.rho),
        "the defaults should be near the edge, not comfortable: rho {}",
        envelope.rho
    );
    assert!(envelope.meets_deadline);
    assert!(close(envelope.generated_per_day, wf.demand_per_day * wf.generation_multiplier));
}

/// The central coupling: a person whose capacity to judge falls takes the workflow out of its
/// envelope without anything about the workflow changing.
#[test]
fn a_falling_capacity_to_judge_drives_the_workflow_out_of_its_envelope() {
    let wf = WorkflowParameters::default();
    let params = capacity_params();

    // A well-trained person under a demanding settled bar: their capacity erodes because
    // assistance is solved for rather than chosen.
    let mut person = params.clone();
    person.expectation_ceiling = 1.9;
    let states = capacity::simulate(0.95, 0.4, &person, capacity::Strategy::Tracking, 0.55);
    assert!(
        states.last().map(|s| s.capacity).unwrap_or(1.0) < 0.85,
        "the fixture did not actually deskill: capacity ends at {:?}",
        states.last().map(|s| s.capacity)
    );

    let path = trajectory(&states, &wf, Policy::Push, &params);
    let first = first_year_outside(&path, Requirement::Stability);
    assert!(
        first.is_some(),
        "the workflow never left its envelope despite capacity falling from 0.95 to {}",
        path.last().map(|e| e.escape).unwrap_or(f64::NAN)
    );
    let year = first.expect("asserted");
    assert!(year > 0, "the workflow was already outside its envelope at year zero");
    assert!(!path[year as usize].stable);
    // Nothing about the workflow changed, so the breach is attributable to the person.
    assert!(path[year as usize].generated_per_day == wf.generated_per_day());
}

/// The critical capacity must be the boundary it claims to be: stable just above, unstable just
/// below. A threshold that does not bracket its own predicate is a number, not a threshold.
#[test]
fn the_critical_capacity_brackets_the_stability_predicate() {
    let wf = WorkflowParameters::default();
    let params = capacity_params();
    let critical = critical_review_capacity(&wf, Policy::Push, &params)
        .expect("the defaults have a crossing: stable at full capacity, not at zero");
    assert!(
        (0.0..1.0).contains(&critical),
        "the crossing is outside the admissible range: {critical}"
    );
    assert!(
        !is_stable_at(&wf, Policy::Push, critical - 0.02, &params),
        "the workflow is still stable below the critical capacity {critical}"
    );
    assert!(
        is_stable_at(&wf, Policy::Push, critical + 0.02, &params),
        "the workflow is not stable above the critical capacity {critical}"
    );
    println!("critical review capacity: {critical:.4}");
}

/// A WIP cap does not fix overload; it relocates it. The workflow becomes stable and the
/// lateness becomes a backlog of work that is never admitted — which is why a buffer is not a
/// substitute for capacity, and why lean's removal of buffers is not undone by adding one
/// without changing anything else.
#[test]
fn a_work_in_process_cap_trades_unbounded_lateness_for_unbounded_backlog() {
    let wf = WorkflowParameters::default();
    let params = capacity_params();
    // A deskilled review stage: the escape share that pushed the pushed policy over one.
    let escape = escape_share(0.5, &params);

    let pushed = evaluate(&wf, Policy::Push, 0.5, &params);
    assert!(!pushed.stable, "the pushed workflow should be unstable here");
    assert!(pushed.cycle_time_days.is_infinite());
    assert!(close(pushed.escape, escape));

    let capped = evaluate(&wf, Policy::WipCap, 0.5, &params);
    assert!(capped.stable, "the cap did not restore a steady state: rho {}", capped.rho);
    assert!(capped.cycle_time_days.is_finite());
    assert!(
        capped.unadmitted_per_day > 0.0,
        "the cap bound without leaving any work unadmitted, so it was not binding"
    );
    // Little's law: the admitted rate times the cycle time is the cap, because that is what the
    // cap was solved for.
    let implied = capped.admitted_per_day * capped.cycle_time_days;
    assert!(
        (implied - wf.wip_cap).abs() < 1e-3 * wf.wip_cap.max(1.0),
        "the cap is not filled: {implied} against a cap of {}",
        wf.wip_cap
    );
    // The target policy reaches the same conclusion by a different route: utilisation is held,
    // so the shortfall appears as unadmitted work rather than as queueing.
    let targeted = evaluate(&wf, Policy::UtilisationTarget, 0.5, &params);
    assert!(targeted.stable);
    assert!(
        (targeted.rho - wf.utilisation_target).abs() < 1e-9,
        "the target policy did not hold its target: rho {}",
        targeted.rho
    );
    assert!(targeted.unadmitted_per_day > 0.0);
}

/// The analytical envelope's central claim — no steady state at or above one — must be what an
/// event-by-event simulation does, since the closed form cannot validate itself.
///
/// Stability is detected by lengthening the horizon: a stable queue's mean cycle time converges,
/// an unstable one's grows without bound. Comparing two analysis-free simulations is a stronger
/// test than comparing either to the approximation.
#[test]
fn the_simulation_agrees_that_utilisation_at_one_is_the_edge() {
    let wf = WorkflowParameters::default();
    let escape = escape_share(1.0, &capacity_params());

    let stable_rate = sim::arrival_rate_for_utilisation(&wf, escape, 0.90);
    let short = sim::simulate(&wf, escape, stable_rate, 600.0, 2026);
    let long = sim::simulate(&wf, escape, stable_rate, 2400.0, 2026);
    assert!(short.items > 500 && long.items > 2000);
    let stable_ratio = long.mean_cycle_time / short.mean_cycle_time;
    assert!(
        (0.7..1.4).contains(&stable_ratio),
        "a stable queue's cycle time should not grow with the horizon, but it went from {} to \
         {} (a factor of {stable_ratio:.2})",
        short.mean_cycle_time,
        long.mean_cycle_time
    );

    let unstable_rate = sim::arrival_rate_for_utilisation(&wf, escape, 1.05);
    let short_u = sim::simulate(&wf, escape, unstable_rate, 600.0, 2026);
    let long_u = sim::simulate(&wf, escape, unstable_rate, 2400.0, 2026);
    let unstable_ratio = long_u.mean_cycle_time / short_u.mean_cycle_time;
    assert!(
        unstable_ratio > 1.8,
        "an unstable queue's cycle time should grow with the horizon, but it went from {} to {} \
         (a factor of {unstable_ratio:.2})",
        short_u.mean_cycle_time,
        long_u.mean_cycle_time
    );
    println!(
        "stable: {:.3} -> {:.3} days (x{stable_ratio:.2}); unstable: {:.1} -> {:.1} days \
         (x{unstable_ratio:.2})",
        short.mean_cycle_time,
        long.mean_cycle_time,
        short_u.mean_cycle_time,
        long_u.mean_cycle_time
    );
}

/// The simulation's own realised utilisation must agree with the number the model computed,
/// which is the check that the load identity is right rather than merely self-consistent.
#[test]
fn the_simulation_realises_the_utilisation_the_model_claims() {
    let wf = WorkflowParameters::default();
    let escape = escape_share(1.0, &capacity_params());
    for target in [0.6, 0.85, 0.95] {
        let rate = sim::arrival_rate_for_utilisation(&wf, escape, target);
        let outcome = sim::simulate(&wf, escape, rate, 3000.0, 7);
        assert!(
            (outcome.rho - target).abs() < 0.02,
            "asked for {target}, the simulation realised {}",
            outcome.rho
        );
        // Little's law on the simulated numbers, which must hold exactly by construction.
        let expected = outcome.arrival_rate * outcome.mean_cycle_time;
        assert!((outcome.mean_wip - expected).abs() < 1e-6 * expected.max(1.0));
        assert!(outcome.mean_cycle_time > wf.base_service());
    }
}

/// Every parameter must be declared, and the table must cover the whole struct: a parameter
/// that exists in the model but not in the provenance table is an unlabelled number.
#[test]
fn every_parameter_is_declared_and_the_table_covers_them() {
    let wf = WorkflowParameters::default();
    let table = wf.provenance_table();
    assert_eq!(
        table.len(),
        12,
        "the provenance table has {} rows for a struct with twelve fields",
        table.len()
    );
    for (name, provenance) in &table {
        assert!(!name.is_empty());
        assert_eq!(
            provenance.label(),
            "declared",
            "{name} claims to be something other than declared"
        );
        assert!(!provenance.detail().is_empty());
    }
}

/// The distinction between the two ways of leaving the envelope is load-bearing, so the two
/// requirements must not be interchangeable: a workflow can miss its deadline while still
/// having a steady state.
#[test]
fn losing_stability_and_missing_the_deadline_are_different_findings() {
    let wf = WorkflowParameters::default();
    let params = capacity_params();
    let relaxed = evaluate(&wf, Policy::WipCap, 0.5, &params);
    assert!(relaxed.stable);
    // The cap holds utilisation, so this fixture must actually breach the deadline or the test
    // proves nothing about the distinction.
    let mut tight = wf.clone();
    tight.deadline_days = 0.01;
    let tight_envelope = evaluate(&tight, Policy::WipCap, 0.5, &params);
    assert!(tight_envelope.stable);
    assert!(!tight_envelope.meets_deadline);
    assert_eq!(first_year_outside(&[tight_envelope], Requirement::Deadline), Some(0));
    assert_eq!(first_year_outside(&[tight_envelope], Requirement::Stability), None);
}
