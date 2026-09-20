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

// Core optimization module
#![allow(dead_code)]
use crate::consumption::{ConsumptionProfileConfig, ConsumptionTiers};
use crate::requirements::{LifeStage, PersonalRequirements, PreferenceWeights, FamilySupport};
use crate::tax::TaxSchedule;
use serde::{Deserialize, Serialize};

/// Employer-side achievement-capacity constraint, from `CRITICS_CURRENT_WORK.md`
/// §1.3 and `MATHEMATICS.md` §14.1.
///
/// A reduction in work percentage is only credible when effective achievement
/// capacity still meets the organisation's required output:
///
/// ```text
/// A_t = H_t * P_t * (1 + alpha_t)   subject to   A_t >= G_t
/// ```
///
/// This operationalizes the reframed question from
/// `PHILOSOPHICAL_SOCIOLOGICAL_ASPECTS.MD` §3a: not "what work percentage
/// maximizes my utility", but "what is the lowest work percentage at which I can
/// still reliably deliver what is expected of me".
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AchievementConstraint {
    /// Baseline productivity per unit of work time, normalized so that
    /// `P_t = 1.0` at full-time capacity.
    pub baseline_productivity: f64,
    /// `alpha_t`: productivity gain from AI and other tools. `0.0` means none.
    pub ai_productivity_gain: f64,
    /// `G_t`: required output index for the period, in the same normalized
    /// units as capacity.
    pub required_output_index: f64,
}

impl AchievementConstraint {
    pub fn new(required_output_index: f64, ai_productivity_gain: f64) -> Self {
        Self {
            baseline_productivity: 1.0,
            ai_productivity_gain: ai_productivity_gain.max(0.0),
            required_output_index: required_output_index.max(0.0),
        }
    }

    /// `A_t`: effective achievement capacity at a given work percentage, where
    /// `H_t = theta * H_full` and `H_full` is normalized to 1.
    pub fn capacity_at(&self, work_percentage: f64) -> f64 {
        work_percentage * self.baseline_productivity * (1.0 + self.ai_productivity_gain)
    }

    /// Whether the worker can still deliver the required output at this work
    /// percentage.
    pub fn is_satisfied_at(&self, work_percentage: f64) -> bool {
        self.capacity_at(work_percentage) >= self.required_output_index
    }

    /// The lowest work percentage that still meets the requirement — the
    /// answer to the §3a question. `None` when even full-time work with the
    /// declared AI gain cannot meet it, which is a genuine finding rather than
    /// an error: it means the assigned goals are infeasible as stated.
    pub fn minimum_viable_work_percentage(&self) -> Option<f64> {
        let capacity_at_full_time = self.capacity_at(1.0);
        if capacity_at_full_time < self.required_output_index {
            return None;
        }
        if self.required_output_index <= 0.0 {
            return Some(0.0);
        }
        // Capacity is linear in work percentage, so this inverts exactly.
        Some(
            (self.required_output_index
                / (self.baseline_productivity * (1.0 + self.ai_productivity_gain)))
                .clamp(0.0, 1.0),
        )
    }

    /// AI gain required to make a given work percentage viable, which is the
    /// "can AI justify 80%?" question from §1.2.
    pub fn required_ai_gain_for(&self, work_percentage: f64) -> Option<f64> {
        if work_percentage <= 0.0 {
            return None;
        }
        let needed = self.required_output_index / (work_percentage * self.baseline_productivity);
        if needed <= 1.0 {
            Some(0.0)
        } else {
            Some(needed - 1.0)
        }
    }
}

/// Outcome of the employer-side constraint at one work percentage.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AchievementStatus {
    pub capacity: f64,
    pub required_output: f64,
    pub satisfied: bool,
    /// Capacity surplus or shortfall against the requirement.
    pub margin: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementAdequacy {
    pub retirement_income_monthly: f64,         // After-tax, annuity + lump-sum withdrawal
    pub self_sustaining_monthly: f64,           // Personal expenses only (no education support)
    pub total_required_monthly: f64,            // Personal + education support
    pub monthly_surplus_deficit: f64,           // Positive = surplus
    pub sustainable_years: f64,                 // Years until savings depleted (4% rule)
    pub education_support_years: f64,           // Years with active education obligations
    pub probability_success_to_95: f64,         // Monte Carlo: % of paths with positive balance at 95
    pub probability_el_required: f64,           // Monte Carlo: % of paths where EL supplement needed
    pub summary: String,                        // Human-readable adequacy assessment
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkScenario {
    pub work_percentage: f64,
    pub gross_income: f64,
    pub after_tax_income: f64,
    pub monthly_after_tax: f64,
    pub tax_only_rate: f64,
    pub effective_tax_rate: f64,
    pub work_hours_per_week: f64,
    pub free_hours_per_week: f64,
    /// `true` when net income covers the mandatory consumption floor.
    /// This is a financial-feasibility statement only; see
    /// [`WorkScenario::achievement_satisfied`] for job-security feasibility.
    pub meets_requirements: bool,
    pub surplus_deficit: f64,
    /// Consumption split by elasticity tier, so the display can show where the
    /// budget squeeze is actually landing rather than one aggregate figure.
    pub consumption_tiers: ConsumptionTiers,
    /// The mandatory floor that `meets_requirements` was tested against.
    pub mandatory_monthly: f64,
    /// The full lifestyle-inclusive basket at this household's sparing settings.
    pub target_monthly: f64,
    /// Employer-side achievement-capacity outcome. `None` when the caller did
    /// not supply a required output index, in which case the constraint is not
    /// applied and cannot make a scenario infeasible.
    pub achievement: Option<AchievementStatus>,
    pub utility_score: f64,
    pub utility_breakdown: UtilityBreakdown,
    pub retirement_adequacy: Option<RetirementAdequacy>,  // NEW: family support analysis
}

impl WorkScenario {
    /// Whether the employer-side achievement constraint is met. Vacuously true
    /// when no constraint was supplied.
    pub fn achievement_satisfied(&self) -> bool {
        self.achievement.map(|a| a.satisfied).unwrap_or(true)
    }

    /// Whether this scenario is feasible on *both* counts: the household can
    /// afford it and the employer's required output is still delivered.
    ///
    /// `find_optimal`/`search_outcome` use this, not `meets_requirements`, so a
    /// work percentage the person could not hold down is never recommended.
    pub fn is_feasible(&self) -> bool {
        self.meets_requirements && self.achievement_satisfied()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityBreakdown {
    pub consumption_utility: f64,
    pub leisure_utility: f64,
    pub family_utility: f64,
    pub health_utility: f64,
    pub security_utility: f64,
    pub total: f64,
}

/// Result of a work-percentage search, including whether any candidate was
/// actually affordable for the person.
#[derive(Debug, Clone)]
pub struct SearchOutcome {
    /// Best candidate: highest utility among feasible ones, or the smallest
    /// shortfall when nothing is feasible.
    pub scenario: WorkScenario,
    /// `true` when `scenario` meets the person's requirements. `false` means
    /// every candidate fell short and `scenario` is a least-bad fallback.
    pub feasible_found: bool,
    /// All evaluated candidates, sorted by utility descending.
    pub all_scenarios: Vec<WorkScenario>,
}

impl SearchOutcome {
    /// Why no candidate was feasible, when none was. Distinguishing "cannot
    /// afford it" from "the employer's goals cannot be met" matters because the
    /// remedies are completely different: one is a budget problem, the other is
    /// a workload or AI-productivity problem.
    pub fn infeasibility_reason(&self) -> Option<InfeasibilityReason> {
        if self.feasible_found {
            return None;
        }
        let any_affordable = self.all_scenarios.iter().any(|s| s.meets_requirements);
        let any_achievable = self.all_scenarios.iter().any(|s| s.achievement_satisfied());
        Some(match (any_affordable, any_achievable) {
            (false, false) => InfeasibilityReason::Both,
            (false, true) => InfeasibilityReason::Unaffordable,
            (true, false) => InfeasibilityReason::AchievementUnreachable,
            // Unreachable in practice: if both held for some candidate it would
            // have been feasible. Kept explicit rather than panicking.
            (true, true) => InfeasibilityReason::Both,
        })
    }
}

/// Which constraint eliminated every candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfeasibilityReason {
    /// Net income never covers the mandatory floor at any candidate.
    Unaffordable,
    /// Employer's required output is unreachable even at full-time work.
    AchievementUnreachable,
    /// Both constraints fail.
    Both,
}

#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    pub full_time_salary: f64,
    pub full_time_hours: f64,
    pub tax_schedule: TaxSchedule,
    pub requirements: PersonalRequirements,
    pub life_stage: LifeStage,
    pub preferences: PreferenceWeights,
    pub current_age: u32,
    pub retirement_age: u32,
    pub discount_rate: f64,  // Time preference
    /// Consumption-side parameters: lifestyle profile, sparing ratio,
    /// utilization discipline, and the quasi-inelastic share.
    pub consumption: ConsumptionProfileConfig,
    /// Optional employer-side achievement constraint (§5.2). When `None`, the
    /// constraint is not applied and only financial feasibility is checked.
    pub achievement: Option<AchievementConstraint>,
    /// Conversion rate scenario used by the security-utility term, so the
    /// optimizer's pension estimate and the Monte Carlo projection do not
    /// disagree about which Umwandlungssatz is being assumed.
    pub conversion_scenario: crate::monte_carlo::ConversionRateScenario,
}

impl OptimizerConfig {
    pub fn new(
        full_time_salary: f64,
        tax_schedule: TaxSchedule,
        requirements: PersonalRequirements,
        life_stage: LifeStage,
        preferences: PreferenceWeights,
    ) -> Self {
        let current_age = life_stage.age();
        Self {
            full_time_salary,
            full_time_hours: 42.0,  // Swiss standard
            tax_schedule,
            requirements,
            life_stage,
            preferences,
            current_age,
            retirement_age: 65,
            discount_rate: 0.03,
            consumption: ConsumptionProfileConfig::default(),
            achievement: None,
            conversion_scenario: crate::monte_carlo::ConversionRateScenario::Statutory,
        }
    }
}

pub struct LifeOptimizer {
    config: OptimizerConfig,
}

impl LifeOptimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        Self { config }
    }

    /// Evaluate a specific work percentage scenario
    pub fn evaluate_scenario(&self, work_percentage: f64) -> WorkScenario {
        // Calculate income
        let gross_income = self.config.full_time_salary * work_percentage;
        let after_tax_income = self.config.tax_schedule.after_tax_income(gross_income);
        let monthly_after_tax = after_tax_income / 12.0;
        let tax_only_rate = self.config.tax_schedule.tax_only_rate(gross_income);
        let effective_tax_rate = self.config.tax_schedule.effective_tax_rate(gross_income);

        // Calculate time allocation
        let work_hours = self.config.full_time_hours * work_percentage;
        let sleep_hours = 8.0 * 7.0;  // 8 hours/day
        let free_hours = 168.0 - work_hours - sleep_hours;

        // Consumption, split by elasticity tier. Feasibility is tested against
        // the mandatory floor, not the full lifestyle basket: a household is
        // not "unable to afford" reduced work merely because it would have to
        // trim discretionary spending. The full basket is still reported as a
        // target, and drives the consumption-utility ratio.
        let requirements = self.config.requirements.adjusted_for_life_stage(&self.config.life_stage);
        let tiers = requirements.elasticity_tiers(&self.config.consumption);
        let mandatory_monthly = tiers.mandatory_monthly();
        let target_monthly = tiers.lifestyle_target_monthly();

        let meets_requirements = monthly_after_tax >= mandatory_monthly;
        let surplus_deficit = monthly_after_tax - mandatory_monthly;

        // Employer-side achievement capacity (§5.2). Absent constraint means
        // the scenario cannot be ruled out on job-security grounds.
        let achievement = self.config.achievement.map(|c| {
            let capacity = c.capacity_at(work_percentage);
            AchievementStatus {
                capacity,
                required_output: c.required_output_index,
                satisfied: capacity >= c.required_output_index,
                margin: capacity - c.required_output_index,
            }
        });

        // Calculate utility components
        let utility_breakdown = self.calculate_utility(
            after_tax_income,
            work_hours,
            free_hours,
            target_monthly,
            meets_requirements,
        );

        WorkScenario {
            work_percentage,
            gross_income,
            after_tax_income,
            monthly_after_tax,
            tax_only_rate,
            effective_tax_rate,
            work_hours_per_week: work_hours,
            free_hours_per_week: free_hours,
            meets_requirements,
            surplus_deficit,
            consumption_tiers: tiers,
            mandatory_monthly,
            target_monthly,
            achievement,
            utility_score: utility_breakdown.total,
            utility_breakdown,
            retirement_adequacy: None,
        }
    }

    /// Calculate utility score for a scenario
    fn calculate_utility(
        &self,
        after_tax_income: f64,
        work_hours: f64,
        free_hours: f64,
        requirements: f64,
        meets_requirements: bool,
    ) -> UtilityBreakdown {
        let prefs = &self.config.preferences;
        let stage = &self.config.life_stage;

        // 1. Consumption utility (log utility with diminishing returns)
        let consumption_ratio = (after_tax_income / 12.0) / requirements;
        let consumption_utility = if meets_requirements {
            prefs.consumption * consumption_ratio.ln()
        } else {
            // Heavy penalty if requirements not met
            prefs.consumption * (consumption_ratio.ln() - 5.0)
        };

        // 2. Leisure utility (concave, diminishing returns)
        let leisure_utility = prefs.leisure * (free_hours / 80.0).powf(0.7) * 10.0;

        // 3. Family utility (depends on life stage)
        let family_time_value = stage.time_value_factor();
        let family_utility = prefs.family * (free_hours / 80.0).powf(0.8) * family_time_value * 10.0;

        // 4. Health utility (stress penalty - convex, increases sharply with overwork)
        let stress_factor = (work_hours / 42.0).powf(2.0);
        let stress_tolerance = stage.stress_tolerance();
        let health_penalty = stress_factor / stress_tolerance;
        let health_utility = prefs.health * (10.0 - health_penalty * 5.0);

        // 5. Security utility (pension and savings)
        let pension_value = self.calculate_pension_adequacy(after_tax_income);
        let security_utility = prefs.security * pension_value;

        let total = consumption_utility + leisure_utility + family_utility + health_utility + security_utility;

        UtilityBreakdown {
            consumption_utility,
            leisure_utility,
            family_utility,
            health_utility,
            security_utility,
            total,
        }
    }

    /// Calculate pension adequacy score
    fn calculate_pension_adequacy(&self, annual_income: f64) -> f64 {
        // Saturating subtraction: a caller may legitimately evaluate a person
        // who is already at or past retirement age, and `u32` subtraction would
        // otherwise panic with an arithmetic underflow.
        let years_to_retirement =
            self.config.retirement_age.saturating_sub(self.config.current_age) as f64;
        if years_to_retirement <= 0.0 {
            // No further accumulation: only AHV is available to this person.
            let ahv_expected = annual_income * 0.30;
            let replacement_rate = ahv_expected / annual_income;
            return ((replacement_rate / 0.60) * 10.0).min(10.0);
        }

        // Swiss pension system (simplified)
        // AHV (1st pillar): ~30% of average income
        // BVG (2nd pillar): depends on contributions
        
        let ahv_expected = annual_income * 0.30;
        let bvg_contribution_rate = 0.083;  // Rough estimate
        let bvg_annual = annual_income * bvg_contribution_rate;
        
        // Project BVG savings (simplified)
        let compound_rate: f64 = 1.02;  // 2% annual return
        let bvg_total = bvg_annual * ((compound_rate.powf(years_to_retirement) - 1.0) / (compound_rate - 1.0));
        // Use the same conversion-rate scenario as the Monte Carlo projection
        // rather than an unconditional 6.8%, so the two engines agree.
        let retirement_year = crate::monte_carlo::retirement_year_from(
            self.config.current_age,
            self.config.retirement_age,
        );
        let conversion_rate = crate::monte_carlo::effective_conversion_rate(
            self.config.conversion_scenario,
            self.config.retirement_age,
            retirement_year,
        );
        let bvg_annual_pension = bvg_total * conversion_rate;
        
        let total_pension = ahv_expected + bvg_annual_pension;
        let current_income = annual_income;
        
        // Pension replacement rate
        let replacement_rate = total_pension / current_income;
        
        // Score: aim for 60-80% replacement
        let score = if replacement_rate >= 0.60 {
            10.0
        } else {
            (replacement_rate / 0.60) * 10.0
        };
        
        score.min(10.0)
    }

    /// Find optimal work percentage using grid search.
    ///
    /// Returns the chosen scenario. Use [`Self::search_outcome`] when the caller
    /// needs to distinguish a genuinely feasible optimum from the best-effort
    /// fallback returned when *no* candidate meets the person's requirements —
    /// conflating the two produces advice like "optimal: 50%" printed next to
    /// "below requirements".
    pub fn find_optimal(&self, candidates: &[f64]) -> (WorkScenario, Vec<WorkScenario>) {
        let outcome = self
            .search_outcome(candidates)
            .expect("find_optimal requires at least one candidate work percentage");
        (outcome.scenario, outcome.all_scenarios)
    }

    /// Search for the best work percentage, reporting whether any candidate was
    /// actually feasible. Returns `None` when `candidates` is empty.
    pub fn search_outcome(&self, candidates: &[f64]) -> Option<SearchOutcome> {
        if candidates.is_empty() {
            return None;
        }

        let mut scenarios: Vec<WorkScenario> = candidates
            .iter()
            .map(|&pct| self.evaluate_scenario(pct))
            .collect();

        let feasible: Vec<&WorkScenario> = scenarios
            .iter()
            .filter(|s| s.is_feasible())
            .collect();

        // Among feasible candidates, the highest utility wins. When none is
        // feasible, fall back to the smallest shortfall rather than the highest
        // utility: with no feasible option the useful advice is "here is the
        // least-bad choice", and utility alone can favour deep under-employment
        // because leisure and health scores rise as income falls.
        let (chosen, feasible_found) = if feasible.is_empty() {
            let least_bad = scenarios
                .iter()
                .max_by(|a, b| {
                    a.surplus_deficit
                        .partial_cmp(&b.surplus_deficit)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })?;
            (least_bad.clone(), false)
        } else {
            let best = feasible
                .iter()
                .max_by(|a, b| {
                    a.utility_score
                        .partial_cmp(&b.utility_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })?;
            ((*best).clone(), true)
        };

        scenarios.sort_by(|a, b| {
            b.utility_score
                .partial_cmp(&a.utility_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Some(SearchOutcome {
            scenario: chosen,
            feasible_found,
            all_scenarios: scenarios,
        })
    }

    /// Calculate lifetime utility for a given work percentage
    pub fn calculate_lifetime_utility(&self, work_percentage: f64) -> f64 {
        let mut total_utility = 0.0;
        let years = self.config.retirement_age.saturating_sub(self.config.current_age) as usize;

        for year in 0..years {
            let age = self.config.current_age + year as u32;
            let discount_factor = (1.0 / (1.0 + self.config.discount_rate)).powi(year as i32);

            // Simulate life stage progression (simplified)
            let stage = self.simulate_life_stage_at_age(age);
            let requirements = self.config.requirements.adjusted_for_life_stage(&stage);
            let mandatory = requirements.mandatory_monthly(&self.config.consumption);

            let scenario = self.evaluate_scenario(work_percentage);
            let period_utility = if scenario.monthly_after_tax >= mandatory {
                scenario.utility_score
            } else {
                scenario.utility_score - 10.0  // Heavy penalty for not meeting needs
            };

            total_utility += discount_factor * period_utility;
        }

        // Add terminal pension value
        let final_scenario = self.evaluate_scenario(work_percentage);
        let pension_value = self.calculate_pension_adequacy(final_scenario.after_tax_income);
        total_utility += pension_value * 5.0;  // Weight pension heavily

        total_utility
    }

    /// Simulate life stage at a given age (simplified)
    fn simulate_life_stage_at_age(&self, age: u32) -> LifeStage {
        let children = self.config.life_stage.children_count();
        
        if children > 0 {
            let child_age = age - self.config.current_age;
            if child_age < 6 {
                LifeStage::NewParent { age, children }
            } else if child_age < 13 {
                LifeStage::SchoolAge { age, children }
            } else if child_age < 18 {
                LifeStage::Teenagers { age, children }
            } else {
                LifeStage::EmptyNest { age }
            }
        } else {
            if age < 35 {
                LifeStage::YoungSingle { age }
            } else if age < 60 {
                LifeStage::YoungCouple { age, dual_income: false }
            } else {
                LifeStage::PreRetirement { age }
            }
        }
    }

    /// Find optimal lifetime strategy (may vary by age)
    pub fn find_optimal_lifetime_strategy(&self) -> Vec<(u32, f64)> {
        let candidates = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let years = (self.config.retirement_age - self.config.current_age) as usize;
        let mut strategy = Vec::new();

        for year in 0..years {
            let age = self.config.current_age + year as u32;
            
            // Find best work percentage for this specific year
            let mut best_pct = 1.0;
            let mut best_utility = f64::NEG_INFINITY;

            for &pct in &candidates {
                let utility = self.calculate_year_utility(age, pct);
                if utility > best_utility {
                    best_utility = utility;
                    best_pct = pct;
                }
            }

            strategy.push((age, best_pct));
        }

        strategy
    }

    fn calculate_year_utility(&self, age: u32, work_percentage: f64) -> f64 {
        let stage = self.simulate_life_stage_at_age(age);
        let requirements = self.config.requirements.adjusted_for_life_stage(&stage);
        let mandatory = requirements.mandatory_monthly(&self.config.consumption);
        let scenario = self.evaluate_scenario(work_percentage);

        // Both constraints must hold for a year to count as feasible: the
        // household must afford it and the employer's goals must still be met.
        if scenario.monthly_after_tax >= mandatory && scenario.achievement_satisfied() {
            scenario.utility_score
        } else {
            scenario.utility_score - 10.0
        }
    }

    /// Calculate retirement adequacy for a work scenario with family support obligations
    pub fn calculate_retirement_adequacy(
        &self,
        _work_percentage: f64,
        pension_balance_at_65: f64,
        family_support: &FamilySupport,
        life_expectancy: u32,
    ) -> RetirementAdequacy {
        // 1. Compute monthly withdrawal using 4% rule
        let monthly_withdrawal = (pension_balance_at_65 * 0.04) / 12.0;

        // 2. Add estimated AHV (state pension, age-dependent)
        let monthly_ahv = self.estimate_ahv_monthly(self.config.retirement_age);
        let retirement_income_monthly = monthly_withdrawal + monthly_ahv;

        // 3. Compute required personal expenses
        let req_adjusted = self
            .config
            .requirements
            .adjusted_for_life_stage(&self.config.life_stage);
        let self_sustaining = req_adjusted.total_monthly();

        // 4. Add education support obligations
        let education_support = family_support.total_monthly_education_support_at_retirement(
            self.config.current_age,
            self.config.retirement_age,
        );
        let total_required = self_sustaining + education_support;

        // 5. Compute sustainability
        let monthly_surplus = retirement_income_monthly - total_required;
        let sustainable_years = if monthly_surplus < 0.0 {
            // Burning down capital; estimate years until depletion
            (pension_balance_at_65 / (monthly_surplus.abs() * 12.0)).max(0.0)
        } else {
            // Surplus; sustainable indefinitely with 4% rule
            (life_expectancy - self.config.retirement_age) as f64
        };

        let education_years = family_support
            .education_support_by_year(
                self.config.current_age,
                self.config.retirement_age,
                life_expectancy,
            )
            .len() as f64;

        // 6. Generate summary
        let summary = if monthly_surplus > 0.0 {
            format!(
                "Strong: CHF {}/month surplus. Education support secured.",
                (monthly_surplus * 100.0).round() / 100.0
            )
        } else if sustainable_years > (life_expectancy - self.config.retirement_age) as f64 * 0.8 {
            format!(
                "Adequate: Capital drawdown sustainable to age ~{}.",
                (self.config.retirement_age as f64 + sustainable_years) as u32
            )
        } else {
            format!(
                "Constrained: Capital depleted by age ~{}. EL support likely needed.",
                (self.config.retirement_age as f64 + sustainable_years) as u32
            )
        };

        RetirementAdequacy {
            retirement_income_monthly,
            self_sustaining_monthly: self_sustaining,
            total_required_monthly: total_required,
            monthly_surplus_deficit: monthly_surplus,
            sustainable_years,
            education_support_years: education_years,
            probability_success_to_95: 0.0,  // Set by Monte Carlo if available
            probability_el_required: 0.0,    // Set by Monte Carlo if available
            summary,
        }
    }

    /// Estimate AHV monthly pension based on retirement age
    fn estimate_ahv_monthly(&self, retirement_age: u32) -> f64 {
        // Simplified AHV estimate (2024: CHF 1,225–2,450/month for single)
        match retirement_age {
            65 => 1850.0,                          // Standard retirement, full pension
            66..=70 => 1850.0 * (1.0 + (retirement_age - 65) as f64 * 0.052), // Deferred retirement bonus
            _ => 1850.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_separates_tax_only_and_total_deductions() {
        let schedule = TaxSchedule::bern_city_default(false, 0);
        let requirements = PersonalRequirements::bern_family_default(0);
        let life_stage = LifeStage::YoungSingle { age: 45 };
        let preferences = PreferenceWeights::balanced();
        let optimizer = LifeOptimizer::new(OptimizerConfig {
            full_time_salary: 140_000.0,
            full_time_hours: 42.0,
            tax_schedule: schedule.clone(),
            requirements,
            life_stage,
            preferences,
            current_age: 45,
            retirement_age: 65,
            discount_rate: 0.03,
            consumption: ConsumptionProfileConfig::default(),
            achievement: None,
            conversion_scenario: crate::monte_carlo::ConversionRateScenario::Statutory,
        });

        let scenario = optimizer.evaluate_scenario(1.0);

        assert!((scenario.tax_only_rate - schedule.tax_only_rate(140_000.0)).abs() < 1e-9,
                "tax-only rate should match the official Bern table");
        assert!(scenario.effective_tax_rate < schedule.tax_only_rate(140_000.0)
                    + schedule.social_security_rate
                    + schedule.unemployment_rate
                    + schedule.pension_rate,
                "effective rate should be lower than the official table plus payroll charges when standard deductions are applied");
        assert!(scenario.effective_tax_rate > schedule.social_security_rate
                    + schedule.unemployment_rate
                    + schedule.pension_rate,
                "effective rate should still include the mandatory social contributions");
    }
}

