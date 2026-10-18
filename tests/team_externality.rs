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

//! The team externality's contract.
//!
//! The regression these tests exist for is specific. Before this module, a reduced schedule was
//! modelled as producing proportionally less output with no effect on colleagues, while the README
//! said so in a footnote. That made the tool prefer a schedule that pushes work onto other people
//! over one that genuinely produces less — a bias toward externalising cost. So the first test is
//! not about arithmetic at all: it is that the absence of information is reported as an absence,
//! never as "no effect".

use life_optimizer::optimizer::{LifeOptimizer, OptimizerConfig};
use life_optimizer::requirements::{LifeStage, PersonalRequirements, PreferenceWeights};
use life_optimizer::tax::TaxSchedule;
use life_optimizer::team::{
    Severity, TeamParameters, MATERIAL_HOURS, SEVERE_HOURS,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// **The regression.** A reduction with no team parameters must be reported as *not modelled*, and
/// must require a warning. "No effect" and "not modelled" are different claims and the tool may not
/// conflate them — that conflation was the bias.
#[test]
fn a_reduction_without_team_parameters_is_reported_as_unmodelled() {
    let externality = TeamParameters::not_modelled().evaluate(0.8, 42.0);
    assert_eq!(externality.severity, Severity::NotModelled);
    assert!(
        externality.requires_warning(),
        "an unmodelled externality must produce a warning"
    );
    assert!(
        externality.summary().contains("NOT MODELLED"),
        "the summary must say the effect is unmodelled, got {:?}",
        externality.summary()
    );
    // And it must not attribute hours to a team that was never named, while still showing
    // the size of the exposure so the warning can be acted on.
    assert!(close(externality.hours_moved_per_week, 0.0));
    assert!(
        close(externality.potential_hours_per_week, 8.4),
        "the potential exposure should still be shown: {}",
        externality.potential_hours_per_week
    );
    assert!(externality.summary().contains("8.4"));
}

/// A full-time schedule moves nothing onto anyone, and says so without a warning.
#[test]
fn a_full_time_schedule_has_no_externality() {
    for parameters in [
        TeamParameters::not_modelled(),
        TeamParameters::with_team(6, 1.0),
    ] {
        let externality = parameters.evaluate(1.0, 42.0);
        assert_eq!(externality.severity, Severity::NoReduction);
        assert!(!externality.requires_warning());
        assert!(close(externality.hours_moved_per_week, 0.0));
    }
}

/// The hours moved are the reduction times the contract times the redistributed share, and they
/// divide by the number of colleagues. Checked against arithmetic done by hand.
#[test]
fn the_hours_moved_are_the_arithmetic_they_claim_to_be() {
    // A 20% reduction on 42 hours with all of it redistributed to 3 colleagues:
    // 0.2 * 42 * 1.0 = 8.4 hours, and 8.4 / 3 = 2.8 hours each.
    let externality = TeamParameters::with_team(3, 1.0).evaluate(0.8, 42.0);
    assert!(
        close(externality.hours_moved_per_week, 8.4),
        "moved {} hours, not 8.4",
        externality.hours_moved_per_week
    );
    assert!(
        close(externality.hours_per_colleague_per_week, 2.8),
        "per colleague {}, not 2.8",
        externality.hours_per_colleague_per_week
    );
    assert!(close(externality.redistribution, 1.0));

    // Half of it absorbed: half the hours.
    let half = TeamParameters::with_team(3, 0.5).evaluate(0.8, 42.0);
    assert!(close(half.hours_moved_per_week, 4.2));

    // None absorbed: the schedule genuinely produces less, and nothing moves.
    let none = TeamParameters::with_team(3, 0.0).evaluate(0.8, 42.0);
    assert!(close(none.hours_moved_per_week, 0.0));
    assert_eq!(none.severity, Severity::Negligible);
    assert!(!none.requires_warning());
}

/// The severity boundaries must bracket the declared thresholds rather than sit near them.
#[test]
fn the_severity_thresholds_are_the_declared_hours() {
    // 2 colleagues, 42 hours, 20% reduction -> 8.4 hours moved -> 4.2 each: severe.
    let three = TeamParameters::with_team(2, 1.0).evaluate(0.8, 42.0);
    assert!(three.hours_per_colleague_per_week >= SEVERE_HOURS);
    assert_eq!(three.severity, Severity::Severe);

    // Spread over many hands it becomes negligible.
    let wide = TeamParameters::with_team(40, 1.0).evaluate(0.8, 42.0);
    assert!(wide.hours_per_colleague_per_week < MATERIAL_HOURS);
    assert_eq!(wide.severity, Severity::Negligible);

    // And material sits between them.
    let few = TeamParameters::with_team(6, 1.0).evaluate(0.8, 42.0);
    assert!(few.hours_per_colleague_per_week >= MATERIAL_HOURS);
    assert!(few.hours_per_colleague_per_week < SEVERE_HOURS);
    assert_eq!(few.severity, Severity::Material);
    assert!(few.requires_warning());
}

/// The penalty is zero unless the user supplies a weight, because what a colleague-hour is worth is
/// a moral position and the tool does not get to hold one on the user's behalf.
#[test]
fn the_penalty_is_zero_unless_the_user_prices_it() {
    let unpriced = TeamParameters::with_team(3, 1.0).evaluate(0.8, 42.0);
    assert!(close(unpriced.weighed_penalty, 0.0));
    assert!(close(unpriced.externality_weight, 0.0));
    // But the hours are still reported, so the user can price them if they wish.
    assert!(unpriced.hours_moved_per_week > 0.0);

    let priced = TeamParameters::with_team(3, 1.0)
        .weighed(0.5)
        .evaluate(0.8, 42.0);
    assert!(close(priced.weighed_penalty, 0.5 * 8.4));
    assert!(close(priced.externality_weight, 0.5));
}

/// Every parameter must be declared, and the table must cover the struct.
#[test]
fn every_parameter_is_declared() {
    let table = TeamParameters::not_modelled().provenance_table();
    assert_eq!(table.len(), 3);
    for (name, rationale) in &table {
        assert!(!name.is_empty());
        assert!(
            rationale.starts_with("declared"),
            "the rationale for {name} does not say it is declared: {rationale}"
        );
        assert!(rationale.len() > 40, "the rationale for {name} is not a rationale");
    }
}

/// The scenario must actually carry the externality, or the warning in the presentation layer
/// would be reading a value nothing ever set.
#[test]
fn the_built_scenario_carries_the_externality() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let requirements = PersonalRequirements::bern_family_default(0);
    let life_stage = LifeStage::YoungSingle { age: 45 };

    let mut config = OptimizerConfig::new(
        140_000.0,
        schedule,
        requirements,
        life_stage,
        PreferenceWeights::balanced(),
    );
    config.team = TeamParameters::with_team(4, 1.0);

    let optimizer = LifeOptimizer::new(config);
    let reduced = optimizer.evaluate_scenario(0.8);
    let full = optimizer.evaluate_scenario(1.0);

    // 20% of 42 hours over 4 colleagues is 2.1 hours each: material, and warned about.
    assert!(close(reduced.team_externality.hours_moved_per_week, 8.4));
    assert!(close(reduced.team_externality.hours_per_colleague_per_week, 2.1));
    assert_eq!(reduced.team_externality.severity, Severity::Material);
    assert!(reduced.team_externality.requires_warning());

    // The full-time scenario is clean, which is what makes the reduction's warning mean something.
    assert_eq!(full.team_externality.severity, Severity::NoReduction);
    assert!(!full.team_externality.requires_warning());
}

/// A default configuration must be unmodelled, not silently zero-effect.
#[test]
fn the_default_configuration_is_unmodelled_rather_than_silent() {
    let schedule = TaxSchedule::bern_city_default(false, 0);
    let requirements = PersonalRequirements::bern_family_default(0);
    let config = OptimizerConfig::new(
        120_000.0,
        schedule,
        requirements,
        LifeStage::YoungSingle { age: 40 },
        PreferenceWeights::balanced(),
    );
    assert!(!config.team.is_modelled());
    let optimizer = LifeOptimizer::new(config);
    let scenario = optimizer.evaluate_scenario(0.7);
    assert_eq!(scenario.team_externality.severity, Severity::NotModelled);
    assert!(scenario.team_externality.requires_warning());
}
