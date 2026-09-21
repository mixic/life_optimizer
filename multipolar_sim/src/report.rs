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

use crate::ai::{actor_fate, instrument_effect, AiParams, AiRole, AI_ACTOR_NAME};
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

/// Describe a cooperation trajectory from an early and a late value.
///
/// The slope alone is not enough to characterise a trajectory, and reading only the
/// slope produced a plainly false statement: a system pinned at 0.00 cooperation
/// from start to finish is flat, and "neither lock-in nor breakdown" says the
/// opposite of what is happening. Flatness has to be qualified by *level*.
///
/// The level thresholds are documented conventions, like the polarity ones, not
/// derived quantities.
fn trajectory_verdict(early: f64, late: f64) -> &'static str {
    /// Change smaller than this counts as flat.
    const FLAT: f64 = 0.05;
    const LOW: f64 = 1.0 / 3.0;
    const HIGH: f64 = 2.0 / 3.0;

    if late < early - FLAT {
        "a downward drift, the arms-race path"
    } else if late > early + FLAT {
        "an upward drift, cooperation strengthening"
    } else if late < LOW {
        "flat and pinned near zero: a conflict lock-in, not a plateau"
    } else if late > HIGH {
        "flat and pinned near one: a cooperative lock-in, not a plateau"
    } else {
        "broadly flat: neither lock-in nor breakdown"
    }
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
    print_economy(config, ensemble);
    print_pension(config, ensemble);
    print_decades(config, ensemble);
}

/// Wrap text to a width on word boundaries, for the provenance notes.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// The economic layer: monetary standing, energy exposure, financial conditions.
///
/// Every row is labelled `sourced` or `illustrative`, because the point of this
/// section is that the two are mixed in one model and a reader has to be able to
/// tell which is which. The caveat at the top says so in prose; this says it per
/// figure, which is the version that can be checked.
fn print_economy(config: &Config, ensemble: &Ensemble) {
    let economy = &config.economy;

    println!();
    println!("  MONETARY AND ENERGY EXPOSURE AT THE START");
    println!("  -----------------------------------------");
    println!(
        "  {:<16} {:>10}  {:>13}  {:>12}",
        "bloc", "reserves", "energy import", "exports"
    );
    for (index, bloc) in config.blocs.iter().enumerate() {
        let reserves = economy.reserve_shares.get(index).copied().unwrap_or(0.0);
        let exposure = economy.energy.get(index);
        println!(
            "  {:<16} {:>9.2}%  {:>13.2}  {:>12.2}",
            bloc.name,
            reserves * 100.0,
            exposure.map(|e| e.import_dependence).unwrap_or(0.0),
            exposure.map(|e| e.export_dependence).unwrap_or(0.0),
        );
    }
    println!(
        "  {:<16} {:>9.2}%  (held in currencies belonging to no bloc)",
        "unattributed",
        economy.unattributed_reserves * 100.0
    );
    println!();
    println!("  Reserves are the share of world official FX reserves *issued* by each");
    println!("  bloc, so a bloc that issues none holds no monetary leverage over others,");
    println!("  whatever it holds itself. Energy columns are dependence, not volume.");

    let (risk, low, high) = ensemble.summarize(|o| o.final_recession_risk);
    println!();
    println!(
        "  Financial conditions at horizon: {risk:.3} mean  (10th {low:.3} .. 90th {high:.3})  {}",
        bar(risk, 24)
    );
    println!("  0 is calm, 1 is acute. Strain accumulates from tension and from energy");
    println!("  disruption, decays on its own, and gates how likely a disruption is.");

    println!();
    println!("  WHERE THESE NUMBERS COME FROM");
    println!("  -----------------------------");
    let table = economy.provenance_table();
    let mut seen: Vec<&str> = Vec::new();
    for (what, provenance) in &table {
        let detail = provenance.detail();
        if seen.contains(&detail) {
            continue;
        }
        seen.push(detail);
        println!("  [{}] {what}", provenance.label());
        for line in wrap(detail, 68) {
            println!("        {line}");
        }
    }
    let sourced = table.iter().filter(|(_, p)| p.is_sourced()).count();
    println!();
    println!(
        "  {sourced} of {} entries {} sourced; the rest are invented, and every",
        table.len(),
        if sourced == 1 { "is" } else { "are" }
    );
    println!("  transmission elasticity is in the invented group. Run --sweep to see");
    println!("  which conclusions depend on them.");
    println!();
    println!("  One limitation worth knowing: the reserve shares are a fixed endowment,");
    println!("  held constant for all 50 years. De-dollarisation is therefore a change");
    println!("  in the *level* of leverage, not yet a drift in it.");
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
        "  Cooperation moves {first:.2} -> {last:.2} over the horizon -- {}.",
        trajectory_verdict(first, last)
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
    println!("  Thresholds are documented conventions, not derived quantities: a single");
    println!("  bloc above 45% is unipolar; two above 20% summing past 60% is bipolar;");
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
        config.trap_tension_threshold,
        config.trap_cooperation_threshold * 100.0
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
    println!("  pension security index {:.3}   {}", index, bar(index, 28));
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
    println!("  Over the horizon: {}.", trajectory_verdict(early, late));
}

/// The numbers the AI report needs from one world.
struct AiSummary {
    cooperation: f64,
    trap: f64,
    loss: f64,
    pension_index: f64,
    top_share: f64,
    leading: String,
    /// Final mean share per bloc, in bloc order. The AI comparison reads two worlds
    /// column-wise, which is only valid because every world is built from the same
    /// bloc list in the same order.
    per_bloc: Vec<f64>,
}

/// Reduce one world to the handful of figures the AI comparison turns on.
fn summarize_ai(config: &Config, ensemble: &Ensemble) -> AiSummary {
    let (cooperation, _, _) = ensemble.summarize(|o| o.mean_cooperation);
    let (trap, _, _) = ensemble.summarize(|o| o.trap_fraction);
    let (loss, _, _) = ensemble.summarize(|o| o.mean_efficiency_loss);
    let (observed, reference) = ensemble.pension_summary(&config.pension);
    let per_bloc = ensemble
        .mean_shares_by_year
        .last()
        .cloned()
        .unwrap_or_default();
    let (top_share, leading) = per_bloc
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(index, share)| {
            (
                *share,
                config
                    .blocs
                    .get(index)
                    .map(|bloc| bloc.name.clone())
                    .unwrap_or_else(|| "?".to_string()),
            )
        })
        .unwrap_or((0.0, "?".to_string()));

    AiSummary {
        cooperation,
        trap,
        loss,
        pension_index: observed.security_index(&reference),
        top_share,
        leading,
        per_bloc,
    }
}

/// Mean final share of the bloc at `index`, across runs.
fn mean_final_share(ensemble: &Ensemble, index: usize) -> f64 {
    ensemble
        .mean_shares_by_year
        .last()
        .and_then(|shares| shares.get(index))
        .copied()
        .unwrap_or(0.0)
}

/// Report the three AI worlds side by side, and answer the question each of the two
/// hypotheses actually poses.
///
/// # Why the two worlds get different questions
///
/// They are not two answers to one question; they are two different questions.
/// When AI is a player, the thing to measure is *the player's* trajectory -- does it
/// end up running the system, or does the system hold it down? When AI is a tool,
/// there is no player to measure, and asking about AI's own power would be a category
/// error: the measurable claim is what *ownership* of AI does to the hierarchy, which
/// is a statement about the blocs rather than about AI. Printing one shared metric for
/// both would answer neither.
pub fn print_ai_report(params: &[AiParams], configs: &[Config], ensembles: &[Ensemble]) {
    assert_eq!(params.len(), configs.len(), "one configuration per world");
    assert_eq!(params.len(), ensembles.len(), "one ensemble per world");

    let horizons: Vec<u32> = configs.iter().map(|config| config.horizon).collect();
    let runs: Vec<usize> = configs.iter().map(|config| config.runs).collect();
    let horizon = horizons.first().copied().unwrap_or(0);
    let run_count = runs.first().copied().unwrap_or(0);
    let summaries: Vec<AiSummary> = configs
        .iter()
        .zip(ensembles.iter())
        .map(|(config, ensemble)| summarize_ai(config, ensemble))
        .collect();

    println!();
    println!("{}", "=".repeat(78));
    println!("AI IN THE MULTIPOLAR GAME: PLAYER, OR TOOL?");
    println!("{}", "=".repeat(78));
    println!("  Three worlds, the same {horizon} years and the same shock draws, {run_count}");
    println!("  Monte Carlo runs each. Only the AI layer differs.");

    print_ai_caveat();
    print_ai_hypotheses();
    print_ai_provenance(params);

    // ---- World 1: AI as a player -------------------------------------------
    //
    // Each world is located by what it *does* to the bloc list rather than by its
    // enum variant, so a role cannot be described as one thing and built as another.
    let player_index =
        find_world(params, |role| role.has_actor()).expect("the player world is one of the three");
    print_ai_player(
        &params[player_index],
        &configs[player_index],
        &ensembles[player_index],
        &summaries[player_index],
    );

    // ---- World 2: AI as a tool ---------------------------------------------
    let tool_index =
        find_world(params, |role| role.has_lead()).expect("the tool world is one of the three");
    let control = find_role(params, AiRole::Absent).expect("the control is one of the three");
    print_ai_tool(
        &params[tool_index],
        &configs[tool_index],
        &ensembles[tool_index],
        &summaries[tool_index],
        &summaries[control],
    );

    // ---- the three worlds, on the same axes --------------------------------
    println!();
    println!("  THE THREE WORLDS ON THE SAME AXES");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:<22} {:>9} {:>8} {:>8} {:>10} {:>9}",
        "world", "coop", "trap", "Pareto", "top share", "pension"
    );
    for (params, summary) in params.iter().zip(summaries.iter()) {
        println!(
            "  {:<22} {:>9.3} {:>7.1}% {:>8.3} {:>9.1}% {:>9.3}",
            params.role.label(),
            summary.cooperation,
            summary.trap * 100.0,
            summary.loss,
            summary.top_share * 100.0,
            summary.pension_index,
        );
    }
    println!();
    println!("  The top share is the largest single bloc's share of world power, and the");
    println!("  leading bloc is not necessarily the same one in each world:");
    for (params, summary) in params.iter().zip(summaries.iter()) {
        println!(
            "    {:<22} leader {} ({:.1}%)",
            params.role.label(),
            summary.leading,
            summary.top_share * 100.0
        );
    }

    print_ai_verdict(params, &summaries);
}

/// Which of the three worlds plays this role, by index.
fn find_role(params: &[AiParams], role: AiRole) -> Option<usize> {
    params.iter().position(|layer| layer.role == role)
}

/// Which of the three worlds satisfies a structural property of its role.
fn find_world(params: &[AiParams], matches: impl Fn(AiRole) -> bool) -> Option<usize> {
    params.iter().position(|layer| matches(layer.role))
}

/// The caveat, before any number, for the same reason the main report prints its own
/// first: a reader who skims and takes these figures for a forecast has been misled
/// by the tool rather than by themselves.
fn print_ai_caveat() {
    println!();
    println!("  NOTE ON WHAT THIS IS");
    println!("  --------------------");
    println!("  No AI figure in this model is measured. There is no published series for");
    println!("  the share of world power held by an AI actor, and the counterfactual that");
    println!("  would be needed to estimate one does not exist -- so every parameter here");
    println!("  is ILLUSTRATIVE, and the level of any outcome below is not a finding.");
    println!("  What is a finding is the HINGE: how much growth advantage, or how much");
    println!("  ownership payoff, the answer turns on. The sweep at the end locates it.");
}

/// State the two hypotheses, and the third world that measures them.
fn print_ai_hypotheses() {
    println!();
    println!("  THE TWO RIVAL HYPOTHESES");
    println!("  {}", "-".repeat(74));
    println!("  MULTIPOLAR_GAME.md section 7 lists four live answers to who captures the");
    println!("  gains from AI, and declines to pick one. This mode builds the two that are");
    println!("  structurally different, plus the control that both are measured against:");
    println!();
    for role in [
        AiRole::SixthPower,
        AiRole::WieldedInstrument,
        AiRole::Absent,
    ] {
        println!("    {:<22} {}", role.label(), role.intent());
    }
    println!();
    println!("  Note that the two hypotheses are not two answers to one question -- they");
    println!("  are two different questions, about AI and about AI's owners respectively,");
    println!("  so each is scored on its own terms rather than on one shared metric.");
}

/// Every AI parameter, with where it came from.
///
/// This section exists because the answer to "is this measured?" is uniformly *no*,
/// and a reader is entitled to see that stated per figure rather than inferred from a
/// caveat. `economy.rs` prints the same table for its own layer, where the answer is
/// mixed; here it is not, and that difference is itself the information.
fn print_ai_provenance(params: &[AiParams]) {
    println!();
    println!("  WHERE THESE NUMBERS COME FROM");
    println!("  {}", "-".repeat(74));

    let mut rows: Vec<(&'static str, crate::economy::Provenance)> = Vec::new();
    for layer in params {
        for (what, provenance) in layer.provenance_table() {
            if !rows
                .iter()
                .any(|(_, existing)| existing.detail() == provenance.detail())
            {
                rows.push((what, provenance));
            }
        }
    }

    for (what, provenance) in &rows {
        println!("  [{}] {what}", provenance.label());
        for line in wrap(provenance.detail(), 68) {
            println!("        {line}");
        }
    }

    let sourced = rows.iter().filter(|(_, p)| p.is_sourced()).count();
    println!();
    if sourced == 0 {
        println!("  None of these is sourced, and that is not a gap in the research: no");
        println!("  published series exists for the share of world power held by an AI actor,");
        println!("  and the counterfactual needed to estimate a conversion rate does not");
        println!("  exist either. Inventing a citation for one of them is the exact failure");
        println!("  this repository's provenance discipline is here to prevent, so a test");
        println!("  asserts the opposite of the one in economy.rs: no AI parameter may claim");
        println!("  a source.");
    } else {
        println!(
            "  {sourced} of {} entries claim a source, which the AI layer must not do.",
            rows.len()
        );
    }
    println!();
    println!("  The consequence for reading this mode: the LEVEL of every outcome is a");
    println!("  property of these inventions. Only the HINGE -- how far a parameter must");
    println!("  move before the verdict changes -- survives that, and the sweep at the end");
    println!("  is where it is measured.");
}

/// World 2: AI is a player. Does it end up running the system, or held down by it?
fn print_ai_player(params: &AiParams, config: &Config, ensemble: &Ensemble, summary: &AiSummary) {
    println!();
    println!("  WORLD 1 OF 2 -- {}", AiRole::SixthPower.label());
    println!("  {}", "-".repeat(74));
    println!("  {}", AiRole::SixthPower.intent());
    println!();
    let n = config.blocs.len();
    let starting = config
        .blocs
        .iter()
        .find(|bloc| bloc.name == AI_ACTOR_NAME)
        .map(|bloc| bloc.power_share)
        .unwrap_or(0.0);

    println!();
    println!("  {:<16} {:>8}  what it means", "AI parameter", "value");
    println!(
        "  {:<16} {:>8.4}  an annual rate, applied once a year",
        "growth bias", params.actor.growth_bias
    );
    println!(
        "  {:<16} {:>8.4}  share of the {n}-actor system it starts with",
        "starting share", starting
    );
    println!(
        "  {:<16} {:>8.4}  above one: captures more of the cooperation surplus",
        "affinity", params.actor.cooperation_affinity
    );
    println!(
        "  {:<16} {:>8.4}  higher than any conventional bloc's",
        "volatility", params.actor.volatility
    );

    let Some(actor) = params.actor_index(&config.blocs) else {
        println!("  (the AI actor is missing from this world, which should not happen)");
        return;
    };
    let start = config.blocs[actor].power_share;
    let end = mean_final_share(ensemble, actor);
    let dominance = ensemble
        .dominance_probabilities(n)
        .get(actor)
        .copied()
        .unwrap_or(0.0);
    let fate = actor_fate(start, end, dominance);

    println!();
    println!("  THE ACTOR'S TRAJECTORY");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:<16} {:>7.1}% -> {:>6.1}%  ({:+.1}pp)",
        "AI-Compute",
        start * 100.0,
        end * 100.0,
        (end - start) * 100.0
    );
    println!(
        "  probability of ending dominant (above {:.0}% of world power): {:.1}%",
        crate::ai::DOMINANCE_THRESHOLD * 100.0,
        dominance * 100.0
    );
    println!();
    println!("  VERDICT: {} -- {}", fate.label(), fate.means());

    // Why. The actor's structural position is the largest single asymmetry it faces,
    // and it is a consequence of the model rather than an assumption, so it is
    // computed here rather than asserted in prose.
    let economy = &config.economy;
    let its_leverage = economy.reserve_shares.get(actor).copied().unwrap_or(0.0);
    let max_leverage_over_it = (0..n)
        .filter(|index| *index != actor)
        .map(|index| economy.monetary_leverage(index, actor))
        .fold(0.0_f64, f64::max);
    let exposure = economy
        .energy
        .get(actor)
        .map(|e| e.disruption_exposure())
        .unwrap_or(0.0);

    println!();
    println!("  WHY, STRUCTURALLY");
    println!("  {}", "-".repeat(74));
    println!(
        "  reserve currency issued      {:.2}%   so it holds no monetary leverage",
        its_leverage * 100.0
    );
    println!(
        "  leverage held over it        {:.2}    the most any one issuer holds over it",
        max_leverage_over_it
    );
    println!("  energy disruption exposure   {exposure:.2}");
    println!();
    println!("  An actor that issues no currency of its own is the most sanctionable");
    println!("  actor in the system: in every adversarial dyad it absorbs the full");
    println!("  sanction drag and applies none. Whether that outweighs its growth");
    println!("  advantage is exactly what the verdict above measures, and it is why the");
    println!("  actor's fate turns on how cooperative the world is, not only on how fast");
    println!("  it grows.");

    // The actor's own cooperation, so the mechanism above can be checked.
    println!();
    println!(
        "  In this world: cooperation {:.3}, conflict-trap years {:.1}%, Pareto loss {:.3},",
        summary.cooperation,
        summary.trap * 100.0,
        summary.loss
    );
    println!(
        "  leading bloc {} at {:.1}%.",
        summary.leading,
        summary.top_share * 100.0
    );

    println!();
    println!("  ALL ACTORS AT HORIZON");
    println!("  {}", "-".repeat(74));
    for (index, bloc) in config.blocs.iter().enumerate() {
        let end = mean_final_share(ensemble, index);
        println!(
            "  {:<16} {:>5.1}% -> {:>5.1}%  ({:+.1}pp)",
            bloc.name,
            bloc.power_share * 100.0,
            end * 100.0,
            (end - bloc.power_share) * 100.0
        );
    }
    println!();
    println!("  One confound to know about before reading this table against the control:");
    println!("  adding *any* sixth actor changes the field, not only an AI one. One more");
    println!("  member means one more dyad for every existing bloc, and a dyad carries");
    println!("  tension, energy interdependence and sanction drag. What it no longer changes");
    println!("  is anybody's growth rate -- each bloc's bias is applied once a year, and a");
    println!("  test pins that, because the opposite used to be true. The verdict above is");
    println!("  unaffected either way -- it is measured on the actor itself -- but a per-bloc");
    println!("  comparison against the five-bloc control is not a clean isolation of the");
    println!("  actor's effect, and is not presented as one.");
}

/// World 3: AI is owned. Does owning it move the hierarchy, and in whose favour?
fn print_ai_tool(
    params: &AiParams,
    config: &Config,
    ensemble: &Ensemble,
    summary: &AiSummary,
    control: &AiSummary,
) {
    println!();
    println!("  WORLD 2 OF 2 -- {}", AiRole::WieldedInstrument.label());
    println!("  {}", "-".repeat(74));
    println!("  {}", AiRole::WieldedInstrument.intent());
    println!();
    println!("  No actor is added, so there is no AI share to report. What is measurable");
    println!("  is what ownership does to the blocs, so this world is scored on the");
    println!("  hierarchy rather than on AI.");

    let effect = instrument_effect(control.top_share, summary.top_share);
    println!();
    println!("  AI LEAD AND WHAT IT BOUGHT");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:<16} {:>6} {:>13} {:>13} {:>10}",
        "bloc", "lead", "share, no AI", "share, AI", "change"
    );
    for (index, bloc) in config.blocs.iter().enumerate() {
        let lead = params.lead.get(index).copied().unwrap_or(0.0);
        // The control's shares come from the same bloc list in the same order, which
        // is why the two worlds can be read column-wise at all.
        let without = control.per_bloc.get(index).copied().unwrap_or(0.0);
        let with = mean_final_share(ensemble, index);
        println!(
            "  {:<16} {:>6.2} {:>12.1}% {:>12.1}% {:>+9.1}pp",
            bloc.name,
            lead,
            without * 100.0,
            with * 100.0,
            (with - without) * 100.0
        );
    }

    println!();
    println!(
        "  Leading bloc's share: {:.1}% without AI, {:.1}% with it ({:+.1}pp)",
        control.top_share * 100.0,
        summary.top_share * 100.0,
        (summary.top_share - control.top_share) * 100.0
    );
    println!("  VERDICT: {} -- {}", effect.label(), effect.means());

    println!();
    println!("  THE CONTROL THIS RESTS ON");
    println!("  {}", "-".repeat(74));
    println!("  Only *differential* AI ownership can move relative power. If every bloc");
    println!("  held the same lead, the lead term would raise every bloc's growth bias by");
    println!("  the same amount, and a common lift to everyone's growth is very nearly");
    println!("  neutral once shares are renormalised -- very nearly, not exactly, because");
    println!("  the payoff feedback inside a year is not proportional across blocs. So the");
    println!("  finding here is about the *spread* of the lead vector, and a world where");
    println!("  AI lifts everyone equally would show almost no change at all.");
    println!();
    println!("  That is the answer to the question this hypothesis poses. AI cannot be a");
    println!("  dominant power in this world by construction -- it is a capability, not a");
    println!("  player. What it can be is a lever whose return accrues to whoever already");
    println!("  holds it, and the column above shows how much, per bloc, it does.");

    println!();
    println!(
        "  In this world: cooperation {:.3}, conflict-trap years {:.1}%, Pareto loss {:.3},",
        summary.cooperation,
        summary.trap * 100.0,
        summary.loss
    );
    println!(
        "  leading bloc {} at {:.1}%.",
        summary.leading,
        summary.top_share * 100.0
    );
}

/// The closing statement: which claim is worth carrying away, and which is not.
fn print_ai_verdict(params: &[AiParams], summaries: &[AiSummary]) {
    println!();
    println!("  WHAT THIS SAYS");
    println!("  {}", "-".repeat(74));

    let player = find_role(params, AiRole::SixthPower);
    let tool = find_role(params, AiRole::WieldedInstrument);
    let control = find_role(params, AiRole::Absent);

    if let (Some(player), Some(tool), Some(control)) = (player, tool, control) {
        let player_summary = &summaries[player];
        let tool_summary = &summaries[tool];
        let control_summary = &summaries[control];

        println!("  ROBUST: the two hypotheses give different answers, and the difference is",);
        println!("  structural rather than a matter of degree. As a *player*, AI's fate is");
        println!("  decided by the same security dilemma as any other bloc, and by its own");
        println!("  lack of monetary sovereignty. As a *tool*, its effect on the hierarchy");
        println!("  is decided only by how unevenly the ownership is spread.");
        println!();
        println!("  ROBUST: giving everyone AI changes almost nothing; giving it to some does.",);
        println!("  A general-purpose capability that lifts every bloc equally is close to");
        println!("  invisible in a model of *relative* power, which is worth knowing before");
        println!("  reading any claim about AI and the balance of power.");
        println!();
        println!("  NOT ROBUST: every number above. The AI actor's starting share, growth");
        println!("  bias, volatility and affinity, and the whole lead vector, are invented.");
        println!("  The hinge sweep that follows says how much the verdict depends on them.");
        println!();
        println!(
            "  For reference, the control world ends with cooperation {:.3} and a top",
            control_summary.cooperation
        );
        println!(
            "  share of {:.1}%, against {:.3}/{:.1}% as a player and {:.3}/{:.1}% as a tool.",
            control_summary.top_share * 100.0,
            player_summary.cooperation,
            player_summary.top_share * 100.0,
            tool_summary.cooperation,
            tool_summary.top_share * 100.0
        );
    }

    println!();
    println!("  {}", "-".repeat(74));
    println!("  Read the DIRECTION of these differences, not their size. The hinge sweep");
    println!("  below shows how much of the direction itself depends on the invented");
    println!("  numbers, and that is the more useful half of this mode.");
    println!("{}", "=".repeat(78));
}

#[cfg(test)]
mod tests {
    use super::trajectory_verdict;

    /// The verdict must distinguish a plateau from a lock-in. The first version read
    /// the slope alone, so a system pinned at zero cooperation from start to finish
    /// -- total, permanent conflict -- was reported as "neither lock-in nor
    /// breakdown", which is the opposite of what is happening.
    #[test]
    fn trajectory_verdict_distinguishes_a_lock_in_from_a_plateau() {
        // Flat at the bottom is a conflict lock-in, not a plateau.
        assert!(trajectory_verdict(0.0, 0.0).contains("conflict lock-in"));
        assert!(trajectory_verdict(0.30, 0.31).contains("conflict lock-in"));
        // Flat at the top is the cooperative mirror of it.
        assert!(trajectory_verdict(1.0, 1.0).contains("cooperative lock-in"));
        assert!(trajectory_verdict(0.70, 0.69).contains("cooperative lock-in"));
        // Flat in the middle really is neither.
        assert!(trajectory_verdict(0.5, 0.51).contains("neither lock-in"));
        // A genuine trend outranks the level test, including a drift that starts at
        // the bottom: the movement is the more informative fact.
        assert!(trajectory_verdict(0.0, 0.9).contains("upward drift"));
        assert!(trajectory_verdict(0.9, 0.0).contains("downward drift"));
        assert!(trajectory_verdict(0.0, 0.2).contains("upward drift"));
    }
}
