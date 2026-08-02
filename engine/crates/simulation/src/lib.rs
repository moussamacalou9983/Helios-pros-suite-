//! # simulation
//!
//! Top-level orchestration of the HeliosPro Phase 1 scientific engine: the
//! complete chain from solar geometry through plane-of-array irradiance to
//! PV cell temperature — the "bilan énergétique complet" (complete energy
//! balance) called for by the development brief.
//!
//! This crate contains no new physics of its own; it wires together, in
//! the physically correct order, the crates built earlier in this phase:
//!
//! 1. [`solar_position::position::solar_position`] — where is the Sun?
//! 2. [`irradiance::extraterrestrial`] — how much energy would reach
//!    a surface with no atmosphere, right now?
//! 3. [`irradiance::decomposition::erbs_decomposition`] — if only GHI
//!    was measured, split it into DNI and DHI.
//! 4. [`solar_geometry::incidence::angle_of_incidence`] — what angle do
//!    the Sun's rays make with the collector surface?
//! 5. [`transposition`] — how much irradiance actually lands on
//!    the tilted plane-of-array (POA)?
//! 6. [`thermal_model`] — given that POA irradiance (plus ambient
//!    temperature and wind), how hot does the cell get?
//!
//! The `pv_module`, `electrical_model`, and `losses` crates exist in this
//! workspace as scaffolding for upcoming work but are not yet wired into
//! this orchestration — see each crate's module documentation for its
//! current status. This crate's [`compute_energy_balance`] therefore still
//! stops at plane-of-array irradiance and cell temperature, exactly as in
//! the original Phase 1 brief; extending it to produce DC/AC electrical
//! output is the next unit of work, not a silently deferred one.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod balance;
mod inputs;

pub use balance::compute_energy_balance;
pub use inputs::{
    EnergyBalanceInput, EnergyBalanceResult, ThermalModel, TranspositionModel, WeatherInput,
};
