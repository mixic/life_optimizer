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

//! The state-transition model.
//!
//! One module per month is created for the first `modules` months, then the codebase
//! is fixed and only maintenance happens. Holding the *output* constant is deliberate:
//! the question is what a fixed body of code costs under two authoring modes, not how
//! much more code AI can produce. Growth is a separate question and modelling it here
//! would confound the two.

use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Poisson};

use crate::params::Params;

/// Which authoring mode a run uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// Every module is human-authored.
    Human,
    /// A fraction `ai_share` of modules is AI-authored; the rest are human.
    Ai,
}

/// One month's hours, by activity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonthLedger {
    pub month: usize,
    pub authoring: f64,
    pub review: f64,
    pub maintenance: f64,
    /// Mean comprehension across modules existing at the end of the month.
    pub comprehension_mean: f64,
    /// Modules created so far.
    pub modules: usize,
}

impl MonthLedger {
    pub fn total(&self) -> f64 {
        self.authoring + self.review + self.maintenance
    }
}

/// A full run.
#[derive(Debug, Clone, PartialEq)]
pub struct Ledger {
    pub months: Vec<MonthLedger>,
}

impl Ledger {
    /// Cumulative hours after each month.
    pub fn cumulative(&self) -> Vec<f64> {
        let mut running = 0.0;
        self.months
            .iter()
            .map(|month| {
                running += month.total();
                running
            })
            .collect()
    }

    pub fn total(&self) -> f64 {
        self.months.iter().map(MonthLedger::total).sum()
    }
}

/// A module's state during a run.
#[derive(Debug, Clone, Copy)]
struct ModuleState {
    complexity: f64,
    ai_authored: bool,
    comprehension: f64,
    /// Probability the creation-time review caught a defect. Fixed at creation,
    /// because the review happens once.
    catch_probability: f64,
    created_month: usize,
}

fn half_life(params: &Params, ai_authored: bool) -> f64 {
    if ai_authored {
        params.half_life_ai
    } else {
        params.half_life_human
    }
}

/// Hours to resolve one expected maintenance event on a module.
fn resolve_cost(params: &Params, module: &ModuleState) -> f64 {
    module.complexity
        * (params.diag_hours_per_unit * (1.0 + params.diag_slope * (1.0 - module.comprehension))
            + params.fix_hours_per_unit)
}

/// Expected maintenance events this month on a module.
fn event_rate(params: &Params, module: &ModuleState) -> f64 {
    let mode_penalty = if module.ai_authored { params.ai_defect_multiplier } else { 0.0 };
    params.hazard * module.complexity * (1.0 + mode_penalty)
        + params.leak_hazard * (1.0 - module.catch_probability)
}

/// Create the module for month `index`, charging authoring and review.
///
/// Returns the state and the `(authoring, review)` hours.
fn create(
    params: &Params,
    index: usize,
    scenario: Scenario,
    month: usize,
) -> (ModuleState, f64, f64) {
    let complexity = params.complexity(index);
    let ai_authored = match scenario {
        Scenario::Human => false,
        Scenario::Ai => params.is_ai_authored(index),
    };
    let initial = if ai_authored { params.c_ai } else { params.c_human };

    let authoring =
        params.alpha * complexity * if ai_authored { 1.0 - params.savings } else { 1.0 };
    // Review is charged in both scenarios. A model that only charges it for AI code
    // would be rigging the comparison; the difference must come from comprehension.
    let review =
        params.review_hours_per_unit * complexity * (1.0 + params.review_slope * (1.0 - initial));

    // The gate is only as good as the reader's comprehension at the time of review.
    let catch_probability = (initial / params.catch_threshold).min(1.0);
    let comprehension = initial + params.review_gain * (1.0 - initial);

    (
        ModuleState {
            complexity,
            ai_authored,
            comprehension,
            catch_probability,
            created_month: month,
        },
        authoring,
        review,
    )
}

/// Run the model with expected values (the mean-field, deterministic version).
pub fn run(params: &Params, scenario: Scenario) -> Ledger {
    let mut modules: Vec<ModuleState> = Vec::with_capacity(params.modules);
    let mut months = Vec::with_capacity(params.horizon_months);

    for month in 1..=params.horizon_months {
        let mut authoring = 0.0;
        let mut review = 0.0;
        let mut maintenance = 0.0;

        if month <= params.modules {
            let (state, a, r) = create(params, month - 1, scenario, month);
            modules.push(state);
            authoring += a;
            review += r;
        }

        for module in modules.iter_mut() {
            if module.created_month == month {
                continue;
            }
            let lambda = event_rate(params, module);
            module.comprehension *=
                (-std::f64::consts::LN_2 / half_life(params, module.ai_authored)).exp();
            maintenance += lambda * resolve_cost(params, module);
            // An interaction is what rebuilds the model: the chance of at least one
            // event this month, applied as a pull toward full comprehension.
            let touched = 1.0 - (-lambda).exp();
            module.comprehension += touched * params.review_gain * (1.0 - module.comprehension);
        }

        let comprehension_mean = if modules.is_empty() {
            0.0
        } else {
            modules.iter().map(|m| m.comprehension).sum::<f64>() / modules.len() as f64
        };

        months.push(MonthLedger {
            month,
            authoring,
            review,
            maintenance,
            comprehension_mean,
            modules: modules.len(),
        });
    }

    Ledger { months }
}

/// Run the model drawing real Poisson arrivals, for one seed.
///
/// This exists to price the mean-field assumption: the deterministic run charges
/// `lambda` events every month, while a real codebase gets zero, one or four. The
/// difference between the two is how much the phase diagram's clean boundaries smear.
pub fn run_stochastic(params: &Params, scenario: Scenario, seed: u64) -> Ledger {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut modules: Vec<ModuleState> = Vec::with_capacity(params.modules);
    let mut months = Vec::with_capacity(params.horizon_months);

    for month in 1..=params.horizon_months {
        let mut authoring = 0.0;
        let mut review = 0.0;
        let mut maintenance = 0.0;

        if month <= params.modules {
            let (state, a, r) = create(params, month - 1, scenario, month);
            modules.push(state);
            authoring += a;
            review += r;
        }

        for module in modules.iter_mut() {
            if module.created_month == month {
                continue;
            }
            let lambda = event_rate(params, module);
            module.comprehension *=
                (-std::f64::consts::LN_2 / half_life(params, module.ai_authored)).exp();

            let events = if lambda > 0.0 {
                Poisson::new(lambda)
                    .expect("event rate is finite and non-negative")
                    .sample(&mut rng)
            } else {
                0.0
            };
            maintenance += events * resolve_cost(params, module);
            if events > 0.0 {
                module.comprehension += params.review_gain * (1.0 - module.comprehension);
            }
        }

        let comprehension_mean = if modules.is_empty() {
            0.0
        } else {
            modules.iter().map(|m| m.comprehension).sum::<f64>() / modules.len() as f64
        };

        months.push(MonthLedger {
            month,
            authoring,
            review,
            maintenance,
            comprehension_mean,
            modules: modules.len(),
        });
    }

    Ledger { months }
}

/// The first month at which cumulative AI benefit is negative, if any.
///
/// Benefit is `cumulative(Human) - cumulative(Ai)`: positive means AI spent fewer hours.
/// Months before any AI-authored module exists are excluded, because a difference of
/// zero there is not a crossing, it is an absence of the thing being measured.
pub fn crossover_month(human: &Ledger, ai: &Ledger) -> Option<usize> {
    let human = human.cumulative();
    let ai = ai.cumulative();
    if human.len() != ai.len() {
        return None;
    }
    let mut first_ai_month = None;
    for (index, (h, a)) in human.iter().zip(ai.iter()).enumerate() {
        let benefit = h - a;
        if benefit.abs() > 1e-9 && first_ai_month.is_none() {
            first_ai_month = Some(index);
        }
        if let Some(start) = first_ai_month {
            if index >= start && benefit < 0.0 {
                return Some(index + 1);
            }
        }
    }
    None
}

/// Cumulative benefit per month, for reporting.
pub fn benefit_curve(human: &Ledger, ai: &Ledger) -> Vec<f64> {
    human.cumulative().iter().zip(ai.cumulative().iter()).map(|(h, a)| h - a).collect()
}
