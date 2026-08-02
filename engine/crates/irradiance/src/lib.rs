//! # irradiance
//!
//! Extraterrestrial solar irradiance (the irradiance that would be
//! received with no atmosphere present, used as the normalizing reference
//! for clearness-index calculations) and the exact/estimated relationships
//! between Global Horizontal Irradiance (GHI), Direct Normal Irradiance
//! (DNI), and Diffuse Horizontal Irradiance (DHI).
//!
//! This crate depends on [`solar_geometry`] for the Earth-Sun distance and
//! solar zenith angle that the irradiance calculations require, but
//! contains no solar-position astronomy of its own — see `solar_geometry`
//! for that.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod decomposition;
pub mod extraterrestrial;
