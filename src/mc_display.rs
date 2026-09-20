// Display module for Monte Carlo pension simulation results
use crate::monte_carlo::{MonteCarloResult, WorkPctComparison, RegimeSwitchingResult, PensionRange};
use colored::*;

/// Display the monthly pension across conversion-rate (Umwandlungssatz)
/// scenarios, plus the transparent range and the statutory-vs-actual gap.
///
/// The point of this section is to *remove* unjustified precision: the pension
/// a person actually receives depends on their fund's conversion rate, and many
/// Swiss funds apply materially less than the 6.8% statutory minimum.
pub fn print_conversion_rate_scenarios(range: &PensionRange, ahv_monthly: f64) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".yellow());
    println!("{}", "    MONTHLY PENSION BY CONVERSION RATE (Umwandlungssatz)".bold().yellow());
    println!("{}", "═══════════════════════════════════════════════════════════".yellow());

    println!("\n  {}", format!(
        "Based on projected BVG capital of CHF {:.0} at retirement in {}",
        range.capital, range.retirement_year
    ).dimmed());

    println!("\n  {:<28} {:>10} {:>16}", "Conversion rate scenario", "Rate", "Monthly BVG");
    println!("  {}", "─".repeat(56));
    println!("  {:<28} {:>10} {:>16}",
        "Statutory minimum (BVG)",
        format!("{:.2}%", crate::monte_carlo::STATUTORY_CONVERSION_RATE * 100.0),
        format!("CHF {:.0}", range.statutory));
    println!("  {:<28} {:>10} {:>16}",
        "Typical Swiss fund",
        format!("{:.2}%", crate::monte_carlo::TYPICAL_FUND_CONVERSION_RATE * 100.0),
        format!("CHF {:.0}", range.typical).yellow().to_string());
    println!("  {:<28} {:>10} {:>16}",
        format!("Projected for {}", range.retirement_year),
        format!("{:.2}%", range.future_rate * 100.0),
        format!("CHF {:.0}", range.future));

    if let Some(custom) = range.custom {
        println!("  {:<28} {:>10} {:>16}",
            "Your pension fund",
            format!("{:.2}%", range.selected_rate * 100.0),
            format!("CHF {:.0}", custom).green().bold().to_string());
    }
    println!("  {}", "─".repeat(56));

    println!("\n  {} {}",
        "Realistic range:".bold(),
        format!("CHF {:.0} – {:.0} per month", range.range.0, range.range.1).bold());

    if let Some(custom) = range.custom {
        let diff = range.difference_from_statutory(custom);
        if diff > 0.0 {
            println!("  {} {}",
                "Your rate is below the statutory minimum by".yellow(),
                format!("CHF {:.0}/month", diff).yellow().bold());
        } else if diff < 0.0 {
            println!("  {} {}",
                "Your rate is above the statutory minimum by".green(),
                format!("CHF {:.0}/month", diff.abs()).green().bold());
        }
    }

    println!("\n  {}", "Note on interpretation:".bold());
    println!("  {}", "  • The statutory rate understates the risk for most people:".dimmed());
    println!("  {}", format!(
        "    at the typical fund rate you receive CHF {:.0}/month less than the statutory figure.",
        range.statutory_minus_typical
    ).dimmed());
    println!("  {}", "  • Rates are projected to keep declining (rising life expectancy,".dimmed());
    println!("  {}", "    low rates, demographics), so the projection is the planning case.".dimmed());
    println!("  {}", "  • The actual amount depends on YOUR pension fund's rate.".dimmed());
    println!("  {}", "    Use --conversion-rate to compute with the precise figure.".dimmed());

    if ahv_monthly > 0.0 {
        println!("\n  {}", format!(
            "Add AHV (Pillar 1) of ~CHF {:.0}/month for total retirement income.",
            ahv_monthly
        ).dimmed());
    }

    println!("{}", "═══════════════════════════════════════════════════════════\n".yellow());
}

/// Report the pension distribution arising from conversion-rate uncertainty
/// (`FutureWork.md` §10.5, long-term item) with explicit downside risk.
///
/// Percentiles and CVaR are shown side by side because they answer different
/// questions: the P10 is the level you would exceed 90% of the time, while CVaR
/// is the average outcome *given* that you land in the worst decile — which is
/// what actually matters for a decision you cannot reverse.
pub fn print_conversion_rate_uncertainty(
    result: &crate::monte_carlo::StochasticConversionRateResult,
) {
    println!("\n{}", "───────────────────────────────────────────────────────────".dimmed());
    println!("{}", "CONVERSION-RATE UNCERTAINTY (stochastic)".bold());
    println!("{}", "───────────────────────────────────────────────────────────".dimmed());

    println!("  {}", format!(
        "Rate drawn ~ N({:.2}%, {:.2}pp), truncated to [{:.2}%, {:.2}%]",
        result.mean_rate * 100.0,
        result.rate_std_dev * 100.0,
        crate::monte_carlo::MIN_PROJECTED_CONVERSION_RATE * 100.0,
        crate::monte_carlo::STATUTORY_CONVERSION_RATE * 100.0,
    ).dimmed());

    println!("\n  {:<24} {:>14}", "Monthly BVG pension", "CHF");
    println!("  {}", "─".repeat(40));
    println!("  {:<24} {:>14}", "P10 (bad luck)", format!("{:.0}", result.p10_monthly));
    println!("  {:<24} {:>14}", "P25", format!("{:.0}", result.p25_monthly));
    println!("  {:<24} {:>14}", "Median".bold(), format!("{:.0}", result.median_monthly).bold());
    println!("  {:<24} {:>14}", "P75", format!("{:.0}", result.p75_monthly));
    println!("  {:<24} {:>14}", "P90 (good luck)", format!("{:.0}", result.p90_monthly));
    println!("  {}", "─".repeat(40));
    println!("  {:<24} {:>14}", "CVaR (worst 10%)".red(), format!("{:.0}", result.cvar_10_monthly).red());

    println!("\n  {}", format!(
        "Probability of falling below CHF {:.0}/month: {:.0}%",
        result.floor_monthly,
        result.prob_below_floor * 100.0
    ).bold());

    if result.prob_below_floor > 0.25 {
        println!("  {}", "  A material chance of falling short of your needs. Treat the".yellow());
        println!("  {}", "  median as optimistic and plan against the CVaR figure.".yellow());
    } else if result.prob_below_floor > 0.10 {
        println!("  {}", "  Some downside risk — verify your fund's actual rate.".yellow());
    } else {
        println!("  {}", "  Downside risk is limited under these assumptions.".green());
    }

    // CVaR is bounded from below by the truncation floor, so it can coincide
    // with the P10. Say so rather than letting the reader assume the two
    // statistics are independent.
    if (result.cvar_10_monthly - result.p10_monthly).abs() < 1.0 {
        println!("\n  {}", "  CVaR equals P10 here because the rate distribution is truncated at".dimmed());
        println!("  {}", "  the projection floor: the worst decile collapses onto that bound.".dimmed());
        println!("  {}", "  Truncation compresses both tails, so the true downside is somewhat".dimmed());
        println!("  {}", "  worse than this figure shows.".dimmed());
    }

    println!("{}", "───────────────────────────────────────────────────────────\n".dimmed());
}

pub fn print_monte_carlo_summary(
    conservative: &MonteCarloResult,
    base: &MonteCarloResult,
    optimistic: &MonteCarloResult,
    monthly_needs: f64,
    retirement_age: u32,
    life_expectancy: u32,
) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "    MONTE CARLO PENSION SIMULATION  (10,000 paths)".bold().cyan());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());

    println!("\n{}", "Scenarios:".bold());
    println!("  Conservative  — 2.0% real return, 6.0% volatility (60% bonds)");
    println!("  Base Case     — 3.2% real return, 8.5% volatility (balanced)");
    println!("  Optimistic    — 4.5% real return, 11.0% volatility (equity-heavy)");
    println!("  All include 1.5% annual inflation");

    println!("\n{}", format!(
        "  Retirement age: {}   |   Life expectancy: {}   |   {} years in retirement",
        retirement_age, life_expectancy, life_expectancy - retirement_age
    ).dimmed());

    println!("\n{}", "─── BVG Capital at Retirement (CHF) ─────────────────────────".bold());
    print_capital_table(conservative, base, optimistic);

    println!("\n{}", "─── Monthly Pension in TODAY'S CHF (inflation-adjusted) ─────".bold());
    println!("  {}", "BVG (occupational) annuity only — AHV is added in the section below.".dimmed());
    println!("  {:<20} {:>12} {:>12} {:>12}", "Scenario", "Pessimistic", "Median", "Optimistic");
    println!("  {:<20} {:>12} {:>12} {:>12}",
        "Conservative",
        format!("CHF {:.0}", conservative.p10_real_pension),
        format!("CHF {:.0}", conservative.median_real_pension),
        format!("CHF {:.0}", conservative.p90_real_pension),
    );
    println!("  {:<20} {:>12} {:>12} {:>12}",
        "Base Case",
        format!("CHF {:.0}", base.p10_real_pension),
        format!("CHF {:.0}", base.median_real_pension).green().to_string(),
        format!("CHF {:.0}", base.p90_real_pension),
    );
    println!("  {:<20} {:>12} {:>12} {:>12}",
        "Optimistic",
        format!("CHF {:.0}", optimistic.p10_real_pension),
        format!("CHF {:.0}", optimistic.median_real_pension),
        format!("CHF {:.0}", optimistic.p90_real_pension),
    );
    println!("  {}", format!("  Your estimated retirement needs: CHF {:.0}/month", monthly_needs).dimmed());

    println!("\n{}", "─── Total Monthly Income in Retirement ──────────────────────".bold());
    println!("  (BVG pension + AHV CHF {:.0}/month)", base.ahv_monthly);
    println!("  {:<20} {:>12} {:>12} {:>12}", "Scenario", "P10 (bad)", "Median", "P90 (good)");
    for (label, r) in [("Conservative", conservative), ("Base Case", base), ("Optimistic", optimistic)] {
        let median_str = format!("CHF {:.0}", r.median_total_monthly);
        let colored_median = if r.median_total_monthly >= monthly_needs {
            median_str.green().to_string()
        } else {
            median_str.yellow().to_string()
        };
        println!("  {:<20} {:>12} {:>12} {:>12}",
            label,
            format!("CHF {:.0}", r.p10_total_monthly),
            colored_median,
            format!("CHF {:.0}", r.p90_total_monthly),
        );
    }

    println!("\n{}", "─── Retirement Quality of Life (0 = poverty, 10 = excellent) ─".bold());
    println!("  {:<20} {:>12} {:>12} {:>12}", "Scenario", "Bad luck", "Median", "Good luck");
    for (label, r) in [("Conservative", conservative), ("Base Case", base), ("Optimistic", optimistic)] {
        let qol_str = format!("{:.1}/10", r.median_qol_score);
        let colored_qol = color_qol(&qol_str, r.median_qol_score);
        println!("  {:<20} {:>12} {:>12} {:>12}",
            label,
            format!("{:.1}/10", r.p10_qol_score),
            colored_qol,
            format!("{:.1}/10", r.p90_qol_score),
        );
    }

    println!("\n{}", "─── Sustainability Until Age 90 ─────────────────────────────".bold());
    for (label, r) in [("Conservative", conservative), ("Base Case", base), ("Optimistic", optimistic)] {
        let prob_ok = r.prob_adequate * 100.0;
        let prob_dep = r.prob_depletion * 100.0;
        let status = if prob_ok >= 80.0 {
            format!("{:.0}% adequate", prob_ok).green().to_string()
        } else if prob_ok >= 60.0 {
            format!("{:.0}% adequate", prob_ok).yellow().to_string()
        } else {
            format!("{:.0}% adequate", prob_ok).red().to_string()
        };
        let depletion_info = if prob_dep > 0.5 {
            format!(", {:.0}% risk capital depletes", prob_dep).red().to_string()
        } else {
            format!(", {:.0}% depletion risk", prob_dep).dimmed().to_string()
        };
        println!("  {:<20} {}{}", label, status, depletion_info);
    }

    println!("\n{}", "─── Key Insight ─────────────────────────────────────────────".bold());
    let base_qol = base.median_qol_score;
    if base_qol >= 7.0 {
        println!("  {} Pension is well-funded across most scenarios.", "✓".green().bold());
        println!("  You have flexibility to reduce work % and enjoy life now.");
    } else if base_qol >= 5.0 {
        println!("  {} Pension is borderline — monitor closely.", "⚠".yellow().bold());
        println!("  Consider: more Pillar 3a, or slight increase in work %.");
    } else {
        println!("  {} Pension is likely insufficient — action required!", "✗".red().bold());
        println!("  Increase work %, maximize Pillar 3a, or delay retirement.");
    }

    println!("{}", "═══════════════════════════════════════════════════════════\n".cyan());
}

fn print_capital_table(
    conservative: &MonteCarloResult,
    base: &MonteCarloResult,
    optimistic: &MonteCarloResult,
) {
    println!("  {:<20} {:>14} {:>14} {:>14}", "Scenario", "P10 (bad)", "Median", "P90 (good)");
    for (label, r) in [("Conservative", conservative), ("Base Case", base), ("Optimistic", optimistic)] {
        println!("  {:<20} {:>14} {:>14} {:>14}",
            label,
            format!("CHF {:.0}", r.p10_capital),
            format!("CHF {:.0}", r.median_capital).bold().to_string(),
            format!("CHF {:.0}", r.p90_capital),
        );
    }
}

fn color_qol(s: &str, score: f64) -> String {
    if score >= 7.0 {
        s.green().to_string()
    } else if score >= 5.0 {
        s.yellow().to_string()
    } else {
        s.red().to_string()
    }
}

/// Print comparison table: how work % affects pension quality
pub fn print_work_pct_pension_comparison(comparisons: &[WorkPctComparison], monthly_needs: f64) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "    WORK % vs PENSION QUALITY (Base Case, Median)".bold().cyan());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("  {:<10} {:>14} {:>14} {:>12} {:>12}",
        "Work %", "Contribution/yr", "Median Capital", "Pension/mo", "QoL Score");
    println!("  {}", "─".repeat(65));

    for c in comparisons {
        let r = &c.base;
        let pension_str = format!("CHF {:.0}", r.median_total_monthly);
        let colored_pension = if r.median_total_monthly >= monthly_needs {
            pension_str.green().to_string()
        } else {
            pension_str.yellow().to_string()
        };
        let qol_str = format!("{:.1}/10", r.median_qol_score);
        let colored_qol = color_qol(&qol_str, r.median_qol_score);

        println!("  {:<10} {:>14} {:>14} {:>12} {:>12}",
            format!("{:.0}%", c.work_pct * 100.0),
            format!("CHF {:.0}", r.annual_contribution),
            format!("CHF {:.0}", r.median_capital),
            colored_pension,
            colored_qol,
        );
    }

    println!("  {}", "─".repeat(65));
    println!("  {}", format!("Retirement needs: CHF {:.0}/month", monthly_needs).dimmed());
    println!("{}", "═══════════════════════════════════════════════════════════\n".cyan());
}

/// Display the regime-switching (recession/inflation-aware) simulation
pub fn print_regime_switching_result(result: &RegimeSwitchingResult, monthly_needs: f64) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".magenta());
    println!("{}", "     ECONOMIC REGIME-SWITCHING MODEL".bold().magenta());
    println!("{}", "     (Boom / Normal / Recession / Stagflation — Markov chain)".dimmed());
    println!("{}", "═══════════════════════════════════════════════════════════".magenta());

    println!("\n{}", "  This model simulates realistic clustering of economic cycles".dimmed());
    println!("{}", "  instead of assuming constant returns — recessions and inflation".dimmed());
    println!("{}", "  shocks happen in connected multi-year runs, as they do in reality.".dimmed());

    println!("\n{}", "─── Regime Frequencies (long-run) ────────────────────────────".bold());
    println!("  Boom: ~19%   Normal: ~62%   Recession: ~14%   Stagflation: ~5%");
    println!("  (calibrated to post-WWII developed-market business cycles)");

    println!("\n{}", "─── BVG Capital at Retirement ─────────────────────────────────".bold());
    println!("  P10 (bad luck):   CHF {:.0}", result.p10_capital);
    println!("  Median:           CHF {:.0}", result.median_capital);
    println!("  P90 (good luck):  CHF {:.0}", result.p90_capital);

    println!("\n{}", "─── Monthly Pension in TODAY'S CHF ────────────────────────────".bold());
    println!("  P10 (bad luck):   CHF {:.0}", result.p10_real_pension);
    let median_str = format!("CHF {:.0}", result.median_real_pension);
    let colored_median = if result.median_real_pension >= monthly_needs {
        median_str.green().to_string()
    } else {
        median_str.yellow().to_string()
    };
    println!("  Median:           {}", colored_median);
    println!("  P90 (good luck):  CHF {:.0}", result.p90_real_pension);
    println!("  {}", format!("  Target needs: CHF {:.0}/month", monthly_needs).dimmed());

    println!("\n{}", "─── Quality of Life & Sustainability ───────────────────────────".bold());
    println!("  QoL score:        {:.1}/10 median  ({:.1} bad luck — {:.1} good luck)",
        result.median_qol_score, result.p10_qol_score, result.p90_qol_score);
    let prob_ok = result.prob_adequate * 100.0;
    let status = if prob_ok >= 80.0 {
        format!("{:.0}%", prob_ok).green().to_string()
    } else if prob_ok >= 60.0 {
        format!("{:.0}%", prob_ok).yellow().to_string()
    } else {
        format!("{:.0}%", prob_ok).red().to_string()
    };
    println!("  Adequate outcome: {} of simulations meet retirement needs", status);
    println!("  Depletion risk:   {:.1}% chance capital runs out before target age", result.prob_depletion * 100.0);
    println!("  Recession years:  ~{:.1} years of recession/stagflation experienced during career (avg)",
        result.avg_recession_years_during_career);

    println!("{}", "═══════════════════════════════════════════════════════════\n".magenta());
}

/// Display the sequence-of-returns stress test (forced recession at retirement)
pub fn print_stress_test_result(result: &RegimeSwitchingResult, base_median_pension: f64, monthly_needs: f64) {
    println!("\n{}", "═══════════════════════════════════════════════════════════".red());
    println!("{}", "    STRESS TEST: RECESSION HITS RIGHT AT RETIREMENT".bold().red());
    println!("{}", "     (sequence-of-returns risk — the single biggest pension danger)".dimmed());
    println!("{}", "═══════════════════════════════════════════════════════════".red());

    println!("\n{}", "  Forces a 3-year recession/stagflation shock spanning the 2 years".dimmed());
    println!("{}", "  before retirement through 1 year after — worst-case timing.".dimmed());

    println!("\n{}", "─── Impact vs Normal (Regime-Switching Median) ─────────────────".bold());
    let pension_drop = (1.0 - result.median_real_pension / base_median_pension) * 100.0;
    println!("  Normal median pension:    CHF {:.0}/month", base_median_pension);
    println!("  Stress-test pension:      {}",
        format!("CHF {:.0}/month  ({:.0}% lower)", result.median_real_pension, pension_drop).red());
    println!("  Stress-test P10 (worst):  CHF {:.0}/month", result.p10_real_pension);

    println!("\n{}", "─── Sustainability Under Stress ─────────────────────────────────".bold());
    let prob_ok = result.prob_adequate * 100.0;
    let status = if prob_ok >= 70.0 {
        format!("{:.0}%", prob_ok).green().to_string()
    } else if prob_ok >= 50.0 {
        format!("{:.0}%", prob_ok).yellow().to_string()
    } else {
        format!("{:.0}%", prob_ok).red().to_string()
    };
    println!("  Still adequate:   {} of stress scenarios", status);
    println!("  Depletion risk:   {}",
        format!("{:.1}% chance capital runs out", result.prob_depletion * 100.0).red());
    println!("  {}", format!("  Target needs: CHF {:.0}/month", monthly_needs).dimmed());

    println!("\n{}", "    Mitigation:".yellow().bold());
    println!("     • Shift to more conservative allocation 3-5 years before retirement");
    println!("     • Keep 1-2 years of expenses in cash/bonds as a buffer");
    println!("     • Consider a flexible retirement date rather than a fixed one");
    println!("     • Pillar 3a withdrawal timing can be staggered across tax years");

    println!("{}", "═══════════════════════════════════════════════════════════\n".red());
}
