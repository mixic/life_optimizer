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
    AchievementConstraint, Enforcement, InfeasibilityReason, LifeOptimizer, OptimizerConfig,
    Robustness,
};
use life_optimizer::requirements::{LifeStage, PersonalRequirements, PreferenceWeights};
use life_optimizer::tax::TaxSchedule;

const CANDIDATES: [f64; 6] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

fn optimizer_with(constraint: Option<AchievementConstraint>, salary: f64) -> LifeOptimizer {
    optimizer_with_enforcement(constraint, salary, Enforcement::Strict)
}

fn optimizer_with_enforcement(
    constraint: Option<AchievementConstraint>,
    salary: f64,
    enforcement: Enforcement,
) -> LifeOptimizer {
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
    config.enforcement = enforcement;
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

// ─────────────────────────────────────────────────────────────────────────────
// The six outcome-based scenarios of CRITICS_CURRENT_WORK.md §1.4.
//
// Items 1 and 2's fixed-goal half are covered above. What follows is one test
// per remaining item, so that a future change which quietly drops one of them
// fails here rather than in prose.
// ─────────────────────────────────────────────────────────────────────────────

/// §1.4 item 2 — AI productivity as an explicit **range**, not a constant.
///
/// A schedule that delivers only at the optimistic end is a bet on the tool, and
/// the model must say so rather than averaging the two ends into a single
/// reassuring number.
#[test]
fn a_gain_range_separates_robust_schedules_from_bets() {
    // Goal 1.0, AI somewhere between +0% and +25%. At 80% work the goals are met
    // only if the optimistic end materialises.
    let ranged = AchievementConstraint::new(1.0, 0.0).with_ai_gain_range(0.25);

    assert_eq!(ranged.robustness_at(0.8), Robustness::OptimisticOnly);
    assert!(
        !ranged.is_satisfied_at(0.8),
        "meeting the goals only at the top of the range is not a credible reduction"
    );
    assert!(
        ranged.is_satisfied_optimistically_at(0.8),
        "but the optimistic case must be reported, not hidden"
    );

    // Full time is robust across the whole range: +0% already suffices.
    assert_eq!(ranged.robustness_at(1.0), Robustness::Robust);

    // A goal beyond the optimistic end is unreachable at any point in the range.
    let impossible = AchievementConstraint::new(1.4, 0.0).with_ai_gain_range(0.25);
    assert_eq!(impossible.robustness_at(1.0), Robustness::Unreachable);
    assert!(!impossible.is_satisfied_optimistically_at(1.0));

    // The range must never make a schedule look better than its pessimistic end:
    // widening it downwards cannot rescue a schedule.
    let widened = AchievementConstraint::new(1.0, 0.0).with_ai_gain_range(0.25);
    let narrowed = AchievementConstraint::new(1.0, 0.0).with_ai_gain_range(0.05);
    assert_eq!(widened.robustness_at(0.8), Robustness::OptimisticOnly);
    assert_eq!(
        narrowed.robustness_at(0.8),
        Robustness::Unreachable,
        "the optimistic end is what decides reachability, and 5% is not enough"
    );

    // And a single-valued constraint is the degenerate range it always was.
    let point = AchievementConstraint::new(0.8, 0.0);
    assert_eq!(point.ai_productivity_gain_high, point.ai_productivity_gain);
    assert_eq!(point.robustness_at(0.8), Robustness::Robust);
}

/// §1.4 item 3, first channel — **quality**: AI throughput does not arrive free of
/// rework, so only the gain that survives verification can be banked.
#[test]
fn rework_retention_makes_the_ai_justification_harder() {
    // +25% naive, half of which survives verification: +12.5% effective.
    let naive = AchievementConstraint::new(1.0, 0.25);
    let realistic = AchievementConstraint::new(1.0, 0.25).with_quality(0.5, 0.0);

    assert_eq!(naive.robustness_at(0.8), Robustness::Robust);
    assert_eq!(
        realistic.robustness_at(0.8),
        Robustness::Unreachable,
        "0.8 * 1.125 = 0.9, which does not deliver a goal of 1.0"
    );

    // The question "how much AI would 80% need?" doubles when half the gain is
    // eaten by rework: 0.25 naive, 0.5 after retention.
    assert!((naive.required_ai_gain_for(0.8).unwrap() - 0.25).abs() < 1e-9);
    assert!((realistic.required_ai_gain_for(0.8).unwrap() - 0.5).abs() < 1e-9);

    // Zero retention means no amount of AI helps: the gain is entirely rework.
    let useless = AchievementConstraint::new(1.0, 0.25).with_quality(0.0, 0.0);
    assert_eq!(
        useless.required_ai_gain_for(0.8),
        None,
        "if none of the gain survives, a larger gain buys nothing"
    );
    assert_eq!(useless.robustness_at(0.8), Robustness::Unreachable);
}

/// §1.4 item 3, second channel — the **compression** cost of squeezing the same
/// output into fewer hours. Raw capacity can meet the target exactly and the
/// schedule still fails, because the pace needed to do it degrades delivery.
#[test]
fn compressing_the_same_output_into_fewer_hours_costs_quality() {
    // Raw capacity at 80% with +25% AI is exactly 1.0, which "meets" a goal of 1.0.
    let naive = AchievementConstraint::new(1.0, 0.25);
    assert!((naive.capacity_at(0.8) - 1.0).abs() < 1e-9);
    assert_eq!(naive.robustness_at(0.8), Robustness::Robust);

    // With a quality cost of one unit per unit of pace above baseline, the same
    // schedule delivers 0.75 and fails.
    let honest = AchievementConstraint::new(1.0, 0.25).with_quality(1.0, 1.0);
    assert!((honest.quality_factor_at(0.8) - 0.75).abs() < 1e-9);
    assert!((honest.delivered_pessimistic(0.8) - 0.75).abs() < 1e-9);
    assert_eq!(
        honest.robustness_at(0.8),
        Robustness::Unreachable,
        "meeting the target while producing defects is not meeting the target"
    );

    // Full time needs no stretch and so pays no quality penalty.
    assert!((honest.quality_factor_at(1.0) - 1.0).abs() < 1e-9);
    assert_eq!(honest.robustness_at(1.0), Robustness::Robust);

    // The lowest viable percentage is now above the raw-capacity answer, and the
    // answer must still be a real percentage rather than a bisection artefact.
    let minimum = honest
        .minimum_viable_work_percentage()
        .expect("full time still delivers");
    assert!(
        minimum > 0.8 && minimum <= 1.0,
        "quality drag must push the minimum above the raw 0.8, got {minimum}"
    );
    assert!(
        honest.delivered_pessimistic(minimum) >= 1.0 - 1e-6,
        "and the returned percentage must actually deliver the goal"
    );
}

/// §1.4 item 4 — **hidden work**: a schedule that needs more capacity than the
/// contract provides is worked at the higher figure whatever the contract says.
/// It is reported as workload and must never be presented as spare time.
#[test]
fn hidden_work_is_counted_as_workload_not_as_leisure() {
    // Goal 1.0 with no AI: 80% delivers 0.8, so holding the job would take 100%.
    let constraint = AchievementConstraint::new(1.0, 0.0);
    assert!((constraint.hidden_work_percentage(0.8) - 0.2).abs() < 1e-9);

    let optimizer = optimizer_with(Some(constraint), 150_000.0);
    let part_time = optimizer.evaluate_scenario(0.8);

    assert!(
        (part_time.hidden_work_hours_per_week - 42.0 * 0.2).abs() < 1e-9,
        "hidden work must be reported in hours, got {}",
        part_time.hidden_work_hours_per_week
    );
    // The trap the critique names: free time computed from the contract alone
    // would claim 20% more leisure than the job actually allows.
    let contract_only_free_hours = 168.0 - 0.8 * 42.0 - 56.0;
    assert!(
        (part_time.free_hours_per_week - contract_only_free_hours).abs() < 1e-9,
        "leisure is taken from the contract here, and the hidden hours are reported \
         separately rather than silently added to it"
    );
    assert!(
        !part_time.achievement_satisfied(),
        "and the schedule is not offered as a reduction"
    );

    // A robust schedule needs no cover at all.
    let robust = optimizer_with(Some(AchievementConstraint::new(0.8, 0.0)), 150_000.0)
        .evaluate_scenario(0.8);
    assert_eq!(
        robust.hidden_work_hours_per_week, 0.0,
        "delivering at the pessimistic end means nothing is hidden"
    );
}

/// §1.4 item 5 — **replacement risk**, as a price rather than a catastrophe.
///
/// Strict enforcement refuses to offer a missed schedule. Risk-weighted
/// enforcement offers it and charges the implied chance of losing the job against
/// its security, which is what makes the trade-off visible.
#[test]
fn replacement_risk_is_priced_when_it_is_declared() {
    let goal = 1.0;
    let unpriced = AchievementConstraint::new(goal, 0.0);
    assert_eq!(
        unpriced.replacement_risk_at(0.8),
        0.0,
        "with no declared sensitivity, job loss is not priced at all"
    );

    // A sensitivity of 2.5 means a 20% shortfall implies a 50% chance of replacement.
    let priced = AchievementConstraint::new(goal, 0.0).with_replacement_risk(2.5);
    assert!((priced.replacement_risk_at(0.8) - 0.5).abs() < 1e-9);
    assert!((priced.replacement_risk_at(1.0) - 0.0).abs() < 1e-9);
    assert!(
        priced.replacement_risk_at(0.5) <= 1.0,
        "risk is a probability and must stay bounded"
    );

    // Strict: the search is forced to full time. Risk-weighted with a modest
    // price: the reduced schedule is offered, and carries its risk with it.
    let strict = optimizer_with_enforcement(
        Some(priced),
        150_000.0,
        Enforcement::Strict,
    );
    let weighted = optimizer_with_enforcement(
        Some(priced),
        150_000.0,
        Enforcement::RiskWeighted,
    );

    let strict_outcome = strict.search_outcome(&CANDIDATES).unwrap();
    assert!(
        (strict_outcome.scenario.work_percentage - 1.0).abs() < 1e-9,
        "strict enforcement must refuse the shortfall"
    );

    let weighted_outcome = weighted.search_outcome(&CANDIDATES).unwrap();
    assert!(
        weighted_outcome.all_scenarios.iter().any(|s| !s.achievement_satisfied()
            && s.is_feasible()),
        "risk-weighted enforcement must offer at least one shortfall, priced"
    );
    for scenario in &weighted_outcome.all_scenarios {
        assert_eq!(
            scenario.is_feasible(),
            scenario.meets_requirements,
            "under risk-weighted enforcement only affordability can block"
        );
    }

    // And the price must bite: at a swingeing sensitivity, the shortfall is no
    // longer worth choosing and 100% wins again. This is what makes it a
    // trade-off rather than decoration.
    let punitive = AchievementConstraint::new(goal, 0.0).with_replacement_risk(100.0);
    let punished = optimizer_with_enforcement(
        Some(punitive),
        150_000.0,
        Enforcement::RiskWeighted,
    );
    let punished_outcome = punished.search_outcome(&CANDIDATES).unwrap();
    assert!(
        (punished_outcome.scenario.work_percentage - 1.0).abs() < 1e-9,
        "a near-certain replacement must outweigh the leisure, got {}",
        punished_outcome.scenario.work_percentage
    );
}

/// An assignment that *no* work percentage delivers is a workload problem, not a
/// trade-off, and must not be optimised for leisure.
///
/// Risk weighting exists to price a shortfall the worker could have avoided by
/// working more. When the required output is unreachable even at 100% with the
/// optimistic AI gain, there is no such choice: every candidate fails by the same
/// amount, so a utility search with the delivery requirement relaxed takes the
/// lowest percentage — recommending 50% work and reporting the goals as
/// undeliverable in the same breath. The search must instead fall back to the
/// least-bad option and report that nothing met the goals.
#[test]
fn an_impossible_assignment_is_reported_rather_than_optimised() {
    // Twice what full time delivers, with no AI leverage: unreachable everywhere.
    let unreachable = AchievementConstraint::new(2.0, 0.0).with_replacement_risk(2.5);
    let optimizer =
        optimizer_with_enforcement(Some(unreachable), 150_000.0, Enforcement::RiskWeighted);
    let outcome = optimizer.search_outcome(&CANDIDATES).unwrap();

    assert!(
        !outcome.feasible_found,
        "no percentage delivers the goals, so nothing is feasible — even under \
         risk-weighted enforcement"
    );
    assert!(
        (outcome.scenario.work_percentage - 1.0).abs() < 1e-9,
        "the least-bad option is full time, not the most leisurely one; got {}",
        outcome.scenario.work_percentage
    );
    // Note what is *not* asserted: `scenario.is_feasible()` is still true here,
    // because under risk-weighted enforcement only affordability can block a
    // single scenario. The "nothing delivers this" verdict lives at the level of
    // the search, which is why `feasible_found` — not that predicate — is what the
    // report uses to decide whether to call the result an optimum.
    assert_eq!(
        outcome.scenario.achievement.map(|a| a.robustness),
        Some(Robustness::Unreachable),
        "the chosen fallback is unreachable at every percentage, which is the \
         finding the report has to carry"
    );

    // The guard must not fire for an assignment that *is* reachable optimistically:
    // that is exactly the contingent shortfall the mode is for.
    let contingent = AchievementConstraint::new(1.0, 0.1)
        .with_ai_gain_range(0.4)
        .with_replacement_risk(2.5);
    let optimizer =
        optimizer_with_enforcement(Some(contingent), 150_000.0, Enforcement::RiskWeighted);
    let outcome = optimizer.search_outcome(&CANDIDATES).unwrap();
    assert!(
        outcome.feasible_found,
        "a schedule that delivers at the optimistic end is still offerable"
    );
    assert!(
        outcome
            .all_scenarios
            .iter()
            .any(|s| s.delivers_optimistically() && !s.achievement_satisfied()),
        "and the offered schedule is the contingent one, not a robust one"
    );
}

/// §1.4 item 6 — **adaptation**: a reduction is only available after a period of
/// demonstrated delivery, so the first years are worked at full time and the
/// average workload — which is what leisure is measured from — is higher than the
/// contract.
#[test]
fn an_evaluation_period_is_averaged_into_the_workload() {
    // Age 40, retirement 65: a 25-year horizon.
    let constraint = AchievementConstraint::new(0.8, 0.0).with_evaluation_period(2.0);
    // 2 years at 1.0 and 23 at 0.8.
    let expected = (2.0 * 1.0 + 23.0 * 0.8) / 25.0;
    assert!((constraint.amortised_work_percentage(0.8, 25.0) - expected).abs() < 1e-9);

    let optimizer = optimizer_with(Some(constraint), 150_000.0);
    let scenario = optimizer.evaluate_scenario(0.8);

    assert!(
        (scenario.work_hours_per_week - 0.8 * 42.0).abs() < 1e-9,
        "the contract is unchanged"
    );
    assert!(
        scenario.effective_work_hours_per_week > scenario.work_hours_per_week,
        "the probation must raise the average workload: {} vs {}",
        scenario.effective_work_hours_per_week,
        scenario.work_hours_per_week
    );
    assert!(
        (scenario.effective_work_hours_per_week - 42.0 * expected).abs() < 1e-9,
        "effective hours must be the amortised figure"
    );
    assert!(
        scenario.free_hours_per_week < 168.0 - 0.8 * 42.0 - 56.0,
        "and leisure must be taken from the average, not the contract"
    );

    // With no evaluation period the two collapse back together, which is what
    // keeps the original model's answers intact.
    let immediate = optimizer_with(Some(AchievementConstraint::new(0.8, 0.0)), 150_000.0)
        .evaluate_scenario(0.8);
    assert!(
        (immediate.effective_work_hours_per_week - immediate.work_hours_per_week).abs() < 1e-9
    );
}
