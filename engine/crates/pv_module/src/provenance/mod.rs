//! Traceability: where a value came from, what kind of value it is, and
//! how much to trust it.
//!
//! Three ideas, each specified in
//! `docs/design/27-specification-finale-pv-module.md`:
//!
//! - [`DataClass`] (§1) — the value's epistemic nature: manufacturer
//!   data, measurement, normative constant, derived quantity, estimate,
//!   fitted parameter, model parameter, or metadata.
//! - [`Provenance`] (§2) — the source, its version and date, the document
//!   reference, and how the value was extracted from it.
//! - [`Measurement`] (§10) — the wrapper carrying a physical value
//!   together with its class, provenance, uncertainty and confidence.
//!
//! The unifying decision is that all three travel **with the value**
//! rather than in tables indexed by field name, because classification
//! and provenance are properties of the value, not of the field: the same
//! open-circuit voltage is manufacturer data on a datasheet, measured
//! data in a test report, and an estimate when typed from memory. See
//! `docs/adr/0006-measurement-provenance-conflicts.md`.

mod classification;
mod measurement;
mod source;

pub use classification::DataClass;
pub use measurement::{Measurement, Uncertainty, UncertaintyEvaluation, UncertaintyKind};
pub use source::{
    CalendarDate, Confidence, ExtractionMethod, Provenance, SourceIdentifier, SourceType,
};
