// human_capacity -- the integration tests
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

//! The psychological layer's contract, moved out of the module and into `tests/` so that
//! nothing outside the crate's own test suite can quietly stop checking it.
//!
//! Each test is named for the failure it would catch rather than for the function it calls,
//! because every one of them exists because that failure actually happened.

use human_capacity::*;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Autonomy must be zero if *any* factor is zero. This is the model's whole claim, so it
/// is the first thing checked: the multiplication is not decoration, and a sum would let
/// a person with total self-knowledge and no capacity register as autonomous.
#[test]
fn autonomy_is_zero_if_any_factor_is_zero() {
    assert_eq!(autonomy(0.0, 1.0, 0.0), 0.0);
    assert_eq!(autonomy(1.0, 0.0, 0.0), 0.0);
    assert_eq!(autonomy(1.0, 1.0, 1.0), 0.0);
    assert!(close(autonomy(1.0, 1.0, 0.0), 1.0));
    // And it is strictly increasing in each factor.
    assert!(autonomy(0.5, 0.5, 0.5) < autonomy(0.6, 0.5, 0.5));
    assert!(autonomy(0.5, 0.5, 0.5) < autonomy(0.5, 0.6, 0.5));
    assert!(autonomy(0.5, 0.5, 0.5) > autonomy(0.5, 0.5, 0.6));
}

/// The two channels must move in opposite directions, and the learning share at which they
/// balance must actually balance them.
///
/// This replaces a test that encoded the model's first structural error — it had
/// substitution and scaffolding fighting inside a *single* practice term, so "does
/// assistance help?" had no answer independent of capacity. The honest structure is that
/// producing and learning are different activities: assistance removes reps from one and
/// adds efficiency to the other, and the training share decides which reaches the person.
#[test]
fn assistance_removes_production_reps_and_adds_learning_efficiency() {
    let channels = Channels {
        substitution: 0.8,
        scaffolding: 1.4,
    };
    // Producing with assistance yields fewer reps, monotonically.
    assert!(channels.production_practice(0.0) > channels.production_practice(0.5));
    assert!(channels.production_practice(0.5) > channels.production_practice(1.0));
    assert!((channels.production_practice(1.0) - 0.2).abs() < 1e-12);
    // Learning with assistance yields more, monotonically.
    assert!(channels.learning_practice(1.0) > channels.learning_practice(0.0));
    assert!((channels.learning_practice(0.0) - 1.0).abs() < 1e-12);

    // The break-even share must be where the two exactly cancel at full assistance.
    let t = channels
        .break_even_learning_share()
        .expect("channels are nonzero");
    assert!(
        (t - 0.8 / (0.8 + 1.4)).abs() < 1e-12,
        "break-even share is {t}"
    );
    let blended =
        (1.0 - t) * channels.production_practice(1.0) + t * channels.learning_practice(1.0);
    assert!(
        (blended - 1.0).abs() < 1e-12,
        "at the break-even share the blend should be 1.0, got {blended}"
    );
    // Below the break-even share, assistance builds *less* capacity than the same
    // training split without it; above, more. This is the correct comparison and the
    // obvious wrong one is tempting: the break-even share does not mean build equals
    // decay, only that assisted practice equals unassisted practice. Whether the build
    // clears decay at all is a separate question, answered by `deskilling_threshold`.
    let params = ModelParameters::default();
    let low = capacity_change(1.0, 0.5, t * 0.5, &params)
        - capacity_change(0.0, 0.5, t * 0.5, &params);
    assert!(low < 0.0, "assistance built more capacity below break-even");
    let high_share = (t + 0.4).min(1.0);
    let high = capacity_change(1.0, 0.5, high_share, &params)
        - capacity_change(0.0, 0.5, high_share, &params);
    assert!(
        high > 0.0,
        "assistance built less capacity above break-even"
    );
    // And the sign must flip exactly at the break-even share, not near it.
    let at = capacity_change(1.0, 0.5, t, &params) - capacity_change(0.0, 0.5, t, &params);
    assert!(
        at.abs() < 1e-12,
        "the blend is not flat at break-even: {at}"
    );
}

/// The deskilling threshold must agree with the dynamics it was derived from.
///
/// This is the check that matters: the closed form is an inequality inversion, and an
/// inversion is exactly where a sign error hides. The test asserts that setting sigma just
/// above the threshold drives capacity down and just below it drives capacity up.
#[test]
fn the_deskilling_threshold_agrees_with_the_dynamics() {
    let params = ModelParameters::default();
    let h = 0.7;
    let t = params.training_share;
    let threshold = deskilling_threshold(h, t, &params).expect("defined below the ceiling");
    for (delta, expect_fall) in [(-0.05, false), (0.05, true)] {
        let mut probe = params.clone();
        probe.channels.substitution = threshold + delta;
        let change = capacity_change(1.0, h, t, &probe);
        assert_eq!(
            change < 0.0,
            expect_fall,
            "sigma {} : change {change} at capacity {h}",
            probe.channels.substitution
        );
    }
}

/// Expectations must rise, then stop.
///
/// The second half of this test is a regression test for a real defect: the first version of
/// the model compounded the bar for the whole horizon, reached fourteen times baseline in
/// thirty years, drove every person to full delegation, and left the cohort with one distinct
/// outcome. The unbounded exponential was the mistake, so the ceiling is asserted rather than
/// assumed, and the assertion is followed by the population check that it protects.
#[test]
fn expectations_rise_to_a_ceiling_and_then_stop() {
    let params = ModelParameters::default();
    assert_eq!(expectation_at(0, &params), 1.0, "the runway must be flat");
    assert_eq!(
        expectation_at(params.expectation_lag_years - 1, &params),
        1.0,
        "the bar moved before the runway elapsed"
    );
    let mut previous = 0.0_f64;
    for year in 0..60 {
        let value = expectation_at(year, &params);
        assert!(
            value >= previous - 1e-12,
            "expectations fell at year {year}"
        );
        assert!(
            value <= params.expectation_ceiling + 1e-12,
            "expectations passed the ceiling at year {year}: {value}"
        );
        previous = value;
    }
    // Long enough and it must actually reach the ceiling, or the ceiling is decoration.
    assert!(
        (expectation_at(200, &params) - params.expectation_ceiling).abs() < 1e-9,
        "the ceiling is never reached, so it cannot be what bounds the model"
    );
    // A machine that sets no tempo leaves the bar where it was.
    let mut still = params.clone();
    still.tempo_growth = 0.0;
    for year in [0, 5, 20, 200] {
        assert_eq!(expectation_at(year, &still), 1.0);
    }
}

/// The cohort must remain a cohort: a population model whose every member ends in the same
/// state is not describing a population.
///
/// This is the test the saturated version failed. `p10 == median == mean` at the horizon is
/// the signature of a single attractor reached by everyone, and it means any dispersion
/// statistic in the summary is measuring nothing.
#[test]
fn the_cohort_keeps_dispersion_at_the_horizon() {
    let params = ModelParameters::default();
    let cohort = draw_cohort(500, &params, 20_260_101);
    let summary = summarise(&run_cohort(&cohort, &params, Strategy::Tracking), 0.15);
    assert!(
        summary.p10_capacity < summary.median_capacity - 1e-6,
        "the cohort collapsed to one capacity: p10 {} and median {}",
        summary.p10_capacity,
        summary.median_capacity
    );
    assert!(
        summary.mean_autonomy > 0.0,
        "mean autonomy is zero, so every person is fully delegated and the autonomy \
         channel carries no information"
    );
    assert!(
        summary.share_led > 0.0 && summary.share_led < 1.0,
        "the autonomy floor classifies everyone the same way ({}), which makes it a \
         constant rather than a threshold",
        summary.share_led
    );
}

/// The runway must buy *time*, and this test pins down that it does not buy a different
/// destination.
///
/// Both halves matter. The first is the claim the document makes; the second is the model's
/// honest limitation, and writing it as an assertion means that anyone who changes the
/// dynamics so that the runway does change the endpoint will be told, loudly, that the
/// document's central sentence about the runway no longer describes the code.
#[test]
fn the_runway_buys_years_of_autonomy_rather_than_a_different_endpoint() {
    let params = ModelParameters::default();
    let cohort = draw_cohort(400, &params, 7_700_042);
    let mut none = params.clone();
    none.expectation_lag_years = 0;
    let mut six = params.clone();
    six.expectation_lag_years = 6;
    let bare = summarise(&run_cohort(&cohort, &none, Strategy::Tracking), 0.15);
    let long = summarise(&run_cohort(&cohort, &six, Strategy::Tracking), 0.15);

    assert!(
        long.mean_autonomous_years > bare.mean_autonomous_years,
        "a longer runway bought no additional years of autonomy: {} then {}",
        bare.mean_autonomous_years,
        long.mean_autonomous_years
    );
    assert!(
        long.mean_autonomous_years - bare.mean_autonomous_years > 0.5,
        "the runway's effect is too small to be the edge the argument claims: {} then {}",
        bare.mean_autonomous_years,
        long.mean_autonomous_years
    );
    assert!(
        (long.mean_capacity - bare.mean_capacity).abs() < 0.05,
        "the runway changed the endpoint, so the model is no longer one in which the bar \
         rather than the path sets the destination: {} then {}",
        bare.mean_capacity,
        long.mean_capacity
    );
}

/// The residual defect rate must have a floor and the floor must be the correlated share,
/// not zero. A model in which enough assistance bought perfection would make the argument
/// against maximal assistance rest on preference; it rests on arithmetic instead.
#[test]
fn the_residual_defect_rate_has_a_floor_that_effort_cannot_remove() {
    let params = ModelParameters::default();
    let floor = residual_defect_floor(&params);
    assert!(close(
        floor,
        params.shared_blind_spot * (1.0 - params.machine_detection)
    ));
    assert!(
        floor > 0.0,
        "the floor should be positive for a correlated tester"
    );
    // As capacity vanishes, the rate approaches the floor plus the undetected remainder of
    // the independent share — never below the floor itself.
    let at_zero = residual_defect_rate(0.0, &params);
    assert!(at_zero >= floor - 1e-12);
    // And more capacity strictly reduces it.
    assert!(residual_defect_rate(0.9, &params) < residual_defect_rate(0.1, &params));
    // If there were no blind spot, assistance could in principle reach perfection.
    let mut none = params.clone();
    none.shared_blind_spot = 0.0;
    assert_eq!(residual_defect_floor(&none), 0.0);
}

/// Emotional coherence must rise with the human share of interaction, not fall.
///
/// The first version of this test asserted the wrong direction, and the wrong direction is
/// plausible enough to be worth naming: the model's risk is that coherence *falls*, so a
/// test that "checks it moves" in either direction passes while proving nothing. Coherence
/// is `αH / (αH + β(1−H))`, which is zero when nobody is human and one when everybody is.
#[test]
fn emotional_coherence_and_its_critical_share_are_consistent() {
    let params = ModelParameters::default();
    let mut previous = f64::NEG_INFINITY;
    for step in 0..=100 {
        let share = f64::from(step) / 100.0;
        let value = emotional_coherence(share, &params);
        assert!(value >= previous - 1e-12, "coherence fell at share {share}");
        assert!((0.0..=1.0).contains(&value));
        previous = value;
    }
    assert_eq!(emotional_coherence(1.0, &params), 1.0);
    assert_eq!(emotional_coherence(0.0, &params), 0.0);

    let target = 0.6;
    let critical = critical_human_share(target, &params).expect("both coefficients nonzero");
    assert!(
        (emotional_coherence(critical, &params) - target).abs() < 1e-9,
        "coherence at the critical share is {} not {target}",
        emotional_coherence(critical, &params)
    );
    // The critical share is a real threshold: crossing it takes coherence below target,
    // and the direction is the one the argument needs.
    assert!(emotional_coherence(critical - 0.02, &params) < target);
    assert!(emotional_coherence(critical + 0.02, &params) > target);
}

/// The required assistance must rise with the expectation and fall with capacity, and it
/// must be clamped — an expectation nobody can meet cannot demand more than all of it.
#[test]
fn required_assistance_rises_with_the_bar_and_falls_with_capacity() {
    let params = ModelParameters::default();
    let a_low = required_assistance(1.0, 0.8, &params);
    let a_high = required_assistance(2.0, 0.8, &params);
    assert!(a_high > a_low, "a higher bar must demand more assistance");
    let a_capable = required_assistance(2.0, 0.9, &params);
    let a_weak = required_assistance(2.0, 0.2, &params);
    assert!(
        a_capable < a_weak,
        "a more capable person should need less: {a_capable} against {a_weak}"
    );
    // Clamped at both ends.
    assert_eq!(required_assistance(1.0, 1.0, &params), 0.0);
    assert_eq!(required_assistance(1e6, 0.0, &params), 1.0);
}

/// The equilibrium capacity must be the fixed point of the dynamics it is solved from.
/// Checked by evolving the system and asserting it stays put, which is what "equilibrium"
/// means and what a bisection on the wrong sign would not satisfy.
#[test]
fn the_equilibrium_capacity_is_a_fixed_point() {
    let params = ModelParameters::default();
    let t = params.training_share;
    for assistance in [0.0, 0.2, 0.5] {
        let Some(h) = equilibrium_capacity(assistance, t, &params) else {
            continue;
        };
        let change = capacity_change(assistance, h, t, &params);
        assert!(
            change.abs() < 1e-6,
            "assistance {assistance}: capacity {h} is not a fixed point (change {change})"
        );
        // And it is stable: a perturbation decays back toward it.
        let up = capacity_change(assistance, (h + 0.02).min(1.0), t, &params);
        let down = capacity_change(assistance, (h - 0.02).max(0.0), t, &params);
        assert!(
            up < 0.0 || h + 0.02 >= 1.0,
            "the fixed point should be stable from above"
        );
        assert!(
            down > 0.0 || h - 0.02 <= 0.0,
            "the fixed point should be stable from below"
        );
    }
}

/// Tracking must not leave capacity above the fixed-assistance counterfactual at the same
/// final assistance level: the tempo is what removes the practice, and a model in which
/// tracking preserved capacity would have no mechanism in it.
#[test]
fn tracking_the_tempo_costs_more_capacity_than_holding_assistance_still() {
    let params = ModelParameters::default();
    let tracked = simulate(0.8, 0.5, &params, Strategy::Tracking, params.human_share);
    let held = simulate(0.8, 0.5, &params, Strategy::Fixed(0.2), params.human_share);
    let tracked_end = tracked.last().expect("non-empty horizon").capacity;
    let held_end = held.last().expect("non-empty horizon").capacity;
    assert!(
        tracked_end < held_end,
        "tracking ended at capacity {tracked_end}, holding at {held_end}"
    );
    // And the trajectory's assistance must actually have risen, or the comparison is empty.
    let first = tracked.first().expect("non-empty").assistance;
    let last = tracked.last().expect("non-empty").assistance;
    assert!(
        last > first,
        "tracked assistance did not rise: {first} to {last}"
    );
}

/// The runway must matter, and in the direction claimed: a longer lag before expectations
/// rise must leave more capacity at the horizon, because it buys years in which the person
/// practises at low assistance.
#[test]
fn a_longer_runway_preserves_capacity() {
    let base = ModelParameters::default();
    let mut previous: Option<(u32, f64)> = None;
    for lag in [0u32, 1, 2, 4, 8, 12] {
        let mut params = base.clone();
        params.expectation_lag_years = lag;
        let path = simulate(0.8, 0.5, &params, Strategy::Tracking, params.human_share);
        let end = path.last().expect("non-empty").capacity;
        if let Some((earlier_lag, earlier_end)) = previous {
            assert!(
                end >= earlier_end - 1e-9,
                "capacity fell when the runway lengthened from {earlier_lag} to {lag}: \
                 {earlier_end} then {end}"
            );
        }
        previous = Some((lag, end));
    }
}

/// Whether maximum assistance is optimal is a **parameter question, not a finding** — and
/// saying so is the honest form of the claim.
///
/// The first version of this test asserted that the default parameters produce an interior
/// optimum. They do not: at `throughput_gain = 1.5` the speed is worth more than the
/// capacity costs, and full assistance wins. The temptation is to retune the defaults until
/// the "right" answer appears, which would make the model an illustration of its author's
/// conclusion rather than a test of it.
///
/// Instead the test maps the boundary. There must exist a speed-up small enough that
/// assistance is not worth its capacity cost — the regime in which the argument bites — and
/// a speed-up large enough that it is, which is the regime the argument is *not* needed for.
/// A model that could not produce both regimes would be a fixed conclusion wearing a
/// parameter.
#[test]
fn the_optimal_assistance_trades_speed_against_capacity() {
    let base = ModelParameters::default();
    let mut regimes = Vec::new();
    for gain in [0.05, 0.1, 0.2, 0.4, 0.8, 1.5, 3.0] {
        let mut params = base.clone();
        params.throughput_gain = gain;
        let mut best = (0.0_f64, f64::NEG_INFINITY);
        let mut means = Vec::new();
        for step in 0..=10 {
            let level = f64::from(step) / 10.0;
            let path = simulate(
                0.7,
                0.4,
                &params,
                Strategy::Fixed(level),
                params.human_share,
            );
            let mean: f64 =
                path.iter().map(|s| s.effective_output).sum::<f64>() / path.len() as f64;
            means.push(mean);
            if mean > best.1 {
                best = (level, mean);
            }
        }
        // Effective output must rise then fall, or rise throughout. What it must never do
        // is fall then rise: capacity costs are monotone in assistance.
        for window in means.windows(3) {
            assert!(
                !(window[1] < window[0] && window[2] > window[1]),
                "effective output fell then rose at gain {gain}: {means:?}"
            );
        }
        regimes.push((gain, best.0));
    }

    let interior = regimes.iter().filter(|(_, level)| *level < 1.0).count();
    assert!(
        interior > 0,
        "no speed-up is small enough for assistance to cost more than it earns, so the \
         model cannot express the argument's premise: {regimes:?}"
    );
    assert!(
        interior < regimes.len(),
        "no speed-up is large enough for full assistance to win, so the model has no \
         opposite regime to test the argument against: {regimes:?}"
    );

    // And the optimal assistance must be non-decreasing in the speed-up: a faster tool is
    // never worth *less* use. This is the comparative static that gives the boundary its
    // meaning.
    for pair in regimes.windows(2) {
        assert!(
            pair[1].1 >= pair[0].1 - 1e-12,
            "optimal assistance fell from {} at gain {} to {} at gain {}",
            pair[0].1,
            pair[0].0,
            pair[1].1,
            pair[1].0
        );
    }
}

/// Every parameter must have a stated origin, and the origin must say *declared* — because
/// nothing in this module is calibrated against data.
#[test]
fn every_parameter_is_declared_and_says_so() {
    let params = ModelParameters::default();
    let table = params.provenance_table();
    assert!(table.len() >= 13, "the table has only {} rows", table.len());
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

/// The cohort summary must be the summary of its own trajectories, and the seed must make
/// two policies comparable on the same people.
#[test]
fn the_cohort_summary_describes_the_trajectories_it_was_given() {
    let params = ModelParameters::default();
    let cohort = draw_cohort(200, &params, 12345);
    assert_eq!(cohort.len(), 200);
    let again = draw_cohort(200, &params, 12345);
    for (a, b) in cohort.iter().zip(again.iter()) {
        assert_eq!(a.capacity_0, b.capacity_0);
        assert_eq!(a.self_model_0, b.self_model_0);
    }
    let paths = run_cohort(&cohort, &params, Strategy::Tracking);
    let summary = summarise(&paths, 0.25);
    assert_eq!(summary.persons, 200);
    assert!((0.0..=1.0).contains(&summary.mean_capacity));
    assert!((0.0..=1.0).contains(&summary.mean_autonomy));
    assert!((0.0..=1.0).contains(&summary.share_led));
    assert!(summary.p10_capacity <= summary.median_capacity);
    assert!(summary.peak_effective_output >= summary.mean_effective_output);
}