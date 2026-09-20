//! Life Optimizer — domain library.
//!
//! This exposes the tax, consumption, optimization, and simulation modules as a
//! library so the `tests/` integration suite can exercise them directly. The
//! `life-optimizer` binary is a thin CLI wrapper over these modules; it is not
//! duplicated logic.
//!
//! Module map:
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`tax`] | Progressive Swiss tax, social security, deduction lookup |
//! | [`requirements`] | Personal consumption basket, life stages, preferences |
//! | [`optimizer`] | Multi-objective work-percentage utility optimization |
//! | [`monte_carlo`] | Pension projection, conversion-rate scenarios, stress tests |
//! | [`economic_regimes`] | Markov regime-switching return/inflation model |
//! | [`display`] / [`mc_display`] | Terminal presentation |

pub mod tax;
pub mod requirements;
pub mod consumption;
pub mod optimizer;
pub mod display;
pub mod monte_carlo;
pub mod mc_display;
pub mod economic_regimes;
