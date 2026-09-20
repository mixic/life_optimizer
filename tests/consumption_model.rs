//! Integration tests for the elasticity-tiered consumption model.
//!
//! Covers `THEORY_OF_SPARING.md` §7c and §8, and `CRITICS_CURRENT_WORK.md` §2:
//! the tier split, the sparing ratio, the utilization discipline, the
//! quasi-inelastic share, and the lifestyle profiles from `MATHEMATICS.md` §3.1.

use life_optimizer::consumption::{
    ConsumptionProfile, ConsumptionProfileConfig, MIN_UTILIZATION, REFERENCE_UTILIZATION,
    SECOND_HAND_PRICE_RATIO,
};
use life_optimizer::requirements::PersonalRequirements;

const CHILDLESS: u32 = 0;

fn requirements(children: u32) -> PersonalRequirements {
    PersonalRequirements::bern_family_default(children)
}

/// Full discipline with no sparing and a normal lifestyle is the neutral
/// point: multiplier exactly 1.0.
///
/// This is the invariant that failed first time round — the utilization
/// penalty was normalized against an arbitrary baseline, so the default
/// configuration silently inflated discretionary spending by ~54% and made
/// households look less affordable than they were.
#[test]
fn neutral_configuration_has_unit_multiplier() {
    let neutral = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_sparing_ratio(0.0);
    assert!(
        (neutral.discretionary_multiplier() - 1.0).abs() < 1e-9,
        "full discipline + no sparing must be neutral, got {}",
        neutral.discretionary_multiplier()
    );
}

/// The utilization penalty must be bounded and must not exceed 1.0 at full
/// discipline. A default that penalizes spending is a calibration bug.
#[test]
fn utilization_penalty_is_bounded_and_normalized() {
    let undisciplined =
        ConsumptionProfileConfig::new(ConsumptionProfile::Normal).with_utilization_discipline(0.0);
    let disciplined =
        ConsumptionProfileConfig::new(ConsumptionProfile::Normal).with_utilization_discipline(1.0);

    assert!((disciplined.utilization_penalty() - 1.0).abs() < 1e-9);
    assert!(
        undisciplined.utilization_penalty() > 1.0,
        "a household that filters nothing should pay a penalty"
    );
    assert!(
        undisciplined.utilization_penalty() < 1.5,
        "the penalty must stay modest, got {}",
        undisciplined.utilization_penalty()
    );

    // Utilization is monotone in discipline and respects its bounds.
    assert!((undisciplined.utilization_rate() - MIN_UTILIZATION).abs() < 1e-9);
    assert!((disciplined.utilization_rate() - REFERENCE_UTILIZATION).abs() < 1e-9);
}

/// The sparing discount: with the profile held neutral, more sparing must
/// strictly lower the multiplier, and the full-sparing case must equal the
/// second-hand price ratio.
#[test]
fn sparing_ratio_lowers_the_multiplier() {
    let base =
        ConsumptionProfileConfig::new(ConsumptionProfile::Normal).with_utilization_discipline(1.0);

    let half = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_sparing_ratio(0.5);
    let full = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_sparing_ratio(1.0);

    assert!(half.discretionary_multiplier() < base.discretionary_multiplier());
    assert!(full.discretionary_multiplier() < half.discretionary_multiplier());
    assert!(
        (full.discretionary_multiplier() - SECOND_HAND_PRICE_RATIO).abs() < 1e-9,
        "buying everything spared should cost the second-hand ratio, got {}",
        full.discretionary_multiplier()
    );
}

/// The four lifestyle profiles must be strictly ordered, per the table in
/// `MATHEMATICS.md` §3.1.
#[test]
fn lifestyle_profiles_are_strictly_ordered() {
    let m = |p: ConsumptionProfile| p.discretionary_multiplier();
    assert!(m(ConsumptionProfile::ExtremeSaving) < m(ConsumptionProfile::Moderate));
    assert!(m(ConsumptionProfile::Moderate) < m(ConsumptionProfile::Normal));
    assert!(m(ConsumptionProfile::Normal) < m(ConsumptionProfile::Luxury));

    // The documented values.
    assert!((m(ConsumptionProfile::ExtremeSaving) - 0.50).abs() < 1e-9);
    assert!((m(ConsumptionProfile::Moderate) - 0.80).abs() < 1e-9);
    assert!((m(ConsumptionProfile::Normal) - 1.00).abs() < 1e-9);
    assert!((m(ConsumptionProfile::Luxury) - 1.75).abs() < 1e-9);
}

/// Profile parsing must accept documented spellings and reject junk rather
/// than silently defaulting.
#[test]
fn profile_parsing_is_strict() {
    assert_eq!(
        ConsumptionProfile::parse("extreme-saving"),
        Some(ConsumptionProfile::ExtremeSaving)
    );
    assert_eq!(
        ConsumptionProfile::parse("EXTREME_SAVING"),
        Some(ConsumptionProfile::ExtremeSaving)
    );
    assert_eq!(
        ConsumptionProfile::parse("Luxury"),
        Some(ConsumptionProfile::Luxury)
    );
    assert_eq!(
        ConsumptionProfile::parse("normal"),
        Some(ConsumptionProfile::Normal)
    );
    assert_eq!(
        ConsumptionProfile::parse("moderate"),
        Some(ConsumptionProfile::Moderate)
    );
    assert_eq!(ConsumptionProfile::parse("lavish"), None);
    assert_eq!(ConsumptionProfile::parse(""), None);
}

/// Every field of the requirement basket must land in exactly one tier — no
/// double counting and nothing dropped.
#[test]
fn tiers_partition_the_basket_exactly() {
    let req = requirements(CHILDLESS);
    let neutral =
        ConsumptionProfileConfig::new(ConsumptionProfile::Normal).with_utilization_discipline(1.0);
    let tiers = req.elasticity_tiers(&neutral);

    let tier_total = tiers.mandatory_monthly() + tiers.elastic + tiers.committed_outflows;
    assert!(
        (tier_total - req.total_monthly()).abs() < 1e-9,
        "tiers must sum to the full basket: tiers={tier_total}, total={}",
        req.total_monthly()
    );

    // Housing/childcare are the big inelastic items and must not be in elastic.
    assert!(tiers.inelastic >= req.housing);
    assert!(tiers.inelastic >= req.childcare);
}

/// The mandatory floor must always be below the full basket whenever the
/// household has any discretionary or committed spending. Feasibility is tested
/// against the floor, so getting this backwards would invert the whole model.
#[test]
fn mandatory_floor_is_below_full_basket() {
    for children in [0u32, 1, 2, 3] {
        let req = requirements(children);
        let config = ConsumptionProfileConfig::new(ConsumptionProfile::Normal);
        let mandatory = req.mandatory_monthly(&config);
        let target = req.target_monthly(&config);

        assert!(
            mandatory < target,
            "children={children}: floor {mandatory} should be below basket {target}"
        );
        // The gap is exactly the discretionary + committed spending.
        let gap = target - mandatory;
        assert!(gap > 0.0);
    }
}

/// A luxury lifestyle must make the household less affordable, and extreme
/// saving more affordable — the central claim of `MATHEMATICS.md` §3.1 that the
/// same work percentage can be feasible under one profile and infeasible under
/// another.
#[test]
fn profile_changes_the_lifestyle_basket_but_not_the_floor() {
    let req = requirements(2);

    let saving = ConsumptionProfileConfig::new(ConsumptionProfile::ExtremeSaving);
    let luxury = ConsumptionProfileConfig::new(ConsumptionProfile::Luxury);

    // The floor is lifestyle-independent: rent does not fall because you are
    // frugal, which is the section 7c point.
    assert!(
        (req.mandatory_monthly(&saving) - req.mandatory_monthly(&luxury)).abs() < 1e-9,
        "the mandatory floor must not depend on the lifestyle profile"
    );

    assert!(
        req.target_monthly(&saving) < req.target_monthly(&luxury),
        "extreme saving must need a smaller basket than luxury"
    );
}

/// The quasi-inelastic share must move spending out of the elastic tier and
/// into the quasi-inelastic one, leaving the floor and basket totals in the
/// right relationship.
#[test]
fn quasi_inelastic_share_moves_spending_between_tiers() {
    let req = requirements(CHILDLESS);
    let none = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_quasi_inelastic_share(0.0);
    let some = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_quasi_inelastic_share(0.5);

    let a = req.elasticity_tiers(&none);
    let b = req.elasticity_tiers(&some);

    assert!(
        b.quasi_inelastic > a.quasi_inelastic,
        "share should populate the quasi tier"
    );
    assert!(b.elastic < a.elastic, "share should drain the elastic tier");
    // Locking spending into the quasi tier raises the floor, because the floor
    // includes quasi-inelastic costs.
    assert!(b.mandatory_monthly() > a.mandatory_monthly());
}

/// Sparing must actually reduce the reported basket, which is the whole point:
/// a household practising sparing needs less income for the same lifestyle.
#[test]
fn sparing_reduces_the_lifestyle_basket() {
    let req = requirements(2);
    let no_sparing =
        ConsumptionProfileConfig::new(ConsumptionProfile::Normal).with_utilization_discipline(1.0);
    let sparing = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_sparing_ratio(0.8);

    let a = req.elasticity_tiers(&no_sparing);
    let b = req.elasticity_tiers(&sparing);

    assert!(b.elastic < a.elastic, "sparing must lower elastic spending");
    // The floor is untouched: you cannot spare on rent.
    assert!((a.mandatory_monthly() - b.mandatory_monthly()).abs() < 1e-9);
}

/// A household with entirely inelastic spending cannot reduce its basket by
/// sparing at all. This is the §7c warning the tier report exists to surface.
#[test]
fn fully_inelastic_household_gains_nothing_from_sparing() {
    let mut req = requirements(CHILDLESS);
    // Strip everything sparing could touch.
    req.discretionary = 0.0;
    req.education = 0.0;
    req.savings_goal = 0.0;
    req.vacation = 0.0;

    let sparing = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_utilization_discipline(1.0)
        .with_sparing_ratio(1.0);
    let tiers = req.elasticity_tiers(&sparing);

    assert!((tiers.elastic - 0.0).abs() < 1e-9);
    assert!(
        (tiers.lifestyle_target_monthly() - tiers.mandatory_monthly()).abs() < 1e-9,
        "with no elastic spending the basket equals the floor"
    );
}

/// Parameters must be clamped rather than trusted, so an out-of-range CLI value
/// cannot produce a negative or exploding multiplier.
#[test]
fn out_of_range_parameters_are_clamped() {
    let high = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_sparing_ratio(5.0)
        .with_utilization_discipline(9.0)
        .with_quasi_inelastic_share(3.0);
    assert!((high.sparing_ratio - 1.0).abs() < 1e-9);
    assert!((high.utilization_discipline - 1.0).abs() < 1e-9);
    assert!((high.quasi_inelastic_share - 1.0).abs() < 1e-9);
    assert!(high.discretionary_multiplier() > 0.0);

    let low = ConsumptionProfileConfig::new(ConsumptionProfile::Normal)
        .with_sparing_ratio(-2.0)
        .with_utilization_discipline(-1.0)
        .with_quasi_inelastic_share(-4.0);
    assert!((low.sparing_ratio - 0.0).abs() < 1e-9);
    assert!((low.utilization_discipline - 0.0).abs() < 1e-9);
    assert!((low.quasi_inelastic_share - 0.0).abs() < 1e-9);
    assert!(low.discretionary_multiplier().is_finite());
}

/// Every requirement field must be non-negative for the tiers to be meaningful.
#[test]
fn default_baskets_have_positive_totals() {
    for children in [0u32, 1, 2, 3, 4] {
        let req = requirements(children);
        let config = ConsumptionProfileConfig::new(ConsumptionProfile::Normal);
        let tiers = req.elasticity_tiers(&config);
        assert!(
            tiers.inelastic > 0.0,
            "children={children}: inelastic must be positive"
        );
        assert!(tiers.mandatory_monthly() > 0.0);
        assert!(tiers.lifestyle_target_monthly() >= tiers.mandatory_monthly());
    }
}
