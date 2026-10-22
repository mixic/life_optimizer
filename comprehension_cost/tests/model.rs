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

//! The model's invariants, and the two controls that stop it saying anything it likes.
//!
//! The most important test here is [`the_model_can_report_a_win`]. A model built to
//! show that AI costs more is worthless if it cannot show the opposite, and the only
//! way to demonstrate that is to remove the mechanism under test and check the answer
//! flips. If that test ever fails, the phase diagram in the report is an artefact.

use comprehension_cost::model::{benefit_curve, crossover_month, run, run_stochastic, Scenario};
use comprehension_cost::params::{Params, Provenance};
use comprehension_cost::sweep::{is_falsified, sweep, verdict, Verdict};

/// A parameter set with every comprehension asymmetry switched off.
///
/// AI-authored code starts as well understood, is retained as well, and carries no
/// defect penalty. The only surviving difference is the authoring saving.
fn symmetric(savings: f64) -> Params {
    let mut p = Params::default();
    p.savings = savings;
    p.c_ai = p.c_human;
    p.half_life_ai = p.half_life_human;
    p.ai_defect_multiplier = 0.0;
    p.leak_hazard = 0.0;
    p
}

fn benefit_end(p: &Params) -> f64 {
    let human = run(p, Scenario::Human);
    let ai = run(p, Scenario::Ai);
    benefit_curve(&human, &ai).last().copied().unwrap_or(f64::NAN)
}

/// **The control.** With the comprehension asymmetry removed, AI must win.
///
/// This is what makes the headline result a finding rather than a restatement of the
/// assumptions. The model is capable of reporting a benefit; it reports a loss only
/// when the comprehension and maintenance channels are switched on.
#[test]
fn the_model_can_report_a_win() {
    for savings in [0.15, 0.33] {
        let params = symmetric(savings);
        let human = run(&params, Scenario::Human);
        let ai = run(&params, Scenario::Ai);
        let benefit = benefit_end(&params);
        assert!(
            benefit > 0.0,
            "savings {savings}: benefit {benefit:.1} h should be positive when AI code is \
             understood exactly as well as human code"
        );
        assert_eq!(
            crossover_month(&human, &ai),
            None,
            "savings {savings}: there should be no crossover"
        );
    }
}

/// With the saving set to zero and the asymmetry removed, the two runs are identical.
///
/// A model that reported a non-zero difference here would be charging something twice.
#[test]
fn the_null_scenario_is_exactly_zero() {
    let params = symmetric(0.0);
    let benefit = benefit_end(&params);
    assert!(
        benefit.abs() < 1e-9,
        "with no saving and no asymmetry the benefit must be exactly zero, got {benefit:.6}"
    );
    let human = run(&params, Scenario::Human);
    let ai = run(&params, Scenario::Ai);
    for (h, a) in human.months.iter().zip(ai.months.iter()) {
        assert!((h.total() - a.total()).abs() < 1e-9, "month {}", h.month);
    }
}

/// AI-authored modules must be spread across the complexity range, not handed the
/// cheapest half.
///
/// The first version of `is_ai_authored` did the latter, and because complexity rises
/// with module index it gave AI the cheap end of the codebase. That understated the
/// authoring saving by half and biased the model against its own hypothesis — a bug
/// found by noticing the authoring-only channel produced +6 h where the arithmetic
/// predicted +12 h.
#[test]
fn ai_authored_modules_are_spread_across_the_complexity_range() {
    let params = Params::default();
    let ai: Vec<usize> = (0..params.modules).filter(|i| params.is_ai_authored(*i)).collect();
    assert_eq!(ai.len(), 5, "a 50% share of ten modules is five modules");
    assert_eq!(ai, vec![1, 3, 5, 7, 9], "every other module, not the first five");

    let all_mean: f64 =
        (0..params.modules).map(|i| params.complexity(i)).sum::<f64>() / params.modules as f64;
    let ai_mean: f64 = ai.iter().map(|i| params.complexity(*i)).sum::<f64>() / ai.len() as f64;
    assert!(
        (ai_mean - all_mean).abs() < 0.2,
        "AI modules average complexity {ai_mean:.2} against a codebase mean of {all_mean:.2}"
    );

    // And the saving the assignment implies matches the arithmetic, which is the check
    // that would have caught the original bug.
    let params = symmetric(0.15);
    let predicted: f64 =
        ai.iter().map(|i| params.alpha * params.complexity(*i) * params.savings).sum();
    assert!(
        (benefit_end(&params) - predicted).abs() < 1e-6,
        "benefit {:.2} should equal the predicted saving {predicted:.2}",
        benefit_end(&params)
    );
}

/// Longer retention is worth something, monotonically.
#[test]
fn better_retention_is_never_worse() {
    let mut previous = f64::NEG_INFINITY;
    for half_life in [6.0, 9.0, 12.0, 18.0, 24.0] {
        let mut params = Params::default();
        params.half_life_ai = half_life;
        let benefit = benefit_end(&params);
        assert!(
            benefit >= previous - 1e-9,
            "half-life {half_life}: benefit {benefit:.1} fell below the shorter half-life's \
             {previous:.1}"
        );
        previous = benefit;
    }
    // And retention is not a rounding effect: the 24-month case must be strictly better
    // than the 6-month one, or the asymmetry the hypothesis is about does nothing.
    let mut fast = Params::default();
    fast.half_life_ai = 6.0;
    let mut slow = Params::default();
    slow.half_life_ai = 24.0;
    assert!(
        benefit_end(&slow) > benefit_end(&fast) + 1.0,
        "retention asymmetry moves the answer by {:.1} h, which is too little to matter",
        benefit_end(&slow) - benefit_end(&fast)
    );
}

/// A larger defect penalty on AI code is never better.
#[test]
fn the_defect_multiplier_costs_hours() {
    let mut previous = f64::INFINITY;
    for epsilon in [0.0, 0.1, 0.2, 0.5] {
        let mut params = Params::default();
        params.ai_defect_multiplier = epsilon;
        let benefit = benefit_end(&params);
        assert!(
            benefit <= previous + 1e-9,
            "epsilon {epsilon}: benefit {benefit:.1} rose above the smaller penalty's \
             {previous:.1}"
        );
        previous = benefit;
    }
}

/// Comprehension is a probability and stays in `[0, 1]` for every module, month and seed.
#[test]
fn comprehension_stays_a_probability() {
    for seed in 0..8 {
        let ledger = run_stochastic(&Params::default(), Scenario::Ai, seed);
        for month in &ledger.months {
            assert!(
                (0.0..=1.0).contains(&month.comprehension_mean),
                "seed {seed} month {}: mean comprehension {}",
                month.month,
                month.comprehension_mean
            );
        }
    }
    for month in run(&Params::default(), Scenario::Human).months {
        assert!(
            (0.0..=1.0).contains(&month.comprehension_mean),
            "month {}: {}",
            month.month,
            month.comprehension_mean
        );
    }
}

/// The mean-field run and the stochastic run must agree in expectation.
///
/// The deterministic model charges `lambda` events every month; the stochastic one
/// draws them. Averaged over many seeds the totals should converge, and a large gap
/// would mean one of the two is wrong rather than merely noisy.
#[test]
fn the_mean_field_and_the_simulation_agree_in_expectation() {
    let params = Params::default();
    let deterministic = run(&params, Scenario::Ai).total();
    let paths = 400;
    let mean: f64 =
        (0..paths).map(|seed| run_stochastic(&params, Scenario::Ai, seed).total()).sum::<f64>()
            / paths as f64;
    let error = (mean - deterministic).abs() / deterministic;
    assert!(
        error < 0.03,
        "mean-field {deterministic:.0} h against simulated mean {mean:.0} h is a {:.1}% gap",
        error * 100.0
    );
}

/// Exactly one parameter is sourced, and it is the contested one.
///
/// If a hypothesis ever gets relabelled as measured, this fails.
#[test]
fn only_the_contested_parameter_is_sourced() {
    let params = Params::default();
    let sourced: Vec<&str> = params
        .provenance()
        .into_iter()
        .filter(|(_, _, provenance)| provenance.is_sourced())
        .map(|(name, _, _)| name)
        .collect();
    assert_eq!(sourced.len(), 1, "exactly one parameter is sourced; found {sourced:?}");
    assert!(
        sourced[0].starts_with("savings"),
        "the sourced parameter must be the authoring saving, not {sourced:?}"
    );
    for (name, _value, provenance) in params.provenance() {
        if let Provenance::Sourced(citation) = provenance {
            assert!(
                citation.contains("METR") && citation.contains("BairesDev"),
                "{name} cites only one side of a contested quantity: {citation}"
            );
        }
    }
}

/// The verdict rule reproduces the pre-registration on the real grid.
#[test]
fn the_verdict_rule_is_the_preregistered_one() {
    let params = Params::default();
    let cells = sweep(&params, &[0.0, 0.15, 0.33], &[6.0, 9.0, 24.0], &[0.0, 0.1, 0.5]);
    let decision = verdict(&cells, &[0.0, 0.15, 0.33]);
    assert!(
        matches!(decision, Verdict::Supported),
        "the central cell crosses over, so the pre-registered verdict is Supported; got {decision:?}"
    );
    assert!(!is_falsified(&cells), "the falsification condition must not fire on this grid");

    // A grid in which nothing crosses over must produce NotSupported, so the rule can
    // actually say no.
    let wins = sweep(&symmetric(0.33), &[0.33], &[24.0], &[0.0]);
    assert!(
        matches!(verdict(&wins, &[0.33]), Verdict::NotSupported),
        "a grid with no crossover must not report support"
    );
    assert!(is_falsified(&wins));
}
