//! # tests
//!
//! Cross-crate regression and integration tests for the HeliosPro
//! scientific engine. This crate holds no production code — its own unit
//! tests are the deliverable, plus the integration test binaries under
//! `tests/`.
//!
//! Unlike the per-crate unit tests (each testing one crate's public API in
//! isolation against physical/geometric invariants or cited reference
//! values), this crate exercises multiple crates together through
//! `simulation::compute_energy_balance`, pinning specific numeric outputs
//! as regression baselines: a change in engine output for these fixed
//! inputs, however small, must be a *reviewed, intentional* change (e.g. an
//! upgrade to a higher-precision solar-position series), never a silent
//! side effect of an unrelated refactor. This is standard practice for a
//! scientific engine (see `docs/design/14-strategie-tests.md`, "golden
//! datasets") and is distinct from validating the engine against *external*
//! reference software or measurements, which is the job of the (currently
//! unimplemented, scaffolding-only) `validation` crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
