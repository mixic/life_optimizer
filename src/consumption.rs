//! Consumption realism: elasticity tiers and the sparing model.
//!
//! Implements the consumption-side theory already written down in
//! `THEORY_OF_SPARING.md` §7c and §8, and `CRITICS_CURRENT_WORK.md` §2. The
//! optimizer previously treated one flat "requirements" total as mandatory,
//! which conflated costs a household genuinely cannot avoid with spending it
//! chooses. That made the feasibility test far stricter than the theory it
//! claims to implement: the document states that the same work percentage
//! "can be feasible under an extreme-saving profile and infeasible under a
//! luxury profile" — which is only true if lifestyle spending is separable
//! from the mandatory floor.
//!
//! The model is laid out in three tiers:
//!
//! | Tier | Meaning | Responds to sparing? |
//! |---|---|---|
//! | Inelastic | Rent, food, transport, insurance, childcare, healthcare | No |
//! | Quasi-inelastic | Locked in by switching costs (ecosystem effects) | Partially |
//! | Elastic | Frills and durables | Yes, via the sparing ratio |
//!
//! Total consumption follows `THEORY_OF_SPARING.md` §8:
//!
//! ```text
//! C_t = (R_t + E_t)  +  Q_t  +  L_t  +  D_t
//!       inelastic       quasi   elastic
//! ```
//!
//! with the elastic tier priced at its sparing-adjusted effective cost:
//!
//! ```text
//! multiplier = lifestyle_profile * (1 - sigma * (1 - phi)) * utilization_penalty
//! ```
//!
//! where `sigma` is the sparing ratio, `phi` the second-hand price ratio, and
//! the utilization penalty inflates the effective cost of low-utilization
//! purchases, as the document argues: a discount changes price, not
//! utilization.
//!
//! The multiplier is normalized so that a household which applies **full**
//! purchase discipline and does no sparing sits at exactly `1.0`. That is the
//! neutral reference the model is measured against; anything below it is a real
//! saving, and anything above it is the cost of buying without filtering.

use serde::{Deserialize, Serialize};

/// Average second-hand-to-new price ratio `phi` when a purchase is spared.
///
/// Not currently user-settable — it is a slowly-moving market property rather
/// than a household decision. Exposed as a constant so the calibration is
/// visible and testable rather than buried in an expression.
pub const SECOND_HAND_PRICE_RATIO: f64 = 0.60;

/// Utilization `rho_use` achieved when the household applies no prioritization
/// discipline at all. A household that buys on impulse recovers only about
/// two-thirds of the value it pays for, per the document's argument that low
/// utilization inflates effective cost even at a discount.
pub const MIN_UTILIZATION: f64 = 0.65;

/// Utilization `rho_use` achieved under full discipline. This is the reference
/// point: it is where the utilization penalty is exactly 1.0, so a household
/// that filters every purchase pays no penalty and no discount — the neutral
/// case the sparing model is measured against.
pub const REFERENCE_UTILIZATION: f64 = 0.85;

/// Which costs can actually absorb a sparing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElasticityTier {
    /// Rent, food, transport, insurance, childcare, healthcare. No real
    /// elastic margin — `THEORY_OF_SPARING.md` §7a.
    Inelastic,
    /// Nominally discretionary but locked in by switching costs — §7b.
    QuasiInelastic,
    /// Genuinely discretionary and sparing-eligible.
    Elastic,
}

impl ElasticityTier {
    pub fn label(&self) -> &'static str {
        match self {
            ElasticityTier::Inelastic => "Inelastic (non-reducible)",
            ElasticityTier::QuasiInelastic => "Quasi-inelastic (partially reducible)",
            ElasticityTier::Elastic => "Elastic (sparing-eligible)",
        }
    }
}

/// A lifestyle baseline, from `CRITICS_CURRENT_WORK.md` §2.1.
///
/// These are alternative spending profiles, not moral judgments — the document
/// is explicit about that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumptionProfile {
    /// Minimal discretionary spending and strict cost control.
    ExtremeSaving,
    /// Controlled spending with some flexibility.
    Moderate,
    /// An ordinary expected standard of living.
    Normal,
    /// Premium services, travel, and high discretionary spending.
    Luxury,
}

impl ConsumptionProfile {
    /// Discretionary multiplier for the profile, from the calibration table in
    /// `MATHEMATICS.md` §3.1.
    pub fn discretionary_multiplier(&self) -> f64 {
        match self {
            ConsumptionProfile::ExtremeSaving => 0.50,
            ConsumptionProfile::Moderate => 0.80,
            ConsumptionProfile::Normal => 1.00,
            ConsumptionProfile::Luxury => 1.75,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConsumptionProfile::ExtremeSaving => "extreme saving",
            ConsumptionProfile::Moderate => "moderate",
            ConsumptionProfile::Normal => "normal",
            ConsumptionProfile::Luxury => "luxury",
        }
    }

    /// Parse a CLI value. Returns `None` for anything unrecognised so callers
    /// can reject it rather than silently defaulting.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "extreme-saving" | "extreme" | "saving" => Some(ConsumptionProfile::ExtremeSaving),
            "moderate" => Some(ConsumptionProfile::Moderate),
            "normal" | "standard" => Some(ConsumptionProfile::Normal),
            "luxury" => Some(ConsumptionProfile::Luxury),
            _ => None,
        }
    }
}

/// Household sparing and lifestyle parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConsumptionProfileConfig {
    pub profile: ConsumptionProfile,
    /// `sigma` in `[0, 1]`: fraction of elastic spending sourced second-hand,
    /// borrowed, or shared rather than new.
    pub sparing_ratio: f64,
    /// In `[0, 1]`: how strictly the household applies the prioritization
    /// hierarchy before any purchase. Raises effective utilization.
    pub utilization_discipline: f64,
    /// In `[0, 1]`: share of nominally discretionary spending actually locked in
    /// by switching costs, and therefore only partially responsive to sparing.
    pub quasi_inelastic_share: f64,
}

impl Default for ConsumptionProfileConfig {
    /// A normal lifestyle with no sparing. This is deliberately behaviour-
    /// preserving relative to a flat basket: the sparing model contributes no
    /// discount until the household actually declares one.
    fn default() -> Self {
        Self {
            profile: ConsumptionProfile::Normal,
            sparing_ratio: 0.0,
            utilization_discipline: 0.0,
            quasi_inelastic_share: 0.0,
        }
    }
}

impl ConsumptionProfileConfig {
    pub fn new(profile: ConsumptionProfile) -> Self {
        Self {
            profile,
            ..Self::default()
        }
    }

    pub fn with_sparing_ratio(mut self, ratio: f64) -> Self {
        self.sparing_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    pub fn with_utilization_discipline(mut self, discipline: f64) -> Self {
        self.utilization_discipline = discipline.clamp(0.0, 1.0);
        self
    }

    pub fn with_quasi_inelastic_share(mut self, share: f64) -> Self {
        self.quasi_inelastic_share = share.clamp(0.0, 1.0);
        self
    }

    /// Utilization rate implied by the discipline setting.
    ///
    /// Interpolates between [`MIN_UTILIZATION`] (no discipline) and
    /// [`REFERENCE_UTILIZATION`] (full discipline).
    pub fn utilization_rate(&self) -> f64 {
        MIN_UTILIZATION
            + (REFERENCE_UTILIZATION - MIN_UTILIZATION)
                * self.utilization_discipline.clamp(0.0, 1.0)
    }

    /// The utilization penalty `rho_use^-1`, normalized so that full discipline
    /// yields exactly 1.0. Applying the raw reciprocal would impose a penalty
    /// even on a household practising no sparing at all, which silently
    /// inflated the reference basket; normalizing keeps the no-sparing case
    /// neutral, as the theory intends.
    pub fn utilization_penalty(&self) -> f64 {
        REFERENCE_UTILIZATION / self.utilization_rate().max(0.10)
    }

    /// The elastic-tier multiplier: sparing discount times utilization penalty,
    /// then scaled by the lifestyle profile.
    ///
    /// Note the two effects pull in opposite directions, which is the point of
    /// the theory — sparing lowers the price paid, while low utilization raises
    /// the effective cost per unit of realized value.
    pub fn discretionary_multiplier(&self) -> f64 {
        let sparing_discount = 1.0 - self.sparing_ratio * (1.0 - SECOND_HAND_PRICE_RATIO);
        self.profile.discretionary_multiplier() * sparing_discount * self.utilization_penalty()
    }

    /// Whether the household declared any sparing behaviour at all.
    pub fn practices_sparing(&self) -> bool {
        self.sparing_ratio > 0.0 || self.utilization_discipline > 0.0
    }
}

/// Consumption split by elasticity tier, in CHF per month.
///
/// The point of carrying this structure rather than a single total is the one
/// `THEORY_OF_SPARING.md` §7c makes: a household with rising inelastic costs can
/// practice maximum sparing discipline on the elastic tier and still see little
/// movement in total consumption. Reporting only the aggregate hides where the
/// squeeze is landing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConsumptionTiers {
    /// `R_t + E_t` — rent and essential costs. Never reduced by sparing.
    pub inelastic: f64,
    /// `Q_t` — nominally discretionary but locked in by switching costs.
    pub quasi_inelastic: f64,
    /// `L_t` — sparing-eligible spending, already multiplied.
    pub elastic: f64,
    /// `D_t` — fixed contractual outflows (savings goal, vacation sinking fund).
    pub committed_outflows: f64,
    /// The multiplier actually applied to the elastic tier.
    pub applied_multiplier: f64,
}

impl ConsumptionTiers {
    /// The mandatory floor: costs the household cannot avoid next month.
    ///
    /// This is the right quantity for the feasibility test. The full basket
    /// including lifestyle spending is still reported as a target, but a
    /// household is not "unable to afford" 80% work merely because it would
    /// have to trim discretionary spending.
    pub fn mandatory_monthly(&self) -> f64 {
        self.inelastic + self.quasi_inelastic
    }

    /// The full lifestyle-inclusive basket: what the household would like to
    /// spend, including savings goal and discretionary spending at its
    /// sparing-adjusted level.
    pub fn lifestyle_target_monthly(&self) -> f64 {
        self.mandatory_monthly() + self.elastic + self.committed_outflows
    }

    /// Total monthly consumption `C_t`.
    pub fn total_monthly(&self) -> f64 {
        self.lifestyle_target_monthly()
    }

    /// Amount the household would spend before applying sparing and the
    /// lifestyle profile — the baseline against which savings from sparing can
    /// be quantified.
    pub fn unadjusted_discretionary(&self) -> f64 {
        if self.applied_multiplier > 0.0 {
            self.elastic / self.applied_multiplier
        } else {
            self.elastic
        }
    }
}
