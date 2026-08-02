//! # solar_geometry
//!
//! Solar time, geocentric ephemeris (declination, right ascension,
//! Earth-Sun distance, apparent sidereal time, equation of time), and the
//! angle of incidence between the Sun's rays and an arbitrarily oriented
//! surface — the astronomical foundation consumed by `solar_position`
//! (topocentric zenith/azimuth) and `irradiance` (extraterrestrial
//! irradiance, which needs Earth-Sun distance from [`ephemeris`]).
//!
//! ## Architecture Decision Record — ADR-004: solar position algorithm scope
//!
//! The brief calls for "l'algorithme SPA NREL" (the NREL Solar Position
//! Algorithm, Reda, I. & Andreas, A. (2004), "Solar Position Algorithm for
//! Solar Radiation Applications", *Solar Energy*, 76(5), 577-589,
//! NREL/TP-560-34302). That paper's headline ±0.0003° accuracy is achieved
//! with a full VSOP87-derived periodic term table (Appendix A, Tables
//! A4.1-A4.4: roughly 250 numeric coefficients for Earth's heliocentric
//! longitude/latitude/radius plus the 63-term IAU 1980 nutation series).
//!
//! In the development session that first produced this module, outbound
//! network access to the primary source (nrel.gov) was unavailable, and no
//! independent, reliably machine-readable mirror of that exact coefficient
//! table could be retrieved. Rather than transcribing ~250 numbers from
//! memory with no way to verify them (which would be exactly the "code
//! approximatif" the project's engineering rules forbid), this crate
//! implements:
//!
//! 1. **The full multi-step SPA *procedure***: geocentric ecliptic solar
//!    coordinates -> nutation & obliquity -> apparent sidereal time ->
//!    equatorial coordinates -> local hour angle -> topocentric horizontal
//!    coordinates -> atmospheric refraction -> surface incidence angle
//!    (this last transform lives in `solar_position`). Every one of these
//!    steps is exact, closed-form spherical astronomy — there is no series
//!    truncation involved and no meaningful accuracy loss versus
//!    Reda & Andreas (2004) at this stage.
//! 2. **A reduced-order (but fully cited) periodic series**, in
//!    [`ephemeris`], for the two steps that in the original SPA rely on
//!    the large VSOP87/nutation tables: geocentric solar longitude and
//!    nutation. These use the "low-precision" series explicitly published
//!    for this purpose by Meeus, J. (1998), *Astronomical Algorithms*, 2nd
//!    ed., Willmann-Bell: Chapter 25 ("Solar Coordinates", accuracy stated
//!    by Meeus as ~0.01°) for longitude/eccentricity/radius vector, and
//!    Chapter 22, Eq. 22.1 (explicitly presented by Meeus as a *reduced*
//!    nutation series "sufficient for many purposes", accuracy ~0.5
//!    arcsec) for nutation. Topocentric parallax correction (max amplitude
//!    ≈ 8.8 arcsec ≈ 0.0025°, per Meeus Ch. 40 Eq. 40.6) is omitted for the
//!    same reason: it is below the ~0.01° accuracy floor already set by
//!    the longitude series, so implementing it would not improve overall
//!    accuracy while adding a well-known source of azimuth-quadrant bugs.
//!
//! **Resulting accuracy**: ~0.01-0.02° in zenith/azimuth, versus SPA's
//! ±0.0003°. Sensitivity analysis (see `transposition` and `simulation`
//! validation tests) shows this level of positional accuracy contributes
//! well under 0.1 % error to plane-of-array irradiance away from
//! sunrise/sunset, comfortably inside the project's <1 % energy-yield
//! deviation target versus PVsyst/SAM/pvlib/Archelios Calc.
//!
//! **Upgrade path**: downstream crates depend only on
//! [`ephemeris::Ephemeris`] and [`solar_position::SolarPosition`] (in the
//! `solar_position` crate), not on the internals of the series used to
//! compute them. Swapping in the full Reda & Andreas Table A4.1-A4.4
//! coefficients — once independently verified against the primary NREL
//! source — is a change confined to `ephemeris.rs` and requires no
//! downstream crate to change. This is tracked as explicit follow-up work:
//! see `docs/design/16-phase1-validation-report.md`.
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
pub mod ephemeris;
pub mod incidence;
pub mod time;
