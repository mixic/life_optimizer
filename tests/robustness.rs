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

//! The robustness block's contract: the weights it prints are the ones it used, the perturbations
//! cover every declared input, and an answer sitting on the edge of the search is not presented as
//! a finding.

use life_optimizer::optimizer::{LifeOptimizer, OptimizerConfig};
use life_optimizer::requirements::{LifeStage, PersonalRequirements, PreferenceWeights};
use life_optimizer::robustness::{
    analyse, perturb_requirements, perturb_weights, CRITERIA, FACTORS,
};
use life_optimizer::tax::TaxSchedule;

const CANDIDATES: [f64; 6] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

fn config() -> OptimizerConfig {
    OptimizerConfig::new(
        120_000.0,
        TaxSchedule::bern_city_default(false, 0),
        PersonalRequirements::bern_family_default(0),
        LifeStage::YoungSingle { age: 40 },
        PreferenceWeights::balanced(),
    )
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Perturbing a weight must preserve the vector's sum, because the question is about relative
/// importance. Changing the total scale would move every scenario together and answer nothing.
#[test]
fn perturbing_a_weight_preserves_the_sum() {
    let base = PreferenceWeights::balanced();
    for (index, name) in CRITERIA.iter().enumerate() {
        for factor in FACTORS {
            let moved = perturb_weights(&base, index, factor);
            let total = moved.consumption
                + moved.leisure
                + moved.family
                + moved.health
                + moved.security;
            assert!(
                close(total, 1.0),
                "moving {name} by x{factor} left the weights summing to {total}"
            );
            assert!(moved.validate(), "the perturbed weights are not valid");
        }
    }
}

/// Doubling one weight must raise it and lower the others; halving must do the reverse. A
/// perturbation that left the others untouched would be changing the scale, not the balance.
#[test]
fn doubling_one_weight_lowers_the_others_and_halving_raises_them() {
    let base = PreferenceWeights::balanced();
    let doubled = perturb_weights(&base, 2, 2.0); // family
    assert!(doubled.family > base.family);
    assert!(doubled.consumption < base.consumption);
    assert!(doubled.leisure < base.leisure);

    let halved = perturb_weights(&base, 2, 0.5);
    assert!(halved.family < base.family);
    assert!(halved.consumption > base.consumption);
}

/// Scaling the consumption basket must move every money field by the factor. Missing one would
/// leave the "needs" perturbation partly applied, which is worse than not applying it.
#[test]
fn the_requirements_perturbation_scales_every_money_field() {
    let base = PersonalRequirements::bern_family_default(0);
    let doubled = perturb_requirements(&base, 2.0);
    assert!(close(doubled.housing, base.housing * 2.0));
    assert!(close(doubled.food, base.food * 2.0));
    assert!(close(doubled.transport, base.transport * 2.0));
    assert!(close(doubled.insurance, base.insurance * 2.0));
    assert!(close(doubled.childcare, base.childcare * 2.0));
    assert!(close(doubled.healthcare, base.healthcare * 2.0));
    assert!(close(doubled.savings_goal, base.savings_goal * 2.0));
    assert!(close(doubled.discretionary, base.discretionary * 2.0));
    // And the total moves with it, which is the point of scaling them all.
    assert!(close(doubled.total_monthly(), base.total_monthly() * 2.0));
}

/// The block must cover every declared weight at both factors, plus the consumption basket.
#[test]
fn the_report_covers_every_declared_input() {
    let report = analyse(&config(), &CANDIDATES);
    assert_eq!(
        report.perturbations.len(),
        CRITERIA.len() * FACTORS.len() + FACTORS.len(),
        "the block does not cover every input"
    );
    for name in CRITERIA {
        for factor in FACTORS {
            assert!(
                report
                    .perturbations
                    .iter()
                    .any(|perturbation| perturbation.name == name
                        && close(perturbation.factor, factor)),
                "{name} at x{factor} is missing from the block"
            );
        }
    }
    // The weights printed must be the ones used, or the block would describe a different run.
    assert!(close(report.weights.family, 0.25), "the balanced weights are not the ones printed");
    assert_eq!(report.searched, (0.5, 1.0));
}

/// The baseline must be the answer the search actually gives, not a re-derivation.
#[test]
fn the_baseline_is_the_answer_the_search_gives() {
    let config = config();
    let expected = LifeOptimizer::new(config.clone())
        .search_outcome(&CANDIDATES)
        .map(|outcome| outcome.scenario.work_percentage);
    let report = analyse(&config, &CANDIDATES);
    match (expected, report.base) {
        (Some(expected), Some(base)) => assert!(close(expected, base)),
        (None, None) => {}
        (expected, base) => panic!("the block reports {base:?} where the search gives {expected:?}"),
    }
}

/// An optimum that sits on the edge of the searched range must say so. The CLI's own default run
/// lands there, and presenting it as a stable recommendation would be the most confident possible
/// statement about the least informative possible answer.
#[test]
fn an_optimum_on_the_edge_of_the_search_is_flagged() {
    let report = analyse(&config(), &CANDIDATES);
    assert!(
        report.at_search_edge(),
        "the fixture should land on an edge; it produced {:?}",
        report.base
    );
    let verdict = report.verdict();
    assert!(
        verdict.contains("edge") || verdict.contains("low end") || verdict.contains("high end"),
        "the verdict does not mention the search edge: {verdict:?}"
    );
    // And a narrow search range must be reported as the range it was, so the reader can see what
    // "the edge" means.
    assert!(close(report.searched.0, 0.5));
    assert!(close(report.searched.1, 1.0));
}

/// Shrinking the candidate list must move the reported edge with it: the field describes the
/// search that was run, not a constant.
#[test]
fn the_searched_range_follows_the_candidates() {
    let report = analyse(&config(), &[0.6, 0.7, 0.8]);
    assert!(close(report.searched.0, 0.6));
    assert!(close(report.searched.1, 0.8));
}
