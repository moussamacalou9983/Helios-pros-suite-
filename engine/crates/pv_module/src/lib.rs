//! # pv_module
//!
//! The PV module domain model: a typed, validated, traceable description
//! of a photovoltaic module as its manufacturer publishes it or as a
//! laboratory measures it.
//!
//! ## What this crate is, and is not
//!
//! This crate **represents** a module. It does not **compute** its
//! performance. Extracting single-diode parameters, evaluating an I-V
//! curve at arbitrary conditions, and finding a maximum power point all
//! belong to `electrical_model`, which consumes this crate.
//!
//! The boundary test: if a function needs an *operating* irradiance or
//! temperature as input, it does not belong here. The STC, NOCT, NMOT and
//! PTC conditions held in this crate are *reference measurement
//! conditions*, not operating conditions.
//!
//! ## The founding principle
//!
//! > Source data, physical model, model parameter, derived quantity and
//! > computed result are five different things.
//!
//! Neither the CEC database, nor Sandia's, nor a manufacturer datasheet
//! is absolute physical truth. Each is a *source*, with its own
//! authority, date, method and uncertainty. Direct inspection of the CEC
//! library found 29 records whose stated area contradicts their own
//! dimensions by up to a factor of two
//! (`docs/design/26-validation-empirique-cec-sandia.md`).
//!
//! Five mechanisms follow from that principle:
//!
//! | Mechanism | Module |
//! |---|---|
//! | Epistemic classification of every value | [`DataClass`] |
//! | Provenance attached to the value | [`Provenance`], [`Measurement`] |
//! | Formal conflict handling, never silent arbitration | [`validation`] |
//! | Physical data kept apart from model parameters | see `electrical_model` |
//! | Reference conditions kept explicit and distinct | [`OperatingCondition`] |
//!
//! **A conflicting value is never arbitrated silently.** A conflict
//! blocks construction and names both candidate values; it is passable
//! only through a [`validation::ConflictResolution`] recording who
//! decided and why.
//!
//! ## Units
//!
//! Every physical quantity crosses a public boundary as a strong type
//! from `core_units`, never as a bare `f64`. External formats may use
//! °C, mm, cm², %/°C and so on; conversion happens once, at the adapter
//! boundary.
//!
//! ## Specification
//!
//! `docs/design/27-specification-finale-pv-module.md` is the normative
//! reference for this crate, with
//! `docs/design/28-plan-tests-pv-module.md` for the test plan and
//! ADR-0004, ADR-0005 and ADR-0006 for the architectural decisions.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapters;
pub mod provenance;
pub mod validation;

mod bifacial;
mod condition;
mod configuration;
mod electrical;
mod geometry;
mod identity;
mod module;
mod technology;
mod thermal;

pub use bifacial::{BifacialCharacteristics, RearSideParameters, StatedBifacialGain};
pub use condition::{
    ElectricalState, OperatingCondition, OperatingConditionKind, OperatingPoint, Spectrum,
    TemperatureSpecification,
};
pub use configuration::{CellConfiguration, CellDivision};
pub use electrical::{ElectricalCharacteristics, ElectricalLimits, PowerTolerance};
pub use geometry::{ModuleDimensions, ModuleGeometry, MountingForm};
pub use identity::{Certification, ModuleIdentity};
pub use module::{
    ElectricalOutput, ModuleBuildError, PvModule, PvModuleBuilder, is_known_invariant,
};
pub use provenance::{
    CalendarDate, Confidence, DataClass, ExtractionMethod, Measurement, Provenance,
    SourceIdentifier, SourceType, Uncertainty, UncertaintyEvaluation, UncertaintyKind,
};
pub use technology::{CellArchitecture, CellTechnology, ModuleTechnology};
pub use thermal::ThermalCharacteristics;
pub use validation::{
    Conflict, ConflictResolution, ResolutionDecision, ValidationIssue, ValidationReport,
    ValidationStatus,
};
