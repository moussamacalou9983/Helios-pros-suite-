//! # validation
//!
//! **Status: scaffolding only — not yet implemented.**
//!
//! This crate will hold HeliosPro's external validation framework: curated
//! reference datasets (published measurements, and — subject to their
//! respective license terms — NREL SPA/pvlib/SAM/PVsyst comparison cases)
//! together with automated comparison logic that runs the engine on each
//! case's inputs and produces a structured error report (using
//! [`core_math::stats`] for RMSE/MAE/bias/relative-error, already
//! implemented and unit-tested), per `docs/design/14-strategie-tests.md`.
//!
//! **Why this is not implemented yet**: this development session had no
//! working network access to fetch reference datasets or reference-tool
//! outputs (see the ADR in `solar_geometry::lib` for the concrete failures
//! encountered), so there is currently nothing legitimate for this crate to
//! compare against. Populating it with fabricated "reference" numbers
//! would be scientific misconduct, not a shortcut — it will be built once
//! genuine reference data (from the project owner, from a future session
//! with working network access, or from locally supplied datasets) is
//! available. Until then, `docs/design/16-phase1-validation-report.md`
//! documents the internal validation methodology (physical/geometric
//! invariants, algebraic invariants, cross-validated independent formulas)
//! actually used to gain confidence in the engine so far, and its explicit
//! limits.
//!
//! **Planned references**: Reda, I. & Andreas, A. (2004), NREL/TP-560-34302
//! (SPA reference test cases); Holmgren, W.F., Hansen, C.W., Mikofski, M.A.
//! (2018), "pvlib python: a python package for modeling solar energy
//! systems", *Journal of Open Source Software*, 3(29), 884 (pvlib
//! comparison, open-source/BSD-licensed — a legitimate and appropriate
//! comparison target, distinct from the proprietary tools this project
//! must never copy code or interfaces from); NREL System Advisor Model
//! (SAM) documentation; published PVsyst validation studies (comparison of
//! *outputs only*, never of code or UI).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
