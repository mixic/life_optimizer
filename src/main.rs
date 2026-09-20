// The binary is a thin CLI wrapper over the `life_optimizer` library so that
// the integration tests in `tests/` can exercise the same code paths.
use life_optimizer::{tax, requirements, optimizer, display, monte_carlo, mc_display, consumption};

use clap::{Parser, Subcommand, ArgAction};
use requirements::{LifeStage, PersonalRequirements, PreferenceWeights, FamilySupport};
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
        #[arg(short, long, action = ArgAction::Set, default_value_t = false)]
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

        /// Use enhanced family/childcare deductions for married parents with children.
        #[arg(long, default_value_t = false)]
        family_tax_mode: bool,

        /// Retirement age (default: 65, supports deferred retirement up to 70)
        #[arg(long, default_value = "65")]
        retirement_age: u32,

        /// Life expectancy / target age (default: 90)
        #[arg(long, default_value = "90")]
        life_expectancy: u32,

        /// Annual Pillar 3a contribution in CHF (default: 0, max 7056)
        #[arg(long, default_value = "0")]
        pillar3a: f64,

        /// Children's ages (comma-separated, e.g. "1.5,9" for 1.5 and 9 years old)
        #[arg(long)]
        children_ages: Option<String>,

        /// Monthly education support cost per child during higher education (CHF)
        #[arg(long, default_value = "500")]
        education_cost_per_child: f64,

        /// Actual conversion rate (Umwandlungssatz) applied by your pension fund,
        /// as a decimal — e.g. 0.055 for 5.5%. When supplied it drives the
        /// headline projection; the 6.8% / 5.5% / projected range is still shown.
        #[arg(long)]
        conversion_rate: Option<f64>,

        /// Named pension fund profile (publica, bvk, statutory, typical).
        /// A reference rate only — verify it, or prefer --conversion-rate.
        #[arg(long)]
        pension_fund: Option<String>,

        /// Lifestyle consumption profile (extreme-saving, moderate, normal, luxury)
        #[arg(long, default_value = "normal")]
        consumption_profile: String,

        /// Fraction of elastic spending sourced second-hand/shared/borrowed (0.0-1.0)
        #[arg(long, default_value = "0.0")]
        sparing_ratio: f64,

        /// How strictly you apply the purchase prioritization hierarchy (0.0-1.0)
        #[arg(long, default_value = "0.0")]
        utilization_discipline: f64,

        /// Share of nominally discretionary spending locked in by switching costs (0.0-1.0)
        #[arg(long, default_value = "0.0")]
        quasi_inelastic_share: f64,

        /// Required project output index for your role (G_t). Enables the
        /// employer achievement-capacity constraint; without it, work percentage
        /// is treated as fully discretionary.
        #[arg(long)]
        required_output_index: Option<f64>,

        /// Productivity gain from AI and other tools, as a decimal — e.g. 0.25
        /// for +25%. Only meaningful with --required-output-index.
        #[arg(long, default_value = "0.0")]
        ai_productivity_gain: f64,
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
        #[arg(short, long, action = ArgAction::Set, default_value_t = false)]
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

        /// Use enhanced family/childcare deductions for married parents with children.
        #[arg(long, default_value_t = false)]
        family_tax_mode: bool,
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
        #[arg(short, long, action = ArgAction::Set, default_value_t = false)]
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
        #[arg(short, long, action = ArgAction::Set, default_value_t = false)]
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

        /// Actual conversion rate (Umwandlungssatz) applied by your pension fund,
        /// as a decimal — e.g. 0.055 for 5.5%.
        #[arg(long)]
        conversion_rate: Option<f64>,

        /// Named pension fund profile (publica, bvk, statutory, typical).
        /// A reference rate only — verify it, or prefer --conversion-rate.
        #[arg(long)]
        pension_fund: Option<String>,
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
            // `--canton` is accepted for CLI compatibility but is not yet wired to a
        // tax table: only the Bern schedule exists (`zurich_default` delegates
        // to it). Binding is intentionally skipped so the gap stays visible.
        canton: _canton,
            profile,
            custom_tax_rate,
            family_tax_mode,
            retirement_age,
            life_expectancy,
            pillar3a,
            children_ages,
            education_cost_per_child,
            conversion_rate,
            pension_fund,
            consumption_profile,
            sparing_ratio,
            utilization_discipline,
            quasi_inelastic_share,
            required_output_index,
            ai_productivity_gain,
        } => {
            let consumption = match consumption_config(consumption_profile, sparing_ratio, utilization_discipline, quasi_inelastic_share) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} {}", "error:".red().bold(), e);
                    std::process::exit(2);
                }
            };
            run_optimization(OptimizeParams {
                salary,
                age,
                married,
                children,
                youngest_child_age,
                profile: &profile,
                custom_tax_rate,
                family_tax_mode,
                retirement_age,
                life_expectancy,
                pillar3a,
                children_ages: children_ages.as_deref(),
                education_cost_per_child,
                conversion: resolve_conversion(conversion_rate, pension_fund),
                consumption_profile: consumption,
                achievement: achievement_constraint(required_output_index, ai_productivity_gain),
            });
        }
        Commands::Compare {
            salary,
            age,
            married,
            children,
            percentages,
            custom_tax_rate,
            family_tax_mode,
        } => {
            run_comparison(salary, age, married, children, &percentages, custom_tax_rate, family_tax_mode);
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
            conversion_rate,
            pension_fund,
        } => {
            run_pension_simulation(salary, age, married, work_pct, retirement_age, life_expectancy, pillar3a, custom_tax_rate, resolve_conversion(conversion_rate, pension_fund));
        }
    }
}

/// Explain which constraint eliminated every candidate, and what would change
/// the answer. Silent failure is the least useful outcome for a planning tool.
fn print_infeasibility_hint(
    reason: optimizer::InfeasibilityReason,
    achievement: Option<optimizer::AchievementConstraint>,
) {
    use optimizer::InfeasibilityReason;

    println!("\n{}", "Why no option worked:".bold());
    match reason {
        InfeasibilityReason::Unaffordable => {
            println!("  Net income never covers your mandatory spending floor at any work");
            println!("  percentage. Working more cannot fix a shortfall of this size alone.");
            println!("  Consider: a lower-cost canton, reduced inelastic costs, a second");
            println!("  income, or revisiting the consumption profile (--consumption-profile).");
        }
        InfeasibilityReason::AchievementUnreachable => {
            println!("  Your household finances are fine, but the required project output");
            println!("  cannot be delivered even at 100% work. This is a workload problem,");
            println!("  not a budget problem.");
            if let Some(c) = achievement {
                match c.required_ai_gain_for(1.0) {
                    Some(gain) if gain > 0.0 => println!(
                        "  The goals as stated would need a {:.0}% productivity gain at full time.",
                        gain * 100.0
                    ),
                    _ => {}
                }
            }
            println!("  Consider: renegotiating the project portfolio, or modelling an AI");
            println!("  productivity gain with --ai-productivity-gain.");
        }
        InfeasibilityReason::Both => {
            println!("  Both constraints fail: net income misses your mandatory floor, and");
            println!("  the required output is not deliverable even at 100% work.");
            println!("  Address the budget gap first, then the workload.");
        }
    }
}

/// Resolve the conversion-rate scenario from the two related flags.
///
/// `--conversion-rate` wins over `--pension-fund` when both are given, because
/// an explicit figure from the user's own fund statement is always better than
/// a reference profile. Using a named profile prints a reminder to verify it,
/// since a stale or superseded rate is a plausible-looking but wrong input.
fn resolve_conversion(
    conversion_rate: Option<f64>,
    pension_fund: Option<String>,
) -> monte_carlo::ConversionRateScenario {
    if let Some(rate) = conversion_rate {
        return monte_carlo::ConversionRateScenario::Custom(rate);
    }
    if let Some(id) = pension_fund {
        match monte_carlo::find_pension_fund(&id) {
            Some(profile) => {
                println!(
                    "Using pension fund profile: {} at {:.2}%",
                    profile.name.cyan().bold(),
                    profile.conversion_rate * 100.0
                );
                println!("  {}", format!("  {}", profile.source_note).dimmed());
                println!(
                    "  {}\n",
                    "Verify this rate with your fund — rates change annually.".yellow()
                );
                return monte_carlo::ConversionRateScenario::Custom(profile.conversion_rate);
            }
            None => {
                eprintln!(
                    "{} unknown pension fund '{id}'. Known profiles: {}",
                    "error:".red().bold(),
                    monte_carlo::pension_fund_ids()
                );
                eprintln!("  Or pass your fund's rate directly with --conversion-rate.");
                std::process::exit(2);
            }
        }
    }
    monte_carlo::ConversionRateScenario::Statutory
}

/// Build consumption parameters from the CLI flags, rejecting an unknown
/// profile rather than silently defaulting to normal.
fn consumption_config(
    profile: String,
    sparing_ratio: f64,
    utilization_discipline: f64,
    quasi_inelastic_share: f64,
) -> Result<consumption::ConsumptionProfileConfig, String> {
    let profile = consumption::ConsumptionProfile::parse(&profile)
        .ok_or_else(|| format!(
            "unknown consumption profile '{profile}'. Expected one of: extreme-saving, moderate, normal, luxury"
        ))?;
    Ok(consumption::ConsumptionProfileConfig::new(profile)
        .with_sparing_ratio(sparing_ratio)
        .with_utilization_discipline(utilization_discipline)
        .with_quasi_inelastic_share(quasi_inelastic_share))
}

/// Build the employer-side achievement constraint. Absent a required output
/// index the constraint is not applied, preserving the previous behaviour in
/// which work percentage is fully discretionary.
fn achievement_constraint(
    required_output_index: Option<f64>,
    ai_productivity_gain: f64,
) -> Option<optimizer::AchievementConstraint> {
    required_output_index.map(|g| optimizer::AchievementConstraint::new(g, ai_productivity_gain))
}

/// Validate an age/retirement-age pair before it reaches the optimizer.
///
/// A retirement age at or before the current age leaves no accumulation period;
/// this used to reach `u32` subtraction in the optimizer and panic. Fail with a
/// clear message instead of a backtrace.
fn validate_ages(age: u32, retirement_age: u32, life_expectancy: u32) {
    if retirement_age <= age {
        eprintln!(
            "{} retirement age ({}) must be greater than your current age ({}).",
            "error:".red().bold(),
            retirement_age,
            age
        );
        std::process::exit(2);
    }
    if life_expectancy <= retirement_age {
        eprintln!(
            "{} life expectancy ({}) must be greater than the retirement age ({}).",
            "error:".red().bold(),
            life_expectancy,
            retirement_age
        );
        std::process::exit(2);
    }
}

/// Inputs for one optimization run, grouped so the entry point does not take a
/// seventeen-argument parameter list.
struct OptimizeParams<'a> {
    salary: f64,
    age: u32,
    married: bool,
    children: u32,
    youngest_child_age: Option<u32>,
    profile: &'a str,
    custom_tax_rate: Option<f64>,
    family_tax_mode: bool,
    retirement_age: u32,
    life_expectancy: u32,
    pillar3a: f64,
    children_ages: Option<&'a str>,
    education_cost_per_child: f64,
    conversion: monte_carlo::ConversionRateScenario,
    consumption_profile: consumption::ConsumptionProfileConfig,
    achievement: Option<optimizer::AchievementConstraint>,
}

fn run_optimization(p: OptimizeParams<'_>) {
    let OptimizeParams {
        salary,
        age,
        married,
        children,
        youngest_child_age,
        profile,
        custom_tax_rate,
        family_tax_mode,
        retirement_age,
        life_expectancy,
        pillar3a,
        children_ages,
        education_cost_per_child,
        conversion,
        consumption_profile,
        achievement,
    } = p;

    println!("\n{}", "=== LIFE OPTIMIZER ===".bold().cyan());
    println!("Finding optimal work percentage for your situation...\n");

    validate_ages(age, retirement_age, life_expectancy);

    let mut tax_schedule = if let Some(rate) = custom_tax_rate {
        println!("Using custom tax rate: {:.2}%\n", rate * 100.0);
        TaxSchedule::custom_rate(rate)
    } else {
        TaxSchedule::bern_city_default(married, children)
    };
    tax_schedule.family_tax_mode = family_tax_mode || (married && children > 0);

    let requirements = PersonalRequirements::bern_family_default(children);

    let child_ages_vec = if let Some(youngest) = youngest_child_age {
        vec![youngest]
    } else {
        vec![]
    };

    let life_stage = LifeStage::determine_from_age(age, children > 0, &child_ages_vec);

    let preferences = match profile {
        "family" => PreferenceWeights::family_focused(),
        "career" => PreferenceWeights::career_focused(),
        _ => PreferenceWeights::balanced(),
    };

    let mut config = OptimizerConfig::new(
        salary,
        tax_schedule.clone(),
        requirements.clone(),
        life_stage,
        preferences,
    );
    config.retirement_age = retirement_age;
    config.conversion_scenario = conversion;
    config.consumption = consumption_profile;
    config.achievement = achievement;

    let optimizer = LifeOptimizer::new(config);
    let candidates = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let outcome = optimizer
        .search_outcome(&candidates)
        .expect("candidate list is non-empty");
    let (optimal, all_scenarios) = (outcome.scenario.clone(), outcome.all_scenarios.clone());

    display::print_tax_deduction_breakdown(&tax_schedule, optimal.gross_income);
    // Display work-life balance results
    display::print_optimal_result(&optimal);

    // When nothing was feasible, say which constraint eliminated the options.
    // The remedies are completely different: one is a budget problem, the other
    // a workload or AI-productivity problem.
    if let Some(reason) = outcome.infeasibility_reason() {
        print_infeasibility_hint(reason, achievement);
    }

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
        conversion_scenario: conversion,
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
        conversion_scenario: conversion,
    };
    let regime_result = regime_sim.run_regime_switching();

    // ── Conversion-rate (Umwandlungssatz) transparency ──────────────────────
    let pension_range = regime_sim.pension_range_for_capital(regime_result.median_capital);
    mc_display::print_conversion_rate_scenarios(&pension_range, regime_result.ahv_monthly);

    // Stochastic rate uncertainty, with explicit downside (CVaR).
    let rate_uncertainty = regime_sim.conversion_rate_uncertainty(
        regime_result.median_capital,
        monthly_needs,
    );
    mc_display::print_conversion_rate_uncertainty(&rate_uncertainty);

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

    // ── Family Support & Education Planning ─────────────────────────────────
    if let Some(ages_str) = children_ages {
        println!("\n{}", "=== RETIREMENT ADEQUACY WITH FAMILY SUPPORT ===".bold().cyan());
        
        let family_support = FamilySupport::from_ages_string(ages_str, education_cost_per_child);
        
        // Estimate pension balance at retirement using simplified projection
        let years_to_retirement = (retirement_age - age) as f64;
        let bvg_contribution_annual = salary * optimal.work_percentage * 0.083;  // ~8.3% BVG rate
        let pension_at_65 = bvg_contribution_annual 
            * ((1.02_f64.powf(years_to_retirement) - 1.0) / (1.02 - 1.0))  // 2% annual return
            + (pillar3a * years_to_retirement);  // Pillar 3a linear accumulation
        
        let adequacy = optimizer.calculate_retirement_adequacy(
            optimal.work_percentage,
            pension_at_65,
            &family_support,
            life_expectancy,
        );
        
        println!("\n📊 Monthly Retirement Income:");
        println!("  Pension withdrawal (4% rule): CHF {:>8.0}", 
            (pension_at_65 * 0.04 / 12.0).round());
        println!("  AHV (state pension):          CHF {:>8.0}", 
            (adequacy.retirement_income_monthly - pension_at_65 * 0.04 / 12.0).round());
        println!("  TOTAL:                        CHF {:>8.0}", 
            adequacy.retirement_income_monthly.round());
        
        println!("\n💰 Monthly Expenses:");
        println!("  Personal needs:               CHF {:>8.0}", adequacy.self_sustaining_monthly.round());
        let education_cost = adequacy.total_required_monthly - adequacy.self_sustaining_monthly;
        println!("  Child education support:      CHF {:>8.0}", education_cost.round());
        println!("  TOTAL:                        CHF {:>8.0}", adequacy.total_required_monthly.round());
        
        println!("\n✓ Financial Security:");
        println!("  Monthly surplus/deficit:      CHF {:>+8.0}", adequacy.monthly_surplus_deficit.round());
        println!("  Sustainable until age ~{}:   {} years", 
            (retirement_age as f64 + adequacy.sustainable_years) as u32,
            adequacy.sustainable_years.round());
        println!("  Education support years:      {:.1} years", adequacy.education_support_years);
        println!("\n  Status: {}", adequacy.summary);
    }
}

fn run_comparison(
    salary: f64,
    age: u32,
    married: bool,
    children: u32,
    percentages_str: &str,
    custom_tax_rate: Option<f64>,
    family_tax_mode: bool,
) {
    println!("\n{}", "=== SCENARIO COMPARISON ===".bold().cyan());
    
    let percentages: Vec<f64> = percentages_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let mut tax_schedule = if let Some(rate) = custom_tax_rate {
        println!("Using custom tax rate: {:.2}%\n", rate * 100.0);
        TaxSchedule::custom_rate(rate)
    } else {
        TaxSchedule::bern_city_default(married, children)
    };
    tax_schedule.family_tax_mode = family_tax_mode || (married && children > 0);

    let requirements = PersonalRequirements::bern_family_default(children);
    let life_stage = LifeStage::determine_from_age(age, children > 0, &[]);
    let preferences = PreferenceWeights::balanced();

    let config = OptimizerConfig::new(
        salary,
        tax_schedule.clone(),
        requirements,
        life_stage,
        preferences,
    );

    let optimizer = LifeOptimizer::new(config);
    
    let scenarios: Vec<_> = percentages
        .iter()
        .map(|&pct| optimizer.evaluate_scenario(pct))
        .collect();

    for scenario in &scenarios {
        display::print_tax_deduction_breakdown(&tax_schedule, scenario.gross_income);
    }
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
    
    run_optimization(OptimizeParams {
        salary,
        age,
        married,
        children,
        youngest_child_age: youngest_age,
        profile,
        custom_tax_rate: None,
        family_tax_mode: married && children > 0,
        retirement_age: 65,
        life_expectancy: 90,
        pillar3a: 0.0,
        children_ages: None,
        education_cost_per_child: 500.0,
        conversion: monte_carlo::ConversionRateScenario::Statutory,
        consumption_profile: consumption::ConsumptionProfileConfig::default(),
        achievement: None,
    });
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
    conversion: monte_carlo::ConversionRateScenario,
) {
    println!("\n{}", "=== PENSION SIMULATION ===".bold().cyan());

    validate_ages(age, retirement_age, life_expectancy);

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
    sim.conversion_scenario = conversion;

    let (con, base, opt) = sim.run_all_scenarios();
    mc_display::print_monte_carlo_summary(&con, &base, &opt, monthly_needs, retirement_age, life_expectancy);

    // Conversion-rate transparency, before the regime/stress sections
    let pension_range = sim.pension_range_for_capital(base.median_capital);
    mc_display::print_conversion_rate_scenarios(&pension_range, base.ahv_monthly);

    // Stochastic rate uncertainty, with explicit downside (CVaR).
    let rate_uncertainty = sim.conversion_rate_uncertainty(base.median_capital, monthly_needs);
    mc_display::print_conversion_rate_uncertainty(&rate_uncertainty);

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
