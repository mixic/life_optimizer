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

//! The named AI scenarios: ordered, resolvable, and refusing to guess.

use life_optimizer::ai_scenario::{by_name, describe, is_known, names, SCENARIOS};

/// The scenarios must be ordered from least to most optimistic, or the names would be a claim the
/// numbers do not support. A "sceptical" scenario with a bigger gain than the "optimistic" one is
/// worse than no scenario at all.
#[test]
fn the_scenarios_are_ordered_from_sceptical_to_optimistic() {
    for pair in SCENARIOS.windows(2) {
        assert!(
            pair[1].productivity_gain > pair[0].productivity_gain,
            "{} has gain {} but {} has {}",
            pair[0].name,
            pair[0].productivity_gain,
            pair[1].name,
            pair[1].productivity_gain
        );
        assert!(
            pair[1].quality_retention >= pair[0].quality_retention,
            "{} retains less of its gain than {}",
            pair[1].name,
            pair[0].name
        );
        assert!(
            pair[1].productivity_gain_high >= pair[1].productivity_gain,
            "{} has an optimistic end below its expected end",
            pair[1].name
        );
    }
    assert_eq!(SCENARIOS[0].name, "sceptical");
    assert_eq!(SCENARIOS[2].name, "optimistic");
}

/// The gains must sit inside the range the sourced document reports, or the presets would be
/// asserting something the evidence does not support.
#[test]
fn the_gains_sit_inside_the_published_range() {
    // AI_AS_CATALYST.md records a spread from 19% slower to 56% faster.
    for scenario in SCENARIOS {
        assert!(scenario.productivity_gain >= 0.0, "{} asserts a negative gain", scenario.name);
        assert!(
            scenario.productivity_gain_high <= 0.56,
            "{} asserts {:.0}% at its optimistic end, beyond the published range",
            scenario.name,
            scenario.productivity_gain_high * 100.0
        );
    }
}

/// Every name must resolve, case-insensitively and with surrounding whitespace, because the flag
/// is typed by hand.
#[test]
fn every_name_resolves_however_it_is_typed() {
    for scenario in SCENARIOS {
        assert!(is_known(scenario.name));
        assert_eq!(by_name(scenario.name).map(|s| s.name), Some(scenario.name));
        assert_eq!(by_name(&scenario.name.to_uppercase()).map(|s| s.name), Some(scenario.name));
        assert_eq!(by_name(&format!("  {}  ", scenario.name)).map(|s| s.name), Some(scenario.name));
    }
    assert_eq!(names().len(), 3);
}

/// An unknown name must be refused, not defaulted. A typo that silently selected the optimistic
/// scenario would change the recommendation without the user seeing it.
#[test]
fn an_unknown_name_is_refused_rather_than_defaulted() {
    assert!(by_name("nonsense").is_none());
    assert!(by_name("").is_none());
    assert!(by_name("optimist").is_none());
    // Whitespace and case are tolerated; a different word is not.
    assert!(is_known("Optimistic "));
    assert!(!is_known("optimistically"));
}

/// The description must name the numbers, so a reader can see what the scenario set without
/// running the model again.
#[test]
fn the_description_states_the_numbers_it_sets() {
    let neutral = by_name("neutral").expect("the scenario exists");
    let text = describe(&neutral);
    assert!(text.contains("neutral"));
    assert!(text.contains("25%"), "the gain is not stated: {text}");
    assert!(text.contains("85%"), "the retention is not stated: {text}");
    assert!(text.contains("120%"), "the required output is not stated: {text}");
}

/// Every coefficient must be labelled, and the two that are anchored to the published range must
/// say so rather than claiming measurement.
#[test]
fn every_coefficient_is_labelled() {
    for scenario in SCENARIOS {
        let table = scenario.provenance_table();
        assert_eq!(table.len(), 4, "{} has an incomplete provenance table", scenario.name);
        for (name, note) in &table {
            assert!(!name.is_empty());
            assert!(
                note.contains("declared") || note.contains("anchored"),
                "the note for {name} in {} does not say how it was chosen: {note}",
                scenario.name
            );
        }
        assert!(!scenario.description.is_empty());
    }
}
