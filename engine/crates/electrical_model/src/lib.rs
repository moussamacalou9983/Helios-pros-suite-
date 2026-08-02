//! # electrical_model
//!
//! **Status: scaffolding only — not yet implemented.**
//!
//! This crate will hold the PV single-diode electrical model: given
//! plane-of-array irradiance ([`transposition`]), cell temperature
//! ([`thermal_model`]), and a module's datasheet parameters
//! ([`pv_module`]), compute the module's I-V curve and maximum power
//! point.
//!
//! It is intentionally left unimplemented in this session, per the
//! module-by-module procedure in the development brief (see
//! `pv_module`'s crate documentation for the same note). Implementing this
//! module correctly requires [`pv_module`] to exist first (it consumes
//! `pv_module`'s datasheet struct) and is a substantial unit of work in
//! its own right: the De Soto et al. (2006) five-parameter extraction from
//! datasheet values, translation of those parameters to arbitrary
//! irradiance/temperature via the single-diode equation, and a numerically
//! robust maximum-power-point search (the single-diode I-V relation is
//! transcendental and requires either a Lambert-W closed form or an
//! iterative solver — both need their own dedicated validation against
//! published I-V curves before being trusted).
//!
//! **Planned references**: De Soto, W., Klein, S.A., Beckman, W.A. (2006),
//! "Improvement and validation of a model for photovoltaic array
//! performance", *Solar Energy*, 80(1), 78-88; King, D.L. et al., Sandia
//! PV Array Performance Model (SAND2004-3535) as an alternative empirical
//! characterization; IEC 60904-1 (I-V curve measurement conditions/method).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
