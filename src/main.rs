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
use life_optimizer::{tax, requirements, optimizer, display, monte_carlo, mc_display, consumption, cantons, deductions, early_retirement};
use life_optimizer::early_retirement::{BandedTax, ConsumptionNeed, ContributionPrinciple, Degree, Household, RetirementParameters, TaxBand, TaxTreatment};

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

    /// Optimise an early-retirement plan: when to stop, how to take the money out,
    /// and how much is enough
    ///
    /// Prices the decision rather than projecting a given plan. Projects the AHV and
    /// BVG entitlements at the retirement age, solves the tax-minimising allocation of
    /// the capital across the retirement years as a linear program, sweeps the
    /// retirement age under a Monte Carlo, and prints the four education-contribution
    /// ledgers side by side without ranking them. See EARLY_RETIREMENT.md.
    EarlyRetirement(Box<EarlyRetirementArgs>),
}

/// Parameters for `early-retirement`.
#[derive(clap::Args, Debug, Clone)]
struct EarlyRetirementArgs {
    /// Full-time annual salary in CHF
    #[arg(short, long, default_value = "120000")]
    salary: f64,

    /// Your current age
    #[arg(short, long, default_value = "45")]
    age: u32,

    /// Are you married?
    #[arg(short, long, action = ArgAction::Set, default_value_t = false)]
    married: bool,

    /// Number of children
    #[arg(short, long, default_value = "0")]
    children: u32,

    /// AHV contribution years already accrued
    #[arg(long, default_value = "25")]
    contribution_years: u32,

    /// Existing BVG retirement capital in CHF
    #[arg(long, default_value = "180000")]
    bvg_capital: f64,

    /// Existing Pillar 3a capital in CHF
    #[arg(long, default_value = "60000")]
    pillar3a_capital: f64,

    /// Taxable savings outside the pension wrappers, in CHF
    #[arg(long, default_value = "50000")]
    savings: f64,

    /// The consumption floor that must be funded, per year in CHF
    #[arg(long, default_value = "45000")]
    mandatory: f64,

    /// The lifestyle target the optimisation aims at, per year in CHF
    #[arg(long, default_value = "65000")]
    lifestyle: f64,

    /// The retirement age to plan at the centre of the report
    #[arg(long, default_value = "63")]
    retirement_age: u32,

    /// Life expectancy
    #[arg(long, default_value = "90")]
    life_expectancy: u32,

    /// Work percentage assumed up to retirement (e.g. 0.8 for 80%)
    #[arg(long, default_value = "1.0")]
    work_pct: f64,

    /// Conversion rate (Umwandlungssatz) your fund actually applies at the reference
    /// age, as a decimal. The statutory floor is 0.068, but the average rate applied
    /// across Swiss funds is about 0.052 — a plan priced at the floor overstates the
    /// annuity by roughly a quarter, so prefer your fund's own figure
    #[arg(long, default_value = "0.068")]
    conversion_rate: f64,

    /// Reduction in the conversion rate per year of early withdrawal. There is no
    /// statutory schedule: this is a fund-specific figure. Published funds fall near
    /// 0.0013 to 0.0027 a year, i.e. 0.13 to 0.27 percentage points
    #[arg(long, default_value = "0.002")]
    early_reduction: f64,

    /// Fraction of the BVG capital taken as an annuity rather than as capital
    #[arg(long, default_value = "1.0")]
    annuity_share: f64,

    /// Marginal rate applied to a capital withdrawal, as a decimal. Left unset, the
    /// progressive default bands are used, which is the shape a cantonal tariff takes and
    /// the only case in which spreading a withdrawal across years saves anything. Setting
    /// it replaces the schedule with that flat rate — which is what Zurich and Thurgau
    /// effectively levy, and under a flat rate spreading saves nothing
    #[arg(long)]
    capital_tax_rate: Option<f64>,

    /// Expected real return on invested capital, as a decimal
    #[arg(long, default_value = "0.02")]
    real_return: f64,

    /// Standard deviation of the real return, as a decimal
    #[arg(long, default_value = "0.10")]
    return_std: f64,

    /// Monte Carlo paths
    #[arg(long, default_value = "10000")]
    paths: usize,

    /// Base seed; the same seed reproduces the same paths
    #[arg(long, default_value = "20260101")]
    seed: u64,

    /// Qualification held, for the education ledger
    #[arg(long, default_value = "master")]
    degree: String,

    /// Career length in years, for the education ledger
    #[arg(long, default_value = "35")]
    career_years: u32,

    /// Write the risk table and the allocation as CSV into this directory
    #[arg(long)]
    export: Option<std::path::PathBuf>,
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
    /// for +25%. This is the *pessimistic* end of the range when a range is
    /// declared. Only meaningful with --required-output-index.
    #[arg(long, default_value = "0.0")]
    ai_productivity_gain: f64,

    /// Optimistic end of the AI productivity-gain range — e.g. 0.60 for +60%.
    /// Feasibility is still judged at the pessimistic end: a schedule that only
    /// works if AI delivers at the top of the range is reported as a bet on the
    /// tool rather than a credible reduction.
    #[arg(long)]
    ai_productivity_gain_high: Option<f64>,

    /// Share of the AI gain that survives verification and rework, in [0, 1].
    /// 1.0 (default) assumes AI output is usable as delivered.
    #[arg(long, default_value = "1.0")]
    ai_quality_retention: f64,

    /// Delivered quality lost per unit of pace above your sustainable rate.
    /// 0.0 (default) assumes output per hour can be raised freely.
    #[arg(long, default_value = "0.0")]
    compression_quality_sensitivity: f64,

    /// Replacement probability per unit of relative goal shortfall: 2.5 means a
    /// 10% shortfall implies a 25% chance of replacement. Only meaningful with
    /// --required-output-index.
    #[arg(long, default_value = "0.0")]
    replacement_risk: f64,

    /// Years that must be worked at full time before a reduction becomes
    /// credible. Averaged into the workload, so the leisure gain is not
    /// overstated.
    #[arg(long, default_value = "0.0")]
    evaluation_period_years: f64,

    /// What a missed goal does to the search: `strict` (default) does not offer
    /// the work percentage at all, `risk-weighted` offers it and prices the
    /// replacement probability into its security. Requires --replacement-risk.
    #[arg(long, default_value = "strict")]
    enforcement: String,

    /// Monthly debt repayment or other unavoidable contractual outflow, in CHF.
    /// Added to the mandatory floor: an instalment does not shrink when hours do.
    #[arg(long, default_value = "0.0")]
    monthly_debt: f64,
}

/// Optimise an early-retirement plan.
///
/// # What this prints, and in what order
///
/// The order is the argument of the whole document: the **parameters and where each comes
/// from** first, so a reader knows before seeing any number which ones are law and which
/// are declared; then the **entitlements** at the retirement age, with the bridge years
/// called out; then the **allocation** the LP chooses, with the LP checked against the
/// greedy fill; then the **risk** across retirement ages and the smallest pot that funds
/// the consumption to a stated confidence; and only then the **education ledgers**, which
/// are printed together because ranking them is not this program's business.
fn run_early_retirement(args: &EarlyRetirementArgs) {
    println!();
    println!("{}", "=".repeat(78));
    println!("EARLY RETIREMENT: WHEN TO STOP, HOW TO TAKE IT OUT, AND HOW MUCH IS ENOUGH");
    println!("{}", "=".repeat(78));
    println!("  This prices the decision rather than projecting a plan. The capital is");
    println!("  allocated across the retirement years to minimise tax by linear program,");
    println!("  and the retirement age is swept under a Monte Carlo over returns and");
    println!("  longevity. Every risk is printed as a number; none is summarised away.");

    // ---- 1. the parameters, before any result ---------------------------------
    // Built by struct update rather than by mutating a default, so that every override is
    // visible in one place and the compiler checks the field names.
    let capital_tax = match args.capital_tax_rate {
        Some(flat) => BandedTax {
            allowance: 0.0,
            bands: vec![TaxBand {
                width: 10_000_000.0,
                marginal_rate: flat,
            }],
            provenance: early_retirement::Provenance::Declared {
                rationale: "a flat rate passed on the command line",
            },
        },
        None => RetirementParameters::default().capital_withdrawal_tax,
    };
    let params = RetirementParameters {
        bvg_conversion_rate: args.conversion_rate,
        bvg_early_reduction_per_year: args.early_reduction,
        capital_withdrawal_tax: capital_tax,
        ..RetirementParameters::default()
    };

    println!();
    println!("  1. The parameters, and which of them are law");
    println!("  {}", "-".repeat(74));
    for (name, provenance) in params.provenance_table() {
        println!("    {:<38} {:<9} {}", name, provenance.label(), provenance.detail());
    }
    println!();
    println!("  `sourced` means a statutory figure at the named vintage. `varies` means the");
    println!("  RULE is law but there is no single number: the conversion rate's");
    println!("  early-withdrawal reduction and the capital-benefit tariff are both in that");
    println!("  class, so both are inputs here rather than assumptions buried in the code.");

    // ---- 2. entitlements -------------------------------------------------------
    let household = Household {
        current_age: args.age,
        full_time_salary: args.salary,
        married: args.married,
        children: args.children,
        contribution_years: args.contribution_years,
        bvg_capital_now: args.bvg_capital,
        pillar3a_capital_now: args.pillar3a_capital,
        taxable_savings: args.savings,
        life_expectancy: args.life_expectancy,
    };
    let entitlements = early_retirement::project(&household, &params, args.retirement_age, args.work_pct);

    println!();
    println!("  2. Entitlements at {}", args.retirement_age);
    println!("  {}", "-".repeat(74));
    println!(
        "    AHV:                  {:>12.0} a year  (record {:.0}%, early reduction {:.1}%)",
        entitlements.ahv_annual,
        (entitlements.ahv_annual
            / if args.married {
                params.ahv_max_annual_couple
            } else {
                params.ahv_max_annual_single
            })
            * 100.0,
        entitlements.ahv_early_reduction * 100.0
    );
    println!(
        "    BVG capital:          {:>12.0}          (conversion rate {:.3}%)",
        entitlements.bvg_capital,
        entitlements.effective_conversion_rate * 100.0
    );
    println!(
        "    BVG annuity if whole: {:>12.0} a year",
        entitlements.bvg_annuity_annual
    );
    println!("    Pillar 3a capital:    {:>12.0}", entitlements.pillar3a_capital);
    println!("    Taxable savings:      {:>12.0}", entitlements.bridge_capital);
    println!(
        "    Total capital:        {:>12.0}",
        entitlements.bvg_capital + entitlements.pillar3a_capital + entitlements.bridge_capital
    );
    let bridge_years = 65u32.saturating_sub(args.retirement_age);
    if bridge_years > 0 {
        println!();
        println!(
            "    THE BRIDGE: {} years between retirement and the AHV reference age carry no",
            bridge_years
        );
        println!(
            "    AHV at all, and a non-employed person still owes AHV contributions of"
        );
        println!(
            "    CHF {:.0} to {:.0} a year until the reference age (AHVG Art. 3 Abs. 1bis,",
            params.ahv_non_employed_annual_min, params.ahv_non_employed_annual_max
        );
        println!("    Art. 10). That gap is the largest single cost of retiring early.");
    }
    if args.conversion_rate <= crate::early_retirement::RetirementParameters::default().bvg_conversion_rate + 1e-9
    {
        println!();
        println!("  ON THE CONVERSION RATE: {:.3}% is at or below the statutory floor of 6.8%.", args.conversion_rate * 100.0);
        println!("  The floor is what the law guarantees, but the average rate funds actually");
        println!("  apply is about 5.2% (OAK BV, 1,257 funds). A plan priced at 6.8% overstates");
        println!("  the annuity by roughly a quarter, so pass your fund's own figure.");
    }

    // ---- 3. the allocation the LP chooses --------------------------------------
    let need = ConsumptionNeed {
        mandatory_annual: args.mandatory,
        lifestyle_annual: args.lifestyle,
    };
    let annuity_share = args.annuity_share.clamp(0.0, 1.0);
    let capital_to_place = entitlements.bvg_capital * (1.0 - annuity_share)
        + entitlements.pillar3a_capital;
    let annuity_income = entitlements.ahv_annual
        + entitlements.bvg_capital * annuity_share * entitlements.effective_conversion_rate;
    let years = args
        .life_expectancy
        .saturating_sub(args.retirement_age)
        .max(1) as usize;

    // The committed income each year, and therefore the withdrawal each year *requires*.
    // Before the AHV reference age there is no AHV in it, which is what makes the bridge
    // years expensive.
    let bridge_income: Vec<f64> = (0..years)
        .map(|offset| {
            let age = args.retirement_age + offset as u32;
            if age < 65 {
                annuity_income - entitlements.ahv_annual
            } else {
                annuity_income
            }
        })
        .collect();
    let required = early_retirement::required_withdrawals(&bridge_income, need.lifestyle_annual);
    let total_required: f64 = required.iter().sum();
    let surplus = capital_to_place - total_required;

    println!();
    println!("  3. The allocation: how the capital comes out");
    println!("  {}", "-".repeat(74));
    println!(
        "    Annuity income {:.0} a year, and a lifestyle target of {:.0}, so the withdrawal",
        annuity_income, need.lifestyle_annual
    );
    println!(
        "    REQUIRED is {:.0} over {} years — {:.0} of it in the {} bridge years before the",
        total_required,
        years,
        required
            .iter()
            .take(65u32.saturating_sub(args.retirement_age) as usize)
            .sum::<f64>(),
        bridge_years
    );
    println!("    AHV reference age.");
    println!(
        "    Capital to place {:.0}, leaving a surplus of {:.0} to spread for tax.",
        capital_to_place, surplus
    );
    if surplus < 0.0 {
        println!();
        println!("    THE POT DOES NOT FUND THIS CONSUMPTION AT THIS AGE. The requirement alone");
        println!("    exceeds the capital by {:.0}. No allocation fixes that: the answer is to", -surplus);
        println!("    retire later, spend less, or annuitise more. The risk table below reports");
        println!("    the consequence, which is a statement about arithmetic and not about luck.");
    }
    match early_retirement::optimise_withdrawals(
        surplus.max(0.0),
        &required,
        &params.capital_withdrawal_tax,
    ) {
        early_retirement::AllocationOutcome::Optimal(plan) => {
            let greedy = early_retirement::greedy_withdrawals(
                surplus.max(0.0),
                &required,
                &params.capital_withdrawal_tax,
            );
            println!();
            println!(
                "    tax on the surplus by linear program {:>10.0}    by greedy fill {:>10.0}    agreement {}",
                plan.total_tax,
                greedy.total_tax,
                if (plan.total_tax - greedy.total_tax).abs() < 1.0 {
                    "yes"
                } else {
                    "NO"
                }
            );
            println!(
                "    marginal rate of the next franc of surplus withdrawn: {:.4}",
                plan.marginal_rate_at_optimum
            );
            println!();
            println!(
                "    {:<5} {:>6} {:>12} {:>12} {:>12}",
                "year", "age", "required", "extra", "marginal"
            );
            let mut running = 0.0;
            for offset in 0..years {
                let extra = greedy.capital_withdrawn.get(offset).copied().unwrap_or(0.0);
                if extra <= 0.5 && required[offset] <= 0.5 {
                    continue;
                }
                running += extra;
                println!(
                    "    {:<5} {:>6} {:>12.0} {:>12.0} {:>12.4}",
                    offset + 1,
                    args.retirement_age + offset as u32,
                    required[offset],
                    extra,
                    params.capital_withdrawal_tax.marginal_rate_at(running)
                );
            }
            println!();
            println!("    The KKT reading: at the optimum no year offers a cheaper next franc");
            println!("    than the multiplier above. That is the precise form of \"spread the");
            println!("    withdrawal until the next franc costs the same wherever it goes\".");
            println!("    Note the requirement is funded FIRST and is not the LP's to trade");
            println!("    away: an allocation that spread the capital evenly to save tax would");
            println!("    leave the bridge years short, which is the failure the risk table");
            println!("    below exists to catch.");
        }
        early_retirement::AllocationOutcome::Infeasible(why) => {
            println!("    The allocation could not be solved: {why}");
        }
    }

    // ---- 4. risk, across retirement ages --------------------------------------
    let need = ConsumptionNeed {
        mandatory_annual: args.mandatory,
        lifestyle_annual: args.lifestyle,
    };
    let schedule = TaxSchedule::bern_city_default(args.married, args.children);

    println!();
    println!("  4. Risk, across retirement ages");
    println!("  {}", "-".repeat(74));
    println!(
        "    {} paths an age, the SAME seed everywhere, so the differences are the age",
        args.paths
    );
    println!("    and not the draws. Real return {:.1}% +/- {:.1}%, life expectancy {}.",
        args.real_return * 100.0, args.return_std * 100.0, args.life_expectancy);
    println!();
    println!(
        "    {:<5} {:>11} {:>11} {:>9} {:>11} {:>11} {:>10}",
        "age", "capital", "annuity", "shortfall", "exp. short", "median end", "tax"
    );

    let mut sweep: Vec<(u32, f64, f64, f64, f64, f64)> = Vec::new();
    let mut centred: Option<(f64, f64, f64, f64, f64)> = None;
    for age in 58..=67u32 {
        let projected = early_retirement::project(&household, &params, age, args.work_pct);
        let paths = early_retirement::draw_paths(
            args.paths,
            args.life_expectancy.saturating_sub(age).max(1),
            args.real_return,
            args.return_std,
            args.life_expectancy,
            age,
            args.seed,
        );
        let annuity_income = projected.ahv_annual
            + projected.bvg_capital
                * args.annuity_share.clamp(0.0, 1.0)
                * projected.effective_conversion_rate;
        let taxable = schedule.taxable_income_after_estimated_deductions(annuity_income);
        let annuity_tax = taxable * schedule.tax_rate_on_taxable(taxable);
        let profile = early_retirement::evaluate_strategy(
            age,
            args.annuity_share,
            &projected,
            &need,
            &TaxTreatment::new(params.capital_withdrawal_tax.clone(), annuity_tax),
            &paths,
            args.life_expectancy.saturating_sub(age).max(1),
        );
        println!(
            "    {:<5} {:>11.0} {:>11.0} {:>8.1}% {:>11.0} {:>11.0} {:>10.0}",
            age,
            projected.bvg_capital + projected.pillar3a_capital + projected.bridge_capital,
            annuity_income,
            profile.probability_of_shortfall * 100.0,
            profile.expected_shortfall,
            // The MEDIAN unconsumed wealth, not the mean. The mean is dominated by the
            // lucky paths, where a fixed real withdrawal leaves a fortune behind, and a
            // column reading "8,093,427" is not a statement about a retirement — it is a
            // statement about the right tail of the return draw.
            profile.median_terminal_wealth,
            profile.mean_lifetime_tax
        );
        if age == args.retirement_age {
            centred = Some((
                profile.probability_of_shortfall,
                profile.expected_shortfall,
                profile.mean_terminal_wealth,
                profile.median_terminal_wealth,
                profile.p10_terminal_wealth,
            ));
        }
        sweep.push((
            age,
            projected.bvg_capital + projected.pillar3a_capital + projected.bridge_capital,
            // The median, for the same reason the table prints it: the mean is the right
            // tail of the return draw rather than a statement about a retirement. The
            // first version exported the mean and the plotted figure duly showed unconsumed
            // wealth in the millions, which is how the inconsistency was found — the table
            // and the CSV were reporting different statistics under one name.
            profile.median_terminal_wealth,
            profile.probability_of_shortfall,
            profile.mean_lifetime_tax,
            annuity_income,
        ));
    }
    println!();
    println!("    `shortfall` is the share of paths in which the money ran out before the end");
    println!("    of life; `exp. short` is how large the gap was when it did. `median end` is");
    println!("    the MEDIAN unconsumed wealth at death, and it is a COST of over-saving, not a");
    println!("    safety margin — a plan that funds the consumption and still leaves a large");
    println!("    balance has saved too much, not wisely. The median rather than the mean");
    println!("    because a fixed real withdrawal leaves a fortune on the lucky paths, and a");
    println!("    mean that reads in the millions is a statement about the right tail of the");
    println!("    return draw rather than about a retirement.");

    // ---- 5. the least that is enough ------------------------------------------
    println!();
    println!("  5. The least that is enough");
    println!("  {}", "-".repeat(74));
    let target = 0.10;
    let paths = early_retirement::draw_paths(
        args.paths,
        args.life_expectancy
            .saturating_sub(args.retirement_age)
            .max(1),
        args.real_return,
        args.return_std,
        args.life_expectancy,
        args.retirement_age,
        args.seed,
    );
    let annuity_income = entitlements.ahv_annual
        + entitlements.bvg_capital * args.annuity_share.clamp(0.0, 1.0) * entitlements.effective_conversion_rate;
    let taxable = schedule.taxable_income_after_estimated_deductions(annuity_income);
    let annuity_tax = taxable * schedule.tax_rate_on_taxable(taxable);

    let shortfall_at = |extra: f64| -> f64 {
        let mut probe = entitlements;
        probe.bvg_capital += extra;
        let profile = early_retirement::evaluate_strategy(
            args.retirement_age,
            args.annuity_share,
            &probe,
            &need,
            &TaxTreatment::new(params.capital_withdrawal_tax.clone(), annuity_tax),
            &paths,
            args.life_expectancy
                .saturating_sub(args.retirement_age)
                .max(1),
        );
        profile.probability_of_shortfall
    };

    // Bisect on the extra capital, because the shortfall probability falls monotonically
    // in it. The upper bound is widened until it qualifies rather than assumed: a bound
    // that silently failed would report "no amount is enough".
    let mut high = 100_000.0_f64;
    let mut widened = 0;
    while shortfall_at(high) > target && widened < 12 {
        high *= 2.0;
        widened += 1;
    }
    if shortfall_at(high) > target {
        println!(
            "    Even {:.0} of extra capital does not bring the shortfall probability to",
            high
        );
        println!("    {:.0}% at this annuity share. The binding constraint is the annuity,", target * 100.0);
        println!("    not the pot: annuitising more is the lever that removes this risk.");
    } else {
        let mut low = 0.0_f64;
        for _ in 0..40 {
            let mid = 0.5 * (low + high);
            if shortfall_at(mid) > target {
                low = mid;
            } else {
                high = mid;
            }
        }
        let funded = entitlements.bvg_capital + entitlements.pillar3a_capital + entitlements.bridge_capital;
        println!(
            "    The smallest extra capital that holds the shortfall probability at or",
        );
        println!(
            "    below {:.0}% is {:.0}, against the {:.0} already held — a total of {:.0}.",
            target * 100.0,
            high,
            funded,
            funded + high
        );
        if high < 1.0 {
            println!("    Nothing extra is needed: the plan already meets the target.");
        }
        println!();
        println!("    The target is a DECLARED preference, not a standard. 10% was chosen");
        println!("    here; the same bisection answers any other, and the reason to state it");
        println!("    is that \"enough\" is a risk appetite before it is a number.");
    }

    if let Some((shortfall, expected, mean_terminal, median_terminal, p10)) = centred {
        println!();
        println!("  At the chosen age {} and annuity share {:.2}:", args.retirement_age, args.annuity_share);
        println!(
            "    shortfall {:.1}%, expected gap when it happens {:.0}, terminal wealth mean",
            shortfall * 100.0,
            expected
        );
        println!(
            "    {:.0} / median {:.0} / p10 {:.0}. The p10 is the one to plan against.",
            mean_terminal, median_terminal, p10
        );
    }

    // ---- 6. the education ledgers, printed together ----------------------------
    let degree = Degree::parse(&args.degree).unwrap_or_else(|| {
        eprintln!("warning: unknown --degree {:?}; using master", args.degree);
        Degree::Master
    });
    let (state_cost, cost_provenance) = degree.declared_state_cost();

    println!();
    println!("  6. What a qualification costs the state, and what the holder pays back");
    println!("  {}", "-".repeat(74));
    println!(
        "    Qualification: {} — declared state cost {:.0} ({})",
        degree.label(),
        state_cost,
        cost_provenance.label()
    );
    println!();
    println!(
        "    {:<28} {:>12} {:>14} {:>10}",
        "principle", "surcharge", "lifetime tax", "x over cost"
    );
    let principles = [
        ContributionPrinciple::AbilityToPay,
        ContributionPrinciple::EducationCostRecovery {
            state_cost,
            recovery_years: args.career_years.max(1),
        },
        ContributionPrinciple::BenefitReceived { premium_share: 0.10 },
        ContributionPrinciple::Flat { annual: 3_000.0 },
    ];
    for principle in principles {
        let ledger = early_retirement::contribution_ledger(
            principle,
            state_cost,
            args.salary,
            args.career_years,
            &schedule,
        );
        let name = match principle {
            ContributionPrinciple::AbilityToPay => "ability to pay (existing tax)",
            ContributionPrinciple::EducationCostRecovery { .. } => "recover the state outlay",
            ContributionPrinciple::BenefitReceived { .. } => "benefit received (10% premium)",
            ContributionPrinciple::Flat { .. } => "flat (3,000 a year)",
        };
        println!(
            "    {:<28} {:>12.0} {:>14.0} {:>9.1}x",
            name, ledger.principle_surcharge, ledger.lifetime_income_tax, ledger.tax_multiple_of_state_cost
        );
    }
    println!();
    println!("  THE MODEL DOES NOT RANK THESE, AND WILL NOT. Each is a defensible");
    println!("  distributive principle. What the table establishes is the calculable half of");
    println!("  the argument: under the ordinary progressive schedule a high earner already");
    println!("  repays the state's outlay several times over, so the question is not whether");
    println!("  they pay back but WHICH MULTIPLE is right — and that is a judgement, not a");
    println!("  derivation. A program that picked one would be presenting an argument as a");
    println!("  calculation. Note also that the flat charge is a smaller share of a high");
    println!("  income, which is what \"regressive in effect\" means.");

    // ---- 7. what this does not say --------------------------------------------
    println!();
    println!("  7. What this does not say");
    println!("  {}", "-".repeat(74));
    println!("  It does not forecast returns. The Monte Carlo draws a declared distribution in");
    println!("  real terms; it is not fitted and not a market view. It does not model your");
    println!("  fund's regulations beyond the two figures you passed. It does not model the");
    println!("  three-step limit on drawing second-pillar capital. And it takes the");
    println!("  consumption profile as given: change the profile and the answer changes,");
    println!("  which is the point rather than a defect.");

    // ---- export ---------------------------------------------------------------
    if let Some(dir) = &args.export {
        if std::fs::create_dir_all(dir).is_ok() {
            let mut csv = String::from(
                "age,capital,median_terminal_wealth,shortfall_probability,mean_tax,annuity_income\n",
            );
            for (age, capital, terminal, shortfall, tax, annuity) in &sweep {
                csv.push_str(&format!(
                    "{age},{capital:.2},{terminal:.2},{shortfall:.6},{tax:.2},{annuity:.2}\n"
                ));
            }
            let path = dir.join("early-retirement-risk.csv");
            match std::fs::write(&path, csv) {
                Ok(()) => println!("\n  EXPORTED for plotting: {}", path.display()),
                Err(error) => eprintln!("warning: could not write {}: {error}", path.display()),
            }
        }
    }

    println!("{}", "=".repeat(78));
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
                ai_productivity_gain_high,
                ai_quality_retention,
                compression_quality_sensitivity,
                replacement_risk,
                evaluation_period_years,
                enforcement,
                monthly_debt,
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
                employer: match EmployerInputs::new(EmployerInputs {
                    required_output_index,
                    ai_productivity_gain,
                    ai_productivity_gain_high,
                    ai_quality_retention,
                    compression_quality_sensitivity,
                    replacement_risk,
                    evaluation_period_years,
                    enforcement,
                }) {
                    Ok(inputs) => inputs,
                    Err(message) => {
                        eprintln!("{} {}", "error:".red().bold(), message);
                        std::process::exit(2);
                    }
                },
                monthly_debt,
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
        Commands::EarlyRetirement(args) => {
            run_early_retirement(&args);
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
                if c.ai_quality_retention < 1.0 {
                    println!(
                        "  That figure already assumes only {:.0}% of an AI gain survives \
                         verification and rework.",
                        c.ai_quality_retention * 100.0
                    );
                }
                if c.compression_quality_sensitivity > 0.0 {
                    println!(
                        "  It also includes the quality cost of producing above your \
                         sustainable pace."
                    );
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

/// The employer-side inputs, grouped because none of them means much alone.
///
/// Every field comes from `CRITICS_CURRENT_WORK.md` §1.4. [`Default`] reproduces
/// the pre-critique behaviour exactly: no constraint at all, no quality drag, no
/// replacement risk, no evaluation period, strict enforcement. A household that
/// supplies only `--required-output-index` therefore gets the same answer it got
/// before these knobs existed.
#[derive(Debug, Clone)]
struct EmployerInputs {
    required_output_index: Option<f64>,
    ai_productivity_gain: f64,
    ai_productivity_gain_high: Option<f64>,
    ai_quality_retention: f64,
    compression_quality_sensitivity: f64,
    replacement_risk: f64,
    evaluation_period_years: f64,
    enforcement: String,
}

impl Default for EmployerInputs {
    fn default() -> Self {
        Self {
            required_output_index: None,
            ai_productivity_gain: 0.0,
            ai_productivity_gain_high: None,
            ai_quality_retention: 1.0,
            compression_quality_sensitivity: 0.0,
            replacement_risk: 0.0,
            evaluation_period_years: 0.0,
            enforcement: "strict".to_string(),
        }
    }
}

impl EmployerInputs {
    /// Validate the raw flag values, so a typo is refused rather than silently
    /// changing the model.
    ///
    /// The parameters worth rejecting are the ones that would *quietly* produce a
    /// different answer: a retention outside `[0, 1]`, a negative coefficient
    /// (all of which are clamped internally, and a clamp is not a correction), and
    /// an enforcement string that is not recognised — that last one would
    /// otherwise fall back to strict and look like the user's choice.
    fn new(raw: EmployerInputs) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&raw.ai_quality_retention) {
            return Err(format!(
                "--ai-quality-retention must be between 0 and 1, got {}",
                raw.ai_quality_retention
            ));
        }
        for (flag, value) in [
            ("--compression-quality-sensitivity", raw.compression_quality_sensitivity),
            ("--replacement-risk", raw.replacement_risk),
            ("--evaluation-period-years", raw.evaluation_period_years),
            ("--ai-productivity-gain", raw.ai_productivity_gain),
        ] {
            if value < 0.0 {
                return Err(format!("{flag} cannot be negative, got {value}"));
            }
        }
        if let Some(high) = raw.ai_productivity_gain_high {
            if high < 0.0 {
                return Err(format!(
                    "--ai-productivity-gain-high cannot be negative, got {high}"
                ));
            }
            if high < raw.ai_productivity_gain {
                return Err(format!(
                    "--ai-productivity-gain-high ({high}) is below the pessimistic end \
                     ({}); the range must bracket the pessimistic value",
                    raw.ai_productivity_gain
                ));
            }
        }
        if optimizer::Enforcement::parse(&raw.enforcement).is_none() {
            return Err(format!(
                "--enforcement must be `strict` or `risk-weighted`, got {:?}",
                raw.enforcement
            ));
        }
        // `risk-weighted` exists to price a missed goal. With no risk declared the
        // pricing is vacuous and the mode quietly becomes "ignore the required
        // output": every affordable percentage is feasible, so the utility search
        // takes the *lowest* one. For a portfolio that no percentage can deliver,
        // the report then recommends 50% work and states in the next line that the
        // goals are undeliverable at every percentage. That is not a trade-off, so
        // the combination is refused rather than renamed.
        if optimizer::Enforcement::parse(&raw.enforcement)
            == Some(optimizer::Enforcement::RiskWeighted)
            && raw.replacement_risk <= 0.0
        {
            return Err(
                "--enforcement risk-weighted requires --replacement-risk above 0. The \
                 mode's whole function is to price a missed goal, and with nothing \
                 declared the required output stops constraining the search at all. \
                 Use --enforcement strict to refuse undelivered schedules outright."
                    .to_string(),
            );
        }
        Ok(raw)
    }

    fn enforcement(&self) -> optimizer::Enforcement {
        optimizer::Enforcement::parse(&self.enforcement)
            .unwrap_or(optimizer::Enforcement::Strict)
    }

    /// Build the employer-side constraint. Absent a required output index it is
    /// not applied, preserving the previous behaviour in which work percentage is
    /// fully discretionary.
    fn constraint(&self) -> Option<optimizer::AchievementConstraint> {
        let required = self.required_output_index?;
        let mut constraint = optimizer::AchievementConstraint::new(required, self.ai_productivity_gain)
            .with_quality(
                self.ai_quality_retention,
                self.compression_quality_sensitivity,
            )
            .with_replacement_risk(self.replacement_risk)
            .with_evaluation_period(self.evaluation_period_years);
        if let Some(high) = self.ai_productivity_gain_high {
            constraint = constraint.with_ai_gain_range(high);
        }
        Some(constraint)
    }
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
    /// Employer-side inputs; see [`EmployerInputs`].
    employer: EmployerInputs,
    /// `--monthly-debt`: unavoidable contractual outflows, added to the
    /// mandatory floor (`CRITICS_CURRENT_WORK.md` §2.1).
    monthly_debt: f64,
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
        employer,
        monthly_debt,
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

    let mut requirements = PersonalRequirements::bern_family_default(children);
    // Debt repayment is unavoidable and does not shrink when hours do, so it joins
    // the mandatory floor rather than the discretionary tier (§2.1).
    requirements.debt_repayment = monthly_debt.max(0.0);

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

    let achievement = employer.constraint();
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
    config.enforcement = employer.enforcement();
    config.achievement = achievement;

    let optimizer = LifeOptimizer::new(config);
    let candidates = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let outcome = optimizer
        .search_outcome(&candidates)
        .expect("candidate list is non-empty");
    let (optimal, all_scenarios) = (outcome.scenario.clone(), outcome.all_scenarios.clone());
    let feasible_found = outcome.feasible_found;

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
    display::print_optimal_result_for(&optimal, Some(&tax_basis), feasible_found);

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
        employer: EmployerInputs::default(),
        monthly_debt: 0.0,
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




