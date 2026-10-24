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

//! The properties the report's claims rest on.
//!
//! Two of these are load-bearing and the rest are hygiene. The load-bearing ones are that the
//! behavioural design genuinely cannot see believed competence at `gamma = 0`, and that the filter
//! beats both the subtraction baseline and its own process model — because if either failed, the
//! report's conclusions would be backwards.

use perception_gap::analysis::{evaluate, evaluate_mismatched, gap_profile};
use perception_gap::{observability, simulate, static_fit, subtract, Design, Kalman, Mat2, Params};

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// The behavioural design cannot identify believed competence unless confidence contaminates
/// behaviour.
///
/// This is the report's central claim. At `gamma = 0` both behavioural channels load on true
/// competence and nothing loads on belief, so the observability matrix is rank 1 and no estimator
/// recovers the gap. As `gamma` rises the design becomes nominally identifiable — precisely because
/// the confound leaks belief into the behavioural signal, which is the quantity the study was trying
/// to avoid assuming.
#[test]
fn the_behavioural_design_cannot_see_belief_without_the_confound() {
    let params = Params::default();

    let unconfounded = observability(&params, Design::BehaviouralOnly, 1.0);
    assert_eq!(unconfounded.rank, 1, "with gamma = 0 the behavioural design must be rank 1");
    assert!(
        !unconfounded.condition.is_finite(),
        "a rank-1 design must have an infinite condition number, got {}",
        unconfounded.condition
    );
    assert!(
        unconfounded.null_direction.is_some(),
        "a rank-1 design must report its null direction"
    );

    for gamma in [0.1, 0.3, 0.5] {
        let mut local = params.clone();
        local.gamma = gamma;
        let identified = observability(&local, Design::BehaviouralOnly, 1.0);
        assert_eq!(
            identified.rank, 2,
            "gamma {gamma}: the confound makes the design nominally identifiable"
        );
        assert!(
            identified.condition > 5.0,
            "gamma {gamma}: a confounded design should be poorly conditioned, got {}",
            identified.condition
        );
    }

    // And the design with self-report is well conditioned at every level of the confound.
    for gamma in [0.0, 0.25, 0.5] {
        let mut local = params.clone();
        local.gamma = gamma;
        let full = observability(&local, Design::Full, 1.0);
        assert_eq!(full.rank, 2, "gamma {gamma}: the full design must be identifiable");
        assert!(
            full.condition < 3.0,
            "gamma {gamma}: the full design should be well conditioned, got {}",
            full.condition
        );
    }
}

/// The gap starts slightly negative and ends positive.
///
/// Declared initial values put belief below truth and the assistance effects invert the sign, so the
/// model describes a person who begins with modest underconfidence and ends overconfident. That is a
/// property of the parameters, not a finding, but it is what the figures show and it should be true
/// on purpose rather than by accident.
#[test]
fn the_gap_flips_sign_over_the_horizon() {
    let params = Params::default();
    let profile = gap_profile(&params, 1.0, 20);

    assert!(
        profile.gap[0] < 0.0,
        "the gap should start negative with these declared values, got {}",
        profile.gap[0]
    );
    let last = profile.gap.last().copied().unwrap_or(f64::NAN);
    assert!(last > 0.0, "the gap should end positive, got {last}");
    assert_eq!(
        profile.peak_month, params.horizon,
        "the gap grows monotonically, so its peak is the horizon"
    );
    assert!(
        profile.fastest_month <= 6,
        "at full adoption the gap should open within the first months, got month {}",
        profile.fastest_month
    );
}

/// The gap approaches the same steady state at every adoption share, but at very different speeds.
#[test]
fn adoption_changes_the_speed_not_the_destination() {
    let params = Params::default();
    let (steady_quarter, half_quarter) = perception_gap::steady_state(&params, 0.25);
    let (steady_full, half_full) = perception_gap::steady_state(&params, 1.0);

    assert!(
        close(steady_quarter, steady_full, 1e-9),
        "the steady-state gap should not depend on adoption: {steady_quarter} against {steady_full}"
    );
    assert!(
        half_full < half_quarter / 2.0,
        "full adoption should reach half the gap in under half the time: {half_full:.1} against \
         {half_quarter:.1}"
    );
}

/// The filter must beat the baseline it has to beat, and it must beat its own model.
///
/// Two comparisons, both necessary. Beating subtraction shows the dynamics are worth having;
/// beating the prior-only run shows the estimate is drawn from the observations rather than recited
/// from the transition. In a simulation the second is the one that is easy to fake, because the
/// truth is generated by the model the filter assumes.
#[test]
fn the_filter_beats_subtraction_and_its_own_process_model() {
    let params = Params::default();
    let cell = evaluate(&params, 0.5, 0.0, Design::Full, 60);

    assert_eq!(cell.rank, 2);
    assert!(
        cell.gain_over_subtraction() < 0.8,
        "the filter must beat subtraction by 20%, got ratio {:.2}",
        cell.gain_over_subtraction()
    );
    assert!(
        cell.gain_over_prior() < 0.8,
        "the filter must beat its own model with no data, got ratio {:.2}",
        cell.gain_over_prior()
    );
    assert!(
        cell.kalman_rmse < cell.static_rmse,
        "the dynamics must beat a static fit: {:.3} against {:.3}",
        cell.kalman_rmse,
        cell.static_rmse
    );
}

/// Calibration on the full design is near nominal.
#[test]
fn the_full_design_is_calibrated() {
    let params = Params::default();
    let cell = evaluate(&params, 0.5, 0.0, Design::Full, 80);
    assert!(
        (0.60..=0.76).contains(&cell.kalman_coverage),
        "a nominal 68% band should cover 60-76% of months, got {:.2}",
        cell.kalman_coverage
    );
}

/// Unidentifiability shows up as miscalibration rather than as error.
///
/// The behavioural design's RMSE stays respectable while its band drifts away from the truth. That
/// is why the report does not rank unidentifiable cells on error alone.
#[test]
fn the_unidentifiable_design_drifts_out_of_calibration() {
    let params = Params::default();
    let matched = evaluate_mismatched(&params, 0.5, 0.0, Design::BehaviouralOnly, 80, 0.0);
    let mismatched = evaluate_mismatched(&params, 0.5, 0.0, Design::BehaviouralOnly, 80, 1.0);

    assert_eq!(matched.rank, 1, "this design is unidentifiable at gamma = 0");
    assert!(
        mismatched.kalman_coverage < matched.kalman_coverage,
        "mismatch should degrade calibration: {:.2} against {:.2}",
        matched.kalman_coverage,
        mismatched.kalman_coverage
    );
    // And the full design holds its calibration where the behavioural one does not.
    let full = evaluate_mismatched(&params, 0.5, 0.0, Design::Full, 80, 1.0);
    assert!(
        full.kalman_coverage > mismatched.kalman_coverage,
        "the identifiable design should stay calibrated where the other drifts: {:.2} against {:.2}",
        full.kalman_coverage,
        mismatched.kalman_coverage
    );
}

/// A rank-deficient design returns no estimate rather than a number.
#[test]
fn the_static_fit_refuses_a_rank_deficient_design() {
    let params = Params::default();
    let trace = simulate(&params, 1.0, 7);
    let estimate = static_fit(&trace, &params, Design::BehaviouralOnly);
    assert!(
        estimate.gap.iter().all(|value| !value.is_finite()),
        "a rank-deficient design must not produce finite estimates"
    );
}

/// The subtraction baseline is the difference of the two direct channels, with the sum of their
/// variances.
#[test]
fn the_subtraction_baseline_is_what_it_claims() {
    let params = Params::default();
    let trace = simulate(&params, 0.5, 3);
    let estimate = subtract(&trace, &params);

    for (index, z) in trace.observations.iter().take(5).enumerate() {
        let expected = z[perception_gap::REPORTED_CONFIDENCE] - z[perception_gap::MEASURED_SAVING];
        assert!(close(estimate.gap[index], expected, 1e-12));
    }
    let expected_sigma = (params.sigma_obs[perception_gap::REPORTED_CONFIDENCE].powi(2)
        + params.sigma_obs[perception_gap::MEASURED_SAVING].powi(2))
    .sqrt();
    assert!(close(estimate.sigma[0], expected_sigma, 1e-12));
}

/// The latents stay in the unit interval under every seed and adoption share.
#[test]
fn the_latents_stay_probabilities() {
    let params = Params::default();
    for seed in 0..12 {
        for adoption in [0.25, 0.75, 1.0] {
            let trace = simulate(&params, adoption, seed);
            for value in trace.tau.iter().chain(trace.beta.iter()) {
                assert!(
                    (0.0..=1.0).contains(value),
                    "seed {seed} adoption {adoption}: latent {value} left [0, 1]"
                );
            }
        }
    }
}

/// The 2×2 arithmetic the filter depends on.
#[test]
fn the_matrix_helpers_are_consistent() {
    let m = Mat2([[4.0, 1.0], [2.0, 3.0]]);
    let inverse = m.inverse().expect("invertible");
    let product = m.mul(&inverse);
    assert!(close(product.0[0][0], 1.0, 1e-12));
    assert!(close(product.0[1][1], 1.0, 1e-12));
    assert!(close(product.0[0][1], 0.0, 1e-12));
    assert!(close(m.det(), 10.0, 1e-12));

    let symmetric = Mat2([[2.0, 1.0], [1.0, 2.0]]);
    let (largest, smallest) = symmetric.eigen();
    assert!(close(largest, 3.0, 1e-12));
    assert!(close(smallest, 1.0, 1e-12));

    // A singular matrix has no inverse, and the filter must not pretend otherwise.
    assert!(Mat2([[1.0, 2.0], [2.0, 4.0]]).inverse().is_none());
}

/// A filter with a prior already at the truth and no observations does not move.
#[test]
fn the_prior_only_run_follows_the_transition() {
    let params = Params::default();
    let mut filter = Kalman::new(&params, Design::Full, 1.0);
    let estimate = filter.run_prior_only(36);
    assert_eq!(estimate.gap.len(), 36);
    assert!(
        estimate.gap.last().copied().unwrap_or(0.0) > estimate.gap[0],
        "with full adoption the gap should widen under the transition alone"
    );
    assert!(estimate.sigma.iter().all(|s| s.is_finite() && *s >= 0.0));
}

/// The referee's objection, tested rather than answered.
///
/// The objection: a standard Kalman filter's result is a property of the linearisation, so an
/// extended, unscented or particle filter might give a different answer. In this model every map is
/// affine — the transition is `F x + u` with a known input and the observation is `H x` — so all four
/// filters are the same estimator, and a particle filter must converge to the Kalman filter as the
/// particle count rises. If it does not converge, the model contains a nonlinearity the
/// specification did not declare, and the Kalman results in the report are unsafe.
///
/// The test therefore checks *convergence*, not equality. A particle filter carries Monte Carlo error
/// on top of the estimation error, so it is expected to be worse at finite particle counts; what
/// matters is that the discrepancy shrinks.
#[test]
fn the_particle_filter_converges_to_the_kalman_filter() {
    let params = Params::default();
    let design = Design::Full;
    let seeds = 12u64;

    let mean_rmse = |particles: usize| {
        let mut kalman_total = 0.0;
        let mut particle_total = 0.0;
        for seed in 0..seeds {
            let trace = simulate(&params, 0.5, seed);
            let truth = trace.gap();

            let mut filter = Kalman::new(&params, design, 0.5);
            let kalman = filter.run(&trace);
            let particle =
                perception_gap::particle_filter(&trace, &params, design, 0.5, particles, seed);

            kalman_total += kalman.rmse(&truth);
            particle_total += particle.rmse(&truth);
        }
        (kalman_total / seeds as f64, particle_total / seeds as f64)
    };

    let (kalman_coarse, particle_coarse) = mean_rmse(200);
    let (kalman_fine, particle_fine) = mean_rmse(4000);

    let coarse_gap = (particle_coarse - kalman_coarse).abs();
    let fine_gap = (particle_fine - kalman_fine).abs();

    assert!(
        fine_gap < coarse_gap * 0.75,
        "the particle filter should converge toward the Kalman filter, but the discrepancy went \
         from {coarse_gap:.4} at 200 particles to {fine_gap:.4} at 4000"
    );
    assert!(
        fine_gap / kalman_fine < 0.05,
        "at 4000 particles the two filters should agree within 5%: {particle_fine:.4} against \
         {kalman_fine:.4}"
    );
}
