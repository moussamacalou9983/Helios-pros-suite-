//! # heliospro-energy-balance
//!
//! Top-level orchestration of the HeliosPro Phase 1 scientific engine: the
//! complete chain from solar geometry through plane-of-array irradiance to
//! PV cell temperature — the "bilan énergétique complet" (complete energy
//! balance) called for by the development brief.
//!
//! This crate contains no new physics of its own; it wires together, in
//! the physically correct order, the crates built earlier in this phase:
//!
//! 1. [`heliospro_solar::position::solar_position`] — where is the Sun?
//! 2. [`heliospro_solar::extraterrestrial`] — how much energy would reach
//!    a surface with no atmosphere, right now?
//! 3. [`heliospro_solar::decomposition::erbs_decomposition`] — if only GHI
//!    was measured, split it into DNI and DHI.
//! 4. [`heliospro_solar::incidence::angle_of_incidence`] — what angle do
//!    the Sun's rays make with the collector surface?
//! 5. [`heliospro_transposition`] — how much irradiance actually lands on
//!    the tilted plane-of-array (POA)?
//! 6. [`heliospro_thermal`] — given that POA irradiance (plus ambient
//!    temperature and wind), how hot does the cell get?
//!
//! Downstream phases (electrical modelling, inverter, storage, ...) are
//! explicitly out of scope for this phase, per the development brief, and
//! are not implemented here.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod balance;
mod inputs;

pub use balance::compute_energy_balance;
pub use inputs::{
    EnergyBalanceInput, EnergyBalanceResult, ThermalModel, TranspositionModel, WeatherInput,
};
