//! # heliospro-thermal
//!
//! PV cell temperature models: the operating temperature of a PV cell
//! departs substantially from ambient air temperature under load (typically
//! 20-40°C above ambient at full sun), and this temperature strongly
//! affects electrical output (silicon PV power drops by roughly 0.3-0.5%
//! per °C above 25°C, per manufacturer datasheets' temperature
//! coefficients) — so an accurate thermal model is essential to the
//! engine's overall energy-yield accuracy.
//!
//! Two models are implemented:
//! - [`noct`]: the simple, single-coefficient Nominal Operating Cell
//!   Temperature model, using only a datasheet NOCT value.
//! - [`sandia`]: the empirical two-stage (module, then cell) Sandia Array
//!   Performance Model thermal model, using mounting-configuration-specific
//!   coefficients and explicit wind-speed dependence.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod noct;
pub mod sandia;
