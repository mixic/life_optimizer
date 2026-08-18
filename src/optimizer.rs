// Core optimization module
#![allow(dead_code)]
use crate::requirements::{LifeStage, PersonalRequirements, PreferenceWeights, FamilySupport};
use crate::tax::TaxSchedule;
use serde::{Deserialize, Serialize};

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
    pub meets_requirements: bool,
    pub surplus_deficit: f64,
    pub utility_score: f64,
    pub utility_breakdown: UtilityBreakdown,
    pub retirement_adequacy: Option<RetirementAdequacy>,  // NEW: family support analysis
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

        // Check if requirements are met
        let requirements = self.config.requirements.adjusted_for_life_stage(&self.config.life_stage);
        let monthly_requirements = requirements.total_monthly();
        let meets_requirements = monthly_after_tax >= monthly_requirements;
        let surplus_deficit = monthly_after_tax - monthly_requirements;

        // Calculate utility components
        let utility_breakdown = self.calculate_utility(
            after_tax_income,
            work_hours,
            free_hours,
            monthly_requirements,
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
        let years_to_retirement = (self.config.retirement_age - self.config.current_age) as f64;
        
        // Swiss pension system (simplified)
        // AHV (1st pillar): ~30% of average income
        // BVG (2nd pillar): depends on contributions
        
        let ahv_expected = annual_income * 0.30;
        let bvg_contribution_rate = 0.083;  // Rough estimate
        let bvg_annual = annual_income * bvg_contribution_rate;
        
        // Project BVG savings (simplified)
        let compound_rate: f64 = 1.02;  // 2% annual return
        let bvg_total = bvg_annual * ((compound_rate.powf(years_to_retirement) - 1.0) / (compound_rate - 1.0));
        let bvg_annual_pension = bvg_total * 0.068;  // 6.8% conversion rate
        
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

    /// Find optimal work percentage using grid search
    pub fn find_optimal(&self, candidates: &[f64]) -> (WorkScenario, Vec<WorkScenario>) {
        let mut scenarios: Vec<WorkScenario> = candidates
            .iter()
            .map(|&pct| self.evaluate_scenario(pct))
            .collect();

        // Filter feasible solutions (must meet requirements)
        let feasible: Vec<_> = scenarios.iter()
            .filter(|s| s.meets_requirements)
            .cloned()
            .collect();

        let optimal = if feasible.is_empty() {
            // No feasible solution - return best effort
            scenarios.iter()
                .max_by(|a, b| a.utility_score.partial_cmp(&b.utility_score).unwrap())
                .unwrap()
                .clone()
        } else {
            // Return feasible solution with highest utility
            feasible.iter()
                .max_by(|a, b| a.utility_score.partial_cmp(&b.utility_score).unwrap())
                .unwrap()
                .clone()
        };

        scenarios.sort_by(|a, b| b.utility_score.partial_cmp(&a.utility_score).unwrap());
        (optimal, scenarios)
    }

    /// Calculate lifetime utility for a given work percentage
    pub fn calculate_lifetime_utility(&self, work_percentage: f64) -> f64 {
        let mut total_utility = 0.0;
        let years = (self.config.retirement_age - self.config.current_age) as usize;

        for year in 0..years {
            let age = self.config.current_age + year as u32;
            let discount_factor = (1.0 / (1.0 + self.config.discount_rate)).powi(year as i32);
            
            // Simulate life stage progression (simplified)
            let stage = self.simulate_life_stage_at_age(age);
            let requirements = self.config.requirements.adjusted_for_life_stage(&stage);
            
            let scenario = self.evaluate_scenario(work_percentage);
            let period_utility = if scenario.monthly_after_tax >= requirements.total_monthly() {
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
        let scenario = self.evaluate_scenario(work_percentage);
        
        if scenario.monthly_after_tax >= requirements.total_monthly() {
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
