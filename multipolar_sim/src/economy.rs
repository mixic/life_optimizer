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

//! The economic layer: monetary standing, energy trade, and financial conditions.
//!
//! # What this module is for
//!
//! `game.rs` models the strategic *structure* between a pair of blocs. This module
//! supplies the economics that determines how much that structure costs them, so
//! the simulator can be asked what de-dollarisation or an energy disruption does to
//! the odds of cooperation rather than only what it does to a chart.
//!
//! # Sourced anchors, illustrative elasticities
//!
//! Every quantity here carries a [`Provenance`], and the distinction is load-bearing:
//!
//! * **Sourced** means a published figure with a named source and vintage. The
//!   reserve shares are IMF COFER; the energy network is built from reported trade
//!   shares.
//! * **Illustrative** means invented -- plausible in sign and rough magnitude, not
//!   calibrated. Every *transmission elasticity* is in this class, and unavoidably
//!   so: nobody can measure how a one-point shift in reserve share changes sanction
//!   leverage, because the counterfactual does not exist. Inventing a number for it
//!   is fine; presenting it as measured is not.
//!
//! The same discipline applies to *allocations*. Where a published total covers a
//! group of countries that spans several blocs, the total is sourced and the split
//! across blocs is marked illustrative, so it can be replaced with the real
//! breakdown without re-deriving anything else.
//!
//! # Why the coupling is split in two
//!
//! The economic layer enters the model through two different doors:
//!
//! * **Through the game**: the pair's *interdependence* raises what mutual
//!   cooperation is worth to both sides, so it belongs in the payoff matrix. So does
//!   each bloc's own valuation of cooperation -- the term that makes the two sides'
//!   matrices differ, and therefore the term that lets the model express "cooperation
//!   is worth more to this bloc than to that one".
//! * **Through the simulation**: exposure to an energy disruption, and leverage over
//!   another bloc's money, hit individual blocs differently, so they act on that
//!   bloc's own power and losses.
//!
//! The first door handles what a bloc *decides*; the second handles what it *pays*.
//! Keeping them apart is what lets the model say "this bloc values cooperation more"
//! and "this bloc is more exposed to a rupture" without conflating the two -- and it
//! is why the solver could be widened from a symmetric 2x2 to a bimatrix without a
//! single line of this layer changing.

use crate::blocks::PowerBloc;

/// Where a number came from.
///
/// Mirrors the `Provenance` enum on the tax side of this repository, and exists for
/// the same reason: a reader must be able to tell a published figure from an
/// invention without reading the surrounding prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Published and citable, at the named source and vintage.
    Sourced {
        source: &'static str,
        vintage: &'static str,
    },
    /// Invented: chosen to be plausible in sign and rough magnitude, not calibrated.
    Illustrative { rationale: &'static str },
}

impl Provenance {
    pub fn label(self) -> &'static str {
        match self {
            Provenance::Sourced { .. } => "sourced",
            Provenance::Illustrative { .. } => "illustrative",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Provenance::Sourced { source, .. } => source,
            Provenance::Illustrative { rationale } => rationale,
        }
    }

    pub fn is_sourced(self) -> bool {
        matches!(self, Provenance::Sourced { .. })
    }
}

/// Share of the world's official foreign-exchange reserves issued by each bloc,
/// in bloc order, plus whatever could not be attributed.
///
/// # Source
///
/// IMF COFER, world aggregates, 2025 Q4 (brief published 2026-03-27): US dollar
/// 56.77%, euro 20.25%, renminbi 1.95%. A further 14.90% is held in currencies
/// COFER identifies individually but the brief aggregates -- yen, sterling, and the
/// Australian, Canadian and Swiss dollars -- and 6.13% is in currencies COFER does
/// not identify at all. That last category has more than doubled since 2021, and it
/// is the more interesting de-dollarisation signal: reserve diversification is
/// running ahead of the renminbi's own share rather than into it.
///
/// # The allocation is illustrative; the totals are not
///
/// COFER publishes the per-currency breakdown in its dataset, but the brief this
/// was taken from reports the five minor currencies only as one 14.90% figure. The
/// split of that figure, and of the 6.13% residual, across blocs is therefore
/// **invented** and marked as such -- see [`Economy::reserve_provenance`]. Replacing
/// it needs the COFER table, not a re-derivation of anything else.
pub fn default_reserve_shares(blocs: &[PowerBloc]) -> (Vec<f64>, f64, Provenance) {
    // Looks up a bloc by name without borrowing the output vector, so the sourced
    // and illustrative contributions below can both be added in place.
    let at = |name: &str| blocs.iter().position(|b| b.name == name);
    let mut shares = vec![0.0; blocs.len()];

    if let Some(atlantic) = at("Atlantic") {
        shares[atlantic] += 56.77; // US dollar, sourced
        shares[atlantic] += 20.25; // euro: issued by the euro area, inside this bloc
        shares[atlantic] += 10.7; // illustrative share of the 14.90 minor aggregate
    }
    if let Some(sinic) = at("Sinic") {
        shares[sinic] += 1.95; // renminbi, sourced
    }
    if let Some(indo) = at("Indo-Pacific") {
        // Illustrative allocation of COFER's 14.90% aggregate. Japan holds the
        // largest of the five minor currencies and the Australian dollar is the
        // other Indo-Pacific one, so this bloc takes the larger remainder.
        shares[indo] += 4.2;
    }

    let attributed: f64 = shares.iter().sum();
    let unattributed = (100.0 - attributed).max(0.0);

    (
        shares.into_iter().map(|s| s / 100.0).collect(),
        unattributed / 100.0,
        Provenance::Sourced {
            source: "IMF COFER world aggregates 2025 Q4 (USD 56.77, EUR 20.25, CNY 1.95; \
                     14.90 minor identified, 6.13 unidentified), with an illustrative \
                     split of the 14.90 across blocs",
            vintage: "2025 Q4",
        },
    )
}

/// The provenance of the illustrative part of the reserve allocation, kept separate
/// so the aggregated note above cannot be mistaken for a wholly sourced figure.
pub const RESERVE_ALLOCATION_PROVENANCE: Provenance = Provenance::Illustrative {
    rationale: "the split of COFER's 14.90% minor-currency aggregate and 6.13% \
                unidentified residual across blocs is invented; only the totals are \
                published in the brief used",
};

/// Provenance of the directed energy flows.
///
/// The *directions and the aggregate shares* are reported figures; the split of
/// each reported aggregate across this model's blocs is not, because the blocs are
/// political groupings that no statistical agency reports as a unit.
pub const ENERGY_FLOW_PROVENANCE: Provenance = Provenance::Sourced {
    source: "Russian oil export destinations (Deputy PM Novak, 2025: about 80% of \
             roughly 238 Mt went to China and India) and the composition of EU gas \
             imports (Eurostat via TASS, 2025: Russian LNG 16.1% and pipeline gas \
             16.3%, US 52.5% of LNG, Algeria 27.4%, Norway 24.9%, Azerbaijan 12.8%). \
             The split within each reported aggregate is illustrative",
    vintage: "2025",
};

/// How exposed one bloc is to the energy trade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyExposure {
    /// Share of the bloc's primary energy that is imported.
    pub import_dependence: f64,
    /// Share of the bloc's export earnings that come from energy.
    pub export_dependence: f64,
    pub provenance: Provenance,
}

impl EnergyExposure {
    /// How much an energy disruption hurts this bloc, all else equal.
    ///
    /// Importers lose supply; exporters lose revenue. Both are damage, so both
    /// count, which is why this is a sum rather than a signed quantity.
    pub fn disruption_exposure(&self) -> f64 {
        (self.import_dependence + self.export_dependence).clamp(0.0, 1.0)
    }
}

/// A directed energy flow between two blocs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyFlow {
    pub from: usize,
    pub to: usize,
    /// Share of `from`'s energy exports that reach `to`.
    pub share: f64,
}

/// The whole economic layer's state.
#[derive(Debug, Clone, PartialEq)]
pub struct Economy {
    /// Reserve-currency issuance by bloc, summing to `1 - unattributed`.
    pub reserve_shares: Vec<f64>,
    /// The part of world reserves in currencies belonging to no bloc.
    pub unattributed_reserves: f64,
    pub reserve_provenance: Provenance,
    pub energy: Vec<EnergyExposure>,
    pub flows: Vec<EnergyFlow>,
    /// Global financial-conditions index: 0 calm, 1 acute. Gated into the shock
    /// process, so a strained financial system makes crises more likely.
    pub recession_risk: f64,
}

/// Default energy exposure per bloc.
///
/// The *directions* are the reported ones. The magnitudes are illustrative except
/// where a share is quoted, because the model's blocs are political groupings that
/// no statistical agency reports as a unit -- "Atlantic" spans the United States and
/// the euro area, whose energy positions are opposite.
pub fn default_energy_exposure(blocs: &[PowerBloc]) -> Vec<EnergyExposure> {
    let illustrative = |import: f64, export| {
        (
            import,
            export,
            Provenance::Illustrative {
                rationale: "bloc aggregates are political groupings, not statistical \
                            units; magnitudes are plausible, directions are reported",
            },
        )
    };
    let mut out = Vec::with_capacity(blocs.len());
    for bloc in blocs {
        let (import, export, provenance) = match bloc.name.as_str() {
            // The United States is a net exporter of oil and LNG while the euro area
            // is a large importer, so the bloc aggregate is close to balanced.
            "Atlantic" => illustrative(0.35, 0.25),
            // The world's largest crude importer, with a gas import bill to match.
            "Sinic" => illustrative(0.72, 0.02),
            // The opposite position: energy is the core of its export revenue.
            "Eurasian" => illustrative(0.02, 0.55),
            // Japan, Korea, India and ASEAN are all import-dependent.
            "Indo-Pacific" => illustrative(0.65, 0.06),
            // Sub-Saharan Africa plus the Maghreb: oil and gas exporters -- Nigeria,
            // Angola, Algeria, Libya -- alongside importers, so the aggregate is a net
            // exporter that still carries a real import bill.
            "Africa" => illustrative(0.20, 0.45),
            // The peninsula exporters. The most export-dependent position in the model
            // and the least import-dependent.
            "Gulf" => illustrative(0.05, 0.70),
            // What is left of the old residual: Latin America, the non-aligned parts of
            // South and Southeast Asia, Turkey. Mixed, and closer to balanced.
            "Non-Aligned" => illustrative(0.35, 0.25),
            // The AI actor of `ai.rs`, present only when AI is modelled as a player.
            // Stated rather than left to the generic fallback below, which would
            // have it exporting energy it does not produce: a compute complex holds
            // no oil or gas, and runs on purchased electricity rather than on its
            // own generation.
            "AI-Compute" => illustrative(0.95, 0.00),
            _ => illustrative(0.40, 0.20),
        };
        out.push(EnergyExposure {
            import_dependence: import,
            export_dependence: export,
            provenance,
        });
    }
    out
}

/// Directed energy flows, built from reported trade shares.
///
/// The two anchors that carry real information here are Russia's oil export
/// destinations and the composition of EU gas imports, because both are the direct
/// consequence of the rupture this model is about.
pub fn default_energy_flows(blocs: &[PowerBloc]) -> Vec<EnergyFlow> {
    let index = |name: &str| blocs.iter().position(|b| b.name == name);
    let mut flows = Vec::new();
    let mut push = |from: Option<usize>, to: Option<usize>, share: f64| {
        if let (Some(from), Some(to)) = (from, to) {
            if from != to {
                flows.push(EnergyFlow { from, to, share });
            }
        }
    };

    let atlantic = index("Atlantic");
    let sinic = index("Sinic");
    let eurasian = index("Eurasian");
    let indo = index("Indo-Pacific");
    let africa = index("Africa");
    let gulf = index("Gulf");

    // Sourced: ~80% of Russian oil exports went to China and India in 2025, out of
    // about 238 million tonnes. The split between the two is illustrative.
    push(eurasian, sinic, 0.48);
    push(eurasian, indo, 0.32);
    // The residual reaches Europe and others, much reduced but not zero.
    push(eurasian, atlantic, 0.12);

    // Sourced: Russian LNG was 16.1% and Russian pipeline gas 16.3% of EU imports by
    // value in 2025, against a US share of 52.5% of LNG. The last is intra-bloc
    // trade for this model, so it does not appear as a flow.
    push(eurasian, atlantic, 0.0);

    // Sourced: Algeria 27.4%, Norway 24.9% and Azerbaijan 12.8% of EU pipeline gas.
    // Algeria is the supplier this flow is built from, and it is why the flow leaves
    // the African bloc: before the regional split it left "Non-Aligned", a bloc whose
    // own description never named Africa, so the single most-sourced number in this
    // layer was attached to a region the model did not have.
    push(africa, atlantic, 0.30);

    // The Gulf supplies the two Asian importers. This is the bulk of what the old
    // non-aligned aggregate exported, and the Gulf is what exports it.
    push(gulf, sinic, 0.24);
    push(gulf, indo, 0.22);

    flows
}

impl Economy {
    /// Build the default layer for a bloc set.
    pub fn for_blocs(blocs: &[PowerBloc]) -> Self {
        let (reserve_shares, unattributed_reserves, reserve_provenance) =
            default_reserve_shares(blocs);
        Economy {
            reserve_shares,
            unattributed_reserves,
            reserve_provenance,
            energy: default_energy_exposure(blocs),
            flows: default_energy_flows(blocs),
            recession_risk: 0.0,
        }
    }

    /// Energy interdependence between two blocs, in [0, 1].
    ///
    /// A pair that trades energy heavily has more to lose from a rupture, which is
    /// the mechanism that makes cooperation worth more between them. This is the
    /// channel that enters the *payoff matrix* -- see the module doc.
    pub fn interdependence(&self, a: usize, b: usize) -> f64 {
        let direct: f64 = self
            .flows
            .iter()
            .filter(|f| (f.from == a && f.to == b) || (f.from == b && f.to == a))
            .map(|f| f.share)
            .sum();
        // Dependence on third parties still makes both parties prefer a stable
        // system, but it is a weaker link than trading with each other, so it enters
        // at a reduced weight.
        let via_exposure = self
            .energy
            .get(a)
            .map(|e| e.disruption_exposure())
            .unwrap_or(0.0)
            + self
                .energy
                .get(b)
                .map(|e| e.disruption_exposure())
                .unwrap_or(0.0);
        (direct + 0.25 * via_exposure / 2.0).clamp(0.0, 1.0)
    }

    /// How much leverage `a` holds over `b` through the money they both use, in
    /// [0, 1].
    ///
    /// The issuer of a widely held reserve currency can restrict access to it, so
    /// leverage rises with the issuer's share. It falls as the other side's own
    /// issuance rises, because a bloc with an alternative has somewhere to go. This
    /// is an **illustrative** transmission: the mechanism is well attested, the
    /// coefficient is not measured anywhere.
    pub fn monetary_leverage(&self, a: usize, b: usize) -> f64 {
        const LEVERAGE_PER_SHARE: f64 = 1.4;
        let own = self.reserve_shares.get(a).copied().unwrap_or(0.0);
        let other = self.reserve_shares.get(b).copied().unwrap_or(0.0);
        (LEVERAGE_PER_SHARE * (own - other)).clamp(0.0, 1.0)
    }

    /// Advance the financial-conditions index one year.
    ///
    /// Financial strain accumulates from tension and from disruption to the energy
    /// network, and decays on its own. `recession_risk` then gates the shock process
    /// upstream, which is what turns it from a reported number into a mechanism.
    pub fn step_financial_conditions(&mut self, tension: f64, energy_disruption: f64) {
        const TENSION_WEIGHT: f64 = 0.06;
        const ENERGY_WEIGHT: f64 = 0.30;
        const DECAY: f64 = 0.25;

        let strain = TENSION_WEIGHT * tension + ENERGY_WEIGHT * energy_disruption;
        self.recession_risk = (self.recession_risk * (1.0 - DECAY) + strain).clamp(0.0, 1.0);
    }

    /// Every provenance this layer carries, for the report and for the completeness
    /// test. Keeps the two in step by construction.
    pub fn provenance_table(&self) -> Vec<(&'static str, Provenance)> {
        let mut rows = vec![
            ("reserve shares (totals)", self.reserve_provenance),
            ("reserve allocation (split)", RESERVE_ALLOCATION_PROVENANCE),
            ("energy flows (directions)", ENERGY_FLOW_PROVENANCE),
        ];
        for exposure in self.energy.iter() {
            rows.push(("energy exposure", exposure.provenance));
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::default_blocs;

    fn economy() -> (Vec<PowerBloc>, Economy) {
        let blocs = default_blocs();
        let economy = Economy::for_blocs(&blocs);
        (blocs, economy)
    }

    /// The reserve shares must reproduce the published totals, or the anchor is not
    /// actually the anchor.
    #[test]
    fn reserve_shares_reproduce_the_published_totals() {
        let (_, economy) = economy();
        let total: f64 = economy.reserve_shares.iter().sum::<f64>() + economy.unattributed_reserves;
        assert!(
            (total - 1.0).abs() < 1e-9,
            "reserve shares plus the unattributed residual must sum to 1, got {total}"
        );

        // 56.77 (USD) + 20.25 (EUR) + 10.7 (illustrative Atlantic share of minor
        // currencies) = 87.72% for the Atlantic bloc.
        let atlantic = economy.reserve_shares[0];
        assert!(
            (atlantic - 0.8772).abs() < 1e-9,
            "Atlantic should hold the dollar and the euro, got {atlantic}"
        );

        // The renminbi is the only other individually identified share that maps to
        // a bloc, and it is small. A model that made it large would be contradicting
        // its own source.
        assert!(
            (economy.reserve_shares[1] - 0.0195).abs() < 1e-9,
            "Sinic should be the renminbi share alone, got {}",
            economy.reserve_shares[1]
        );
        assert!(
            economy.reserve_shares[1] < 0.05,
            "the renminbi's reserve share is a few percent, not a rival to the dollar"
        );
    }

    /// The unattributed residual must be carried, not quietly folded into a bloc.
    #[test]
    fn the_unattributed_residual_survives() {
        let (_, economy) = economy();
        assert!(
            (economy.unattributed_reserves - 0.0613).abs() < 1e-9,
            "6.13% of reserves are in currencies COFER does not identify, got {}",
            economy.unattributed_reserves
        );
    }

    /// Leverage must be directional and bounded. This is an asymmetry that acts on a
    /// bloc's *power* rather than on its choices, so it has to be right here.
    #[test]
    fn monetary_leverage_is_directional_and_bounded() {
        let (blocs, economy) = economy();
        // The Atlantic bloc issues the dollar and the euro; the Eurasian bloc issues
        // no reserve currency at all.
        let atlantic_over_eurasian = economy.monetary_leverage(0, 2);
        let eurasian_over_atlantic = economy.monetary_leverage(2, 0);
        assert!(
            atlantic_over_eurasian > eurasian_over_atlantic,
            "leverage must run from issuer to non-issuer: {atlantic_over_eurasian} vs \
             {eurasian_over_atlantic}"
        );
        assert_eq!(
            eurasian_over_atlantic, 0.0,
            "a bloc issuing no reserve currency has no monetary leverage"
        );
        for a in 0..blocs.len() {
            for b in 0..blocs.len() {
                let value = economy.monetary_leverage(a, b);
                assert!(
                    (0.0..=1.0).contains(&value),
                    "leverage out of range for {a} over {b}: {value}"
                );
            }
        }
    }

    /// Interdependence must be symmetric, because it describes the *pair* rather than
    /// either side of it: both blocs feel the same trade link. An asymmetric value
    /// would mean the two sides of one relationship had different beliefs about it,
    /// which would be a defect rather than a modelling choice.
    #[test]
    fn interdependence_is_symmetric() {
        let (blocs, economy) = economy();
        for a in 0..blocs.len() {
            for b in 0..blocs.len() {
                let ab = economy.interdependence(a, b);
                let ba = economy.interdependence(b, a);
                assert!(
                    (ab - ba).abs() < 1e-12,
                    "interdependence must not depend on order: {ab} vs {ba}"
                );
                assert!((0.0..=1.0).contains(&ab), "out of range: {ab}");
            }
        }
    }

    /// The reported energy directions must survive into the model, since they are
    /// the part of this layer that is not invented.
    #[test]
    fn energy_flows_follow_the_reported_directions() {
        let (blocs, economy) = economy();
        let name = |index: usize| blocs[index].name.as_str();
        let total_from = |index: usize| -> f64 {
            economy
                .flows
                .iter()
                .filter(|f| f.from == index)
                .map(|f| f.share)
                .sum()
        };

        let eurasian = blocs.iter().position(|b| b.name == "Eurasian").unwrap();
        let sinic = blocs.iter().position(|b| b.name == "Sinic").unwrap();
        let indo = blocs.iter().position(|b| b.name == "Indo-Pacific").unwrap();

        let to_asia: f64 = economy
            .flows
            .iter()
            .filter(|f| f.from == eurasian && (f.to == sinic || f.to == indo))
            .map(|f| f.share)
            .sum();
        assert!(
            to_asia > 0.7,
            "about 80% of Russian oil exports go to China and India, got {to_asia}"
        );
        assert!(
            total_from(eurasian) <= 1.0 + 1e-9,
            "a bloc cannot export more than it exports"
        );
        let _ = name;

        // And the attribution the regional split was for: the two anchors must leave
        // the blocs that actually supply them. Both used to leave "Non-Aligned", which
        // named neither Africa nor the Gulf.
        let africa = blocs.iter().position(|b| b.name == "Africa").unwrap();
        let gulf = blocs.iter().position(|b| b.name == "Gulf").unwrap();
        let atlantic = blocs.iter().position(|b| b.name == "Atlantic").unwrap();
        let residual = blocs.iter().position(|b| b.name == "Non-Aligned").unwrap();

        assert!(
            total_from(africa) > 0.0,
            "Algeria's 27.4% of EU pipeline gas must leave the African bloc, not the \
             residual it used to be filed under"
        );
        assert!(
            economy
                .flows
                .iter()
                .any(|f| f.from == africa && f.to == atlantic),
            "the Algerian flow is directed at the Atlantic bloc"
        );
        let gulf_to_asia: f64 = economy
            .flows
            .iter()
            .filter(|f| f.from == gulf && (f.to == sinic || f.to == indo))
            .map(|f| f.share)
            .sum();
        assert!(
            gulf_to_asia > 0.4,
            "the Gulf supplies both Asian importers, got {gulf_to_asia}"
        );
        assert_eq!(
            total_from(residual),
            0.0,
            "the residual bloc is what is left after the named regions, so nothing in \
             this layer should be attributed to it"
        );
    }

    /// Financial conditions must be bounded, must respond to both inputs, and must
    /// decay back toward calm when the strain is removed.
    #[test]
    fn financial_conditions_respond_and_decay() {
        let mut calm = Economy::for_blocs(&default_blocs());
        for _ in 0..50 {
            calm.step_financial_conditions(0.0, 0.0);
        }
        assert_eq!(calm.recession_risk, 0.0, "no strain must leave no strain");

        let mut tense = Economy::for_blocs(&default_blocs());
        for _ in 0..50 {
            tense.step_financial_conditions(3.0, 1.0);
        }
        assert!(
            tense.recession_risk > 0.8,
            "sustained tension and energy disruption must produce acute strain, got {}",
            tense.recession_risk
        );
        assert!(tense.recession_risk <= 1.0);

        // Removing the strain must let it decay rather than latch.
        for _ in 0..50 {
            tense.step_financial_conditions(0.0, 0.0);
        }
        assert!(
            tense.recession_risk < 0.01,
            "strain must decay once the cause is gone, got {}",
            tense.recession_risk
        );
    }

    /// Every parameter this layer exposes must carry a provenance that says
    /// something. A blank or placeholder would defeat the point of having the enum.
    #[test]
    fn every_parameter_declares_where_it_came_from() {
        let (_, economy) = economy();
        let rows = economy.provenance_table();
        assert!(
            rows.len() >= 7,
            "expected an entry per exposure plus the reserve rows, got {}",
            rows.len()
        );
        for (what, provenance) in rows {
            assert!(
                !what.trim().is_empty(),
                "a provenance row must name what it describes"
            );
            let detail = provenance.detail();
            assert!(
                detail.len() > 30,
                "{what} has a provenance that says nothing: {detail:?}"
            );
            // A sourced claim must name a source *and* a vintage; otherwise it is
            // not checkable.
            if let Provenance::Sourced { source, vintage } = provenance {
                assert!(!source.trim().is_empty(), "{what} is sourced to nothing");
                assert!(!vintage.trim().is_empty(), "{what} has no vintage");
            }
        }
    }

    /// The energy *flows* are the most-sourced part of this layer, and they were
    /// missing from the provenance table in its first version -- which understated
    /// how much of the model rests on published figures. This pins them in.
    #[test]
    fn the_provenance_table_covers_the_sourced_energy_flows() {
        let (_, economy) = economy();
        assert!(
            ENERGY_FLOW_PROVENANCE.is_sourced(),
            "the reported flow directions are a sourced figure"
        );
        assert!(
            economy
                .provenance_table()
                .iter()
                .any(|(what, provenance)| what.contains("flows") && provenance.is_sourced()),
            "the flow directions must appear in the table, and as sourced"
        );
    }
}
