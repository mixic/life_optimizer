mod tax;
mod requirements;
mod optimizer;
mod display;
mod monte_carlo;
mod mc_display;
mod economic_regimes;

use clap::{Parser, Subcommand};
use requirements::{LifeStage, PersonalRequirements, PreferenceWeights};
use tax::TaxSchedule;
use optimizer::{OptimizerConfig, LifeOptimizer};
use colored::*;

#[derive(Parser)]
#[command(name = "life-optimizer")]
#[command(about = "Optimize work-life balance considering taxes, requirements, and long-term sustainability", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find optimal work percentage for current situation
    Optimize {
        /// Full-time annual salary in CHF
        #[arg(short, long)]
        salary: f64,

        /// Your current age
        #[arg(short, long)]
        age: u32,

        /// Are you married?
        #[arg(short, long, default_value = "false")]
        married: bool,

        /// Number of children
        #[arg(short, long, default_value = "0")]
        children: u32,

        /// Youngest child age (if applicable)
        #[arg(long)]
        youngest_child_age: Option<u32>,

        /// Canton code (e.g., ZH, BE, GE)
        #[arg(long, default_value = "ZH")]
        canton: String,

        /// Preference profile (balanced, family, career)
        #[arg(short, long, default_value = "balanced")]
        profile: String,

        /// Custom tax rate (as decimal, e.g., 0.1382 for 13.82%). Overrides official tables.
        #[arg(long)]
        custom_tax_rate: Option<f64>,

        /// Retirement age (default: 65, supports deferred retirement up to 70)
        #[arg(long, default_value = "65")]
        retirement_age: u32,

        /// Life expectancy / target age (default: 90)
        #[arg(long, default_value = "90")]
        life_expectancy: u32,

        /// Annual Pillar 3a contribution in CHF (default: 0, max 7056)
        #[arg(long, default_value = "0")]
        pillar3a: f64,
    },

    /// Compare specific work percentage scenarios
    Compare {
        /// Full-time annual salary in CHF
        #[arg(short, long)]
        salary: f64,

        /// Your current age
        #[arg(short, long)]
        age: u32,

        /// Are you married?
        #[arg(short, long, default_value = "false")]
        married: bool,

        /// Number of children
        #[arg(short, long, default_value = "0")]
        children: u32,

        /// Work percentages to compare (comma-separated)
        #[arg(short, long, default_value = "0.6,0.8,1.0")]
        percentages: String,

        /// Custom tax rate (as decimal, e.g., 0.1382 for 13.82%). Overrides official tables.
        #[arg(long)]
        custom_tax_rate: Option<f64>,
    },

    /// Calculate lifetime strategy (work % by age)
    Lifetime {
        /// Full-time annual salary in CHF
        #[arg(short, long)]
        salary: f64,

        /// Your current age
        #[arg(short, long)]
        age: u32,

        /// Are you married?
        #[arg(short, long, default_value = "false")]
        married: bool,

        /// Number of children
        #[arg(short, long, default_value = "0")]
        children: u32,

        /// Retirement age (supports deferred retirement up to 70)
        #[arg(short, long, default_value = "65")]
        retirement_age: u32,
    },

    /// Interactive mode (ask questions)
    Interactive,

    /// Monte Carlo pension simulation for a specific work percentage
    Pension {
        /// Full-time annual salary in CHF
        #[arg(short, long)]
        salary: f64,

        /// Your current age
        #[arg(short, long)]
        age: u32,

        /// Are you married?
        #[arg(short, long, default_value = "false")]
        married: bool,

        /// Work percentage to simulate (e.g. 0.8 for 80%)
        #[arg(short, long, default_value = "1.0")]
        work_pct: f64,

        /// Retirement age (default: 65, supports deferred retirement up to 70)
        #[arg(long, default_value = "65")]
        retirement_age: u32,

        /// Life expectancy (default: 90)
        #[arg(long, default_value = "90")]
        life_expectancy: u32,

        /// Annual Pillar 3a contribution in CHF (default: 0)
        #[arg(long, default_value = "0")]
        pillar3a: f64,

        /// Custom tax rate (as decimal, e.g. 0.1382 for 13.82%)
        #[arg(long)]
        custom_tax_rate: Option<f64>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Optimize {
            salary,
            age,
            married,
            children,
            youngest_child_age,
            canton,
            profile,
            custom_tax_rate,
            retirement_age,
            life_expectancy,
            pillar3a,
        } => {
            run_optimization(salary, age, married, children, youngest_child_age, &canton, &profile, custom_tax_rate, retirement_age, life_expectancy, pillar3a);
        }
        Commands::Compare {
            salary,
            age,
            married,
            children,
            percentages,
            custom_tax_rate,
        } => {
            run_comparison(salary, age, married, children, &percentages, custom_tax_rate);
        }
        Commands::Lifetime {
            salary,
            age,
            married,
            children,
            retirement_age,
        } => {
            run_lifetime_strategy(salary, age, married, children, retirement_age);
        }
        Commands::Interactive => {
            run_interactive();
        }
        Commands::Pension {
            salary,
            age,
            married,
            work_pct,
            retirement_age,
            life_expectancy,
            pillar3a,
            custom_tax_rate,
        } => {
            run_pension_simulation(salary, age, married, work_pct, retirement_age, life_expectancy, pillar3a, custom_tax_rate);
        }
    }
}

fn run_optimization(
    salary: f64,
    age: u32,
    married: bool,
    children: u32,
    youngest_child_age: Option<u32>,
    _canton: &str,
    profile: &str,
    custom_tax_rate: Option<f64>,
    retirement_age: u32,
    life_expectancy: u32,
    pillar3a: f64,
) {
    println!("\n{}", "=== LIFE OPTIMIZER ===".bold().cyan());
    println!("Finding optimal work percentage for your situation...\n");

    let tax_schedule = if let Some(rate) = custom_tax_rate {
        println!("Using custom tax rate: {:.2}%\n", rate * 100.0);
        TaxSchedule::custom_rate(rate)
    } else {
        TaxSchedule::bern_city_default(married, children)
    };

    let requirements = PersonalRequirements::bern_family_default(children);

    let children_ages = if let Some(youngest) = youngest_child_age {
        vec![youngest]
    } else {
        vec![]
    };

    let life_stage = LifeStage::determine_from_age(age, children > 0, &children_ages);

    let preferences = match profile {
        "family" => PreferenceWeights::family_focused(),
        "career" => PreferenceWeights::career_focused(),
        _ => PreferenceWeights::balanced(),
    };

    let mut config = OptimizerConfig::new(
        salary,
        tax_schedule,
        requirements.clone(),
        life_stage,
        preferences,
    );
    config.retirement_age = retirement_age;

    let optimizer = LifeOptimizer::new(config);
    let candidates = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let (optimal, all_scenarios) = optimizer.find_optimal(&candidates);

    // Display work-life balance results
    display::print_optimal_result(&optimal);

    // ── Monte Carlo pension simulation ──────────────────────────────────────
    let monthly_needs = requirements
        .adjusted_for_life_stage(
            &LifeStage::determine_from_age(retirement_age, false, &[])
        )
        .total_monthly();

    let (con, base, opt_mc) = monte_carlo::PensionSimulator {
        current_age: age,
        retirement_age,
        life_expectancy,
        current_salary: salary,
        work_percentage: optimal.work_percentage,
        married,
        existing_bvg_capital: 0.0,
        pillar3a_annual: pillar3a,
        monthly_retirement_needs: monthly_needs,
        n_simulations: 10_000,
        seed: Some(42),
    }
    .run_all_scenarios();

    mc_display::print_monte_carlo_summary(&con, &base, &opt_mc, monthly_needs, retirement_age, life_expectancy);

    // ── Regime-switching (recession/inflation-aware) simulation ─────────────
    let regime_sim = monte_carlo::PensionSimulator {
        current_age: age,
        retirement_age,
        life_expectancy,
        current_salary: salary,
        work_percentage: optimal.work_percentage,
        married,
        existing_bvg_capital: 0.0,
        pillar3a_annual: pillar3a,
        monthly_retirement_needs: monthly_needs,
        n_simulations: 10_000,
        seed: Some(42),
    };
    let regime_result = regime_sim.run_regime_switching();
    mc_display::print_regime_switching_result(&regime_result, monthly_needs);

    let stress_result = regime_sim.run_retirement_shock_stress_test();
    mc_display::print_stress_test_result(&stress_result, regime_result.median_real_pension, monthly_needs);

    // Work % vs pension quality comparison
    let comparisons = monte_carlo::compare_work_percentages(
        age, retirement_age, life_expectancy,
        salary, married, monthly_needs, pillar3a, &candidates,
    );
    mc_display::print_work_pct_pension_comparison(&comparisons, monthly_needs);

    display::print_comparison_table(&all_scenarios);
    display::print_recommendations(&optimal, age);
}

fn run_comparison(
    salary: f64,
    age: u32,
    married: bool,
    children: u32,
    percentages_str: &str,
    custom_tax_rate: Option<f64>,
) {
    println!("\n{}", "=== SCENARIO COMPARISON ===".bold().cyan());
    
    let percentages: Vec<f64> = percentages_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let tax_schedule = if let Some(rate) = custom_tax_rate {
        println!("Using custom tax rate: {:.2}%\n", rate * 100.0);
        TaxSchedule::custom_rate(rate)
    } else {
        TaxSchedule::bern_city_default(married, children)
    };
    
    let requirements = PersonalRequirements::bern_family_default(children);
    let life_stage = LifeStage::determine_from_age(age, children > 0, &[]);
    let preferences = PreferenceWeights::balanced();

    let config = OptimizerConfig::new(
        salary,
        tax_schedule,
        requirements,
        life_stage,
        preferences,
    );

    let optimizer = LifeOptimizer::new(config);
    
    let scenarios: Vec<_> = percentages
        .iter()
        .map(|&pct| optimizer.evaluate_scenario(pct))
        .collect();

    display::print_comparison_table(&scenarios);
}

fn run_lifetime_strategy(
    salary: f64,
    age: u32,
    married: bool,
    children: u32,
    retirement_age: u32,
) {
    println!("\n{}", "=== LIFETIME STRATEGY ===".bold().cyan());
    println!("Calculating optimal work percentages from age {} to {}...\n", age, retirement_age);

    let tax_schedule = TaxSchedule::bern_city_default(married, children);
    let requirements = PersonalRequirements::bern_family_default(children);
    let life_stage = LifeStage::determine_from_age(age, children > 0, &[]);
    let preferences = PreferenceWeights::balanced();

    let mut config = OptimizerConfig::new(
        salary,
        tax_schedule,
        requirements,
        life_stage,
        preferences,
    );
    config.retirement_age = retirement_age;

    let optimizer = LifeOptimizer::new(config);
    let strategy = optimizer.find_optimal_lifetime_strategy();

    display::print_lifetime_strategy(&strategy);
}

fn run_interactive() {
    println!("\n{}", "=== INTERACTIVE LIFE OPTIMIZER ===".bold().cyan());
    println!("Let's find your optimal work-life balance!\n");

    use std::io::{self, Write};

    // Helper function to read input
    fn read_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    // Collect information
    let salary: f64 = read_input("What is your full-time annual salary (CHF)? ")
        .parse()
        .unwrap_or(80000.0);

    let age: u32 = read_input("What is your age? ")
        .parse()
        .unwrap_or(35);

    let married = read_input("Are you married? (y/n) ")
        .to_lowercase()
        .starts_with('y');

    let children: u32 = read_input("How many children do you have? ")
        .parse()
        .unwrap_or(0);

    let youngest_age = if children > 0 {
        Some(read_input("What is the age of your youngest child? ")
            .parse()
            .unwrap_or(5))
    } else {
        None
    };

    println!("\nWhat are your priorities?");
    println!("1. Balanced (equal weight to all factors)");
    println!("2. Family-focused (prioritize time with family)");
    println!("3. Career-focused (prioritize income and security)");
    let profile = match read_input("Choose (1/2/3): ").as_str() {
        "2" => "family",
        "3" => "career",
        _ => "balanced",
    };

    println!("\n{}", "Analyzing your situation...".yellow());
    
    run_optimization(salary, age, married, children, youngest_age, "ZH", profile, None, 65, 90, 0.0);
}

fn run_pension_simulation(
    salary: f64,
    age: u32,
    married: bool,
    work_pct: f64,
    retirement_age: u32,
    life_expectancy: u32,
    pillar3a: f64,
    custom_tax_rate: Option<f64>,
) {
    println!("\n{}", "=== PENSION SIMULATION ===".bold().cyan());

    let tax_schedule = if let Some(rate) = custom_tax_rate {
        TaxSchedule::custom_rate(rate)
    } else {
        TaxSchedule::bern_city_default(married, 0)
    };

    let working_income = salary * work_pct;
    let after_tax = tax_schedule.after_tax_income(working_income);
    let monthly_needs = after_tax / 12.0 * 0.75; // 75% of current net

    println!("\n  Work:         {:.0}%  |  Gross: CHF {:.0}/year  |  Net: CHF {:.0}/month",
        work_pct * 100.0, working_income, after_tax / 12.0);
    println!("  Retirement:   age {}  →  age {}  ({} years in retirement)",
        retirement_age, life_expectancy, life_expectancy - retirement_age);
    println!("  Pillar 3a:    CHF {:.0}/year", pillar3a);
    println!("  Target needs: CHF {:.0}/month (75% of current net)", monthly_needs);

    let mut sim = monte_carlo::PensionSimulator::new(
        age, retirement_age, life_expectancy,
        salary, work_pct, married, monthly_needs,
    );
    sim.pillar3a_annual = pillar3a;
    sim.n_simulations = 10_000;

    let (con, base, opt) = sim.run_all_scenarios();
    mc_display::print_monte_carlo_summary(&con, &base, &opt, monthly_needs, retirement_age, life_expectancy);

    // Regime-switching + stress test
    let regime_result = sim.run_regime_switching();
    mc_display::print_regime_switching_result(&regime_result, monthly_needs);

    let stress_result = sim.run_retirement_shock_stress_test();
    mc_display::print_stress_test_result(&stress_result, regime_result.median_real_pension, monthly_needs);

    // Also compare all work percentages
    let candidates = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let comparisons = monte_carlo::compare_work_percentages(
        age, retirement_age, life_expectancy,
        salary, married, monthly_needs, pillar3a, &candidates,
    );
    mc_display::print_work_pct_pension_comparison(&comparisons, monthly_needs);
}

fn print_pension_sustainability(
    scenario: &optimizer::WorkScenario,
    full_time_salary: f64,
    current_age: u32,
    retirement_age: u32,
    life_expectancy: u32,
    married: bool,
) {
    println!("\n{}", "💰 PENSION SUSTAINABILITY ANALYSIS".bold().blue());
    println!("{}", "=".repeat(60));
    
    let years_until_retirement = (retirement_age - current_age) as f64;
    let years_in_retirement = (life_expectancy - retirement_age) as f64;
    
    // Calculate projected pension
    let working_income = full_time_salary * scenario.work_percentage;
    
    // Swiss pension system (simplified)
    // Pillar 1 (AHV): Max ~CHF 29,400/year for single, ~CHF 44,100 for couple
    let ahv_max = if married { 44_100.0 } else { 29_400.0 };
    let ahv_pension = (working_income * 0.30).min(ahv_max);
    
    // Pillar 2 (BVG): Accumulated savings
    let bvg_contribution_rate = 0.083;  // ~8.3% per year
    let bvg_annual_contribution = working_income * bvg_contribution_rate;
    let compound_rate: f64 = 1.02;  // 2% annual return
    let bvg_capital = bvg_annual_contribution * 
        ((compound_rate.powf(years_until_retirement) - 1.0) / (compound_rate - 1.0));
    let bvg_conversion_rate = 0.068;  // 6.8% at age 65
    let bvg_annual_pension = bvg_capital * bvg_conversion_rate;
    
    let total_annual_pension = ahv_pension + bvg_annual_pension;
    let total_monthly_pension = total_annual_pension / 12.0;
    
    // Calculate needs in retirement (typically 70-80% of working income)
    let retirement_needs_monthly = scenario.monthly_after_tax * 0.75;  // 75% of current
    
    let pension_coverage = total_monthly_pension / retirement_needs_monthly;
    
    println!("\n{}", "Current Work Scenario:".bold());
    println!("  Work Percentage:     {:.0}%", scenario.work_percentage * 100.0);
    println!("  Working Income:      CHF {:.0}/year", working_income);
    println!("  Years Until Retire:  {} years", years_until_retirement as u32);
    
    println!("\n{}", format!("Projected Pension (at age {}):", retirement_age).bold());
    println!("  AHV (Pillar 1):      CHF {:.0}/month", ahv_pension / 12.0);
    println!("  BVG (Pillar 2):      CHF {:.0}/month", bvg_annual_pension / 12.0);
    println!("  {} CHF {}/month", "Total Pension:".bold(), format!("{:.0}", total_monthly_pension).cyan().bold());
    
    println!("\n{}", "Retirement Needs:".bold());
    println!("  Estimated Monthly:   CHF {:.0}", retirement_needs_monthly);
    println!("  Years in Retirement: {} years (age {}-{})", years_in_retirement as u32, retirement_age, life_expectancy);
    
    println!("\n{}", "Sustainability Check:".bold());
    if pension_coverage >= 1.0 {
        println!("  Status:              {} ✓", "PENSION ADEQUATE".green().bold());
        println!("  Coverage:            {:.0}% of needs", pension_coverage * 100.0);
        println!("  Monthly Surplus:     CHF +{:.0}", total_monthly_pension - retirement_needs_monthly);
    } else {
        println!("  Status:              {} ⚠", "PENSION INSUFFICIENT".yellow().bold());
        println!("  Coverage:            {:.0}% of needs", pension_coverage * 100.0);
        println!("  Monthly Shortfall:   CHF {:.0}", retirement_needs_monthly - total_monthly_pension);
        
        let total_shortfall = (retirement_needs_monthly - total_monthly_pension) * 12.0 * years_in_retirement;
        println!("  Total Shortfall:     CHF {:.0} over {} years", total_shortfall, years_in_retirement as u32);
        
        println!("\n{}", "  💡 Solutions:".yellow());
        println!("    • Increase work percentage to boost BVG contributions");
        println!("    • Contribute to Pillar 3a (CHF 7,056/year tax deductible)");
        println!("    • Build private savings/investments");
        println!("    • Consider working past age {}", retirement_age);
        
        // Calculate what work % would be needed
        let needed_bvg_pension = retirement_needs_monthly - (ahv_pension / 12.0);
        let needed_bvg_capital = needed_bvg_pension / bvg_conversion_rate;
        let needed_annual_contribution = needed_bvg_capital / 
            ((compound_rate.powf(years_until_retirement) - 1.0) / (compound_rate - 1.0));
        let needed_income = needed_annual_contribution / bvg_contribution_rate;
        let needed_work_pct = needed_income / full_time_salary;
        
        if needed_work_pct <= 1.0 && needed_work_pct > 0.0 {
            println!("    • Working {:.0}% would provide adequate pension", needed_work_pct * 100.0);
        } else {
            println!("    • Even 100% work may not be sufficient - additional savings needed");
        }
    }
    
    println!("\n{}", "  Note: Assumes 2% annual return and 6.8% BVG conversion rate".italic());
    println!("{}", "=".repeat(60));
}
