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

// Display module for pretty printing results
use crate::optimizer::WorkScenario;
use crate::tax::TaxSchedule;
use colored::*;
use tabled::{Table, Tabled, settings::Style};

#[derive(Tabled)]
struct ScenarioRow {
    #[tabled(rename = "Work %")]
    work_pct: String,
    #[tabled(rename = "Gross (CHF)")]
    gross: String,
    #[tabled(rename = "Tax Rate")]
    tax_rate: String,
    #[tabled(rename = "Net/Month")]
    net_monthly: String,
    #[tabled(rename = "Work h/w")]
    work_hours: String,
    #[tabled(rename = "Free h/w")]
    free_hours: String,
    #[tabled(rename = "Feasible")]
    feasible: String,
    #[tabled(rename = "Surplus")]
    surplus: String,
    #[tabled(rename = "Utility")]
    utility: String,
}

pub fn print_optimal_result(scenario: &WorkScenario) {
    print_optimal_result_for(scenario, None)
}

/// As [`print_optimal_result`], but naming the tax basis.
///
/// The tax-rate line used to read "(official Bern tax only)" unconditionally,
/// which was false for every other canton once the two-level model began
/// pricing them. The basis is now supplied by the caller, so the label cannot
/// misdescribe the figure beside it.
pub fn print_optimal_result_for(scenario: &WorkScenario, tax_basis: Option<&str>) {
    // Only call it "optimal" when it is actually affordable *and* the
    // employer's required output is still delivered. Announcing an optimum next
    // to "below requirements" is the kind of self-contradicting output that
    // makes a tool untrustworthy.
    if scenario.is_feasible() {
        println!("\n{}", "🎯 OPTIMAL SOLUTION FOUND!".bold().green());
    } else if !scenario.meets_requirements {
        println!("\n{}", "⚠  NO AFFORDABLE OPTION AT ANY WORK PERCENTAGE".bold().yellow());
        println!("{}", "   Showing the closest option, not a recommendation.".yellow());
    } else {
        println!("\n{}", "⚠  NO OPTION MEETS THE REQUIRED OUTPUT".bold().yellow());
        println!("{}", "   Financially affordable, but the employer's goals are not deliverable.".yellow());
    }
    println!("{}", "=".repeat(60));
    
    println!("\n{}", "Work Configuration:".bold());
    println!("  Work Percentage: {}", format!("{:.0}%", scenario.work_percentage * 100.0).cyan().bold());
    println!("  Gross Income:    {} CHF/year", format!("{:.0}", scenario.gross_income).cyan());
    println!("  After-Tax:       {} CHF/year", format!("{:.0}", scenario.after_tax_income).cyan());
    println!("  Monthly Net:     {} CHF/month", format!("{:.0}", scenario.monthly_after_tax).cyan().bold());
    println!("  Tax Rate:        {:.1}%{}", scenario.tax_only_rate * 100.0,
        match tax_basis {
            Some(basis) => format!(" ({basis})"),
            None => String::new(),
        });
    println!("  Social Security: 12.9% (AHV/IV/EO/ALV/BVG)");
    println!("  Total Deduction: {}", format!("{:.1}%", scenario.effective_tax_rate * 100.0).yellow());

    println!("\n{}", "Time Allocation:".bold());
    println!("  Work Hours:      {:.1} hours/week", scenario.work_hours_per_week);
    println!("  Free Hours:      {:.1} hours/week", scenario.free_hours_per_week);
    println!("  Work Days:       {:.1} days/week", scenario.work_hours_per_week / 8.4);

    println!("\n{}", "Financial Health:".bold());
    if scenario.meets_requirements {
        println!("  Status:          {} ✓", "MEETS ALL REQUIREMENTS".green().bold());
        println!("  Monthly Surplus: {} CHF", format!("+{:.0}", scenario.surplus_deficit).green());
    } else {
        println!("  Status:          {} ✗", "BELOW REQUIREMENTS".red().bold());
        println!("  Monthly Deficit: {} CHF", format!("{:.0}", scenario.surplus_deficit).red());
    }
    println!("  {}", format!(
        "  (tested against the mandatory floor of CHF {:.0}/month, not the full lifestyle basket)",
        scenario.mandatory_monthly
    ).dimmed());

    // ── Employer-side achievement capacity (§5.2) ────────────────────────────
    if let Some(a) = scenario.achievement {
        println!("\n{}", "Employer Achievement Capacity:".bold());
        println!("  Capacity (A):    {:.2}", a.capacity);
        println!("  Required (G):    {:.2}", a.required_output);
        if a.satisfied {
            println!("  Status:          {} (margin {:+.2})",
                "MEETS REQUIRED OUTPUT ✓".green().bold(), a.margin);
        } else {
            println!("  Status:          {} (shortfall {:.2})",
                "BELOW REQUIRED OUTPUT ✗".red().bold(), -a.margin);
            println!("  {}", "  Note: a reduced work percentage is not credible here — the".dimmed());
            println!("  {}", "  organisation's goals would not be delivered.".dimmed());
        }
    }

    print_consumption_breakdown(scenario);

    println!("\n{}", "Utility Score Breakdown:".bold());
    let breakdown = &scenario.utility_breakdown;
    println!("  Consumption:     {:.2}", breakdown.consumption_utility);
    println!("  Leisure:         {:.2}", breakdown.leisure_utility);
    println!("  Family:          {:.2}", breakdown.family_utility);
    println!("  Health:          {:.2}", breakdown.health_utility);
    println!("  Security:        {:.2}", breakdown.security_utility);
    // Note: precision specifiers have no effect on String, so the value must be
    // formatted before being coloured.
    println!("  {} {}", "TOTAL UTILITY:".bold(), format!("{:.2}", breakdown.total).cyan().bold());

    println!("\n{}", "=".repeat(60));
}

/// Show where the budget squeeze actually lands, by elasticity tier.
///
/// `THEORY_OF_SPARING.md` §7c: a household with rising inelastic costs can
/// practice maximum sparing discipline on the elastic tier and still see little
/// movement in total consumption. Reporting only an aggregate "requirements"
/// number hides which tier is under pressure.
pub fn print_consumption_breakdown(scenario: &WorkScenario) {
    let t = &scenario.consumption_tiers;
    println!("\n{}", "Consumption by Elasticity Tier:".bold());
    println!("  {:<38} {:>10}", "Inelastic (non-reducible)", format!("CHF {:.0}", t.inelastic).cyan());
    println!("  {:<38} {:>10}", "Quasi-inelastic (locked in)", format!("CHF {:.0}", t.quasi_inelastic).cyan());
    println!("  {:<38} {:>10}", "Elastic (sparing-eligible)", format!("CHF {:.0}", t.elastic).cyan());
    println!("  {:<38} {:>10}", "Committed outflows (savings, vacation)", format!("CHF {:.0}", t.committed_outflows).cyan());
    println!("  {}", "─".repeat(49));
    println!("  {:<38} {:>10}", "Mandatory floor".bold(), format!("CHF {:.0}", scenario.mandatory_monthly).bold());
    println!("  {:<38} {:>10}", "Full lifestyle basket".bold(), format!("CHF {:.0}", scenario.target_monthly).bold());

    if t.applied_multiplier < 1.0 {
        let saved = t.unadjusted_discretionary() * (1.0 - t.applied_multiplier);
        println!("\n  {}", format!(
            "Sparing multiplier {:.3} applied to discretionary spending.",
            t.applied_multiplier
        ).green());
        if saved > 0.5 {
            println!("  {}", format!(
                "This reduces monthly discretionary cost by CHF {:.0}.", saved
            ).green());
        }
    } else if t.applied_multiplier > 1.0 {
        println!("\n  {}", format!(
            "Lifestyle/utilisation multiplier {:.3} raises effective discretionary cost.",
            t.applied_multiplier
        ).yellow());
        println!("  {}", "  Low utilisation inflates the real cost of what you buy, even at a discount.".dimmed());
    }

    // The insight the tier split exists to surface.
    let total = t.lifestyle_target_monthly();
    if total > 0.0 {
        let inelastic_share = (t.inelastic + t.quasi_inelastic) / total * 100.0;
        if inelastic_share >= 75.0 {
            println!("  {}", format!(
                "Note: {:.0}% of your spending is non-reducible — sparing cannot move this much.",
                inelastic_share
            ).yellow());
        }
    }
}

pub fn print_tax_deduction_breakdown(tax_schedule: &TaxSchedule, gross_income: f64) {
    let deduction = tax_schedule.deduction_breakdown(gross_income);

    println!("\n{}", "💸 TAX DEDUCTION BREAKDOWN".bold().cyan());
    println!("  Gross income: CHF {:.0}", gross_income);
    println!("  Deductible items:");
    println!("    • Childcare:         CHF {:.0}", deduction.childcare);
    println!("    • Commuting:         CHF {:.0}", deduction.commuting);
    println!("    • Work equipment:    CHF {:.0}", deduction.work_equipment);
    println!("    • Health insurance:  CHF {:.0}", deduction.health_insurance);
    println!("    • Rent/apartment:    CHF {:.0}", deduction.rent);
    println!("    • Family-specific:   CHF {:.0}", deduction.family_specific);
    println!("    • Total deductible:  CHF {:.0}", deduction.deductible_total);
    // `non_deductible_total` is deliberately NOT printed. It is 2% of income after
    // the items above, it feeds no tax calculation, and sitting between "total
    // deductible" and "taxable income" it read as part of that arithmetic — which
    // it is not, since taxable income is gross minus `deductible_total` alone.
    println!("  Taxable income after deductions: CHF {:.0}", tax_schedule.taxable_income_after_estimated_deductions(gross_income));
    println!(
        "  {}",
        "  Estimate only — these components are hand-entered, not sourced. The \
         sourced rules are compared below."
            .yellow()
    );
}

pub fn print_comparison_table(scenarios: &[WorkScenario]) {
    print_comparison_table_for(scenarios, None)
}

/// As [`print_comparison_table`], but naming the basis the tax figures came from.
///
/// The breakdown note used to claim "official Stadt Bern rates" unconditionally,
/// which was wrong for every other canton and — worse — was printed next to a
/// `--canton` the caller had just asked for.
pub fn print_comparison_table_for(scenarios: &[WorkScenario], tax_basis: Option<&str>) {
    println!("\n{}", "📊 SCENARIO COMPARISON".bold().blue());
    println!("{}", "=".repeat(60));

    let rows: Vec<ScenarioRow> = scenarios
        .iter()
        .map(|s| {
            let feasible = if s.meets_requirements {
                "✓".green().to_string()
            } else {
                "✗".red().to_string()
            };

            let surplus_color = if s.surplus_deficit >= 0.0 {
                format!("{:+.0}", s.surplus_deficit).green().to_string()
            } else {
                format!("{:.0}", s.surplus_deficit).red().to_string()
            };

            ScenarioRow {
                work_pct: format!("{:.0}%", s.work_percentage * 100.0),
                gross: format!("{:.0}k", s.gross_income / 1000.0),
                tax_rate: format!("{:.1}%", s.effective_tax_rate * 100.0),
                net_monthly: format!("{:.0}", s.monthly_after_tax),
                work_hours: format!("{:.1}", s.work_hours_per_week),
                free_hours: format!("{:.1}", s.free_hours_per_week),
                feasible,
                surplus: surplus_color,
                utility: format!("{:.2}", s.utility_score),
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::modern());
    println!("\n{}", table);
    
    // Add explanation
    let tax_line = match tax_basis {
        Some(basis) => format!("  • Kantons-, Gemeinde- und Kirchensteuer ({basis})"),
        None => "  • Kantons-, Gemeinde- und Kirchensteuer".to_string(),
    };
    println!("\n{}", "ℹ️  Tax Rate Breakdown:".bold().cyan());
    println!("The 'Tax Rate' column shows TOTAL deductions including:");
    println!("{tax_line}");
    println!("  • Social Security: AHV/IV/EO (5.3%) + ALV (1.1%) + BVG (~6.5%) = ~12.9%");
    println!("\nExample: 33% total = ~20% Steuer + ~13% Sozialversicherung");

}

pub fn print_recommendations(scenario: &WorkScenario, age: u32) {
    println!("\n{}", "💡 RECOMMENDATIONS".bold().magenta());
    println!("{}", "=".repeat(60));

    let work_pct = scenario.work_percentage;

    if work_pct >= 0.95 {
        println!("\n{}", "Full-time work (100%):".bold());
        println!("  ✓ Maximizes income and pension contributions");
        println!("  ✓ Best for career advancement");
        println!("  ⚠ Limited time for family and personal pursuits");
        println!("  ⚠ Higher stress and burnout risk");
        
        if age < 35 {
            println!("\n  {} At your age, full-time work can build a strong financial foundation.", "💼".yellow());
        } else if age > 45 {
            println!("\n  {} Consider reducing hours as you approach retirement for better health.", "🏥".yellow());
        }
    } else if work_pct >= 0.75 {
        println!("\n{}", "Part-time work (80%):".bold());
        println!("  ✓ Good balance between income and free time");
        println!("  ✓ One extra day off per week");
        println!("  ✓ Still decent pension contributions");
        println!("  ✓ Lower stress, better health outcomes");
        println!("\n  {} This is often the 'sweet spot' for work-life balance!", "⭐".green());
    } else {
        println!("\n{}", "Reduced work (60-70%):".bold());
        println!("  ✓ Maximum time flexibility");
        println!("  ✓ Ideal for family time, especially with young children");
        println!("  ✓ Lowest stress");
        println!("  ⚠ Reduced income - ensure you meet basic needs");
        println!("  ⚠ Lower pension contributions");
        
        if !scenario.meets_requirements {
            println!("\n  {} WARNING: This may not cover your requirements!", "⚠️".red());
            println!("  Consider: dual income, reducing expenses, or increasing work percentage.");
        }
    }

    // Tax optimization tip
    if scenario.effective_tax_rate > 0.25 {
        println!("\n{}", "💰 Tax Optimization Tips:".bold());
        println!("  • Your effective tax rate is {:.1}% - consider:", scenario.effective_tax_rate * 100.0);
        println!("    - Pillar 3a contributions (CHF 7,056/year deductible)");
        println!("    - Pillar 3b if self-employed");
        println!("    - Childcare costs are partially deductible");
        println!("    - Moving to a lower-tax canton (e.g., ZG, SZ, NW)");
    }

    // Life stage specific advice
    if age < 35 {
        println!("\n{}", "🌱 Early Career (< 35):".bold());
        println!("  • Focus on building skills and income");
        println!("  • Invest aggressively (longer time horizon)");
        println!("  • Consider 100% work to maximize career growth");
    } else if age >= 35 && age < 50 {
        println!("\n{}", "👨‍👩‍👧‍👦 Family Years (35-50):".bold());
        println!("  • Time with children is irreplaceable");
        println!("  • 80% work often optimal if financially feasible");
        println!("  • Partner coordination can enable dual 80% = 160% total");
    } else if age >= 50 {
        println!("\n{}", "🏖️ Pre-Retirement (50+):".bold());
        println!("  • Health becomes increasingly important");
        println!("  • Consider gradual reduction (90% → 80% → 70%)");
        println!("  • Ensure pension is on track for retirement");
        println!("  • More time for hobbies, travel, grandchildren");
    }

    println!("\n{}", "=".repeat(60));
}

pub fn print_lifetime_strategy(strategy: &[(u32, f64)]) {
    println!("\n{}", "📈 OPTIMAL LIFETIME STRATEGY".bold().cyan());
    println!("{}", "=".repeat(60));

    // Group by work percentage
    let mut current_pct = strategy[0].1;
    let mut current_start = strategy[0].0;

    println!("\n{}", "Recommended Work Schedule:".bold());

    for (i, &(age, pct)) in strategy.iter().enumerate() {
        if pct != current_pct || i == strategy.len() - 1 {
            let end_age = if i == strategy.len() - 1 { age } else { strategy[i - 1].0 };
            
            let period = if current_start == end_age {
                format!("Age {}", current_start)
            } else {
                format!("Ages {}-{}", current_start, end_age)
            };

            let work_desc = format!("{:.0}% work", current_pct * 100.0);
            let color = if current_pct >= 0.9 {
                work_desc.yellow()
            } else if current_pct >= 0.75 {
                work_desc.green()
            } else {
                work_desc.cyan()
            };

            println!("  {}: {}", period.bold(), color);

            current_pct = pct;
            current_start = age;
        }
    }

    println!("\n{}", "Key Insights:".bold());
    
    // Calculate average work percentage
    let avg_pct: f64 = strategy.iter().map(|(_, pct)| pct).sum::<f64>() / strategy.len() as f64;
    println!("  • Average work percentage: {:.0}%", avg_pct * 100.0);
    
    // Find transitions
    let mut transitions = 0;
    for i in 1..strategy.len() {
        if strategy[i].1 != strategy[i-1].1 {
            transitions += 1;
        }
    }
    println!("  • Number of transitions: {}", transitions);

    // Calculate total working years
    let total_years = strategy.len();
    let full_time_equivalent = avg_pct * total_years as f64;
    println!("  • Total working years: {}", total_years);
    println!("  • Full-time equivalent: {:.1} years", full_time_equivalent);

    println!("\n{}", "💡 This strategy balances:".bold());
    println!("  • Income needs at each life stage");
    println!("  • Time with family when children are young");
    println!("  • Career progression and pension contributions");
    println!("  • Health and stress management");

    println!("\n{}", "=".repeat(60));
}
