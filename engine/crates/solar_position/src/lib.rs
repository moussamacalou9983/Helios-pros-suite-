//! # solar_position
//!
//! Topocentric solar position: the observer-relative zenith, azimuth and
//! elevation of the Sun, including atmospheric refraction — the final,
//! observer-dependent stage of the SPA-class algorithm whose geocentric
//! ephemeris stage lives in the [`solar_geometry`] crate (see that crate's
//! module documentation for the full precision Architecture Decision
//! Record, ADR-004, governing this algorithm's scope).
//!
//! **Sources**:
//! - Overall multi-step procedure: Reda, I. & Andreas, A. (2004), "Solar
//!   Position Algorithm for Solar Radiation Applications", *Solar Energy*,
//!   76(5), 577-589.
//! - Horizontal-coordinate (elevation/azimuth) transform: Meeus, J. (1998),
//!   *Astronomical Algorithms*, 2nd ed., Willmann-Bell, Chapter 13
//!   ("Transformation of Coordinates"), Eq. 13.5-13.6.
//! - Atmospheric refraction: Sæmundsson's formula as reproduced in Meeus
//!   (1998), Chapter 16, Eq. 16.4, with the standard pressure/temperature
//!   scaling also used in Reda & Andreas (2004), Eq. 3.19.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod position;

pub use position::{AtmosphericConditions, Observer, SolarPosition, solar_position};
