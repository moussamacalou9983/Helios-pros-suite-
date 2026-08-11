//! # core_units
//!
//! Strongly-typed SI unit newtypes for the HeliosPro scientific engine.
//!
//! ## Design rationale (Architecture Decision)
//!
//! Physical quantities passed as bare `f64` between functions are a leading
//! source of silent unit-mismatch bugs in scientific software (e.g. passing
//! degrees where radians are expected, or Celsius where Kelvin is expected).
//! Every public function boundary in the HeliosPro engine therefore takes
//! and returns the newtypes defined in this crate rather than raw `f64`.
//!
//! Each type stores its value internally in the SI coherent unit for that
//! quantity (radians for angle, Kelvin for temperature, W/m² for irradiance,
//! pascals for pressure, metres for length, m/s for velocity), and exposes
//! explicit, named constructors/accessors for any non-SI unit in common
//! engineering use (degrees, Celsius, hours, etc.) so that a conversion is
//! always a visible, intentional step in the code, never an implicit one.
//!
//! This is a zero-cost abstraction: every newtype is `#[repr(transparent)]`-
//! equivalent (a single `f64` field) and all accessor/constructor methods are
//! marked `#[inline]`, so the compiler generates code identical to using bare
//! `f64` values directly.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod angle;
mod area;
mod current;
mod irradiance;
mod length;
mod mass;
mod power;
mod pressure;
mod ratio;
mod temperature;
mod temperature_coefficient;
mod velocity;
mod voltage;
mod wavelength;

pub use angle::Angle;
pub use area::Area;
pub use current::Current;
pub use irradiance::Irradiance;
pub use length::Length;
pub use mass::Mass;
pub use power::Power;
pub use pressure::Pressure;
pub use ratio::Ratio;
pub use temperature::Temperature;
pub use temperature_coefficient::{CoefficientBasis, TemperatureCoefficient};
pub use velocity::Velocity;
pub use voltage::Voltage;
pub use wavelength::Wavelength;
