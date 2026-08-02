//! Full-precision NREL Solar Position Algorithm (SPA): geocentric stage.
//!
//! **Source**: Reda, I. & Andreas, A. (2004), "Solar Position Algorithm for
//! Solar Radiation Applications", *Solar Energy*, 76(5), 577-589
//! (NREL/TP-560-34302). Section numbers below refer to that paper.
//!
//! This is the **default, primary** solar position engine for HeliosPro
//! (see `solar_position::spa` for the topocentric stage that consumes this
//! module's output, and the crate-level documentation in
//! `solar_position::lib` for the `Spa`/`MeeusReduced` selectable
//! abstraction). [`crate::meeus`] is retained solely as a secondary,
//! reduced-precision engine used for cross-validation, per an explicit
//! project decision: the original reduced-series approach was judged
//! sufficient for the <1% energy-yield target, but the project's ambition
//! to be a reference-grade tool requires the full published algorithm as
//! the actual product engine, with Meeus kept only as an independent
//! sanity check.
//!
//! ## Accuracy
//!
//! Reda & Andreas (2004) claim +/-0.0003 degree accuracy for zenith and
//! azimuth over the years -2000 to 6000. This implementation reproduces
//! every term of every periodic series in the paper's Appendix A (see the
//! `tables` submodule for exact provenance), so it inherits that claimed
//! accuracy, subject only to floating-point rounding (double precision
//! throughout, error contribution negligible at this accuracy level).
//!
//! ## Validation
//!
//! The `reda_andreas_2004_worked_example_geocentric` test (in this
//! module's test suite) reproduces the paper's own published worked
//! example (Golden, Colorado, 2003-10-17 19:30:30 UTC) to six decimal
//! degrees, cross-checked against pvlib-python 0.15.2's
//! independent implementation of the same paper (see
//! `engine/validation-data/README.md`). The broader `validation` crate
//! compares this implementation against pvlib across the 91-case dataset
//! generated for this project.

mod tables;

use core_units::Angle;
use tables::{NUTATION_ABCD, NUTATION_YTERM, Term};

/// Every geocentric intermediate and final quantity computed by the SPA
/// algorithm up to (but not including) the topocentric/observer-dependent
/// stage (Reda & Andreas (2004), Sections 3.1-3.9). Exposing every
/// intermediate value (not just the final right ascension/declination)
/// allows the `validation` crate to compare each individual stage against
/// the reference dataset, isolating exactly where any discrepancy would
/// originate.
#[derive(Debug, Clone, Copy)]
pub struct SpaGeocentricPosition {
    /// Julian Ephemeris Millennium (Section 3.1.4).
    pub julian_ephemeris_millennium: f64,
    /// Earth heliocentric longitude L, degrees (Section 3.2).
    pub earth_heliocentric_longitude_deg: f64,
    /// Earth heliocentric latitude B, degrees (Section 3.2).
    pub earth_heliocentric_latitude_deg: f64,
    /// Earth radius vector R, astronomical units (Section 3.2).
    pub earth_radius_vector_au: f64,
    /// Geocentric longitude Theta = L + 180, degrees (Section 3.3).
    pub geocentric_longitude_deg: f64,
    /// Geocentric latitude beta = -B, degrees (Section 3.3).
    pub geocentric_latitude_deg: f64,
    /// X0: mean elongation of the Moon from the Sun, degrees (Section 3.4).
    pub x0_deg: f64,
    /// X1: mean anomaly of the Sun (Earth), degrees (Section 3.4).
    pub x1_deg: f64,
    /// X2: mean anomaly of the Moon, degrees (Section 3.4).
    pub x2_deg: f64,
    /// X3: Moon's argument of latitude, degrees (Section 3.4).
    pub x3_deg: f64,
    /// X4: longitude of the ascending node of the Moon's mean orbit,
    /// degrees (Section 3.4).
    pub x4_deg: f64,
    /// Nutation in longitude, delta_psi, degrees (Section 3.4).
    pub nutation_longitude_deg: f64,
    /// Nutation in obliquity, delta_epsilon, degrees (Section 3.4).
    pub nutation_obliquity_deg: f64,
    /// Mean obliquity of the ecliptic, epsilon0, degrees (Section 3.5).
    pub mean_obliquity_deg: f64,
    /// True obliquity of the ecliptic, epsilon, degrees (Section 3.5).
    pub true_obliquity_deg: f64,
    /// Aberration correction, delta_tau, degrees (Section 3.6).
    pub aberration_correction_deg: f64,
    /// Apparent sun longitude, lambda, degrees (Section 3.7).
    pub apparent_longitude_deg: f64,
    /// Mean sidereal time at Greenwich, nu0, degrees (Section 3.8).
    pub mean_sidereal_time_deg: f64,
    /// Apparent sidereal time at Greenwich, nu, degrees (Section 3.8).
    pub apparent_sidereal_time_deg: f64,
    /// Geocentric right ascension of the Sun, alpha (Section 3.9).
    pub right_ascension: Angle,
    /// Geocentric declination of the Sun, delta (Section 3.9).
    pub declination: Angle,
    /// Equation of time, minutes (Section 3.11).
    pub equation_of_time_minutes: f64,
}

/// Sum a periodic term series: `sum(A * cos(B + C*x))`.
///
/// **Source**: Reda & Andreas (2004), Eq. 3.2.1 (the general form shared by
/// the L, B, and R series).
fn sum_periodic_terms(terms: &[Term], x: f64) -> f64 {
    terms.iter().map(|t| t.a * (t.b + t.c * x).cos()).sum()
}

/// Normalize an angle in degrees to `[0, 360)`, matching Python's `%`
/// operator semantics (always non-negative for a positive modulus), which
/// the reference pvlib implementation relies on throughout.
fn norm_deg_0_360(deg: f64) -> f64 {
    deg.rem_euclid(360.0)
}

/// Compute the full geocentric SPA position at the given Julian Ephemeris
/// Day (`jde`, already corrected for `delta_t = TT - UT1`) and the
/// corresponding UT-based Julian Day (`jd_ut`, used for the mean sidereal
/// time argument, per Reda & Andreas (2004), Eq. 3.8.1, which explicitly
/// uses the UT Julian Day, not JDE).
pub fn compute(jde: f64, jd_ut: f64) -> SpaGeocentricPosition {
    let jce = (jde - core_constants::J2000_EPOCH_JD) / core_constants::DAYS_PER_JULIAN_CENTURY;
    let jme = jce / 10.0;
    let jc_ut = (jd_ut - core_constants::J2000_EPOCH_JD) / core_constants::DAYS_PER_JULIAN_CENTURY;

    // --- Earth heliocentric longitude, latitude, radius vector
    // (Reda & Andreas (2004), Eq. 3.2.1-3.2.7).
    let l0 = sum_periodic_terms(&tables::L0_TERMS, jme);
    let l1 = sum_periodic_terms(&tables::L1_TERMS, jme);
    let l2 = sum_periodic_terms(&tables::L2_TERMS, jme);
    let l3 = sum_periodic_terms(&tables::L3_TERMS, jme);
    let l4 = sum_periodic_terms(&tables::L4_TERMS, jme);
    let l5 = sum_periodic_terms(&tables::L5_TERMS, jme);
    let l_rad =
        (l0 + l1 * jme + l2 * jme.powi(2) + l3 * jme.powi(3) + l4 * jme.powi(4) + l5 * jme.powi(5))
            / 1.0e8;
    let earth_heliocentric_longitude_deg = norm_deg_0_360(l_rad.to_degrees());

    let b0 = sum_periodic_terms(&tables::B0_TERMS, jme);
    let b1 = sum_periodic_terms(&tables::B1_TERMS, jme);
    let b_rad = (b0 + b1 * jme) / 1.0e8;
    let earth_heliocentric_latitude_deg = b_rad.to_degrees();

    let r0 = sum_periodic_terms(&tables::R0_TERMS, jme);
    let r1 = sum_periodic_terms(&tables::R1_TERMS, jme);
    let r2 = sum_periodic_terms(&tables::R2_TERMS, jme);
    let r3 = sum_periodic_terms(&tables::R3_TERMS, jme);
    let r4 = sum_periodic_terms(&tables::R4_TERMS, jme);
    let earth_radius_vector_au =
        (r0 + r1 * jme + r2 * jme.powi(2) + r3 * jme.powi(3) + r4 * jme.powi(4)) / 1.0e8;

    // --- Geocentric longitude/latitude (Reda & Andreas (2004), Eq. 3.3.1-3.3.2).
    let geocentric_longitude_deg = norm_deg_0_360(earth_heliocentric_longitude_deg + 180.0);
    let geocentric_latitude_deg = -earth_heliocentric_latitude_deg;

    // --- Five fundamental arguments X0-X4 (Reda & Andreas (2004), Eq. 3.4.1-3.4.5).
    let x0_deg = 297.85036 + 445267.111480 * jce - 0.0019142 * jce.powi(2) + jce.powi(3) / 189474.0;
    let x1_deg = 357.52772 + 35999.050340 * jce - 0.0001603 * jce.powi(2) - jce.powi(3) / 300000.0;
    let x2_deg = 134.96298 + 477198.867398 * jce + 0.0086972 * jce.powi(2) + jce.powi(3) / 56250.0;
    let x3_deg = 93.27191 + 483202.017538 * jce - 0.0036825 * jce.powi(2) + jce.powi(3) / 327270.0;
    let x4_deg = 125.04452 - 1934.136261 * jce + 0.0020708 * jce.powi(2) + jce.powi(3) / 450000.0;

    // --- Nutation in longitude and obliquity, 63-term series
    // (Reda & Andreas (2004), Eq. 3.4.6-3.4.9). NUTATION_ABCD amplitudes
    // are tabulated in units of 0.0001 arcsecond, hence the /36_000_000
    // divisor (= 3600 arcsec/degree * 10000).
    let mut delta_psi_sum = 0.0_f64;
    let mut delta_eps_sum = 0.0_f64;
    for (coeff, y) in NUTATION_ABCD.iter().zip(NUTATION_YTERM.iter()) {
        let arg_deg = y.y0 as f64 * x0_deg
            + y.y1 as f64 * x1_deg
            + y.y2 as f64 * x2_deg
            + y.y3 as f64 * x3_deg
            + y.y4 as f64 * x4_deg;
        let arg = Angle::from_degrees(arg_deg);
        delta_psi_sum += (coeff.a + coeff.b * jce) * arg.sin();
        delta_eps_sum += (coeff.c + coeff.d * jce) * arg.cos();
    }
    let nutation_longitude_deg = delta_psi_sum / 36_000_000.0;
    let nutation_obliquity_deg = delta_eps_sum / 36_000_000.0;

    // --- Mean and true obliquity of the ecliptic (Reda & Andreas (2004),
    // Eq. 3.5.1-3.5.2; the Laskar (1986) high-order polynomial in
    // U = JME/10).
    let u = jme / 10.0;
    let mean_obliquity_arcsec = 84381.448 - 4680.93 * u - 1.55 * u.powi(2) + 1999.25 * u.powi(3)
        - 51.38 * u.powi(4)
        - 249.67 * u.powi(5)
        - 39.05 * u.powi(6)
        + 7.12 * u.powi(7)
        + 27.87 * u.powi(8)
        + 5.79 * u.powi(9)
        + 2.45 * u.powi(10);
    let mean_obliquity_deg = mean_obliquity_arcsec / 3600.0;
    let true_obliquity_deg = mean_obliquity_deg + nutation_obliquity_deg;

    // --- Aberration correction (Reda & Andreas (2004), Eq. 3.6.1).
    let aberration_correction_deg = -20.4898 / (3600.0 * earth_radius_vector_au);

    // --- Apparent sun longitude (Reda & Andreas (2004), Eq. 3.7.1).
    let apparent_longitude_deg =
        geocentric_longitude_deg + nutation_longitude_deg + aberration_correction_deg;

    // --- Mean and apparent sidereal time at Greenwich (Reda & Andreas
    // (2004), Eq. 3.8.1-3.8.2). Uses the UT-based Julian Day/century, per
    // the paper.
    let mean_sidereal_time_deg = norm_deg_0_360(
        280.46061837
            + 360.98564736629 * (jd_ut - core_constants::J2000_EPOCH_JD)
            + 0.000387933 * jc_ut.powi(2)
            - jc_ut.powi(3) / 38_710_000.0,
    );
    let true_obliquity = Angle::from_degrees(true_obliquity_deg);
    let apparent_sidereal_time_deg =
        mean_sidereal_time_deg + nutation_longitude_deg * true_obliquity.cos();

    // --- Geocentric right ascension and declination (Reda & Andreas
    // (2004), Eq. 3.9.1-3.9.2), including the geocentric latitude beta
    // (unlike the reduced Meeus engine, which treats it as exactly zero).
    let beta = Angle::from_degrees(geocentric_latitude_deg);
    let lambda = Angle::from_degrees(apparent_longitude_deg);
    let right_ascension_rad = f64::atan2(
        lambda.sin() * true_obliquity.cos() - beta.tan() * true_obliquity.sin(),
        lambda.cos(),
    );
    let right_ascension = Angle::from_radians(right_ascension_rad).normalized_0_2pi();

    let declination_rad = (beta.sin() * true_obliquity.cos()
        + beta.cos() * true_obliquity.sin() * lambda.sin())
    .asin();
    let declination = Angle::from_radians(declination_rad);

    // --- Equation of time (Reda & Andreas (2004), Eq. 3.11.1-3.11.3).
    let sun_mean_longitude_deg =
        280.4664567 + 360007.6982779 * jme + 0.03032028 * jme.powi(2) + jme.powi(3) / 49931.0
            - jme.powi(4) / 15300.0
            - jme.powi(5) / 2_000_000.0;
    let e_deg = norm_deg_0_360(
        sun_mean_longitude_deg - 0.0057183 - right_ascension.degrees()
            + nutation_longitude_deg * true_obliquity.cos(),
    );
    let mut equation_of_time_minutes = e_deg * 4.0;
    if equation_of_time_minutes > 20.0 {
        equation_of_time_minutes -= 1440.0;
    } else if equation_of_time_minutes < -20.0 {
        equation_of_time_minutes += 1440.0;
    }

    SpaGeocentricPosition {
        julian_ephemeris_millennium: jme,
        earth_heliocentric_longitude_deg,
        earth_heliocentric_latitude_deg,
        earth_radius_vector_au,
        geocentric_longitude_deg,
        geocentric_latitude_deg,
        x0_deg,
        x1_deg,
        x2_deg,
        x3_deg,
        x4_deg,
        nutation_longitude_deg,
        nutation_obliquity_deg,
        mean_obliquity_deg,
        true_obliquity_deg,
        aberration_correction_deg,
        apparent_longitude_deg,
        mean_sidereal_time_deg,
        apparent_sidereal_time_deg,
        right_ascension,
        declination,
        equation_of_time_minutes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::julian::CalendarDateTime;

    /// The Reda & Andreas (2004) paper's own published worked example
    /// (Table A5.2 area, Golden, Colorado), as independently reproduced by
    /// pvlib-python 0.15.2 (`tests/conftest.py` fixtures `golden_mst` /
    /// `expected_solpos`, BSD-3-Clause license) -- see
    /// `engine/validation-data/README.md` for the exact reproduction
    /// script. Input: 2003-10-17 12:30:30 MST (= 19:30:30 UTC),
    /// lat=39.742476, lon=-105.1786, elevation=1830.14 m, pressure=820 hPa,
    /// temperature=11 degC, delta_t=67 s. This test covers only the
    /// geocentric stage (right ascension, declination, equation of time,
    /// Earth radius vector); the full topocentric zenith/azimuth
    /// comparison (which additionally exercises parallax and refraction)
    /// is in `solar_position::spa::tests`.
    #[test]
    fn reda_andreas_2004_worked_example_geocentric() {
        let dt = CalendarDateTime {
            year: 2003,
            month: 10,
            day: 17,
            hour: 19,
            minute: 30,
            second: 30.0,
        };
        let jd_ut = dt.to_julian_day().unwrap();
        let delta_t = 67.0;
        let jde = jd_ut + delta_t / core_constants::SECONDS_PER_DAY;

        let pos = compute(jde, jd_ut);

        assert!(
            (pos.earth_radius_vector_au - 0.9965422973539708).abs() < 1e-9,
            "R = {}",
            pos.earth_radius_vector_au
        );
        assert!(
            (pos.right_ascension.degrees() - 202.22740782720712).abs() < 1e-6,
            "alpha = {}",
            pos.right_ascension.degrees()
        );
        assert!(
            (pos.declination.degrees() - (-9.314340090849058)).abs() < 1e-6,
            "delta = {}",
            pos.declination.degrees()
        );
        assert!(
            (pos.equation_of_time_minutes - 14.641510770821696).abs() < 1e-6,
            "eot = {}",
            pos.equation_of_time_minutes
        );
        assert!(
            (pos.apparent_sidereal_time_deg - 318.5119098411589).abs() < 1e-6,
            "v = {}",
            pos.apparent_sidereal_time_deg
        );
    }

    #[test]
    fn declination_stays_within_axial_tilt_bounds_across_a_full_year() {
        for day in 0..365u32 {
            let dt = CalendarDateTime {
                year: 2024,
                month: 1,
                day: 1,
                hour: 12,
                minute: 0,
                second: 0.0,
            };
            let jd = dt.to_julian_day().unwrap() + day as f64;
            let pos = compute(jd, jd);
            assert!(pos.declination.degrees().abs() <= 23.45);
            assert!(pos.right_ascension.radians().is_finite());
            assert!(pos.earth_radius_vector_au.is_finite());
        }
    }
}
