//! Integration tests for the employer-side achievement-capacity constraint.
//!
//! Covers `CRITICS_CURRENT_WORK.md` §1.3 and `MATHEMATICS.md` §14.1:
//!
//! ```text
//! A_t = H_t * P_t * (1 + alpha_t)   subject to   A_t >= G_t
//! ```
//!
//! The reframed question from §3a is not "what work percentage maximizes my
//! utility" but "what is the lowest work percentage at which I can still
//! reliably deliver what is expected of me".

use life_optimizer::consumption::{ConsumptionProfile, ConsumptionProfileConfig};
use life_optimizer::monte_carlo::ConversionRateScenario;
use life_optimizer::optimizer::{
    AchievementConstraint, InfeasibilityReason, LifeOptimizer, OptimizerConfig,
};
use life_optimizer::requirements::{LifeStage, PersonalRequirements, PreferenceWeights};
use life_optimizer::tax::TaxSchedule;

const CANDIDATES: [f64; 6] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

fn optimizer_with(constraint: Option<AchievementConstraint>, salary: f64) -> LifeOptimizer {
    let mut config = OptimizerConfig::new(
        salary,
        TaxSchedule::bern_city_default(false, 0),
        PersonalRequirements::bern_family_default(0),
        LifeStage::YoungSingle { age: 40 },
        PreferenceWeights::balanced(),
    );
    config.retirement_age = 65;
    config.consumption =
        ConsumptionProfileConfig::new(ConsumptionProfile::Normal).with_utilization_discipline(1.0);
    config.achievement = constraint;
    config.conversion_scenario = ConversionRateScenario::Statutory;
    LifeOptimizer::new(config)
}

/// Capacity is linear in work percentage and scaled by the AI gain.
#[test]
fn capacity_is_linear_in_work_percentage() {
    let c = AchievementConstraint::new(0.8, 0.0);
    assert!((c.capacity_at(1.0) - 1.0).abs() < 1e-9);
    assert!((c.capacity_at(0.8) - 0.8).abs() < 1e-9);
    assert!((c.capacity_at(0.5) - 0.5).abs() < 1e-9);
    assert!((c.capacity_at(0.0) - 0.0).abs() < 1e-9);
}

/// The AI gain multiplies capacity: `(1 + alpha)`.
#[test]
fn ai_gain_scales_capacity() {
    let no_ai = AchievementConstraint::new(0.8, 0.0);
    let with_ai = AchievementConstraint::new(0.8, 0.25);

    assert!((with_ai.capacity_at(0.8) - 0.8 * 1.25).abs() < 1e-9);
    assert!(with_ai.capacity_at(0.8) > no_ai.capacity_at(0.8));
    // A 25% gain lets 80% work match what 100% delivered before.
    assert!((with_ai.capacity_at(0.8) - 1.0).abs() < 1e-9);
}

/// The §1.2 question: what AI gain is needed to justify reducing to 80%?
#[test]
fn required_ai_gain_answers_the_80_percent_question() {
    // A worker must deliver the output that full-time work used to produce.
    let c = AchievementConstraint::new(1.0, 0.0);
    let needed = c
        .required_ai_gain_for(0.8)
        .expect("80% is achievable with enough AI gain");
    assert!(
        (needed - 0.25).abs() < 1e-9,
        "80% work needs a +25% productivity gain, got {needed}"
    );

    // If the goal is already modest, no AI help is required at all.
    let modest = AchievementConstraint::new(0.5, 0.0);
    assert_eq!(modest.required_ai_gain_for(0.8), Some(0.0));
}

/// The §3a answer: the lowest work percentage that still delivers the goals.
#[test]
fn minimum_viable_work_percentage_inverts_capacity() {
    let c = AchievementConstraint::new(0.6, 0.0);
    let min = c
        .minimum_viable_work_percentage()
        .expect("60% of capacity is reachable");
    assert!((min - 0.6).abs() < 1e-9);

    // With AI, less scheduled time is needed for the same output.
    let with_ai = AchievementConstraint::new(0.6, 0.5);
    let min_ai = with_ai.minimum_viable_work_percentage().expect("reachable");
    assert!(
        min_ai < min,
        "AI gain should lower the minimum viable work percentage"
    );
    assert!((min_ai - 0.4).abs() < 1e-9, "0.6 / 1.5 = 0.4");
}

/// If the assigned goals exceed even full-time capacity, that is a genuine
/// finding — the constraint must report it rather than returning a number.
#[test]
fn unreachable_goals_report_none() {
    let impossible = AchievementConstraint::new(1.5, 0.0);
    assert_eq!(
        impossible.minimum_viable_work_percentage(),
        None,
        "goals needing 150% of capacity are infeasible as stated"
    );

    // Enough AI productivity can still rescue them.
    let rescued = AchievementConstraint::new(1.5, 0.6);
    assert!(rescued.minimum_viable_work_percentage().is_some());
}

/// The constraint must actually eliminate low work percentages from the search.
#[test]
fn constraint_eliminates_infeasible_work_percentages() {
    // Financially comfortable, but the job requires full output.
    let unconstrained = optimizer_with(None, 150_000.0);
    let constrained = optimizer_with(Some(AchievementConstraint::new(1.0, 0.0)), 150_000.0);

    let baseline = unconstrained.search_outcome(&CANDIDATES).unwrap();
    let bounded = constrained.search_outcome(&CANDIDATES).unwrap();

    // Without the constraint, reduced hours are chosen.
    assert!(
        baseline.scenario.work_percentage < 1.0,
        "baseline should reduce hours"
    );
    // With it, only 100% can deliver the required output.
    assert!(
        (bounded.scenario.work_percentage - 1.0).abs() < 1e-9,
        "constraint should force full-time work, got {}",
        bounded.scenario.work_percentage
    );
    assert!(bounded.scenario.achievement_satisfied());
}

/// AI productivity gain must relax the constraint and let reduced hours back in
/// — the positive case the critique asks the model to be able to represent.
#[test]
fn ai_gain_allows_reduced_hours_at_the_same_goal() {
    let goal = 1.0;
    let without_ai = optimizer_with(Some(AchievementConstraint::new(goal, 0.0)), 150_000.0);
    let with_ai = optimizer_with(Some(AchievementConstraint::new(goal, 0.40)), 150_000.0);

    let a = without_ai.search_outcome(&CANDIDATES).unwrap();
    let b = with_ai.search_outcome(&CANDIDATES).unwrap();

    assert!((a.scenario.work_percentage - 1.0).abs() < 1e-9);
    assert!(
        b.scenario.work_percentage < 1.0,
        "a +40% gain should permit reduced hours, got {}",
        b.scenario.work_percentage
    );
    assert!(b.scenario.achievement_satisfied());
}

/// When the constraint cannot be met at any candidate, the search must report
/// that specifically — a workload problem, not an affordability problem.
#[test]
fn unachievable_constraint_reports_workload_reason() {
    // Three children is expensive, so use a high salary to isolate the
    // achievement constraint from affordability.
    let optimizer = optimizer_with(Some(AchievementConstraint::new(2.0, 0.0)), 200_000.0);
    let outcome = optimizer.search_outcome(&CANDIDATES).unwrap();

    assert!(!outcome.feasible_found);
    assert_eq!(
        outcome.infeasibility_reason(),
        Some(InfeasibilityReason::AchievementUnreachable),
        "should blame the workload, not the budget"
    );
    // Financially it is fine, which is exactly the diagnostic value.
    assert!(
        outcome.scenario.meets_requirements,
        "this household can afford its floor"
    );
}

/// `is_feasible` must require both constraints, not either.
#[test]
fn feasibility_requires_both_constraints() {
    let optimizer = optimizer_with(Some(AchievementConstraint::new(1.0, 0.0)), 150_000.0);
    let outcome = optimizer.search_outcome(&CANDIDATES).unwrap();

    for scenario in &outcome.all_scenarios {
        assert_eq!(
            scenario.is_feasible(),
            scenario.meets_requirements && scenario.achievement_satisfied(),
            "is_feasible must conjoin budget and achievement at {}%",
            scenario.work_percentage * 100.0
        );
    }
}

/// With no constraint supplied, behaviour must be unchanged: work percentage is
/// fully discretionary, and no scenario is ruled out on job-security grounds.
#[test]
fn absent_constraint_is_vacuous() {
    let optimizer = optimizer_with(None, 150_000.0);
    let outcome = optimizer.search_outcome(&CANDIDATES).unwrap();

    for scenario in &outcome.all_scenarios {
        assert!(
            scenario.achievement.is_none(),
            "no constraint should be recorded"
        );
        assert!(
            scenario.achievement_satisfied(),
            "an absent constraint cannot fail"
        );
    }
    // And the reduced-hours recommendation is still available.
    assert!(outcome.scenario.work_percentage < 1.0);
}

/// The status margin must be the signed capacity surplus, used by the display.
#[test]
fn achievement_margin_is_signed_correctly() {
    let constraint = AchievementConstraint::new(0.8, 0.0);
    let optimizer = optimizer_with(Some(constraint), 150_000.0);

    let generous = optimizer.evaluate_scenario(1.0).achievement.unwrap();
    assert!(generous.margin > 0.0, "full-time work exceeds a 0.8 goal");

    let tight = optimizer.evaluate_scenario(0.5).achievement.unwrap();
    assert!(
        tight.margin < 0.0,
        "half-time work falls short of a 0.8 goal"
    );
    assert!(!tight.satisfied);
}

/// Zero or negative goals must not produce a negative minimum.
#[test]
fn degenerate_goals_are_handled() {
    let zero = AchievementConstraint::new(0.0, 0.0);
    assert_eq!(zero.minimum_viable_work_percentage(), Some(0.0));
    assert!(zero.is_satisfied_at(0.0));

    // Negative AI gain is clamped away rather than reducing capacity.
    let negative_ai = AchievementConstraint::new(0.8, -0.5);
    assert!(negative_ai.ai_productivity_gain >= 0.0);
    assert!((negative_ai.capacity_at(1.0) - 1.0).abs() < 1e-9);
}
