//! # pv_module
//!
//! **Status: scaffolding only — not yet implemented.**
//!
//! This crate will hold the PV module datasheet domain model: the
//! electrical and physical parameters read from a manufacturer datasheet
//! (`Voc`, `Isc`, `Vmpp`, `Impp` at STC, temperature coefficients, number of
//! cells in series, technology/certifications) that [`electrical_model`]
//! needs to evaluate the single-diode model, and that [`thermal_model`]'s
//! NOCT path needs for the datasheet NOCT value.
//!
//! It is intentionally left unimplemented in this session: the current
//! development brief's explicit procedure is to implement and fully
//! validate one module at a time, starting from `core_constants`, and to
//! never begin a module before the previous one is confirmed compiling,
//! tested, and documented. This file exists purely so the workspace's
//! member list (and therefore `cargo build --workspace` /
//! `cargo test --workspace`) is complete and accurate about the intended
//! final architecture; it contains no logic and makes no scientific claim.
//!
//! **Planned references** (to be cited precisely, per-field, once
//! implemented): IEC 61215-1/2 (module qualification, defines the STC
//! measurement conditions the datasheet parameters are reported at), IEC
//! 61730-1/2 (module safety), IEC 61853-1..4 (energy rating / extended
//! performance matrix, an alternative/complementary characterization to
//! the single STC point), De Soto, W., Klein, S.A., Beckman, W.A. (2006),
//! "Improvement and validation of a model for photovoltaic array
//! performance", *Solar Energy*, 80(1), 78-88 (defines exactly which
//! datasheet fields the single-diode parameter-extraction procedure
//! requires).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
