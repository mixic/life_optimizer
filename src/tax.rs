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

// Tax calculation - EXACT official Stadt Bern 2024 rates
// Source: https://www.bern.ch/themen/stadt-recht-und-politik/bern-in-zahlen/katost/18offver/jahresdaten/t-18-07-010-steuerbelastung-des-arbeitseinkommens.pdf
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBracket {
    pub threshold: f64,
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxDeductionBreakdown {
    pub childcare: f64,
    pub commuting: f64,
    pub work_equipment: f64,
    pub health_insurance: f64,
    pub rent: f64,
    pub family_specific: f64,
    pub deductible_total: f64,
    /// Not a deduction, not part of `deductible_total`, and **not used by any tax
    /// calculation** — nothing reads it but the display.
    ///
    /// It is `2%` of income *after* the deductible items, which is a figure with no
    /// source and no stated purpose. It used to be printed inside the deduction
    /// breakdown, where it read as part of the arithmetic leading to taxable
    /// income; a reader could not reconcile it and would reasonably conclude one of
    /// the lines was wrong. `FutureWork.md` §7's standard is that numbers be
    /// traceable, so an untraceable one is better withdrawn from the output than
    /// shown unexplained.
    ///
    /// The field is retained for serialised compatibility. See
    /// [`crate::deductions`] for the sourced replacement.
    pub non_deductible_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSchedule {
    pub canton_name: String,
    pub municipality_name: String,
    pub married: bool,                          // Track marital status
    pub children: u32,                          // Track children count
    pub family_tax_mode: bool,                  // Married parents can use enhanced family deductions
    // Lookup tables based on official data
    pub tax_table_single: Vec<(f64, f64)>,      // (income, tax_rate)
    pub tax_table_married_no_kids: Vec<(f64, f64)>, // (income, tax_rate) - married 0 kids
    pub tax_table_married_2kids: Vec<(f64, f64)>, // (income, tax_rate) - married 2+ kids
    pub social_security_rate: f64,     // AHV/IV/EO
    pub unemployment_rate: f64,        // ALV
    pub pension_rate: f64,             // BVG
}

impl TaxSchedule {
    /// Official Stadt Bern 2024 rates - exact from government document
    pub fn bern_city_default(married: bool, children: u32) -> Self {
        // Official tax rates from Stadt Bern document (2024)
        // These are Canton + Gemeinde + Kirche ONLY
        let tax_table_single = vec![
            (15_000.0, 0.0000),
            (20_000.0, 0.0217),
            (25_000.0, 0.0541),
            (30_000.0, 0.0725),
            (35_000.0, 0.0878),
            (40_000.0, 0.1008),
            (45_000.0, 0.1119),
            (50_000.0, 0.1200),
            (60_000.0, 0.1345),
            (70_000.0, 0.1453),
            (80_000.0, 0.1538),
            (90_000.0, 0.1626),
            (100_000.0, 0.1724),
            (125_000.0, 0.1939),
            (150_000.0, 0.2134),
            (175_000.0, 0.2317),
            (200_000.0, 0.2473),
        ];

        let tax_table_married_2kids = vec![
            (15_000.0, 0.0000),
            (20_000.0, 0.0000),
            (25_000.0, 0.0000),
            (30_000.0, 0.0000),
            (35_000.0, 0.0000),
            (40_000.0, 0.0000),
            (45_000.0, 0.0033),
            (50_000.0, 0.0097),
            (60_000.0, 0.0282),
            (70_000.0, 0.0471),
            (80_000.0, 0.0626),
            (90_000.0, 0.0759),
            (100_000.0, 0.0845),
            (125_000.0, 0.1083),
            (150_000.0, 0.1291),
            (175_000.0, 0.1503),
            (200_000.0, 0.1720),
        ];

        let tax_table_married_no_kids = vec![
            (25_000.0, 0.0100),
            (30_000.0, 0.0234),
            (35_000.0, 0.0371),
            (40_000.0, 0.0481),
            (45_000.0, 0.0604),
            (50_000.0, 0.0726),
            (60_000.0, 0.0926),
            (70_000.0, 0.1038),
            (80_000.0, 0.1127),
            (90_000.0, 0.1215),
            (100_000.0, 0.1300),
            (125_000.0, 0.1491),
            (150_000.0, 0.1670),
            (175_000.0, 0.1864),
            (200_000.0, 0.2086),
        ];

        Self {
            canton_name: "Bern".to_string(),
            municipality_name: "Bern (Stadt)".to_string(),
            married,
            children,
            family_tax_mode: false,
            tax_table_single,
            tax_table_married_no_kids,
            tax_table_married_2kids,
            social_security_rate: 0.053,  // AHV/IV/EO: 5.3% employee share
            unemployment_rate: 0.011,     // ALV: 1.1% employee share
            pension_rate: if children >= 2 { 0.060 } else { 0.065 }, // BVG ~6-6.5%
        }
    }

    /// Kept for backwards compatibility
    pub fn zurich_default(married: bool, children: u32) -> Self {
        Self::bern_city_default(married, children)
    }

    /// Create a custom tax schedule with user-provided tax rate
    /// This rate should be the TOTAL rate (tax + social security)
    pub fn custom_rate(custom_total_rate: f64) -> Self {
        // Create a dummy schedule that returns the custom rate for any income
        let dummy_table = vec![
            (0.0, custom_total_rate),
            (1_000_000.0, custom_total_rate),
        ];

        Self {
            canton_name: "Custom".to_string(),
            municipality_name: "User-provided rate".to_string(),
            married: false,
            children: 0,
            family_tax_mode: false,
            tax_table_single: dummy_table.clone(),
            tax_table_married_no_kids: dummy_table.clone(),
            tax_table_married_2kids: dummy_table.clone(),
            social_security_rate: 0.0,  // Already included in custom rate
            unemployment_rate: 0.0,      // Already included in custom rate
            pension_rate: 0.0,           // Already included in custom rate
        }
    }

    /// Canton code that this schedule prices, for display purposes.
    ///
    /// `canton_name` was already present but unused by callers, which let the
    /// display hard-code "official Bern tax only" — false for every other
    /// canton. Exposing a short code lets the CLI state the real basis.
    pub fn canton_code(&self) -> &str {
        &self.canton_name
    }

    /// Build a schedule for a canton priced by the two-level model
    /// (base scale x Steuerfuss), from the ESTV-imported scales.
    ///
    /// The schedule is materialised as a lookup table because `TaxSchedule`'s
    /// downstream consumers, and its linear interpolation, work on rate points.
    /// Those points are generated by evaluating the *actual* two-level
    /// arithmetic through [`crate::cantons::cantonal_tax`], so the table is a
    /// sampled view of the real calculation rather than a second, parallel
    /// implementation that could drift from it.
    ///
    /// Returns `None` when the canton cannot be priced, so the caller fails
    /// loudly instead of substituting another canton's rates.
    pub fn from_canton_scale(
        canton: crate::cantons::Canton,
        married: bool,
        children: u32,
    ) -> Option<Self> {
        // Guard: an unpriceable canton must produce no schedule at all.
        if !crate::cantons::is_priceable(canton) {
            return None;
        }

        // Dense at the low end, where progression is steepest, and coarser
        // above, since the downstream lookup interpolates linearly between
        // points.
        let mut points: Vec<f64> = Vec::new();
        let mut income = 0.0_f64;
        while income <= 200_000.0 {
            points.push(income);
            income += 2_500.0;
        }
        while income <= 1_000_000.0 {
            points.push(income);
            income += 25_000.0;
        }

        // Cantonal + communal + church, expressed as a rate on taxable income.
        // The municipal component is included, matching how the Bern table
        // treats its combined cantonal/communal/church figures.
        let mut table: Vec<(f64, f64)> = Vec::with_capacity(points.len());
        for &taxable in &points {
            let tax = crate::cantons::cantonal_tax(canton, taxable, married, true)
                .unwrap_or(0.0);
            let rate = if taxable > 0.0 { tax / taxable } else { 0.0 };
            table.push((taxable, rate));
        }

        Some(Self {
            canton_name: canton.name().to_string(),
            municipality_name: canton.capital().to_string(),
            married,
            children,
            family_tax_mode: false,
            tax_table_single: table.clone(),
            tax_table_married_no_kids: table.clone(),
            tax_table_married_2kids: table,
            social_security_rate: 0.053,
            unemployment_rate: 0.011,
            pension_rate: if children >= 2 { 0.060 } else { 0.065 },
        })
    }

    /// An **estimate** of common Swiss employee deductions, not a sourced model.
    ///
    /// Every component below is hand-entered: `commuting` is 1.5% of gross capped
    /// at CHF 4,000, `rent` is 12% of gross capped at CHF 20,000, and so on. None
    /// of them comes from a published table, and the rental component in particular
    /// has no counterpart in Swiss tax law — rent is not deductible for an employee.
    ///
    /// It survives because it is what the reported figures are computed from, and
    /// changing that moves every number the tool produces. `src/deductions.rs`
    /// holds the sourced replacement, built from the ESTV rule exports, and
    /// `optimize` prints both so the difference is visible before the default is
    /// switched.
    ///
    /// Total deductions are capped at 35% of gross, which binds for families at
    /// moderate incomes — at CHF 60,000 with two children both the plain and the
    /// `family_tax_mode` figures sit at the cap, so the flag has no effect there.
    pub fn deduction_breakdown(&self, gross_income: f64) -> TaxDeductionBreakdown {
        if gross_income <= 0.0 {
            return TaxDeductionBreakdown::default();
        }

        let childcare = if self.children > 0 {
            let base = (self.children as f64 * 4_000.0).min(12_000.0);
            if self.family_tax_mode && self.married {
                (base * 1.4).min(18_000.0)
            } else {
                base
            }
        } else {
            0.0
        };

        let commuting = (gross_income * 0.015).min(4_000.0);
        let work_equipment = (gross_income * 0.01).min(3_500.0);
        let health_insurance = (gross_income * 0.012).min(4_500.0);

        let rent = if self.married {
            (gross_income * 0.11).min(18_000.0)
        } else {
            (gross_income * 0.12).min(20_000.0)
        };

        let family_specific = if self.children > 0 {
            let base = (self.children as f64 * 2_800.0).min(8_000.0);
            if self.family_tax_mode && self.married { (base * 1.5).min(12_000.0) } else { base }
        } else {
            0.0
        };

        let deductible_total = (childcare + commuting + work_equipment + health_insurance + rent + family_specific)
            .min(gross_income * 0.35);

        TaxDeductionBreakdown {
            childcare,
            commuting,
            work_equipment,
            health_insurance,
            rent,
            family_specific,
            deductible_total,
            non_deductible_total: (gross_income - deductible_total).max(0.0) * 0.02,
        }
    }

    pub fn standard_deduction_estimate(&self, gross_income: f64) -> f64 {
        self.deduction_breakdown(gross_income).deductible_total
    }

    pub fn taxable_income_after_estimated_deductions(&self, gross_income: f64) -> f64 {
        let deduction = self.standard_deduction_estimate(gross_income);
        (gross_income - deduction).max(0.0)
    }

    /// Calculate effective tax rate using official lookup table + social security
    pub fn effective_tax_rate(&self, gross_income: f64) -> f64 {
        if gross_income == 0.0 {
            return 0.0;
        }

        let taxable_income = self.taxable_income_after_estimated_deductions(gross_income);

        // Get base tax rate from official table on reduced taxable income.
        let base_tax_rate = self.tax_rate_on_taxable(taxable_income);

        // Add social security contributions
        let social_security_total = self.social_security_rate +
                                     self.unemployment_rate +
                                     self.pension_rate;

        // Total effective rate. This makes the model reflect common Swiss deductions.
        base_tax_rate + social_security_total
    }

    /// Get tax-only rate (without social security) for a **gross** income.
    ///
    /// Deductions are applied first, because the rate schedule is a function of
    /// *taxable* income and applying it to gross income overstates the rate:
    /// 17.24% instead of 15.76% for a single Bern household at CHF 100,000.
    ///
    /// This previously looked the rate up on gross income while
    /// [`effective_tax_rate`](Self::effective_tax_rate) looked it up on deducted
    /// income, so the printed "Tax Rate" and the tax actually charged disagreed
    /// by 1.3–1.8 percentage points on the same schedule. Use
    /// [`tax_rate_on_taxable`](Self::tax_rate_on_taxable) when the taxable figure
    /// is already known, rather than converting it back to a gross one.
    pub fn tax_only_rate(&self, gross_income: f64) -> f64 {
        let taxable = self.taxable_income_after_estimated_deductions(gross_income);
        self.tax_rate_on_taxable(taxable)
    }

    /// The rate schedule applied to an income that is **already taxable**.
    ///
    /// The only way to ask "what rate does the published schedule give on this
    /// taxable income?" without deductions being applied a second time.
    pub fn tax_rate_on_taxable(&self, taxable_income: f64) -> f64 {
        self.lookup_tax_rate(taxable_income)
    }

    /// Lookup tax rate from official table with linear interpolation
    fn lookup_tax_rate(&self, income: f64) -> f64 {
        // Choose correct table based on marital status and children
        let table = if !self.married {
            // Single person
            &self.tax_table_single
        } else if self.children >= 2 {
            // Married with 2+ children
            &self.tax_table_married_2kids
        } else {
            // Married with 0-1 children
            &self.tax_table_married_no_kids
        };
        
        if income <= table[0].0 {
            return table[0].1;
        }
        
        if income >= table[table.len() - 1].0 {
            return table[table.len() - 1].1;
        }
        
        // Linear interpolation between table points
        for i in 1..table.len() {
            if income <= table[i].0 {
                let (x0, y0) = table[i - 1];
                let (x1, y1) = table[i];
                
                // Linear interpolation: y = y0 + (y1-y0)*(x-x0)/(x1-x0)
                let rate = y0 + (y1 - y0) * (income - x0) / (x1 - x0);
                return rate;
            }
        }
        
        table[table.len() - 1].1
    }

    /// Calculate after-tax income
    pub fn after_tax_income(&self, gross_income: f64) -> f64 {
        let tax_rate = self.effective_tax_rate(gross_income);
        gross_income * (1.0 - tax_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_official_rates_single() {
        let schedule = TaxSchedule::bern_city_default(false, 0);

        // The published table's rates, at the published thresholds. These are
        // `tax_rate_on_taxable` values: the document quotes a rate for a given
        // *taxable* income, and `tax_only_rate` takes gross income and deducts
        // first. Using the gross method here subtracted the deduction estimate a
        // second time, which is how the base inconsistency stayed hidden.
        let rate_40k = schedule.tax_rate_on_taxable(40_000.0);
        assert!((rate_40k - 0.1008).abs() < 0.001, 
                "40k tax should be 10.08%, got {:.2}%", rate_40k * 100.0);
        
        let rate_60k = schedule.tax_rate_on_taxable(60_000.0);
        assert!((rate_60k - 0.1345).abs() < 0.001,
                "60k tax should be 13.45%, got {:.2}%", rate_60k * 100.0);
        
        let rate_80k = schedule.tax_rate_on_taxable(80_000.0);
        assert!((rate_80k - 0.1538).abs() < 0.001,
                "80k tax should be 15.38%, got {:.2}%", rate_80k * 100.0);
        
        let rate_100k = schedule.tax_rate_on_taxable(100_000.0);
        assert!((rate_100k - 0.1724).abs() < 0.001,
                "100k tax should be 17.24%, got {:.2}%", rate_100k * 100.0);
    }

    #[test]
    fn test_total_rate_includes_social() {
        let schedule = TaxSchedule::bern_city_default(false, 0);

        let taxable_income = schedule.taxable_income_after_estimated_deductions(100_000.0);
        let tax_only_100k = schedule.lookup_tax_rate(taxable_income);
        let total_100k = schedule.effective_tax_rate(100_000.0);
        let social_security = total_100k - tax_only_100k;

        // With Swiss deductions, the effective social-security share still remains in the
        // expected range of roughly 11–13% of gross income.
        assert!(social_security > 0.11 && social_security < 0.13,
                "Social security should remain roughly 11–13% of gross income, got {:.1}%", social_security * 100.0);
    }

    #[test]
    fn test_interpolation() {
        let schedule = TaxSchedule::bern_city_default(false, 0);
        
        // Between the published 80k and 90k points, on the taxable figure those
        // points refer to. See `test_exact_official_rates_single` for why this is
        // the taxable variant rather than `tax_only_rate`.
        let rate_85k = schedule.tax_rate_on_taxable(85_000.0);
        
        // Should be between 15.38% and 16.26%
        assert!(rate_85k > 0.1538 && rate_85k < 0.1626,
                "85k rate should be between 15.38% and 16.26%, got {:.2}%", rate_85k * 100.0);
    }

    /// `non_deductible_total` must not affect any tax figure.
    ///
    /// It is `2%` of income *after* the deductible items, it has no source, and it
    /// used to be printed between "total deductible" and "taxable income" — where
    /// it read as part of the arithmetic even though taxable income is gross minus
    /// `deductible_total` alone. This test exists so the field cannot quietly
    /// acquire a role: if it ever feeds a calculation, the removal from the display
    /// becomes a real omission rather than a tidying-up.
    #[test]
    fn non_deductible_total_feeds_no_tax_figure() {
        for schedule in [
            TaxSchedule::bern_city_default(false, 0),
            TaxSchedule::bern_city_default(true, 2),
        ] {
            for gross in [40_000.0, 60_000.0, 100_000.0, 250_000.0] {
                let breakdown = schedule.deduction_breakdown(gross);

                // Taxable income is gross minus the deductible total, full stop.
                assert!(
                    (schedule.taxable_income_after_estimated_deductions(gross)
                        - (gross - breakdown.deductible_total).max(0.0))
                        .abs()
                        < 1e-9,
                    "at gross {gross}: taxable income is not gross minus deductible_total"
                );

                // And it is not silently folded into the deductible total.
                assert!(
                    breakdown.deductible_total <= gross * 0.35 + 1e-9,
                    "at gross {gross}: the total exceeds the documented 35% cap"
                );
                assert!(
                    breakdown.non_deductible_total >= 0.0
                        && breakdown.non_deductible_total < breakdown.deductible_total,
                    "at gross {gross}: the undocumented figure should stay minor"
                );
            }
        }
    }

    /// The two public rate accessors must agree on the same income.
    ///
    /// They did not: `tax_only_rate` looked the schedule up on gross income while
    /// `effective_tax_rate` looked it up on deducted income, so the printed "Tax
    /// Rate" was 1.3-1.8 percentage points above the rate backing the tax the
    /// tool actually charged. The invariant is that the tax-only component of the
    /// effective rate equals the tax-only rate.
    #[test]
    fn rate_accessors_agree_on_the_same_income() {
        for schedule in [
            TaxSchedule::bern_city_default(false, 0),
            TaxSchedule::bern_city_default(true, 2),
        ] {
            for gross in [40_000.0, 60_000.0, 100_000.0, 140_000.0] {
                let taxable = schedule.taxable_income_after_estimated_deductions(gross);
                let via_gross = schedule.tax_only_rate(gross);
                let via_taxable = schedule.tax_rate_on_taxable(taxable);
                assert!(
                    (via_gross - via_taxable).abs() < 1e-12,
                    "at gross {gross}: tax_only_rate {via_gross} != \
                     tax_rate_on_taxable(taxable) {via_taxable}"
                );

                // And the effective rate is exactly the tax-only rate plus the
                // social-security components.
                let social = schedule.social_security_rate
                    + schedule.unemployment_rate
                    + schedule.pension_rate;
                let effective = schedule.effective_tax_rate(gross);
                assert!(
                    (effective - (via_gross + social)).abs() < 1e-12,
                    "at gross {gross}: effective {effective} != tax-only {via_gross} + social {social}"
                );
            }
        }
    }

    #[test]
    fn test_standard_deductions_reduce_total_rate_for_realistic_swiss_case() {
        let schedule = TaxSchedule::bern_city_default(false, 0);
        let gross = 140_000.0;

        let deduction = schedule.standard_deduction_estimate(gross);
        assert!(deduction > 0.0, "Swiss work-related deductions should reduce taxable income");

        let taxable_income = schedule.taxable_income_after_estimated_deductions(gross);
        assert!(taxable_income < gross,
                "taxable income should be reduced by standard Swiss deductions");

        let effective_rate = schedule.effective_tax_rate(gross);
        assert!(effective_rate < 0.33,
                "total deduction for a realistic Bern salary should be below 33% after standard deductions");
    }

    #[test]
    fn test_family_specific_deductions_are_applied_for_children() {
        let schedule = TaxSchedule::bern_city_default(true, 2);
        let gross = 140_000.0;

        let deduction = schedule.standard_deduction_estimate(gross);
        assert!(deduction > 0.0, "children should add Swiss family-related deductions");
        assert!(deduction > schedule.standard_deduction_estimate(0.0),
                "family deductions should increase with children");
    }
}
