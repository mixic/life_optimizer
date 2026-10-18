// knowledge_work -- just-in-time logic in AI-assisted knowledge work
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

//! The command-line face of the workflow model: the envelope, the thresholds, the year the
//! workflow leaves them, and the direction analysis that refuses to rank.
//!
//! Everything printed comes from the same functions the tests check, and every parameter is
//! declared. The report therefore carries its own limits on its face rather than in a footnote,
//! because a reader who takes a level from it as a forecast has been misled by the presentation
//! and not by the numbers.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use human_capacity as capacity;
use knowledge_work::direction::{self, Outcome, CRITERIA};
use knowledge_work::envelope;
use knowledge_work::workflow::{self, Policy, WorkflowParameters};

/// The declared autonomy floor, shared with the psychological model's own report.
const AUTONOMY_FLOOR: f64 = 0.15;
/// The person the report follows: well trained, modest self-knowledge, under a demanding bar.
const PERSON_CAPACITY: f64 = 0.95;
const PERSON_SELF_MODEL: f64 = 0.40;
const PERSON_HUMAN_SHARE: f64 = 0.55;
const PERSON_CEILING: f64 = 1.70;
/// The settlement the weight grid is drawn at. 495 weightings over five criteria.
const WEIGHT_STEPS: usize = 8;

/// One configuration of the same workflow: a policy plus how the period is spent.
#[derive(Debug, Clone, Copy, PartialEq)]
struct OptionSpec {
    label: &'static str,
    policy: Policy,
    training_share: f64,
}

/// The three configurations compared. They are three answers to "what happens to the slack the
/// machine created", and the third is the one the framework exists to make visible: the buffer
/// spent on capacity to judge rather than on throughput.
fn options() -> [OptionSpec; 3] {
    [
        OptionSpec {
            label: "lean (push everything)",
            policy: Policy::Push,
            training_share: 0.05,
        },
        OptionSpec {
            label: "buffered (WIP cap)",
            policy: Policy::WipCap,
            training_share: 0.05,
        },
        OptionSpec {
            label: "learning buffer",
            policy: Policy::WipCap,
            training_share: 0.25,
        },
    ]
}

/// One option, evaluated: what it does to the person and what the workflow does with it.
struct OptionResult {
    spec: OptionSpec,
    review_capacity: f64,
    envelope: workflow::Envelope,
    outcome: Outcome,
}

fn evaluate_option(spec: OptionSpec, wf: &WorkflowParameters) -> OptionResult {
    let params = capacity::ModelParameters {
        training_share: spec.training_share,
        expectation_ceiling: PERSON_CEILING,
        ..capacity::ModelParameters::default()
    };
    let states = capacity::simulate(
        PERSON_CAPACITY,
        PERSON_SELF_MODEL,
        &params,
        capacity::Strategy::Tracking,
        PERSON_HUMAN_SHARE,
    );
    let review_capacity = states.last().map(|state| state.capacity).unwrap_or(0.0);
    let autonomy_years = states
        .iter()
        .filter(|state| state.autonomy >= AUTONOMY_FLOOR)
        .count() as f64;

    // A quarter of the period spent learning is a quarter not spent reviewing, so the review
    // capacity the workflow gets is scaled by the time that is actually available. Without this
    // the learning option would be free, which is the error the whole framework is about.
    let mut theirs = wf.clone();
    theirs.review_rate *= (1.0 - spec.training_share).max(0.0);
    let envelope = workflow::evaluate(&theirs, spec.policy, review_capacity, &params);

    let escaped_defects = envelope.admitted_per_day * theirs.defect_share * envelope.escape;
    let outcome = Outcome {
        throughput: envelope.admitted_per_day,
        // An unstable configuration has no mean cycle time; the sentinel is the deadline, which
        // is the only honest answer and is reported as a failure rather than a duration.
        cycle_time: if envelope.stable {
            envelope.cycle_time_days
        } else {
            theirs.deadline_days * 10.0
        },
        escaped_defects,
        autonomy_years,
        backlog: envelope.unadmitted_per_day,
    };
    OptionResult {
        spec,
        review_capacity,
        envelope,
        outcome,
    }
}

fn main() {
    let mut export: Option<PathBuf> = None;
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut index = 0;
    while index < raw.len() {
        match raw[index].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--export" => match raw.get(index + 1) {
                Some(path) => {
                    export = Some(PathBuf::from(path));
                    index += 2;
                }
                None => {
                    eprintln!("warning: --export needs a directory path");
                    index += 1;
                }
            },
            other => {
                eprintln!("warning: ignoring unrecognised argument {other:?}");
                index += 1;
            }
        }
    }

    report(export.as_deref());
}

fn print_usage() {
    println!(
        "\
Just-in-time logic in AI-assisted knowledge work

USAGE:
  knowledge_work [--export <dir>]

Reports the operating envelope of a review workflow: the utilisation beyond which
productivity gains become fragility, how much of the machine's output the workflow
can absorb, and the year it leaves the envelope as the capacity to judge erodes.

  --export <dir>   write the CSV the figures are drawn from
  --help           this message

Every parameter is declared, not calibrated; see OPTIMIZING_FOR_WHAT.md."
    );
}

fn report(export: Option<&Path>) {
    let wf = WorkflowParameters::default();
    let params = capacity::ModelParameters::default();

    print_header(&wf);

    // ---- the load identity at full capacity --------------------------------
    let at_full = envelope::envelope_at(&wf, 1.0, &params);
    println!();
    println!("  THE LOAD IDENTITY");
    println!("  {}", "-".repeat(74));
    println!(
        "  One generated item consumes review time in three ways: reviewed, then either"
    );
    println!(
        "  reworked because the defect was caught, or reworked later because it escaped."
    );
    println!();
    println!("    L(e) = 1 + d[(1 - e)c_caught + e c_escape]");
    println!();
    println!(
        "  With d = {:.2}, c_caught = {:.2} and c_escape = {:.2}, a review stage that catches"
        ,
        wf.defect_share, wf.caught_rework_cost, wf.escape_rework_cost
    );
    println!(
        "  everything still carries L = {:.2} services per item, so counting only",
        at_full.load
    );
    println!(
        "  first-pass review understates the load by {:.0}%.",
        (at_full.load - 1.0) * 100.0
    );
    println!();
    println!(
        "  At full capacity the workflow runs at {:.1}% of its review capacity. That is inside"
        ,
        at_full.rho_at_generated * 100.0
    );
    println!(
        "  the envelope, with about three points of margin -- which is what a lean target of"
    );
    println!(
        "  high utilisation leaves, and why a small loss of capacity to judge is enough to"
    );
    println!("  matter.");

    // ---- the envelope by capacity -----------------------------------------
    println!();
    println!("  THE OPERATING ENVELOPE");
    println!("  {}", "-".repeat(74));
    println!(
        "  {:>9} {:>8} {:>7} {:>8} {:>10} {:>9} {:>10}",
        "capacity", "escape", "load", "rho", "absorbable", "steady", "on time"
    );
    for point in envelope::sweep(&wf, &params, 20) {
        println!(
            "  {:>9.2} {:>8.3} {:>7.3} {:>8.3} {:>10.3} {:>9} {:>10}",
            point.review_capacity,
            point.escape,
            point.load,
            point.rho_at_generated,
            point.absorbable_share.unwrap_or(f64::NAN),
            if point.stable { "yes" } else { "NO" },
            if point.meets_deadline { "yes" } else { "no" }
        );
    }
    println!();
    println!("  capacity = the person's capacity to judge, on the psychological model's scale.");
    println!("  absorbable = the share of generated work that can be admitted while still");
    println!("  meeting the commitment. At 1.00 the workflow can take everything.");

    // ---- the two thresholds ------------------------------------------------
    println!();
    println!("  TWO THRESHOLDS, AND THEY ARE NOT THE SAME");
    println!("  {}", "-".repeat(74));
    let stability = workflow::critical_review_capacity(&wf, Policy::Push, &params);
    let absorption = envelope::critical_absorption_capacity(&wf, &params);
    match (absorption, stability) {
        (Some(commitment), Some(steady)) => {
            println!(
                "  The commitment binds first, at capacity {commitment:.3}: below it the workflow"
            );
            println!(
                "  cannot absorb its own output. Stability fails later, at {steady:.3}. The gap"
            );
            println!(
                "  between them is the range in which the queue still converges but the promise"
            );
            println!("  is already broken -- which is the range most organisations actually occupy.");
        }
        (None, Some(steady)) => {
            println!("  The workflow cannot meet its commitment even at full capacity.");
            println!("  Stability fails at {steady:.3}.");
        }
        (Some(commitment), None) => {
            println!("  The commitment binds at {commitment:.3}; the queue is stable throughout.");
        }
        (None, None) => println!("  Neither threshold exists in the admissible range."),
    }

    println!();
    println!("  What the commitment costs, by how tight it is:");
    println!();
    println!("    {:>9} {:>14}", "deadline", "critical cap.");
    for (deadline, critical) in envelope::deadline_sensitivity(
        &wf,
        &params,
        &[10.0, 7.0, 5.0, 3.0, 2.0, 1.0, 0.5],
    ) {
        match critical {
            Some(value) => println!("    {:>9.1} {:>14.3}", deadline, value),
            None => println!("    {:>9.1} {:>14}", deadline, "unmeetable"),
        }
    }
    println!();
    println!("  A commitment the workflow cannot meet at any capacity is not a tight");
    println!("  commitment, it is a design failure, and it is reported as one.");

    // ---- the deskilled trajectory -----------------------------------------
    println!();
    println!("  THE YEAR IT LEAVES THE ENVELOPE");
    println!("  {}", "-".repeat(74));
    let mut probe = params.clone();
    probe.expectation_ceiling = PERSON_CEILING;
    let states = capacity::simulate(
        PERSON_CAPACITY,
        PERSON_SELF_MODEL,
        &probe,
        capacity::Strategy::Tracking,
        PERSON_HUMAN_SHARE,
    );
    let path = workflow::trajectory(&states, &wf, Policy::Push, &params);
    println!(
        "  Same workflow throughout. The only thing that changes is the person's capacity to"
    );
    println!(
        "  judge, which falls because assistance is solved for rather than chosen."
    );
    println!();
    println!(
        "  {:>5} {:>10} {:>8} {:>7} {:>8} {:>12}",
        "year", "capacity", "escape", "load", "rho", "verdict"
    );
    for (year, point) in path.iter().enumerate() {
        if year % 3 != 0 && year + 1 != path.len() {
            continue;
        }
        let verdict = if !point.stable {
            "NO STEADY STATE"
        } else if !point.meets_deadline {
            "over commitment"
        } else {
            "inside"
        };
        println!(
            "  {:>5} {:>10.3} {:>8.3} {:>7.3} {:>8.3} {:>12}",
            year, states[year].capacity, point.escape, point.load, point.rho, verdict
        );
    }
    println!();
    let breach_stability = workflow::first_year_outside(&path, workflow::Requirement::Stability);
    let breach_deadline = workflow::first_year_outside(&path, workflow::Requirement::Deadline);
    match breach_deadline {
        Some(year) => println!("  First year over the commitment: {year}."),
        None => println!("  The commitment is never breached."),
    }
    match breach_stability {
        Some(year) => println!("  First year with no steady state: {year}."),
        None => println!("  The queue is never lost."),
    }
    println!();
    println!(
        "  Nothing about the workflow changed. What changed is the person, and the workflow's"
    );
    println!(
        "  stability is a property of the person's capacity to catch what the machine produced."
    );

    // ---- the three configurations -----------------------------------------
    println!();
    println!("  WHAT THE SLACK IS SPENT ON");
    println!("  {}", "-".repeat(74));
    println!(
        "  Three answers to the same question. The third spends the buffer on the capacity to"
    );
    println!("  judge rather than on throughput, and it is the only one that buys any.");
    println!();
    let results: Vec<OptionResult> = options()
        .into_iter()
        .map(|spec| evaluate_option(spec, &wf))
        .collect();
    println!(
        "  {:<24} {:>8} {:>8} {:>8} {:>9} {:>8} {:>7}",
        "configuration", "deskills", "through", "cycle", "escapes", "auton.", "steady"
    );
    for result in &results {
        println!(
            "  {:<24} {:>8.3} {:>8.2} {:>8.2} {:>9.2} {:>8.0} {:>7}",
            result.spec.label,
            result.review_capacity,
            result.outcome.throughput,
            result.outcome.cycle_time,
            result.outcome.escaped_defects,
            result.outcome.autonomy_years,
            if result.envelope.stable { "yes" } else { "NO" }
        );
    }
    println!();
    println!("  deskills = capacity to judge at the horizon; through = items admitted per day;");
    println!("  cycle = days; escapes = escaped defects per day; auton. = years spent judging.");

    // ---- the direction analysis -------------------------------------------
    println!();
    println!("  PRODUCTIVITY TOWARD WHAT");
    println!("  {}", "-".repeat(74));
    println!(
        "  The framework does not rank these. It reports, for every weighting of the five"
    );
    println!(
        "  criteria, which configuration comes first -- and then hands the question back."
    );
    println!();
    let outcomes: Vec<Outcome> = results.iter().map(|result| result.outcome).collect();
    let acceptance = direction::acceptance(&outcomes, WEIGHT_STEPS);
    println!(
        "  {:>8} {:>24}  {:>20}",
        "share", "configuration", "rank under declared"
    );
    let declared = &acceptance.declared_ranking;
    for (position, index) in declared.iter().enumerate() {
        println!(
            "  {:>7.0}% {:<24}  {:>20}",
            acceptance.shares[*index] * 100.0,
            results[*index].spec.label,
            format!("#{}", position + 1)
        );
    }
    println!();
    println!(
        "  {} weightings over the five criteria: {}.",
        acceptance.weightings,
        (0..CRITERIA)
            .map(direction::criterion_name)
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    let contested: Vec<usize> = (0..outcomes.len())
        .filter(|index| acceptance.shares[*index] > 0.0)
        .collect();
    if contested.len() > 1 {
        println!(
            "  No rank order exists: {} configurations win under some defensible weighting.",
            contested.len()
        );
        println!(
            "  The declared weighting is one point on that surface and is printed above as one"
        );
        println!(
            "  of them, not as the answer. Which criterion should outrank which is the"
        );
        println!("  question this document cannot settle, and does not pretend to.");
    } else {
        println!("  One configuration wins under every weighting in the grid, so on these");
        println!("  criteria the trade-off is not a trade-off. State that, and check whether");
        println!("  the criteria are the right ones.");
    }
    if !acceptance.never_first.is_empty() {
        println!();
        println!(
            "  Never first under any weighting: {}.",
            acceptance
                .never_first
                .iter()
                .map(|index| results[*index].spec.label)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // ---- limits ------------------------------------------------------------
    println!();
    println!("  WHAT THIS REPORT DOES NOT ESTABLISH");
    println!("  {}", "-".repeat(74));
    println!("  - That any real workflow runs at these numbers. Every parameter is declared,");
    println!("    and none was measured in a workplace.");
    println!("  - The service-time distributions. The approximation uses only two coefficients of");
    println!("    variation, and it overstates waiting by up to 28.5% at moderate utilisation");
    println!("    (measured against the exact M/M/c result).");
    println!("  - That the workflow feeds back into capacity. It does not: the person's capacity");
    println!("    path is exogenous here. A model in which review load changed that path would");
    println!("    be more alarming and is not this one.");
    println!("  - Which configuration is better. That is a weighting, and the weighting is a");
    println!("    moral position rather than a measurement.");

    if let Some(dir) = export {
        match write_bundle(dir, &wf, &params, &path, &states, &results, &acceptance) {
            Ok(files) => {
                println!();
                println!("  EXPORTED for plotting ({}):", dir.display());
                for file in files {
                    println!("    {}", file.display());
                }
                println!();
                println!("  Draw the figures with:");
                println!("    python tools/plot_workflow.py {}", dir.display());
            }
            Err(error) => eprintln!("warning: could not write the export: {error}"),
        }
    }

    println!();
    println!("{}", "=".repeat(78));
}

fn print_header(wf: &WorkflowParameters) {
    println!();
    println!("{}", "=".repeat(78));
    println!("JUST-IN-TIME LOGIC IN AI-ASSISTED KNOWLEDGE WORK");
    println!("{}", "=".repeat(78));
    println!(
        "  {} reviewers at {:.0} items/day, generating {:.0} items/day from a demand of {:.0}.",
        wf.servers, wf.review_rate, wf.generated_per_day(), wf.demand_per_day
    );
    println!(
        "  The machine multiplied what the team can generate by {:.1}. The review that catches",
        wf.generation_multiplier
    );
    println!("  its defects is still human, and the utilisation target is still lean's.");
    println!();
    println!("  EVERY PARAMETER IS DECLARED. Nothing here is calibrated, so no LEVEL is a");
    println!("  prediction. What the model contributes is the thresholds -- where stability");
    println!("  fails, where the commitment fails, and what has to be true for either to hold.");
}

/// Write the CSV bundle the figures are drawn from.
fn write_bundle(
    dir: &Path,
    wf: &WorkflowParameters,
    params: &capacity::ModelParameters,
    path: &[workflow::Envelope],
    states: &[capacity::State],
    results: &[OptionResult],
    acceptance: &direction::Acceptance,
) -> std::io::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let mut files = Vec::new();
    let mut write = |name: &str, body: String| -> std::io::Result<()> {
        let target = dir.join(name);
        std::fs::write(&target, body)?;
        files.push(target);
        Ok(())
    };

    let mut envelope_csv = String::from(
        "capacity,escape,load,rho_generated,rho_max_deadline,absorbable,stable,on_time\n",
    );
    for point in envelope::sweep(wf, params, 40) {
        let _ = writeln!(
            envelope_csv,
            "{:.4},{:.6},{:.6},{:.6},{},{},{},{}",
            point.review_capacity,
            point.escape,
            point.load,
            point.rho_at_generated,
            point
                .rho_max_for_deadline
                .map(|value| format!("{value:.6}"))
                .unwrap_or_else(|| "NaN".to_string()),
            point
                .absorbable_share
                .map(|value| format!("{value:.6}"))
                .unwrap_or_else(|| "NaN".to_string()),
            u8::from(point.stable),
            u8::from(point.meets_deadline),
        );
    }
    write("envelope.csv", envelope_csv)?;

    let mut deadline_csv = String::from("deadline_days,critical_capacity\n");
    for (deadline, critical) in
        envelope::deadline_sensitivity(wf, params, &[12.0, 10.0, 8.0, 6.0, 5.0, 4.0, 3.0, 2.5, 2.0, 1.5, 1.0])
    {
        let _ = writeln!(
            deadline_csv,
            "{deadline:.2},{}",
            critical
                .map(|value| format!("{value:.6}"))
                .unwrap_or_else(|| "NaN".to_string())
        );
    }
    write("deadline.csv", deadline_csv)?;

    let mut trajectory_csv =
        String::from("year,capacity,escape,load,rho,cycle_time,stable,on_time\n");
    for (year, point) in path.iter().enumerate() {
        let _ = writeln!(
            trajectory_csv,
            "{},{:.6},{:.6},{:.6},{:.6},{},{},{}",
            year,
            states
                .get(year)
                .map(|state| state.capacity)
                .unwrap_or(f64::NAN),
            point.escape,
            point.load,
            point.rho,
            if point.cycle_time_days.is_finite() {
                format!("{:.6}", point.cycle_time_days)
            } else {
                "NaN".to_string()
            },
            u8::from(point.stable),
            u8::from(point.meets_deadline),
        );
    }
    write("trajectory.csv", trajectory_csv)?;

    let mut options_csv = String::from(
        "option,training_share,review_capacity,throughput,cycle_time,escaped_defects,\
         autonomy_years,backlog,stable,share\n",
    );
    for (index, result) in results.iter().enumerate() {
        let _ = writeln!(
            options_csv,
            "\"{}\",{:.4},{:.6},{:.6},{:.6},{:.6},{:.4},{:.6},{},{:.6}",
            result.spec.label,
            result.spec.training_share,
            result.review_capacity,
            result.outcome.throughput,
            result.outcome.cycle_time,
            result.outcome.escaped_defects,
            result.outcome.autonomy_years,
            result.outcome.backlog,
            u8::from(result.envelope.stable),
            acceptance.shares.get(index).copied().unwrap_or(0.0),
        );
    }
    write("options.csv", options_csv)?;

    let mut meta = String::from("key,value\n");
    let stability = workflow::critical_review_capacity(wf, Policy::Push, params);
    let absorption = envelope::critical_absorption_capacity(wf, params);
    let _ = writeln!(meta, "weightings,{}", acceptance.weightings);
    let _ = writeln!(meta, "autonomy_floor,{AUTONOMY_FLOOR}");
    let _ = writeln!(meta, "horizon_years,{}", params.horizon_years);
    let _ = writeln!(
        meta,
        "critical_stability,{}",
        stability
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| "NaN".to_string())
    );
    let _ = writeln!(
        meta,
        "critical_commitment,{}",
        absorption
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| "NaN".to_string())
    );
    let _ = writeln!(meta, "deadline_days,{:.4}", wf.deadline_days);
    let _ = writeln!(meta, "servers,{}", wf.servers);
    let _ = writeln!(meta, "review_rate,{:.4}", wf.review_rate);
    write("meta.csv", meta)?;

    let mut provenance = String::new();
    let _ = writeln!(provenance, "WORKFLOW MODEL -- EXPORT PROVENANCE");
    let _ = writeln!(provenance, "{}", "-".repeat(70));
    let _ = writeln!(
        provenance,
        "Every parameter below is DECLARED. None is calibrated against workplace data."
    );
    let _ = writeln!(
        provenance,
        "The files support the SHAPE of the thresholds, not the levels."
    );
    let _ = writeln!(provenance);
    for (name, source) in wf.provenance_table() {
        let _ = writeln!(provenance, "  {name:<36} {}", source.label());
    }
    write("provenance.txt", provenance)?;

    Ok(files)
}
