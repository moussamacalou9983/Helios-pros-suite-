//! # core_math
//!
//! Generic numerical toolbox shared by the HeliosPro scientific engine:
//! calendar/Julian Day time conversion, angle-independent interpolation, and
//! error statistics used both in physical models and in the engine's own
//! validation test suites.
//!
//! This crate intentionally contains only primitives that are actually
//! consumed elsewhere in the engine or in its validation harness — see
//! `docs/design/06-algorithmes.md` for the algorithm inventory this
//! supports. It has no dependency beyond the Rust standard library.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod interpolation;
pub mod julian;
pub mod stats;
