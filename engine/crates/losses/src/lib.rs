//! # losses
//!
//! **Status: scaffolding only — not yet implemented.**
//!
//! This crate will hold the systematic loss models applied downstream of
//! the [`electrical_model`] DC output: soiling, mismatch, DC/AC wiring
//! (Joule losses), inverter clipping/efficiency, availability, and
//! long-term degradation — each producing an explicit, individually
//! reported loss term rather than a single opaque "system losses" factor
//! (per the project rule against unjustified simplification: every loss
//! percentage must be traceable to a named physical cause).
//!
//! Intentionally left unimplemented in this session, per the
//! module-by-module procedure in the development brief.
//!
//! **Planned references**: Duffie, J.A. & Beckman, W.A. (2013), *Solar
//! Engineering of Thermal Processes*, 4th ed. (mismatch, wiring losses);
//! IEC 60364-5-52 / IEC 62548 (cable sizing and the resulting Joule loss
//! bound used as a design constraint); IEC 61724-1 (definition of
//! Performance Ratio and its standard loss categories, used as the
//! reporting taxonomy this crate will populate).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
