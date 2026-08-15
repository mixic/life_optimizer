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
    println!("\n{}", "🎯 OPTIMAL SOLUTION FOUND!".bold().green());
    println!("{}", "=".repeat(60));
    
    println!("\n{}", "Work Configuration:".bold());
    println!("  Work Percentage: {}", format!("{:.0}%", scenario.work_percentage * 100.0).cyan().bold());
    println!("  Gross Income:    {} CHF/year", format!("{:.0}", scenario.gross_income).cyan());
    println!("  After-Tax:       {} CHF/year", format!("{:.0}", scenario.after_tax_income).cyan());
    println!("  Monthly Net:     {} CHF/month", format!("{:.0}", scenario.monthly_after_tax).cyan().bold());
    println!("  Tax Rate:        {:.1}% (official Bern tax only)", scenario.tax_only_rate * 100.0);
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

    println!("\n{}", "Utility Score Breakdown:".bold());
    let breakdown = &scenario.utility_breakdown;
    println!("  Consumption:     {:.2}", breakdown.consumption_utility);
    println!("  Leisure:         {:.2}", breakdown.leisure_utility);
    println!("  Family:          {:.2}", breakdown.family_utility);
    println!("  Health:          {:.2}", breakdown.health_utility);
    println!("  Security:        {:.2}", breakdown.security_utility);
    println!("  {} {:.2}", "TOTAL UTILITY:".bold(), breakdown.total.to_string().cyan().bold());

    println!("\n{}", "=".repeat(60));
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
    println!("  Non-deductible estimate: CHF {:.0}", deduction.non_deductible_total);
    println!("  Taxable income after deductions: CHF {:.0}", tax_schedule.taxable_income_after_estimated_deductions(gross_income));
}

pub fn print_comparison_table(scenarios: &[WorkScenario]) {
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
    println!("\n{}", "ℹ️  Tax Rate Breakdown:".bold().cyan());
    println!("The 'Tax Rate' column shows TOTAL deductions including:");
    println!("  • Kantons-, Gemeinde- und Kirchensteuer (official Stadt Bern rates)");
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
