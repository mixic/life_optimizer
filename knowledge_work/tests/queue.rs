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

//! The queueing building blocks, checked against the exact results they approximate.
//!
//! The approximation is the load-bearing object in the whole framework, so it is not enough
//! that it produces plausible curves. It has to reproduce the exact answers where exact answers
//! exist, and its error elsewhere has to be *measured* rather than asserted.

use knowledge_work::queue::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

/// At one server the VUT exponent must be exactly one, or the approximation is not a
/// generalisation of Kingman but a different formula wearing its name.
#[test]
fn the_utilisation_term_reduces_to_kingman_at_one_server() {
    for rho in [0.1, 0.3, 0.5, 0.7, 0.9, 0.99] {
        let vut = utilisation_term(rho, 1);
        let kingman = rho / (1.0 - rho);
        assert!(close(vut, kingman), "at rho {rho} the term is {vut}, Kingman gives {kingman}");
    }
}

/// The exponent must exceed one for pooled servers and increase with the pool size, which is
/// the property that makes consolidation buy resilience as well as efficiency.
#[test]
fn pooling_slows_the_divergence_right_up_to_saturation() {
    let mut previous = 1.0_f64;
    for servers in [1, 2, 3, 4, 8, 16] {
        let exponent = (2.0 * (servers as f64 + 1.0)).sqrt() - 1.0;
        assert!(
            exponent > previous - 1e-12,
            "the exponent fell at {servers} servers: {exponent} after {previous}"
        );
        previous = exponent;
    }
    // And the consequence: at the same utilisation a bigger pool waits less.
    for rho in [0.7, 0.85, 0.95] {
        assert!(
            utilisation_term(rho, 8) < utilisation_term(rho, 1),
            "pooling did not reduce the utilisation term at rho {rho}"
        );
    }
}

/// With exponential arrivals and exponential service the VUT must reproduce the M/M/1 result
/// exactly. `V = (1 + 1)/2 = 1`, so any discrepancy is a bug in the term rather than in the
/// approximation's accuracy.
#[test]
fn the_approximation_reproduces_mm1_exactly() {
    let service = 0.125_f64;
    for rho in [0.2, 0.5, 0.8, 0.9, 0.95] {
        let vut = queue_delay(rho, 1, 1.0, 1.0, service);
        let exact = mm1_queue_delay(rho, service);
        assert!(
            close(vut, exact),
            "at rho {rho} the approximation gives {vut}, M/M/1 gives {exact}"
        );
    }
}

/// Against the exact M/M/c result the approximation must stay close **where waiting is
/// material**, and how close is a fact about the method that the document has to report rather
/// than assume.
///
/// The restriction is not convenience. A first version of this test compared every point in the
/// grid and reported a worst-case error of 6575% at `ρ = 0.30` with sixteen servers — where the
/// exact waiting time is about `10⁻⁷` service times and the approximation gives `3 × 10⁻⁴`. Both
/// say "there is no queue"; the relative error is enormous because the denominator is nothing.
/// The next test pins that corner down deliberately so the limitation is a documented property
/// of the approximation rather than a hole in the evidence.
///
/// The bound is set from the measured behaviour, not from hope: the approximation overstates
/// waiting by up to about a third at moderate utilisation and is within a couple of per cent
/// near saturation. What matters for the argument is the *direction* of the error, which is
/// asserted separately —
/// it errs toward a worse queue, so the envelope the framework derives is not inflated by it.
#[test]
fn the_approximation_tracks_erlang_c_where_waiting_is_material() {
    let service = 0.125_f64;
    let material = 0.05 * service;
    let mut worst_over: f64 = 0.0;
    let mut worst_under: f64 = 0.0;
    let mut over_at = (0.0_f64, 0_usize);
    let mut under_at = (0.0_f64, 0_usize);
    let mut compared = 0_usize;
    for servers in [1_usize, 2, 3, 4, 8, 16] {
        for rho in [0.3, 0.5, 0.7, 0.8, 0.9, 0.95, 0.98] {
            let exact = mmc_queue_delay(rho, servers, service);
            if !exact.is_finite() || exact < material {
                continue;
            }
            compared += 1;
            let approx = queue_delay(rho, servers, 1.0, 1.0, service);
            let ratio = approx / exact;
            if ratio - 1.0 > worst_over {
                worst_over = ratio - 1.0;
                over_at = (rho, servers);
            }
            if 1.0 - ratio > worst_under {
                worst_under = 1.0 - ratio;
                under_at = (rho, servers);
            }
        }
    }
    println!(
        "material region ({compared} points): overstates by up to {:.1}% at rho {:.2}, {} \
         servers; understates by up to {:.1}% at rho {:.2}, {} servers",
        worst_over * 100.0,
        over_at.0,
        over_at.1,
        worst_under * 100.0,
        under_at.0,
        under_at.1
    );
    assert!(compared >= 10, "the grid produced only {compared} material points");
    assert!(
        worst_over < 0.40,
        "the approximation overstates waiting by {:.1}% at rho {} with {} servers",
        worst_over * 100.0,
        over_at.0,
        over_at.1
    );
    // The direction that would threaten the argument: if the approximation understated the
    // queue, every threshold derived from it would be too permissive. It does not.
    assert!(
        worst_under < 0.05,
        "the approximation understates waiting by {:.1}% at rho {} with {} servers, which would \
         make the derived envelope too permissive",
        worst_under * 100.0,
        under_at.0,
        under_at.1
    );
}

/// The light-traffic corner, where the approximation is worst and where it does not matter.
///
/// At low utilisation with a large pool the exact waiting time is vanishingly small and the
/// power law overstates it by more than an order of magnitude. Documenting it as a test is the
/// difference between "the method has a known limitation" and "the method happened to be
/// checked on a favourable grid".
#[test]
fn the_approximation_is_unreliable_where_there_is_no_queue_to_speak_of() {
    let service = 0.125_f64;
    let (rho, servers) = (0.30, 16_usize);
    let exact = mmc_queue_delay(rho, servers, service);
    let approx = queue_delay(rho, servers, 1.0, 1.0, service);
    let ratio = approx / exact;
    println!(
        "light-traffic corner rho {rho}, {servers} servers: {:.5} service times against an \
         exact {:.7}, a factor of {ratio:.0}",
        approx / service,
        exact / service
    );
    assert!(ratio > 20.0, "the expected light-traffic failure did not appear: a factor of {ratio}");
    // And the failure is irrelevant in the units the argument uses: both answers are a fraction
    // of one service time, so neither is a queue.
    assert!(
        approx < 0.01 * service,
        "the approximation produced {approx}, which is a material wait after all"
    );
}

/// Waiting must grow without bound as utilisation reaches one, and the function must say so
/// rather than return a large number that a caller could mistake for slowness.
#[test]
fn waiting_diverges_at_saturation_and_says_so() {
    let service = 0.125_f64;
    let mut previous = 0.0;
    for rho in [0.5, 0.7, 0.9, 0.95, 0.99] {
        let delay = queue_delay(rho, 4, 1.0, 1.5, service);
        assert!(delay > previous, "waiting fell as utilisation rose at {rho}");
        previous = delay;
    }
    assert!(queue_delay(1.0, 4, 1.0, 1.5, service).is_infinite());
    assert!(queue_delay(1.2, 4, 1.0, 1.5, service).is_infinite());
    // The convexity is the point: the last five points of utilisation cost more than the
    // first five-tenths.
    let cheap = queue_delay(0.5, 4, 1.0, 1.5, service);
    let expensive = queue_delay(0.95, 4, 1.0, 1.5, service);
    assert!(
        expensive > 4.0 * cheap,
        "the utilisation term is not convex enough to carry the argument: {cheap} then {expensive}"
    );
}

/// Little's law must round trip, because the WIP cap is solved by using both directions.
#[test]
fn little_law_round_trips() {
    let arrivals = 3.5_f64;
    let cycle = 4.0_f64;
    let items = wip(arrivals, cycle);
    assert!(close(items, 14.0));
    let back = arrival_rate_for(items, cycle).expect("cycle time is positive");
    assert!(close(back, arrivals));
    assert!(arrival_rate_for(items, 0.0).is_none());
    assert!(arrival_rate_for(items, -1.0).is_none());
}

/// The inversion must find a utilisation that actually meets the target, and must refuse when
/// the target is unreachable instead of returning a number outside the unit interval.
#[test]
fn the_inversion_finds_the_utilisation_that_meets_a_wait_target() {
    let service = 0.125_f64;
    let target = 0.5_f64;
    let rho = utilisation_for_delay(target, 4, 1.0, 1.5, service)
        .expect("a half-day wait is reachable at some utilisation");
    assert!((0.0..1.0).contains(&rho), "rho {rho} is not a utilisation");
    let achieved = queue_delay(rho, 4, 1.0, 1.5, service);
    assert!(
        (achieved - target).abs() < 1e-6,
        "the inversion returned {rho}, which waits {achieved} not {target}"
    );
    // A target nothing can meet is refused, not clamped.
    assert!(utilisation_for_delay(-1.0, 4, 1.0, 1.5, service).is_none());
}
