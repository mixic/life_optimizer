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

// The binary is a thin CLI wrapper over the `life_optimizer` library so that
// the integration tests in `tests/` can exercise the same code paths.
use life_optimizer::{tax, requirements, optimizer, display, monte_carlo, mc_display, consumption, cantons, deductions};

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
    ///
    /// Boxed because this variant carries the most parameters of any command, and
    /// the size difference between enum variants is a real cost: every `Commands`
    /// value would otherwise be as large as this one.
    Optimize(Box<OptimizeArgs>),
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

        /// Canton code (e.g. ZH, BE, AG). Omit for Bern, and the output says so.
        #[arg(long)]
        canton: Option<String>,
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

/// Parameters for `optimize`, extracted from the `Commands` variant so that variant
/// can be boxed.
///
/// `Commands` is a plain enum, so every value is as large as its biggest variant.
/// `optimize` takes by far the most parameters, and adding the household-fact flags
/// pushed the difference past the size at which that cost is worth a `Box`.
#[derive(clap::Args, Debug, Clone)]
struct OptimizeArgs {
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

    /// Canton code (e.g. ZH, BE, AG). 23 of the 26 cantons are priced; the
    /// rest fail with exactly what is missing rather than falling back to
    /// another canton. See SWISS_TAX_DATA.md. Omitting it uses Bern, and
    /// the output says so.
    #[arg(long)]
    canton: Option<String>,

    /// Preference profile (balanced, family, career)
    #[arg(short, long, default_value = "balanced")]
    profile: String,

    /// Custom tax rate (as decimal, e.g., 0.1382 for 13.82%). Overrides official tables.
    #[arg(long)]
    custom_tax_rate: Option<f64>,

    /// Use enhanced family/childcare deductions for married parents with children.
    #[arg(long, default_value_t = false)]
    family_tax_mode: bool,

    /// Target retirement age
    #[arg(long, default_value = "65")]
    retirement_age: u32,

    /// Life expectancy for planning
    #[arg(long, default_value = "90")]
    life_expectancy: u32,

    /// Annual pillar 3a contribution in CHF
    #[arg(long, default_value = "0.0")]
    pillar3a: f64,

    /// Comma-separated list of children's ages (e.g. "5,8,12")
    #[arg(long)]
    children_ages: Option<String>,

    /// Annual education cost per child in CHF
    #[arg(long, default_value = "500.0")]
    education_cost_per_child: f64,

    /// Actual conversion rate (Umwandlungssatz) applied by your pension fund,
    /// as a decimal (e.g. 0.05 for 5%). Overrides the statutory reference rate.
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

    /// Imputed rental value (Eigenmietwert) of your home, in CHF/year. This is
    /// an *income addition* as well as the base for every property deduction,
    /// so it raises taxable income. Omit if you rent.
    #[arg(long)]
    imputed_rental_value: Option<f64>,

    /// You receive an AHV/IV pension. Unlocks the pensioner deductions, which
    /// are otherwise never applied.
    #[arg(long, default_value_t = false)]
    pensioner: bool,

    /// Declared private insurance premiums and savings interest, in CHF/year.
    /// These are capped by each canton, so the published ceiling applies where
    /// your figure exceeds it.
    #[arg(long)]
    insurance_premiums: Option<f64>,

    /// What you pay towards a child's education costs, in CHF/year. St. Gallen
    /// allows a flat deduction conditional on making such a contribution, so
    /// the rule stays unapplied without this.
    #[arg(long)]
    education_contribution: Option<f64>,

    /// Required project output index for your role (G_t). Enables the
    /// employer achievement-capacity constraint; without it, work percentage
    /// is treated as fully discretionary.
    #[arg(long)]
    required_output_index: Option<f64>,

    /// Productivity gain from AI and other tools, as a decimal — e.g. 0.25
    /// for +25%. Only meaningful with --required-output-index.
    #[arg(long, default_value = "0.0")]
    ai_productivity_gain: f64,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Optimize(args) => {
            let OptimizeArgs {
                salary,
                age,
                married,
                children,
                youngest_child_age,
                canton,
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
                imputed_rental_value,
                pensioner,
                insurance_premiums,
                education_contribution,
            } = *args;
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
                canton: canton.as_deref(),
                conversion: resolve_conversion(conversion_rate, pension_fund),
                consumption_profile: consumption,
                achievement: achievement_constraint(required_output_index, ai_productivity_gain),
                household_facts: HouseholdFacts {
                    imputed_rental_value,
                    receives_pension: pensioner,
                    insurance_premiums,
                    education_contribution,
                },
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
            canton,
        } => {
            run_comparison(CompareParams {
                salary,
                age,
                married,
                children,
                percentages: &percentages,
                custom_tax_rate,
                family_tax_mode,
                canton: canton.as_deref(),
            });
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

/// Resolve the `--canton` flag against the tax data that actually exists.
///
/// Before this, `--canton` was accepted, defaulted to `ZH`, and silently
/// ignored — `--canton ZH` produced Bern numbers. A flag that looks like it
/// changes the answer but does not is worse than an absent one, so it now
/// behaves in one of three explicit ways:
///
/// * **not supplied** → Bern, and say so. Bern is no longer the only priceable
///   canton, but it is the default for backward compatibility with earlier
///   output, and the message makes the choice visible rather than implicit.
/// * **supplied and priceable** → use it.
/// * **supplied but not priceable** → fail with exactly what is missing. This
///   is the objective's "fail loudly rather than silently falling back to
///   Bern" requirement, and it is enforced at the CLI boundary.
fn resolve_canton(requested: Option<&str>) -> Option<cantons::Canton> {
    let Some(code) = requested else {
        println!(
            "{}",
            "No --canton given; using Bern."
                .dimmed()
        );
        return Some(cantons::Canton::Bern);
    };

    let Some(canton) = cantons::Canton::from_code(code) else {
        eprintln!(
            "{} '{}' is not a Swiss canton code. Valid codes: {}",
            "error:".red().bold(),
            code,
            cantons::Canton::all_codes()
        );
        std::process::exit(2);
    };

    // `is_priceable`, not `data.is_priced()`: Bern is usable through its own
    // complete standalone rate table even though the two-level decomposition
    // (base scale x Steuerfuss) has not been done for it.
    if !cantons::is_priceable(canton) {
        let data = cantons::canton_tax_data(canton);
        let missing = data.missing_fields(canton);
        eprintln!(
            "{} cannot price canton {} ({}). Missing: {}.",
            "error:".red().bold(),
            canton.code(),
            canton.name(),
            missing.join(", ")
        );
        eprintln!();
        // Name only what is actually absent. Four cantons (BL, FR, GE, VS) have
        // an imported scale and are blocked by their *multiplier* instead, so a
        // fixed "the scale has not been supplied" sentence would be false for
        // them — and would send the reader looking for a file already present.
        eprintln!("  Cantonal tax = simple_tax(income) x Steuerfuss, where both sides");
        eprintln!(
            "  must come from the same year. For {}: {}",
            canton.code(),
            missing.join(", ")
        );
        if missing.contains(&"cantonal base tax scale") {
            eprintln!(
                "  The scale is imported from an ESTV \"Tarife\" export; this canton's \
                 export is"
            );
            eprintln!("  not in the repository yet.");
        }
        if missing.contains(&"cantonal Steuerfuss") {
            eprintln!(
                "  The multiplier is read from the ESTV Steuerfuss workbook, whose \
                 {} cell is",
                cantons::SELF_ASSESSMENT_YEAR
            );
            eprintln!("  not a plain number (blank, or a footnote this project will not guess at).");
        }
        if missing.contains(&"capital municipal Steuerfuss") {
            eprintln!("  The capital city's multiplier is missing from the same workbook.");
        }
        eprintln!();
        // List the cantons that *are* priced, derived rather than hard-coded:
        // the count has climbed from 3 to 22 in this repository's lifetime, and
        // a stale hint would send the user away from a canton that works.
        let priced: Vec<&str> = cantons::ALL_CANTONS
            .iter()
            .filter(|c| cantons::is_priceable(**c))
            .map(|c| c.code())
            .collect();
        eprintln!("  Two ways forward:");
        eprintln!(
            "    - use a canton that is priced ({}): {}",
            priced.len(),
            priced.join(", ")
        );
        eprintln!(
            "    - pass --custom-tax-rate with your observed rate from your tax \
             assessment (most accurate)"
        );
        eprintln!();
        eprintln!("  See SWISS_TAX_DATA.md §3 for the table format needed.");
        std::process::exit(2);
    }

    Some(canton)
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
    /// `--canton`. `None` when the flag was not supplied.
    canton: Option<&'a str>,
    conversion: monte_carlo::ConversionRateScenario,
    consumption_profile: consumption::ConsumptionProfileConfig,
    achievement: Option<optimizer::AchievementConstraint>,
    /// Extra household facts the deduction model uses; see [`HouseholdFacts`].
    household_facts: HouseholdFacts,
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
        canton,
        conversion,
        consumption_profile,
        achievement,
        household_facts,
    } = p;

    println!("\n{}", "=== LIFE OPTIMIZER ===".bold().cyan());
    println!("Finding optimal work percentage for your situation...\n");

    validate_ages(age, retirement_age, life_expectancy);

    // Resolve the schedule together with a short label for its basis, so the
    // "Tax Rate" line reports what actually produced the figure.
    let (mut tax_schedule, tax_basis): (TaxSchedule, String) =
        resolve_tax_schedule(canton, custom_tax_rate, married, children);
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

    // Deductions are the input every tax figure depends on, and two models
    // currently disagree about them by roughly a factor of five for a single
    // earner: the hand-entered ~35%-cap estimate in `tax.rs`, and the sourced
    // ESTV rules in `deductions.rs`. Both are printed so the difference is
    // visible on a real scenario rather than only in a test fixture.
    print_deduction_model_comparison(
        &tax_schedule,
        &tax_basis,
        optimal.gross_income,
        married,
        children,
        household_facts,
    );

    // Display work-life balance results, naming the tax basis that produced them
    // so the rate line cannot be read as a different canton's figures.
    display::print_optimal_result_for(&optimal, Some(&tax_basis));

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

    display::print_comparison_table_for(&all_scenarios, Some(&tax_basis));
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

/// Build the tax schedule for a run, and announce which basis produced it.
///
/// Returns the schedule alongside a short label naming its source, because the
/// figure is meaningless without it: "10.7%" is a different claim in Aargau than
/// in Bern, and the label is what makes the number checkable. Both the optimize
/// and comparison commands go through here so they cannot drift apart — the
/// comparison command previously always used Bern's table while printing a
/// `--canton` it had accepted.
fn resolve_tax_schedule(
    canton: Option<&str>,
    custom_tax_rate: Option<f64>,
    married: bool,
    children: u32,
) -> (TaxSchedule, String) {
    let Some(rate) = custom_tax_rate else {
        // Resolve the canton explicitly. This fails loudly for cantons whose tax
        // scale has not been loaded, rather than quietly returning Bern numbers
        // for a Zürich household.
        let canton = resolve_canton(canton).expect("resolve_canton exits on failure");
        if canton == cantons::Canton::Bern {
            // Bern's standalone Stadt Bern table is complete and in use.
            println!(
                "{}",
                "  Tax basis: Bern — official Stadt Bern rate table (2024).".dimmed()
            );
            return (
                TaxSchedule::bern_city_default(married, children),
                "official Bern table".to_string(),
            );
        }
        let Some(schedule) = TaxSchedule::from_canton_scale(canton, married, children) else {
            // `resolve_canton` only lets priceable cantons through, so reaching
            // here is an inconsistency in the registry rather than bad input.
            eprintln!(
                "{} canton {} ({}) passed the priceability check but produced \
                 no schedule. This is a bug in the canton registry, not your input.",
                "error:".red().bold(),
                canton.code(),
                canton.name()
            );
            eprintln!("  Use --custom-tax-rate, or omit --canton to use Bern.");
            std::process::exit(2);
        };

        // Name the tariff shape, rather than implying a band table was applied:
        // a flat-rate canton has no bands, and a formula canton (BL) publishes
        // algebraic expressions instead.
        let detail = match cantons::imported_scale(canton) {
            Some(scale) if scale.is_formula() => "ESTV formula tariff x Steuerfuss".to_string(),
            Some(scale) if scale.is_flat_rate() => match scale.flat_rate_percent {
                Some(rate) => format!("flat {rate}% x Steuerfuss"),
                None => "flat rate x Steuerfuss".to_string(),
            },
            _ => "ESTV scale x Steuerfuss".to_string(),
        };
        println!(
            "{}",
            format!(
                "  Tax basis: {} — {} ({}).",
                canton.name(),
                detail,
                canton.capital()
            )
            .dimmed()
        );
        println!(
            "  {}",
            "  Verify against your own tax assessment before relying on it.".yellow()
        );
        return (schedule, format!("{} {}", canton.code(), detail));
    };

    // An observed personal rate supersedes any canton table, so the canton is
    // not resolved in this branch -- resolving it would reject a run that does
    // not need cantonal data at all.
    println!("Using custom tax rate: {:.2}%\n", rate * 100.0);
    (
        TaxSchedule::custom_rate(rate),
        "your observed rate".to_string(),
    )
}

/// Everything `compare` needs. A struct rather than eight positional arguments,
/// which is what the equivalent `optimize` parameters already use.
struct CompareParams<'a> {
    salary: f64,
    age: u32,
    married: bool,
    children: u32,
    percentages: &'a str,
    custom_tax_rate: Option<f64>,
    family_tax_mode: bool,
    canton: Option<&'a str>,
}

/// The household facts the deduction model needs beyond income and family size.
///
/// A struct rather than three more positional parameters, and deliberately with
/// `None`/`false` defaults that mean *not supplied* — the deduction engine skips
/// a category whose facts are missing rather than assuming them, so a caller that
/// omits one gets a smaller deduction and a report, never a guess.
#[derive(Debug, Clone, Copy, Default)]
struct HouseholdFacts {
    /// `--imputed-rental-value`: the home's `Eigenmietwert`.
    imputed_rental_value: Option<f64>,
    /// `--pensioner`.
    receives_pension: bool,
    /// `--insurance-premiums`.
    insurance_premiums: Option<f64>,
    /// `--education-contribution`. Gates St. Gallen's conditional flat deduction.
    education_contribution: Option<f64>,
}

impl HouseholdFacts {
    /// Apply these facts to a household under construction.
    fn apply(self, household: &mut deductions::Household) {
        household.imputed_rental_value = self.imputed_rental_value;
        household.receives_pension = self.receives_pension;
        household.insurance_premiums = self.insurance_premiums;
        household.education_contribution = self.education_contribution;
    }
}

/// Print the sourced deduction assessment beside the hand-entered estimate.
///
/// Exists because the two models disagree materially and the direction matters:
/// the estimate overstates deductions, so it *understates* tax. Showing only one
/// number would hide that, and showing the sourced one alone would silently
/// change every figure the tool reports — which is a decision, not a detail.
///
/// Nothing downstream consumes the sourced figure yet; this is comparison only.
fn print_deduction_model_comparison(
    schedule: &TaxSchedule,
    tax_basis: &str,
    gross_income: f64,
    married: bool,
    children: u32,
    facts: HouseholdFacts,
) {
    // The federal rules always apply, and a canton's add to them. Both are shown
    // so the reader can see which half a figure came from.
    let mut household = deductions::Household::employee(gross_income, married, children);
    facts.apply(&mut household);
    let federal = deductions::assess("Bund", &household);
    let cantonal = canton_code_from_basis(tax_basis)
        .map(|code| deductions::assess(code, &household))
        .filter(|a| !a.applied.is_empty() || a.means_tested > 0.0);

    let estimate = schedule.standard_deduction_estimate(gross_income);

    println!("\n{}", "🧾 DEDUCTION MODEL COMPARISON".bold().cyan());
    println!("{}", "=".repeat(60));
    println!(
        "  Gross income: CHF {gross_income:.0}   ({tax_basis})"
    );
    println!(
        "  Estimated (in use): CHF {estimate:.0} ({:.1}%)",
        if gross_income > 0.0 { estimate / gross_income * 100.0 } else { 0.0 }
    );
    println!(
        "  Sourced (federal):  CHF {:.0} ({:.1}%)",
        federal.total(),
        federal.effective_rate() * 100.0
    );
    if let Some(cantonal) = &cantonal {
        println!(
            "  Sourced ({}):  CHF {:.0} ({:.1}%)",
            cantonal.jurisdiction,
            cantonal.total(),
            cantonal.effective_rate() * 100.0
        );
    }

    // The itemisation is the point: a total with no breakdown cannot be checked.
    for (label, assessment) in [("Bund", Some(&federal)), ("canton", cantonal.as_ref())] {
        let Some(assessment) = assessment else { continue };
        for applied in &assessment.applied {
            println!(
                "      {:>9.2}  [{}] {}",
                applied.amount,
                label,
                applied.name
            );
        }
        if assessment.means_tested > 0.0 {
            println!(
                "      {:>9.2}  [{}] means-tested deduction (phase-out scale)",
                assessment.means_tested, label
            );
        }
        // An income ADDITION, printed with a plus so it cannot be misread as
        // another deduction when the list above it is all minus-figures. Only a
        // homeowner has one today.
        if assessment.income_addition > 0.0 {
            println!(
                "      {:>+9.2}  [{}] imputed rental value added to taxable income \
                 (Eigenmietwert)",
                assessment.income_addition, label
            );
        }
    }

    // Anything considered and not applied is reported, so a small total reads as
    // "these facts were not supplied" rather than "there is nothing to deduct".
    let skipped = federal.skipped().len() + cantonal.as_ref().map_or(0, |a| a.skipped().len());
    println!(
        "  {skipped} further rule(s) were considered and not applied: the household \
         supplied none of the facts they depend on."
    );
    println!(
        "  {}",
        "  The sourced model is not yet used for the figures below; it is shown for \
         comparison."
            .yellow()
    );
}

/// The canton code a tax basis label refers to, if it names one.
///
/// The basis is a display string like `"ZH ESTV scale x Steuerfuss"` or
/// `"official Bern table"`, and the deduction engine needs a jurisdiction code.
/// Anything unrecognised yields `None`, which the caller treats as "no cantonal
/// deductions to show" rather than guessing a canton.
fn canton_code_from_basis(tax_basis: &str) -> Option<&str> {
    let token = tax_basis.split_whitespace().next()?;
    if token == "official" {
        return Some("BE");
    }
    if token.len() == 2 && token.chars().all(|c| c.is_ascii_uppercase()) {
        return Some(token);
    }
    None
}

fn run_comparison(p: CompareParams<'_>) {
    let CompareParams {
        salary,
        age,
        married,
        children,
        percentages: percentages_str,
        custom_tax_rate,
        family_tax_mode,
        canton,
    } = p;
    println!("\n{}", "=== SCENARIO COMPARISON ===".bold().cyan());
    
    let percentages: Vec<f64> = percentages_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Same resolution path as `optimize`, so `--canton` means the same thing in
    // both commands. It used to be accepted here and ignored, which meant a
    // comparison for Zürich was silently priced with Bern's table.
    let (mut tax_schedule, tax_basis) =
        resolve_tax_schedule(canton, custom_tax_rate, married, children);
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
    display::print_comparison_table_for(&scenarios, Some(&tax_basis));
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
        // Interactive mode does not ask for a canton; Bern is the working default.
        canton: None,
        conversion: monte_carlo::ConversionRateScenario::Statutory,
        consumption_profile: consumption::ConsumptionProfileConfig::default(),
        achievement: None,
        // The interactive prompt does not ask about these, and assuming them
        // would be exactly the guessing the deduction model refuses to do.
        household_facts: HouseholdFacts::default(),
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
