//! Integration tests for work-percentage search behaviour.
//!
//! These lock in three defects that were found and fixed, and which are easy to
//! reintroduce:
//!
//! 1. A `u32` underflow panic for anyone at or past retirement age.
//! 2. An "optimal solution" announced when *nothing* was affordable.
//! 3. An infeasible fallback ranked by utility, which favoured deep
//!    under-employment (leisure and health scores rise as income falls) even
//!    when a higher work percentage had a much smaller shortfall.

use life_optimizer::monte_carlo::ConversionRateScenario;
use life_optimizer::optimizer::{LifeOptimizer, OptimizerConfig, SearchOutcome};
use life_optimizer::requirements::{LifeStage, PersonalRequirements, PreferenceWeights};
use life_optimizer::tax::TaxSchedule;

const CANDIDATES: [f64; 6] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

fn optimizer_for(age: u32, retirement_age: u32, salary: f64) -> LifeOptimizer {
    optimizer_for_children(age, retirement_age, salary, 0)
}

fn optimizer_for_children(
    age: u32,
    retirement_age: u32,
    salary: f64,
    children: u32,
) -> LifeOptimizer {
    LifeOptimizer::new(optimizer_config_children(
        age,
        retirement_age,
        salary,
        children,
    ))
}

fn search(optimizer: &LifeOptimizer, candidates: &[f64]) -> SearchOutcome {
    optimizer
        .search_outcome(candidates)
        .expect("a non-empty candidate list must produce an outcome")
}

/// Regression: `retirement_age - current_age` is `u32` arithmetic and used to
/// panic with "attempt to subtract with overflow".
#[test]
fn past_retirement_age_does_not_panic() {
    for (age, retirement_age) in [(66, 65), (70, 65), (65, 65), (80, 65)] {
        let optimizer = optimizer_for(age, retirement_age, 100_000.0);
        let scenario = optimizer.evaluate_scenario(1.0);

        assert!(
            (0.0..=10.0).contains(&scenario.utility_breakdown.security_utility),
            "age {age}: security utility {} out of range",
            scenario.utility_breakdown.security_utility
        );
        assert!(
            scenario.utility_score.is_finite(),
            "age {age}: non-finite utility"
        );

        // The full search must survive the same condition.
        let outcome = search(&optimizer, &CANDIDATES);
        assert!(
            outcome.scenario.utility_score.is_finite(),
            "age {age}: non-finite search result"
        );
    }
}

/// Regression: when nothing is affordable the tool must say so, and the
/// fallback must be the smallest shortfall rather than the nicest utility.
///
/// Note the scenario: 50k with two children. Under the elasticity-tiered
/// consumption model a 60k childless household is now *affordable* (its
/// mandatory floor excludes the savings goal and discretionary spending), so an
/// infeasibility test needs a household whose inelastic costs genuinely exceed
/// net income.
#[test]
fn infeasible_search_reports_fallback_and_minimises_shortfall() {
    let optimizer = optimizer_for_children(35, 65, 50_000.0, 2);
    let outcome = search(&optimizer, &CANDIDATES);

    assert!(
        !outcome.feasible_found,
        "50k with two children cannot cover the mandatory floor"
    );
    assert!(!outcome.scenario.meets_requirements);

    let smallest_shortfall = outcome
        .all_scenarios
        .iter()
        .map(|s| s.surplus_deficit)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (outcome.scenario.surplus_deficit - smallest_shortfall).abs() < 1e-9,
        "fallback should be the least-bad option: got {}, best is {}",
        outcome.scenario.surplus_deficit,
        smallest_shortfall
    );

    // The fallback must not be the utility-maximising candidate, which is the
    // specific behaviour that produced misleading advice. This scenario is
    // chosen because the two criteria genuinely diverge here.
    let utility_best = outcome
        .all_scenarios
        .iter()
        .map(|s| s.utility_score)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (outcome.scenario.utility_score - utility_best).abs() > 1e-9,
        "this scenario is meant to demonstrate the two criteria diverging"
    );
}

/// A feasible search must report feasibility and pick the highest-utility
/// affordable candidate.
#[test]
fn feasible_search_reports_feasibility() {
    let optimizer = optimizer_for(45, 65, 140_000.0);
    let outcome = search(&optimizer, &CANDIDATES);

    assert!(outcome.feasible_found, "a 140k salary should be affordable");
    assert!(outcome.scenario.meets_requirements);

    let best_feasible_utility = outcome
        .all_scenarios
        .iter()
        .filter(|s| s.meets_requirements)
        .map(|s| s.utility_score)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!((outcome.scenario.utility_score - best_feasible_utility).abs() < 1e-9);
}

/// The search must never return an infeasible scenario while claiming
/// feasibility — the exact inconsistency the display used to print.
#[test]
fn feasibility_flag_always_matches_the_chosen_scenario() {
    let cases = [
        (28, 65, 95_000.0),
        (38, 65, 110_000.0),
        (40, 65, 60_000.0),
        (45, 65, 140_000.0),
        (50, 65, 180_000.0),
        (60, 65, 120_000.0),
    ];

    for (age, retirement_age, salary) in cases {
        let optimizer = optimizer_for(age, retirement_age, salary);
        let outcome = search(&optimizer, &CANDIDATES);
        assert_eq!(
            outcome.feasible_found,
            outcome.scenario.is_feasible(),
            "age {age}, salary {salary}: feasible_found={} but is_feasible={}",
            outcome.feasible_found,
            outcome.scenario.is_feasible()
        );
    }
}

/// Empty candidate lists must not panic.
#[test]
fn empty_candidates_yields_none() {
    let optimizer = optimizer_for(40, 65, 100_000.0);
    assert!(optimizer.search_outcome(&[]).is_none());
}

/// `find_optimal` is the compatibility wrapper; it must agree with
/// `search_outcome` on both the chosen scenario and the full scenario list.
#[test]
fn find_optimal_agrees_with_search_outcome() {
    let optimizer = optimizer_for(45, 65, 140_000.0);
    let (scenario, all) = optimizer.find_optimal(&CANDIDATES);
    let outcome = search(&optimizer, &CANDIDATES);

    assert!((scenario.work_percentage - outcome.scenario.work_percentage).abs() < 1e-12);
    assert_eq!(all.len(), outcome.all_scenarios.len());
    for (a, b) in all.iter().zip(outcome.all_scenarios.iter()) {
        assert!((a.work_percentage - b.work_percentage).abs() < 1e-12);
    }
}

/// Scenarios are returned sorted by utility descending, which the comparison
/// table relies on.
#[test]
fn all_scenarios_are_sorted_by_utility_descending() {
    let optimizer = optimizer_for(45, 65, 140_000.0);
    let outcome = search(&optimizer, &CANDIDATES);

    for pair in outcome.all_scenarios.windows(2) {
        assert!(
            pair[0].utility_score >= pair[1].utility_score - 1e-12,
            "scenarios must be sorted by utility descending: {} then {}",
            pair[0].utility_score,
            pair[1].utility_score
        );
    }
}

/// Every candidate must be evaluated, so the comparison table has no gaps.
#[test]
fn all_candidates_are_evaluated() {
    let optimizer = optimizer_for(45, 65, 140_000.0);
    let outcome = search(&optimizer, &CANDIDATES);

    assert_eq!(outcome.all_scenarios.len(), CANDIDATES.len());
    for candidate in CANDIDATES {
        assert!(
            outcome
                .all_scenarios
                .iter()
                .any(|s| (s.work_percentage - candidate).abs() < 1e-12),
            "candidate {candidate} was not evaluated"
        );
    }
}

/// The conversion-rate scenario must actually influence the optimizer's
/// security term, otherwise `--conversion-rate` would silently do nothing for
/// the recommendation (only for the pension display).
#[test]
fn conversion_scenario_influences_security_utility() {
    let security = |scenario: ConversionRateScenario| {
        let mut config = optimizer_config(45, 65, 140_000.0);
        config.conversion_scenario = scenario;
        LifeOptimizer::new(config)
            .evaluate_scenario(1.0)
            .utility_breakdown
            .security_utility
    };

    let low = security(ConversionRateScenario::Custom(0.030));
    let high = security(ConversionRateScenario::Statutory);

    assert!(
        high > low,
        "a higher conversion rate must score a better security utility: high={high}, low={low}"
    );
}

fn optimizer_config(age: u32, retirement_age: u32, salary: f64) -> OptimizerConfig {
    optimizer_config_children(age, retirement_age, salary, 0)
}

fn optimizer_config_children(
    age: u32,
    retirement_age: u32,
    salary: f64,
    children: u32,
) -> OptimizerConfig {
    let mut config = OptimizerConfig::new(
        salary,
        TaxSchedule::bern_city_default(false, children),
        PersonalRequirements::bern_family_default(children),
        LifeStage::YoungSingle { age },
        PreferenceWeights::balanced(),
    );
    config.retirement_age = retirement_age;
    config.conversion_scenario = ConversionRateScenario::Statutory;
    config
}
