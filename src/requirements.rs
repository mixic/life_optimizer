// Personal requirements and consumption basket
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalBasketItem {
    pub name: String,
    pub weight: f64,           // Proportion of spending (0.0 to 1.0)
    pub price_start: f64,      // Price at start period (CHF)
    pub price_latest: f64,     // Price at latest period (CHF)
    pub discount_rate: f64,    // Average discount rate (0.0 to 1.0)
    pub category: String,
}

impl PersonalBasketItem {
    pub fn effective_price_start(&self) -> f64 {
        self.price_start * (1.0 - self.discount_rate)
    }

    pub fn effective_price_latest(&self) -> f64 {
        self.price_latest * (1.0 - self.discount_rate)
    }

    pub fn personal_inflation_rate(&self) -> f64 {
        if self.effective_price_start() == 0.0 {
            0.0
        } else {
            (self.effective_price_latest() / self.effective_price_start()) - 1.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalRequirements {
    // Monthly requirements in CHF
    pub housing: f64,
    pub food: f64,
    pub transport: f64,
    pub insurance: f64,
    pub childcare: f64,
    pub healthcare: f64,
    pub education: f64,
    pub vacation: f64,
    pub savings_goal: f64,
    pub discretionary: f64,
}

impl PersonalRequirements {
    pub fn total_monthly(&self) -> f64 {
        self.housing +
        self.food +
        self.transport +
        self.insurance +
        self.childcare +
        self.healthcare +
        self.education +
        self.vacation +
        self.savings_goal +
        self.discretionary
    }

    pub fn total_annual(&self) -> f64 {
        self.total_monthly() * 12.0
    }

    /// Adjust requirements based on life stage
    pub fn adjusted_for_life_stage(&self, stage: &LifeStage) -> Self {
        let mut adjusted = self.clone();
        
        match stage {
            LifeStage::YoungSingle { .. } => {
                adjusted.childcare = 0.0;
                adjusted.education = 0.0;
                adjusted.housing *= 0.7;  // Smaller apartment
            },
            LifeStage::YoungCouple { .. } => {
                adjusted.childcare = 0.0;
                adjusted.education = 0.0;
            },
            LifeStage::NewParent { .. } | LifeStage::SchoolAge { .. } => {
                // Childcare peaks during these years
                adjusted.childcare *= 1.5;
                adjusted.discretionary *= 0.7;
            },
            LifeStage::Teenagers { .. } => {
                adjusted.childcare *= 0.5;  // Less supervision needed
                adjusted.education *= 1.5;  // University costs
                adjusted.food *= 1.3;  // Teenagers eat more!
            },
            LifeStage::EmptyNest { .. } => {
                adjusted.childcare = 0.0;
                adjusted.education = 0.0;
                adjusted.food *= 0.7;
                adjusted.housing *= 0.8;  // Can downsize
            },
            LifeStage::PreRetirement { .. } => {
                adjusted.childcare = 0.0;
                adjusted.education = 0.0;
                adjusted.healthcare *= 1.3;  // Health costs increase
            },
        }
        
        adjusted
    }

    /// Create default requirements for Swiss family (Bern)
    pub fn bern_family_default(children: u32) -> Self {
        Self {
            housing: 2200.0 + (children as f64 * 250.0),  // 3.5-4.5 room apartment (Bern cheaper than ZH)
            food: 800.0 + (children as f64 * 200.0),      // Discounted shopping
            transport: 280.0 + (children as f64 * 50.0),  // Libero or car
            insurance: 380.0 + (children as f64 * 180.0), // Health, liability
            childcare: (children as f64) * 650.0,         // Kita/Hort if needed (Bern cheaper than ZH)
            healthcare: 100.0,                            // Co-pays, dentist
            education: (children as f64) * 100.0,         // Activities, materials
            vacation: 450.0,                              // Annual trips amortized
            savings_goal: 900.0,                          // Emergency + goals
            discretionary: 450.0,                         // Fun money
        }
    }

    /// Kept for backwards compatibility
    pub fn zurich_family_default(children: u32) -> Self {
        Self::bern_family_default(children)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifeStage {
    YoungSingle { age: u32 },
    YoungCouple { age: u32, dual_income: bool },
    NewParent { age: u32, children: u32 },
    SchoolAge { age: u32, children: u32 },
    Teenagers { age: u32, children: u32 },
    EmptyNest { age: u32 },
    PreRetirement { age: u32 },
}

impl LifeStage {
    pub fn age(&self) -> u32 {
        match self {
            LifeStage::YoungSingle { age } => *age,
            LifeStage::YoungCouple { age, .. } => *age,
            LifeStage::NewParent { age, .. } => *age,
            LifeStage::SchoolAge { age, .. } => *age,
            LifeStage::Teenagers { age, .. } => *age,
            LifeStage::EmptyNest { age } => *age,
            LifeStage::PreRetirement { age } => *age,
        }
    }

    pub fn children_count(&self) -> u32 {
        match self {
            LifeStage::NewParent { children, .. } => *children,
            LifeStage::SchoolAge { children, .. } => *children,
            LifeStage::Teenagers { children, .. } => *children,
            _ => 0,
        }
    }

    /// Time value factor (how much is leisure worth at this stage?)
    pub fn time_value_factor(&self) -> f64 {
        match self {
            LifeStage::YoungSingle { .. } => 0.8,
            LifeStage::YoungCouple { .. } => 0.9,
            LifeStage::NewParent { .. } => 1.5,  // Time with young kids is invaluable
            LifeStage::SchoolAge { .. } => 1.3,
            LifeStage::Teenagers { .. } => 1.0,
            LifeStage::EmptyNest { .. } => 1.1,
            LifeStage::PreRetirement { .. } => 1.2,  // Health is wealth
        }
    }

    /// Stress tolerance (how much work stress can you handle?)
    pub fn stress_tolerance(&self) -> f64 {
        match self {
            LifeStage::YoungSingle { .. } => 1.2,  // Can handle more
            LifeStage::YoungCouple { .. } => 1.1,
            LifeStage::NewParent { .. } => 0.6,    // Already stressed with kids
            LifeStage::SchoolAge { .. } => 0.8,
            LifeStage::Teenagers { .. } => 0.9,
            LifeStage::EmptyNest { .. } => 1.0,
            LifeStage::PreRetirement { .. } => 0.7,  // Health concerns
        }
    }

    pub fn determine_from_age(age: u32, has_children: bool, children_ages: &[u32]) -> Self {
        if !has_children {
            if age < 30 {
                LifeStage::YoungSingle { age }
            } else if age < 65 {
                LifeStage::YoungCouple { age, dual_income: false }
            } else {
                LifeStage::PreRetirement { age }
            }
        } else {
            let youngest_child = children_ages.iter().min().unwrap_or(&0);
            let children = children_ages.len() as u32;
            
            if *youngest_child < 6 {
                LifeStage::NewParent { age, children }
            } else if *youngest_child < 13 {
                LifeStage::SchoolAge { age, children }
            } else if *youngest_child < 18 {
                LifeStage::Teenagers { age, children }
            } else {
                if age < 60 {
                    LifeStage::EmptyNest { age }
                } else {
                    LifeStage::PreRetirement { age }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceWeights {
    pub consumption: f64,  // How much you value spending money
    pub leisure: f64,      // How much you value free time
    pub family: f64,       // How much you value family time
    pub health: f64,       // How much you value health/low stress
    pub security: f64,     // How much you value long-term security
}

impl PreferenceWeights {
    pub fn balanced() -> Self {
        Self {
            consumption: 0.25,
            leisure: 0.20,
            family: 0.25,
            health: 0.15,
            security: 0.15,
        }
    }

    pub fn family_focused() -> Self {
        Self {
            consumption: 0.15,
            leisure: 0.15,
            family: 0.40,
            health: 0.15,
            security: 0.15,
        }
    }

    pub fn career_focused() -> Self {
        Self {
            consumption: 0.30,
            leisure: 0.10,
            family: 0.15,
            health: 0.15,
            security: 0.30,
        }
    }

    pub fn validate(&self) -> bool {
        let sum = self.consumption + self.leisure + self.family + self.health + self.security;
        (sum - 1.0).abs() < 0.01
    }
}
