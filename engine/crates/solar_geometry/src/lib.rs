//! # solar_geometry
//!
//! Solar time, geocentric ephemeris, and the angle of incidence between
//! the Sun's rays and an arbitrarily oriented surface — the astronomical
//! foundation consumed by `solar_position` (topocentric zenith/azimuth)
//! and `irradiance` (extraterrestrial irradiance, which needs the Earth-Sun
//! distance computed here).
//!
//! Two independent geocentric ephemeris engines are provided:
//!
//! - [`spa`] — the full NREL Solar Position Algorithm (Reda & Andreas,
//!   2004), complete with its published periodic-term coefficient tables.
//!   **This is HeliosPro's default, primary solar-position engine** (see
//!   `solar_position::lib` for the selectable abstraction).
//! - [`meeus`] — a reduced-order engine (Meeus, 1998) kept specifically as
//!   an **independent secondary engine for cross-validation**; see that
//!   module's documentation for why an independently-sourced comparison
//!   engine is valuable even after the primary engine reaches full
//!   published precision.
//!
//! ## Historical note (ADR-004): why this crate once used Meeus as primary
//!
//! Earlier in this project's development, network access to the primary
//! NREL source was unavailable, and [`meeus`] was temporarily used as the
//! primary (not just secondary) engine, with its precision trade-off
//! explicitly documented and communicated to the project owner (see
//! `docs/design/16-phase1-validation-report.md`, section 16.2, preserved
//! for the historical record). Once network access to an independently
//! verifiable, cross-validated reproduction of the NREL coefficient tables
//! became available (`pvlib-python`, BSD-3-Clause; see
//! `engine/validation-data/README.md`), the project owner required the
//! full published SPA as the official engine, with Meeus demoted to a
//! secondary validation role. [`spa`] is the result. This history is kept
//! here, rather than deleted, because understanding *why* two independent
//! engines coexist is necessary context for anyone maintaining either one.
//!
//! ## Sign and unit conventions (binding for this crate and its consumers)
//!
//! - All angles are [`core_units::Angle`]; there is no bare-`f64`
//!   degrees/radians ambiguity anywhere in the public API.
//! - Longitude is **positive east** of Greenwich (ISO 6709 / WGS84
//!   convention), matching the SIG/GIS coordinate convention used
//!   elsewhere in HeliosPro (`docs/design/07-modeles-physiques.md`).
//! - Azimuth (both solar azimuth and surface azimuth) is measured **from
//!   geographic North, clockwise** (0° = N, 90° = E, 180° = S, 270° = W),
//!   the compass-bearing convention, consistently for the Sun and for
//!   collector surfaces.
//! - Time inputs are UTC calendar fields
//!   ([`core_math::julian::CalendarDateTime`]); timezone handling is an
//!   application-layer concern, not part of the scientific core.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod delta_t;
pub mod incidence;
pub mod meeus;
pub mod spa;
pub mod time;
