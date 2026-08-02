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
//! reference software or measurements, which is the job of the `validation`
//! crate.
//!
//! `tests/numerical_stability.rs` is a third, distinct kind of check again:
//! neither a regression baseline nor an external-accuracy comparison, but a
//! guarantee that the solar-position engines never produce `NaN`/`+-inf`
//! or panic at deliberately degenerate inputs (the geographic poles, years
//! far outside any claimed accuracy window, angles at/near 0/90/180
//! degrees, UTC midnight/day-boundary crossings).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
