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

//! AI in the multipolar game: a player, or a tool?
//!
//! `MULTIPOLAR_GAME.md` section 7 lists four live answers to "who captures the
//! gains from AI" and then declines to pick one, on the grounds that "genuine
//! uncertainty is the most defensible position". This module turns that refusal
//! into something executable. Rather than assuming an answer, it builds the two
//! worlds the question actually distinguishes and measures what separates them.
//!
//! * **AI as a sixth power** ([`AiRole::SixthPower`]). The actor holds power of its
//!   own and plays every dyad like any other bloc, so the question is whether it
//!   ends a hegemon or is held down by the blocs around it.
//! * **AI as a wielded tool** ([`AiRole::WieldedInstrument`]). There is no new
//!   player. Each existing bloc's power grows with its own AI lead, so the question
//!   is not who AI *is* but *whose* it is, and whether differential ownership
//!   concentrates the system.
//!
//! A third world, [`AiRole::Absent`], is the control: the blocs exactly as
//! `blocks.rs` defines them, with no AI term at all.
//!
//! # Why the layer is a transformation of the bloc list
//!
//! Both hypotheses are expressible as an edit to the blocs and nothing else -- one
//! appends an actor, the other adds a term to `growth_bias` and
//! `cooperation_affinity`. Keeping it that way means the Monte Carlo, the pension
//! channel and the solver are untouched and cannot silently change meaning, and it
//! means the control world is the *existing* model rather than a reimplementation
//! of it that might drift.
//!
//! # Nothing here is measured, and that is the whole point
//!
//! There is no published series for "share of world power held by an AI actor", and
//! the counterfactual needed to estimate one does not exist. Every number in this
//! module is [`Provenance::Illustrative`], and a test enforces that none of them
//! claims a source -- inventing one is the failure mode this project exists to
//! avoid. So the *level* of any outcome is not a finding. The finding is the
//! **hinge**: how much growth advantage, or how much leadership payoff, the answer
//! turns on. `--ai` prints both the worlds and the sweep that locates the hinges.

use crate::blocks::PowerBloc;
use crate::economy::Provenance;

/// The name the AI actor takes when it is one.
///
/// Chosen to be a single case-insensitively matchable token, like the existing
/// bloc names, so `--bloc` and the economy's per-bloc lookups work on it without a
/// special case.
pub const AI_ACTOR_NAME: &str = "AI-Compute";

/// A bloc at or above this share of system power counts as dominant.
///
/// Deliberately the same 45% the polarity classifier uses in `simulation.rs`, so
/// "the AI actor ends dominant" and "the system ends unipolar" are the same claim
/// about the same threshold rather than two conventions that can disagree.
pub const DOMINANCE_THRESHOLD: f64 = 0.45;

/// A move counts as a change in kind when it changes the actor's share by at least
/// this fraction of the share it started with.
///
/// # Why relative rather than absolute
///
/// The first version of this used a flat two percentage points of world power, and
/// that was simply wrong for the actor it was measuring. An actor starting near 4.8%
/// can lose *a third of itself* -- down to 3.2% -- while moving only 1.6 points, so a
/// flat band scored a collapse as "PARITY", and did so in the same run in which a
/// fall to 0.5% scored as "ABSORBED". The band has to scale with the thing it is
/// judging. A documented convention, like the polarity thresholds, not a derived
/// quantity.
pub const MATERIAL_RELATIVE: f64 = 0.25;

/// ...or by at least this much of world power, whichever is larger.
///
/// The floor exists so that an actor starting near zero -- where a quarter of its
/// share is a meaningless quantity -- still needs a real move to change verdict.
pub const MATERIAL_FLOOR: f64 = 0.005;

/// The size of move that counts as a change in kind for an actor starting at `start`.
pub fn material_change(start: f64) -> f64 {
    (MATERIAL_RELATIVE * start.abs()).max(MATERIAL_FLOOR)
}

/// A change in the leading bloc's share smaller than this is not a move.
///
/// One percentage point. Absolute rather than relative because the leading bloc
/// starts near 30% in these systems, so the two conventions nearly coincide -- and
/// because this one is compared across worlds whose starting distributions differ.
pub const MATERIAL_TOP_SHARE: f64 = 0.01;

/// Illustrative starting share for the AI actor.
///
/// An actor that holds compute, capital and intellectual property but no
/// territory, no population and no reserve currency. `MULTIPOLAR_GAME.md` section
/// 5's framing -- hyperscalers optimizing datacenter capex -- suggests a real but
/// not initially dominant position, which is what a low single-digit share models.
/// The exact figure is invented; the `--ai-share` flag exists to move it.
pub const AI_STARTING_SHARE: f64 = 0.05;

/// Illustrative annual growth bias for the AI actor.
///
/// Set to exactly the fastest-growing conventional bloc's bias in `default_blocs()`
/// (Indo-Pacific at 0.050), so the *default* world claims only that AI grows like the
/// fastest-growing bloc -- not that it outgrows everything. Whether the actor becomes
/// the hegemon at that rate is then a result of the dynamics rather than an
/// assumption baked into the parameter, and the hinge sweep shows how much faster it
/// would have to grow to get there.
///
/// The value was re-stated from `0.010` when growth biases became true annual rates;
/// see the note on `default_blocs()`. Both numbers model the same actor.
pub const AI_GROWTH_BIAS: f64 = 0.050;

/// Illustrative volatility for the AI actor.
///
/// Higher than any conventional bloc's, on the grounds that a frontier technology's
/// capability trajectory is more dispersed than a territorial economy's. This is a
/// judgement about dispersion, not a measurement of it.
pub const AI_VOLATILITY: f64 = 0.055;

/// Illustrative cooperation affinity for the AI actor.
///
/// Above one, which implements a specific claim rather than a general hunch:
/// `MULTIPOLAR_GAME.md` section 7's first live possibility is that "the
/// consumer/cloudalist class captures the gains" from AI-driven efficiency, being
/// "positioned in circulation, not production". An actor in that position captures
/// more of the cooperation surplus, which is what an affinity above one means.
pub const AI_COOPERATION_AFFINITY: f64 = 1.10;

/// Illustrative: annual power growth bought by one full unit of AI lead.
///
/// The mechanism -- a bloc that leads in a general-purpose technology converts that
/// lead into economic and military capability -- is not in doubt. The conversion
/// rate is unmeasurable, and this number is a placeholder for it.
///
/// Re-stated from `0.010` alongside the growth biases, so that one unit of lead still
/// buys about five percent a year rather than the same nominal figure now meaning a
/// fifth of that. See the note on `default_blocs()`.
pub const DEFAULT_LEAD_GROWTH_EFFECT: f64 = 0.050;

/// Illustrative: how much AI leadership changes what cooperation is worth.
///
/// **Zero, deliberately.** `MULTIPOLAR_GAME.md` section 4 sets out the dispute
/// rather than resolving it: the pessimistic realist case has AI as one more axis
/// of zero-sum rivalry, while the optimistic case (Diane Coyle's, as the chapter
/// reports it) has AI competition not being zero-sum at all because the technology
/// is bound up with global supply chains and cross-border data flows. Those imply
/// opposite signs for this coefficient, so picking either would be hiding a
/// judgement inside a default -- the same choice `payoffs_for` makes when it
/// declines to let interdependence touch the temptation to defect.
///
/// The `--ai` hinge sweep runs this in both directions precisely so the disputed
/// sign is *shown* rather than assumed.
///
/// # What it moves, and what it took to make it move
///
/// This raises the owning bloc's [`PowerBloc::cooperation_valuation`], which enters
/// the payoff matrix -- so it changes how often the bloc cooperates, and the sweep
/// row is a real test of the chapter's optimistic case.
///
/// It did not always. The first version of this feature raised
/// `cooperation_affinity` instead, which is applied *downstream* of the solver to the
/// power a bloc banks; the coefficient therefore could not move the cooperation rate
/// at all, and the sweep row was flat at every value. That flatness was reported
/// honestly at the time as a limitation of a symmetric 2x2, and it is what prompted
/// widening the solver to a bimatrix -- which is why the row now moves.
pub const DEFAULT_LEAD_COOPERATION_EFFECT: f64 = 0.0;

/// How AI enters the system. The rival hypotheses, plus the control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRole {
    /// No AI layer at all: the control the other two worlds are measured against.
    Absent,
    /// AI is an independent actor with a share of system power of its own.
    SixthPower,
    /// AI is a capability the existing blocs own. It is not a player.
    WieldedInstrument,
}

impl AiRole {
    pub fn label(self) -> &'static str {
        match self {
            AiRole::Absent => "NO AI LAYER",
            AiRole::SixthPower => "AI AS A SIXTH POWER",
            AiRole::WieldedInstrument => "AI AS A WIELDED TOOL",
        }
    }

    /// Plain-language statement of what the world assumes, for the report.
    pub fn intent(self) -> &'static str {
        match self {
            AiRole::Absent => "the blocs as they are, with no AI term at all",
            AiRole::SixthPower => "AI is a player: it holds power of its own and plays every dyad",
            AiRole::WieldedInstrument => {
                "AI is owned: no new player, but each bloc's power grows with its own AI lead"
            }
        }
    }

    /// Whether this role puts an AI actor into the bloc list.
    pub fn has_actor(self) -> bool {
        matches!(self, AiRole::SixthPower)
    }

    /// Whether this role scales the existing blocs by their own AI lead.
    pub fn has_lead(self) -> bool {
        matches!(self, AiRole::WieldedInstrument)
    }
}

/// The AI layer, as a parameterisation of the bloc list.
#[derive(Debug, Clone, PartialEq)]
pub struct AiParams {
    pub role: AiRole,
    /// The AI actor. Used only when `role.has_actor()`.
    pub actor: PowerBloc,
    /// Per-bloc AI/compute lead in `[0, 1]`, indexed like the bloc list it is
    /// applied to. Used only when `role.has_lead()`.
    pub lead: Vec<f64>,
    /// Illustrative: annual power growth bought by one unit of AI lead.
    pub lead_growth_effect: f64,
    /// Illustrative: how much AI leadership changes what cooperation is worth.
    /// Zero by default, deliberately -- see [`DEFAULT_LEAD_COOPERATION_EFFECT`].
    pub lead_cooperation_effect: f64,
}

/// The AI actor as a bloc.
fn ai_actor() -> PowerBloc {
    PowerBloc::new(
        AI_ACTOR_NAME,
        AI_STARTING_SHARE,
        AI_GROWTH_BIAS,
        AI_VOLATILITY,
        AI_COOPERATION_AFFINITY,
    )
}

/// Default per-bloc AI lead, in `[0, 1]`.
///
/// The *shape* -- two frontier leaders, a significant third tier, then everyone
/// else -- is the widely reported one and is the reason this vector is not flat.
/// The *magnitudes* are invented: no agency publishes an "AI lead" index, and any
/// figure here would be a judgement dressed as a measurement.
///
/// Matching is by name, like `default_energy_exposure`, so the vector follows the
/// bloc list rather than assuming a length or an order. An unrecognised bloc gets
/// the low default rather than an average, because assuming a new bloc is a
/// frontier AI power is a stronger claim than assuming it is not.
pub fn default_ai_lead(blocs: &[PowerBloc]) -> Vec<f64> {
    blocs
        .iter()
        .map(|bloc| match bloc.name.as_str() {
            "Atlantic" => 1.00,
            "Sinic" => 0.85,
            "Indo-Pacific" => 0.40,
            "Eurasian" => 0.15,
            // Africa, the Gulf and the residual non-aligned middle all take the low
            // default. Naming them explicitly at a higher figure would claim a
            // frontier AI position for a region on no evidence, which is the same
            // judgement this function's note above refuses to make for any bloc.
            _ => 0.10,
        })
        .collect()
}

impl AiParams {
    /// The control world: the blocs exactly as given.
    pub fn absent() -> Self {
        AiParams {
            role: AiRole::Absent,
            actor: ai_actor(),
            lead: Vec::new(),
            lead_growth_effect: 0.0,
            lead_cooperation_effect: 0.0,
        }
    }

    /// AI as an independent actor.
    pub fn sixth_power() -> Self {
        AiParams {
            role: AiRole::SixthPower,
            actor: ai_actor(),
            lead: Vec::new(),
            lead_growth_effect: 0.0,
            lead_cooperation_effect: 0.0,
        }
    }

    /// AI as a capability owned by the existing blocs.
    pub fn wielded_instrument(blocs: &[PowerBloc]) -> Self {
        AiParams {
            role: AiRole::WieldedInstrument,
            actor: ai_actor(),
            lead: default_ai_lead(blocs),
            lead_growth_effect: DEFAULT_LEAD_GROWTH_EFFECT,
            lead_cooperation_effect: DEFAULT_LEAD_COOPERATION_EFFECT,
        }
    }

    /// The same AI lead for every bloc: the control for the tool world.
    ///
    /// If everyone holds the same lead, the lead term raises every bloc's growth
    /// bias by the same amount, and a common lift to everyone's growth is close to
    /// a no-op once shares are renormalised. *Close to*, not exactly: the payoff
    /// feedback inside a year is not proportional across blocs, so a common lift
    /// still nudges the shares slightly. The test on this control is sized for
    /// that, and the report says so rather than claiming an exact invariance it
    /// does not have.
    pub fn uniform_lead(blocs: &[PowerBloc]) -> Self {
        AiParams {
            role: AiRole::WieldedInstrument,
            actor: ai_actor(),
            lead: vec![1.0; blocs.len()],
            lead_growth_effect: DEFAULT_LEAD_GROWTH_EFFECT,
            lead_cooperation_effect: DEFAULT_LEAD_COOPERATION_EFFECT,
        }
    }

    /// Apply this layer to a bloc list, returning the bloc list the run should use.
    ///
    /// The result is always a valid power distribution: shares sum to one. The AI
    /// actor's share is *added* to the system rather than taken from the existing
    /// blocs, and the list is then renormalised so that every bloc's recorded
    /// starting share is the share it actually starts with. Without that step the
    /// report would compare a raw starting figure against a normalised horizon
    /// figure and overstate every other bloc's decline.
    ///
    /// # Why any existing actor is removed first
    ///
    /// Only [`AiRole::SixthPower`] has an actor. The tool world's whole premise is
    /// that AI is *not* a player, and the control's is that there is no AI layer at
    /// all, so an actor left in either would contradict the world being reported --
    /// and would do so silently. Dropping it first also makes `--bloc AI-Compute:...`
    /// compose predictably with `--ai`: the actor is rebuilt from this layer rather
    /// than left as a second, disagreeing copy, so the numbers the report prints for
    /// the actor are always the numbers the run used.
    pub fn apply(&self, blocs: &[PowerBloc]) -> Vec<PowerBloc> {
        let mut out = blocs.to_vec();
        out.retain(|bloc| !bloc.name.eq_ignore_ascii_case(AI_ACTOR_NAME));

        match self.role {
            AiRole::Absent => {}
            AiRole::SixthPower => out.push(self.actor.clone()),
            AiRole::WieldedInstrument => {
                for (index, bloc) in out.iter_mut().enumerate() {
                    let lead = self.lead.get(index).copied().unwrap_or(0.0).clamp(0.0, 1.0);
                    bloc.growth_bias += self.lead_growth_effect * lead;
                    // Valuation, not affinity. This is what makes the coefficient
                    // able to change the *cooperation rate*: valuation enters the
                    // payoff matrix, so a bloc that leads in AI and therefore values
                    // cooperation more will actually choose it more often. Affinity
                    // is downstream of the solver and could never do that, which is
                    // why the first version of this sweep row moved nothing.
                    bloc.cooperation_valuation =
                        (bloc.cooperation_valuation + self.lead_cooperation_effect * lead).max(0.0);
                }
            }
        }

        normalise_shares(&mut out);
        out
    }

    /// Where the AI actor sits in the list this layer produces, if it is there.
    pub fn actor_index(&self, blocs: &[PowerBloc]) -> Option<usize> {
        blocs
            .iter()
            .position(|bloc| bloc.name.eq_ignore_ascii_case(AI_ACTOR_NAME))
    }

    /// Every parameter this layer carries, for the report and for the completeness
    /// test. Keeps the two in step by construction, exactly as
    /// `Economy::provenance_table` does on the economic side.
    pub fn provenance_table(&self) -> Vec<(&'static str, Provenance)> {
        match self.role {
            AiRole::Absent => vec![("no AI layer", NO_LAYER_PROVENANCE)],
            AiRole::SixthPower => vec![
                ("AI actor starting share", AI_SHARE_PROVENANCE),
                ("AI actor growth bias", AI_GROWTH_PROVENANCE),
                ("AI actor volatility", AI_VOLATILITY_PROVENANCE),
                ("AI actor cooperation affinity", AI_AFFINITY_PROVENANCE),
                ("AI actor energy position", AI_ENERGY_PROVENANCE),
                ("AI actor reserve-currency share", AI_MONEY_PROVENANCE),
            ],
            AiRole::WieldedInstrument => vec![
                ("per-bloc AI lead", AI_LEAD_PROVENANCE),
                ("AI lead growth effect", LEAD_GROWTH_PROVENANCE),
                ("AI lead cooperation effect", LEAD_COOPERATION_PROVENANCE),
            ],
        }
    }
}

/// The three worlds `--ai` compares, in report order.
pub fn ai_worlds(blocs: &[PowerBloc]) -> Vec<AiParams> {
    vec![
        AiParams::absent(),
        AiParams::sixth_power(),
        AiParams::wielded_instrument(blocs),
    ]
}

/// Force a share vector into a valid distribution, flooring at zero.
///
/// A local copy rather than a call into `simulation.rs`, because that one is
/// private and this module must be usable to *build* a configuration, which is
/// before any simulation exists.
fn normalise_shares(blocs: &mut [PowerBloc]) {
    for bloc in blocs.iter_mut() {
        if !bloc.power_share.is_finite() || bloc.power_share < 0.0 {
            bloc.power_share = 0.0;
        }
    }
    let total: f64 = blocs.iter().map(|bloc| bloc.power_share).sum();
    // A list that already sums to one is left exactly alone. Dividing by
    // 1.0000000000000002 is noise, but it is noise that made `AiParams::absent()`
    // return shares differing from the model it is the control *for* -- and a control
    // that is merely almost identical contaminates every difference measured against
    // it. Whether the sum lands on one exactly depends on the bloc list, so this must
    // not be left to luck.
    if (total - 1.0).abs() < 1e-12 {
        return;
    }
    if total > 0.0 {
        for bloc in blocs.iter_mut() {
            bloc.power_share /= total;
        }
    } else {
        let even = 1.0 / blocs.len().max(1) as f64;
        for bloc in blocs.iter_mut() {
            bloc.power_share = even;
        }
    }
}

/// What became of AI when it is a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorFate {
    /// It ended above the dominance threshold in most runs.
    Hegemon,
    /// It reliably gained ground without reaching dominance.
    Ascendant,
    /// It held roughly the share it started with.
    Parity,
    /// It lost ground: the blocs around it held it down.
    Absorbed,
}

impl ActorFate {
    pub fn label(self) -> &'static str {
        match self {
            ActorFate::Hegemon => "HEGEMON",
            ActorFate::Ascendant => "ASCENDANT",
            ActorFate::Parity => "PARITY",
            ActorFate::Absorbed => "ABSORBED",
        }
    }

    /// What the verdict means in one line, for the report.
    pub fn means(self) -> &'static str {
        match self {
            ActorFate::Hegemon => "AI usually ends the system's dominant power",
            ActorFate::Ascendant => "AI gains ground steadily but does not come to dominate",
            ActorFate::Parity => "AI holds the position it started from",
            ActorFate::Absorbed => "AI is held down: the blocs absorb it rather than the reverse",
        }
    }
}

/// Read the actor's fate from its trajectory and its dominance probability.
///
/// Hegemony is tested first, because it is the strongest claim and the others are
/// all statements about a non-hegemon. The materiality band scales with the actor's
/// own starting share -- see [`material_change`] for why an absolute band was wrong.
pub fn actor_fate(start: f64, end: f64, dominance_probability: f64) -> ActorFate {
    if dominance_probability >= 0.5 {
        return ActorFate::Hegemon;
    }
    let threshold = material_change(start);
    if end > start + threshold {
        return ActorFate::Ascendant;
    }
    if end < start - threshold {
        return ActorFate::Absorbed;
    }
    ActorFate::Parity
}

/// What AI does to the hierarchy when the blocs own it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentEffect {
    /// The most AI-led blocs pull away from the rest.
    Concentrating,
    /// Differential leadership does not materially move the hierarchy.
    Neutral,
    /// The lead vector levels the system rather than concentrating it.
    Levelling,
}

impl InstrumentEffect {
    pub fn label(self) -> &'static str {
        match self {
            InstrumentEffect::Concentrating => "CONCENTRATING",
            InstrumentEffect::Neutral => "NEUTRAL",
            InstrumentEffect::Levelling => "LEVELLING",
        }
    }

    pub fn means(self) -> &'static str {
        match self {
            InstrumentEffect::Concentrating => "owning AI buys power, so the AI leaders pull away",
            InstrumentEffect::Neutral => "differential AI ownership does not move the hierarchy",
            InstrumentEffect::Levelling => "AI ownership redistributes power away from the leader",
        }
    }
}

/// Compare the leading bloc's share with and without the AI-lead term.
pub fn instrument_effect(baseline_top_share: f64, with_ai_top_share: f64) -> InstrumentEffect {
    let change = with_ai_top_share - baseline_top_share;
    if change > MATERIAL_TOP_SHARE {
        InstrumentEffect::Concentrating
    } else if change < -MATERIAL_TOP_SHARE {
        InstrumentEffect::Levelling
    } else {
        InstrumentEffect::Neutral
    }
}

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------
//
// All of these are `Illustrative`, and a test enforces that none of them is
// `Sourced`. That is not caution for its own sake: the failure this guards against
// is a model that looks authoritative because a number has a citation next to it,
// when the citation is to a *mechanism* rather than to a measurement of the
// quantity being used.

/// The control world carries no AI parameters of its own.
pub const NO_LAYER_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "no AI layer is applied in this world; it is the existing model \
                and exists so the other two can be measured against something",
};

pub const AI_SHARE_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "no published figure exists for the share of world power held by an AI \
                actor. A low single-digit start models an actor with compute, capital and \
                intellectual property but no territory, population or currency",
};

pub const AI_GROWTH_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "set to exactly the fastest-growing conventional bloc's bias, so the default \
                world claims only that AI grows like the fastest-growing bloc. Both are \
                annual rates, applied once a year",
};

pub const AI_VOLATILITY_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "higher than any conventional bloc's, on the judgement that a frontier \
                technology's capability trajectory is more dispersed than a territorial \
                economy's. A claim about dispersion, not a measurement of it",
};

pub const AI_AFFINITY_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "above one, which implements MULTIPOLAR_GAME.md section 7's first live \
                possibility -- that the class positioned in circulation rather than \
                production captures most of the AI-driven efficiency gain",
};

pub const AI_ENERGY_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "a compute complex holds no oil or gas to export and depends on purchased \
                electricity rather than its own production, so it is modelled as an energy \
                importer with no export earnings. The mechanism is reported; the magnitude \
                is not",
};

pub const AI_MONEY_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "an AI actor issues no reserve currency, so it holds no monetary leverage \
                over anyone while every bloc that does issue one holds maximum leverage \
                over it. This is a structural consequence of the model, not a measured \
                position, and it is the single largest asymmetry the actor faces",
};

pub const AI_LEAD_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "the shape -- two frontier leaders, a significant third tier, then everyone \
                else -- follows widely reported capability concentration, but no agency \
                publishes an AI lead index and every magnitude here is a judgement",
};

pub const LEAD_GROWTH_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "that a lead in a general-purpose technology converts into economic and \
                military capability is not in doubt; the conversion rate is unmeasurable, \
                and this is a placeholder for it, exposed so --ai can locate the hinge",
};

pub const LEAD_COOPERATION_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "zero by default because MULTIPOLAR_GAME.md section 4 presents the dispute \
                and declines to resolve it: the realist case implies AI sharpens zero-sum \
                rivalry, the optimistic case that it need not. Picking either sign would \
                hide a judgement inside a default, so --ai sweeps both directions. It acts \
                on the bloc's cooperation valuation, which enters the payoff matrix, so \
                unlike the first version of this coefficient it genuinely moves the \
                cooperation rate",
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::default_blocs;
    use crate::economy::Economy;

    fn share_total(blocs: &[PowerBloc]) -> f64 {
        blocs.iter().map(|bloc| bloc.power_share).sum()
    }

    /// The control world has to be the existing model exactly. If it were merely
    /// close, every difference the other two worlds show would be contaminated by
    /// the control's own deviation.
    #[test]
    fn the_control_world_changes_nothing_at_all() {
        let blocs = default_blocs();
        let applied = AiParams::absent().apply(&blocs);
        assert_eq!(
            applied, blocs,
            "the no-AI world must be the blocs exactly as defined"
        );
    }

    /// Adding an actor has to leave a valid power distribution, and its recorded
    /// starting share has to be its *actual* starting share -- the value the report
    /// compares each horizon share against.
    #[test]
    fn the_sixth_power_world_adds_a_renormalised_actor() {
        let blocs = default_blocs();
        let applied = AiParams::sixth_power().apply(&blocs);

        assert_eq!(applied.len(), blocs.len() + 1, "one actor is added");
        assert!(
            (share_total(&applied) - 1.0).abs() < 1e-9,
            "the applied list must be a valid distribution, got {}",
            share_total(&applied)
        );

        let index = AiParams::sixth_power()
            .actor_index(&applied)
            .expect("the actor must be findable by name");
        let actor = &applied[index];
        assert_eq!(actor.name, AI_ACTOR_NAME);

        // 0.05 added to a system that already sums to 1 is 0.05/1.05 = 4.76% of it,
        // not 5%. Reporting the raw figure would overstate the actor and understate
        // every other bloc's decline.
        let expected = AI_STARTING_SHARE / (1.0 + AI_STARTING_SHARE);
        assert!(
            (actor.power_share - expected).abs() < 1e-9,
            "the actor's recorded start must be its share of the new total: {} vs {expected}",
            actor.power_share
        );

        // And the existing blocs are scaled down, not left claiming a total above 1.
        for (before, after) in blocs.iter().zip(applied.iter()) {
            assert!(
                after.power_share < before.power_share,
                "{} must be diluted by the actor's arrival",
                before.name
            );
        }
    }

    /// The tool world must add no player. That is the entire distinction between the
    /// two hypotheses, so it is pinned.
    #[test]
    fn the_wielded_tool_world_adds_no_player() {
        let blocs = default_blocs();
        let applied = AiParams::wielded_instrument(&blocs).apply(&blocs);

        assert_eq!(applied.len(), blocs.len(), "no actor may be added");
        assert!(
            AiParams::wielded_instrument(&blocs)
                .actor_index(&applied)
                .is_none(),
            "the tool world must contain no AI actor"
        );
        assert!((share_total(&applied) - 1.0).abs() < 1e-9);
        assert!(
            (share_total(&applied) - share_total(&blocs)).abs() < 1e-9,
            "the tool world changes growth, not the starting distribution"
        );
    }

    /// AI leadership must actually reach the bloc that holds it, and it must reach
    /// the leader more than the laggard. Without this the tool world would be a
    /// no-op dressed up as a comparison.
    #[test]
    fn the_lead_term_reaches_the_blocs_that_hold_it() {
        let blocs = default_blocs();
        let params = AiParams::wielded_instrument(&blocs);
        let applied = params.apply(&blocs);

        let lead_of = |name: &str| {
            params.lead[blocs
                .iter()
                .position(|bloc| bloc.name == name)
                .expect("default bloc")]
        };
        let bias_of = |name: &str| {
            applied
                .iter()
                .find(|bloc| bloc.name == name)
                .expect("default bloc")
                .growth_bias
        };

        // Atlantic leads, Eurasian barely registers.
        assert!(
            lead_of("Atlantic") > lead_of("Eurasian"),
            "the lead vector must not be flat"
        );
        assert!(
            bias_of("Atlantic") > bias_of("Eurasian"),
            "the larger lead must buy the larger growth advantage: {} vs {}",
            bias_of("Atlantic"),
            bias_of("Eurasian")
        );
        // The laggard still gets something, so the term is a gradient rather than a
        // winner-take-all switch.
        assert!(
            bias_of("Eurasian") > blocs[2].growth_bias,
            "even a small lead must add something"
        );
    }

    /// The control for the tool world: a common lead for everyone. This must be
    /// *near*-neutral, and the test says near rather than exact because the payoff
    /// feedback inside a year is not proportional across blocs.
    #[test]
    fn a_uniform_ai_lead_lifts_everyone_alike() {
        let blocs = default_blocs();
        let applied = AiParams::uniform_lead(&blocs).apply(&blocs);

        let added: Vec<f64> = blocs
            .iter()
            .zip(applied.iter())
            .map(|(before, after)| after.growth_bias - before.growth_bias)
            .collect();
        for value in &added {
            assert!(
                (value - DEFAULT_LEAD_GROWTH_EFFECT).abs() < 1e-12,
                "a uniform lead must add the same bias everywhere, got {value}"
            );
        }
    }

    /// The two hypotheses must actually be different worlds, or `--ai` would be
    /// comparing an assumption against itself and the output would still look
    /// plausible -- the dangerous kind of failure.
    #[test]
    fn the_two_hypotheses_produce_genuinely_different_worlds() {
        let blocs = default_blocs();
        let actor_world = AiParams::sixth_power().apply(&blocs);
        let tool_world = AiParams::wielded_instrument(&blocs).apply(&blocs);

        assert_ne!(
            actor_world, tool_world,
            "the two rival hypotheses must not describe the same system"
        );
        assert_eq!(
            actor_world.len(),
            blocs.len() + 1,
            "the player world has one more actor than the control"
        );
        assert_eq!(
            tool_world.len(),
            blocs.len(),
            "the tool world owns the capability with the existing blocs, so it adds and \
             removes nobody"
        );
        assert!(
            tool_world
                .iter()
                .any(|bloc| bloc.growth_bias > 0.010 + 1e-12),
            "the tool world must have moved at least one bloc's growth"
        );
    }

    /// The strongest structural claim the actor world makes: an actor with no
    /// currency of its own is the most sanctionable actor in the system. It is a
    /// consequence of the model, so it must hold by construction -- if it stopped
    /// holding, the report's explanation of the actor's fate would become false.
    #[test]
    fn the_ai_actor_holds_no_monetary_leverage_and_is_maximally_exposed() {
        let blocs = AiParams::sixth_power().apply(&default_blocs());
        let economy = Economy::for_blocs(&blocs);
        let actor = blocs
            .iter()
            .position(|bloc| bloc.name == AI_ACTOR_NAME)
            .expect("the actor is present");
        let atlantic = blocs
            .iter()
            .position(|bloc| bloc.name == "Atlantic")
            .expect("default bloc");

        assert_eq!(
            economy.reserve_shares[actor], 0.0,
            "an AI actor issues no reserve currency"
        );
        assert_eq!(
            economy.monetary_leverage(actor, atlantic),
            0.0,
            "with no issuance it holds no leverage over anyone"
        );
        assert_eq!(
            economy.monetary_leverage(atlantic, actor),
            1.0,
            "and every issuer holds maximum leverage over it"
        );
    }

    /// Only the player world may contain an actor. A stray one in either of the other
    /// two would contradict the premise that world is reported under -- silently, and
    /// in the direction that flatters the hypothesis being tested.
    ///
    /// This matters because `--help` invites `--bloc AI-Compute:...` as a way to edit
    /// the actor, so a user-supplied actor can be present in the base bloc list.
    #[test]
    fn only_the_player_world_contains_an_actor() {
        let mut blocs = default_blocs();
        blocs.push(PowerBloc::new(AI_ACTOR_NAME, 0.40, 0.03, 0.02, 1.0));

        for role in [
            AiRole::Absent,
            AiRole::WieldedInstrument,
            AiRole::SixthPower,
        ] {
            let params = match role {
                AiRole::Absent => AiParams::absent(),
                AiRole::SixthPower => AiParams::sixth_power(),
                AiRole::WieldedInstrument => AiParams::wielded_instrument(&blocs),
            };
            let applied = params.apply(&blocs);
            let actors = applied
                .iter()
                .filter(|bloc| bloc.name.eq_ignore_ascii_case(AI_ACTOR_NAME))
                .count();

            if role.has_actor() {
                assert_eq!(actors, 1, "the player world must have exactly one actor");
            } else {
                assert_eq!(
                    actors,
                    0,
                    "{} must contain no actor, whatever the base list held",
                    role.label()
                );
            }
            assert!(
                (share_total(&applied) - 1.0).abs() < 1e-9,
                "{} is not a valid distribution",
                role.label()
            );
        }
    }

    /// `--bloc AI-Compute:...` and the `--ai-*` flags both address the actor, so the
    /// layer must rebuild it rather than leave a second, disagreeing copy behind.
    #[test]
    fn a_supplied_actor_is_rebuilt_from_the_layer_not_left_as_a_duplicate() {
        let mut blocs = default_blocs();
        blocs.push(PowerBloc::new(AI_ACTOR_NAME, 0.40, 0.03, 0.02, 1.0));
        let params = AiParams::sixth_power();
        let applied = params.apply(&blocs);

        assert_eq!(
            applied
                .iter()
                .filter(|bloc| bloc.name.eq_ignore_ascii_case(AI_ACTOR_NAME))
                .count(),
            1,
            "the actor must not be duplicated"
        );
        let actor = &applied[params.actor_index(&applied).expect("the actor is present")];
        assert_eq!(
            actor.growth_bias, params.actor.growth_bias,
            "the run must use the layer's growth bias, not the one it replaced, or the \
             report would print figures the simulation never used"
        );
    }

    /// The actor's energy position must be stated rather than inherited from the
    /// generic fallback, which would have it exporting energy it does not produce.
    #[test]
    fn the_ai_actor_is_a_pure_energy_importer() {
        let blocs = AiParams::sixth_power().apply(&default_blocs());
        let economy = Economy::for_blocs(&blocs);
        let actor = blocs
            .iter()
            .position(|bloc| bloc.name == AI_ACTOR_NAME)
            .expect("the actor is present");

        let exposure = economy.energy[actor];
        assert!(
            exposure.import_dependence > 0.9,
            "a compute complex runs on purchased electricity, got {}",
            exposure.import_dependence
        );
        assert_eq!(
            exposure.export_dependence, 0.0,
            "it has no energy to export, so the generic fallback would be a false claim"
        );
    }

    /// Fate must read the trajectory in the right direction at every boundary.
    ///
    /// The values sit clearly inside each band rather than exactly on a threshold.
    /// `end == start - material_change(start)` is not a stable test: the threshold is
    /// evaluated as a subtraction, and `0.05 - 0.0125` need not equal the literal a
    /// reader would write for it, so pinning the boundary would test the float
    /// representation rather than the classification.
    #[test]
    fn actor_fate_reads_the_trajectory() {
        // Dominance is the strongest claim and outranks the others.
        assert_eq!(actor_fate(0.05, 0.60, 0.99), ActorFate::Hegemon);
        assert_eq!(actor_fate(0.05, 0.60, 0.50), ActorFate::Hegemon);
        assert_eq!(actor_fate(0.05, 0.60, 0.49), ActorFate::Ascendant);
        // Gaining ground without dominating.
        assert_eq!(actor_fate(0.05, 0.20, 0.0), ActorFate::Ascendant);
        assert_eq!(actor_fate(0.05, 0.07, 0.0), ActorFate::Ascendant);
        // Holding position.
        assert_eq!(actor_fate(0.05, 0.06, 0.0), ActorFate::Parity);
        assert_eq!(actor_fate(0.05, 0.05, 0.0), ActorFate::Parity);
        assert_eq!(actor_fate(0.05, 0.04, 0.0), ActorFate::Parity);
        // Losing ground.
        assert_eq!(actor_fate(0.05, 0.03, 0.0), ActorFate::Absorbed);
        assert_eq!(actor_fate(0.05, 0.00, 0.0), ActorFate::Absorbed);
        // And the direction must be right rather than merely different: an actor that
        // doubles is never reported as absorbed, and one that halves never as
        // ascendant.
        assert_ne!(actor_fate(0.05, 0.10, 0.0), ActorFate::Absorbed);
        assert_ne!(actor_fate(0.05, 0.02, 0.0), ActorFate::Ascendant);
    }

    /// The materiality band must scale with the actor, because a flat band misjudged
    /// the real actor this mode reports on.
    ///
    /// The regression this pins is concrete: at the default starting share of 4.76%,
    /// the actor falling to 3.2% -- a loss of a third of itself -- was scored as
    /// PARITY by a flat two-point band, in the same run in which a fall to 0.5% scored
    /// as ABSORBED. A band whose judgement reverses like that is not measuring
    /// anything.
    #[test]
    fn the_materiality_band_scales_with_the_actor() {
        // The floor binds for an actor starting near zero...
        assert!(
            (material_change(0.001) - MATERIAL_FLOOR).abs() < 1e-12,
            "a tiny actor must fall back to the floor, got {}",
            material_change(0.001)
        );
        // ...and the relative term binds for one that starts large.
        assert!(
            (material_change(0.40) - 0.10).abs() < 1e-12,
            "a large actor must be judged relatively, got {}",
            material_change(0.40)
        );

        // The regression case, at the default starting share.
        let start = AI_STARTING_SHARE / (1.0 + AI_STARTING_SHARE);
        assert_eq!(
            actor_fate(start, 0.032, 0.0),
            ActorFate::Absorbed,
            "losing a third of its share is a change in kind, not parity"
        );
        assert_eq!(actor_fate(start, 0.005, 0.0), ActorFate::Absorbed);
    }

    /// The instrument verdict must separate a real move from noise in both
    /// directions, and must not treat a rise as a fall.
    #[test]
    fn instrument_effect_reads_the_top_share() {
        // A clear rise and a clear fall.
        assert_eq!(
            instrument_effect(0.30, 0.40),
            InstrumentEffect::Concentrating
        );
        assert_eq!(instrument_effect(0.30, 0.20), InstrumentEffect::Levelling);
        // A move smaller than the convention is not a move, in either direction.
        // 0.5pp is half the threshold, so it is unambiguous under floating point --
        // unlike a value written at exactly the threshold, which is not.
        assert_eq!(instrument_effect(0.30, 0.305), InstrumentEffect::Neutral);
        assert_eq!(instrument_effect(0.30, 0.30), InstrumentEffect::Neutral);
        assert_eq!(instrument_effect(0.30, 0.295), InstrumentEffect::Neutral);
    }

    /// Every AI parameter must say where it came from, and *none* of them may claim
    /// a source. Inventing a citation for a quantity nobody has measured is the
    /// precise failure this repository's provenance discipline exists to prevent, so
    /// the assertion runs in the opposite direction from the one in `economy.rs`.
    #[test]
    fn every_ai_parameter_is_labelled_as_invented_and_says_something() {
        let blocs = default_blocs();
        for params in ai_worlds(&blocs) {
            let rows = params.provenance_table();
            assert!(
                !rows.is_empty(),
                "{} must declare its parameters",
                params.role.label()
            );
            for (what, provenance) in rows {
                assert!(
                    !what.trim().is_empty(),
                    "a provenance row must name what it describes"
                );
                assert!(
                    !provenance.is_sourced(),
                    "{what} claims a source; no AI figure in this model is measured"
                );
                assert!(
                    provenance.detail().len() > 30,
                    "{what} has a provenance that says nothing: {:?}",
                    provenance.detail()
                );
            }
        }
    }

    /// A custom bloc set must still get a lead vector of the right length, and an
    /// unknown bloc must not be silently promoted to a frontier AI power.
    #[test]
    fn lead_vectors_follow_the_bloc_list_they_are_applied_to() {
        let mut blocs = default_blocs();
        blocs.push(PowerBloc::new("Antarctic", 0.05, 0.0, 0.01, 1.0));
        let params = AiParams::wielded_instrument(&blocs);

        assert_eq!(params.lead.len(), blocs.len());
        assert_eq!(
            *params.lead.last().expect("a lead per bloc"),
            0.10,
            "an unrecognised bloc gets the low default, not an average"
        );

        // And the uniform control covers every bloc rather than the first five.
        assert_eq!(AiParams::uniform_lead(&blocs).lead.len(), blocs.len());
    }
}
