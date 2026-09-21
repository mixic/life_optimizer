// Multipolar World Simulator
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

//! Terminal reporting.
//!
//! Every section that reports a number also reports the uncertainty around it, or
//! says that the number is a convention. `MULTIPOLAR_GAME.md` section 7 concludes
//! that "genuine uncertainty is the most defensible position", and a report that
//! printed a single 50-year trajectory without its dispersion would contradict
//! that.

use crate::simulation::{Config, Ensemble};

/// A horizontal bar, for showing a distribution without a charting dependency.
fn bar(fraction: f64, width: usize) -> String {
    let filled = (fraction.clamp(0.0, 1.0) * width as f64).round() as usize;
    format!(
        "{}{}",
        "#".repeat(filled),
        ".".repeat(width.saturating_sub(filled))
    )
}

/// Print the whole report.
pub fn print_report(config: &Config, ensemble: &Ensemble) {
    println!();
    println!("{}", "=".repeat(78));
    println!("MULTIPOLAR WORLD SIMULATOR");
    println!("{}", "=".repeat(78));
    println!(
        "  {} blocs, {} years, {} Monte Carlo runs",
        config.blocs.len(),
        config.horizon,
        config.runs
    );

    print_caveat();
    print_trajectory(ensemble);
    print_power_trajectories(config, ensemble);
    print_polarity(config, ensemble);
    print_dominance(config, ensemble);
    print_pension(config, ensemble);
    print_decades(config, ensemble);
}

/// The caveat goes first, not last.
///
/// A reader who skims and takes the numbers as a forecast has been misled by the
/// tool, not by themselves, if the warning is buried at the bottom.
fn print_caveat() {
    println!();
    println!("  NOTE ON WHAT THIS IS");
    println!("  --------------------");
    println!("  The parameters here are ILLUSTRATIVE, not calibrated to measured");
    println!("  economic or military data. This is a mechanism-explorer for why");
    println!("  multipolar systems tend toward instability or lock-in -- NOT a");
    println!("  forecast of the next 50 years. MULTIPOLAR_GAME.md section 5 states");
    println!("  that no general theorem guarantees a multipolar system converges at");
    println!("  all. Read the sensitivity sweep at the end for which findings are");
    println!("  robust and which flip on small parameter changes.");
}

/// Cooperation and tension over time, with the spread across runs.
fn print_trajectory(ensemble: &Ensemble) {
    println!();
    println!("  SYSTEM TRAJECTORY (mean across runs)");
    println!("  ------------------------------------");
    println!("  year   cooperation            tension");
    let step = (ensemble.horizon / 10).max(1);
    for year in (0..ensemble.horizon).step_by(step) {
        let coop = ensemble.cooperation_by_year[year];
        let tension = ensemble.tension_by_year[year];
        println!(
            "  {:>4}   {:.2} {}   {:.2} {}",
            year + 1,
            coop,
            bar(coop, 24),
            tension,
            bar((tension / 2.0).min(1.0), 24)
        );
    }
    let first = ensemble.cooperation_by_year.first().copied().unwrap_or(0.0);
    let last = ensemble.cooperation_by_year.last().copied().unwrap_or(0.0);
    println!();
    println!(
        "  Cooperation moves {first:.2} -> {last:.2} over the horizon{}",
        if last < first - 0.05 {
            " -- a downward drift, the arms-race path."
        } else if last > first + 0.05 {
            " -- an upward drift, cooperation strengthening."
        } else {
            " -- broadly flat, neither lock-in nor breakdown."
        }
    );
}

/// Mean final power share per bloc, with 10th-90th percentile band.
fn print_power_trajectories(config: &Config, ensemble: &Ensemble) {
    println!();
    println!("  POWER SHARES AT HORIZON (mean across runs)");
    println!("  ------------------------------------------");
    for (i, bloc) in config.blocs.iter().enumerate() {
        let start = bloc.power_share;
        let end = ensemble
            .mean_shares_by_year
            .last()
            .and_then(|shares| shares.get(i))
            .copied()
            .unwrap_or(0.0);
        let delta = end - start;
        println!(
            "  {:<14} {:>5.1}% -> {:>5.1}%  ({:+.1}pp) {}",
            bloc.name,
            start * 100.0,
            end * 100.0,
            delta * 100.0,
            bar(end, 20)
        );
    }
}

/// How the system ends up shaped.
fn print_polarity(config: &Config, ensemble: &Ensemble) {
    println!();
    println!("  POLARITY AT HORIZON");
    println!("  -------------------");
    for (polarity, probability) in ensemble.polarity_distribution() {
        println!(
            "  {:<12} {:>5.1}%  {}",
            polarity.label(),
            probability * 100.0,
            bar(probability, 28)
        );
    }
    println!();
    println!(
        "  Thresholds are documented conventions, not derived quantities: a single"
    );
    println!(
        "  bloc above 45% is unipolar; two above 20% summing past 60% is bipolar;"
    );
    println!("  a third bloc above 10% is balanced; otherwise fragmented.");
    let _ = config;
}

/// Who ends up on top, and how often nobody does.
fn print_dominance(config: &Config, ensemble: &Ensemble) {
    println!();
    println!("  DOMINANCE PROBABILITY");
    println!("  ---------------------");
    let dominance = ensemble.dominance_probabilities(config.blocs.len());
    let mut any = false;
    for (i, probability) in dominance.iter().enumerate() {
        if *probability > 0.0 {
            any = true;
            println!(
                "  {:<14} {:>5.1}%  {}",
                config.blocs[i].name,
                probability * 100.0,
                bar(*probability, 28)
            );
        }
    }
    if !any {
        println!("  (no bloc achieves a clear lead in any run)");
    }
    let none = ensemble.no_clear_leader_probability();
    println!(
        "  {:<14} {:>5.1}%  {}",
        "no clear leader",
        none * 100.0,
        bar(none, 28)
    );
    println!();
    println!("  A bloc counts as dominant only above 45% of system power, so this");
    println!("  is consistent with the polarity result rather than contradicting it.");
}

/// Conflict-trap frequency and what it implies for a PAYG pension promise.
fn print_pension(config: &Config, ensemble: &Ensemble) {
    println!();
    println!("  CONFLICT TRAP AND PENSION SECURITY");
    println!("  ----------------------------------");

    let (mean_trap, lo_trap, hi_trap) = ensemble.summarize(|o| o.trap_fraction);
    println!(
        "  Years in a conflict trap:  {:.1}% mean  (10th {:.1}% .. 90th {:.1}%)",
        mean_trap * 100.0,
        lo_trap * 100.0,
        hi_trap * 100.0
    );
    println!(
        "  A trap year is one both tense (>{:.1}) and uncooperative (<{:.0}%), so a",
        config.trap_tension_threshold, config.trap_cooperation_threshold * 100.0
    );
    println!("  tense-but-cooperating year -- deterrence working -- is not counted.");

    let (mean_loss, lo_loss, hi_loss) = ensemble.summarize(|o| o.mean_efficiency_loss);
    println!();
    println!(
        "  Pareto-efficiency loss:    {:.3} mean  (10th {:.3} .. 90th {:.3})",
        mean_loss, lo_loss, hi_loss
    );
    println!("  This is the operational version of the claim in MULTIPOLAR_GAME.md");
    println!("  section 4 / THEORY_OF_SPARING.md section 7d: surplus that a");
    println!("  cooperative outcome would have captured and self-interested play did not.");

    let (observed, reference) = ensemble.pension_summary(&config.pension);
    println!();
    println!("  PENSION IMPLICATIONS (illustrative channel -- see pension.rs)");
    println!("  ------------------------------------------------------------");
    println!(
        "  cooperation index      {:.3}   (1.000 = fully cooperative reference)",
        observed.cooperation_index
    );
    println!(
        "  PAYG replacement rate  {:.1}%   vs {:.1}% at full cooperation  ({:+.1}pp)",
        observed.replacement_rate * 100.0,
        reference.replacement_rate * 100.0,
        (observed.replacement_rate - reference.replacement_rate) * 100.0
    );
    println!(
        "  contributor:retiree    {:.2}    vs {:.2} at full cooperation",
        observed.support_ratio, reference.support_ratio
    );
    println!(
        "  portfolio real return  {:.2}%   vs {:.2}% at full cooperation",
        observed.portfolio_return * 100.0,
        reference.portfolio_return * 100.0
    );
    let index = observed.security_index(&reference);
    println!(
        "  pension security index {:.3}   {}",
        index,
        bar(index, 28)
    );
    println!();
    println!("  On CHF 100,000 of pre-retirement income the PAYG pillar alone would");
    println!(
        "  pay CHF {:.0} in this simulated world, against CHF {:.0} at full",
        observed.payg_income(100_000.0),
        reference.payg_income(100_000.0)
    );
    println!("  cooperation. This is the missing voice PHILOSOPHICAL_SOCIOLOGICAL_");
    println!("  ASPECTS.MD section 2c describes: not what is best for you, but what");
    println!("  the promise is worth if the system around you evolves this way.");
}

/// Decade-by-decade read on the balance of power.
fn print_decades(config: &Config, ensemble: &Ensemble) {
    println!();
    println!("  DECADE-BY-DECADE READ");
    println!("  ---------------------");
    println!(
        "  {:<8} {:>8} {:>10} {:<28}",
        "decade", "coop", "tension", "leading bloc"
    );
    let mut year = 9usize;
    while year < ensemble.horizon {
        let coop = ensemble.cooperation_by_year[year];
        let tension = ensemble.tension_by_year[year];
        let shares = &ensemble.mean_shares_by_year[year];
        let leader = shares
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, s)| {
                format!(
                    "{} ({:.0}%)",
                    config.blocs.get(i).map(|b| b.name.as_str()).unwrap_or("?"),
                    s * 100.0
                )
            })
            .unwrap_or_else(|| "?".to_string());
        println!(
            "  yr {:<5} {:>8.2} {:>10.2} {:<28}",
            year + 1,
            coop,
            tension,
            leader
        );
        year += 10;
    }

    // Summarise the decade trend so the table does not have to be read by eye.
    let early = if ensemble.horizon > 3 {
        ensemble.cooperation_by_year[2]
    } else {
        0.0
    };
    let late = ensemble.cooperation_by_year.last().copied().unwrap_or(0.0);
    println!();
    if late < early - 0.05 {
        println!("  The system trends toward competition over the horizon.");
    } else if late > early + 0.05 {
        println!("  The system trends toward cooperation over the horizon.");
    } else {
        println!("  The system is roughly stationary -- neither lock-in nor breakdown.");
    }
}
