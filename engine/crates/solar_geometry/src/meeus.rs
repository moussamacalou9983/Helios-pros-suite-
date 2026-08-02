//! Reduced-order geocentric solar ephemeris (Meeus 1998): apparent
//! ecliptic longitude, right ascension, declination, Earth-Sun distance,
//! nutation, apparent obliquity, apparent sidereal time, and the equation
//! of time.
//!
//! ## Status: secondary validation engine (not the default)
//!
//! **As of the project's second development phase, this module is no
//! longer HeliosPro's primary solar-position engine.** [`crate::spa`] (the
//! full NREL Solar Position Algorithm, Reda & Andreas 2004, complete with
//! its published coefficient tables) is now the default, primary engine —
//! see `solar_position::lib` for the selectable `Spa`/`MeeusReduced`
//! abstraction. This module is retained specifically as an **independent
//! secondary engine for cross-validation**: because it is derived from a
//! different published source with a different internal structure
//! (closed-form low-order series vs. SPA's large periodic tables), close
//! agreement between the two is meaningful evidence against a shared
//! implementation bug, which comparing SPA against itself obviously could
//! not provide.
//!
//! This module's own accuracy (~0.01-0.02 degrees, see the historical
//! ADR-004 discussion previously in `lib.rs` and now in
//! `docs/design/16-phase1-validation-report.md`) is unchanged and remains
//! fully documented; only its role in the engine changed, not its content.
//!
//! **Sources**:
//! - Meeus, J. (1998). *Astronomical Algorithms*, 2nd ed., Willmann-Bell.
//!   - Chapter 22 ("Nutation and the Obliquity of the Ecliptic"), Eq. 22.1,
//!     22.2, 22.3.
//!   - Chapter 25 ("Solar Coordinates"), Eq. 25.2-25.5, 25.9.
//!   - Chapter 12 ("Sidereal Time at Greenwich"), Eq. 12.4.
//!   - Chapter 28 ("Equation of Time"), Eq. 28.3.

use core_units::Angle;

/// The set of geocentric quantities describing the Sun's apparent position
/// at a given instant (Terrestrial Time), independent of any observer
/// location.
#[derive(Debug, Clone, Copy)]
pub struct Ephemeris {
    /// Julian centuries of Terrestrial Time (TT) since J2000.0, i.e. the
    /// argument `T` of every series in this module.
    pub julian_century_tt: f64,
    /// Apparent geocentric right ascension of the Sun.
    pub right_ascension: Angle,
    /// Apparent geocentric declination of the Sun.
    pub declination: Angle,
    /// Earth-Sun distance, astronomical units.
    pub earth_sun_distance_au: f64,
    /// Apparent sidereal time at Greenwich (includes the nutation
    /// correction), used to derive the local hour angle.
    pub apparent_sidereal_time_greenwich: Angle,
    /// Equation of time: the difference `apparent solar time - mean solar
    /// time`, expressed in minutes of time (a human-readable byproduct; not
    /// used internally by `solar_position::meeus::solar_position`, which
    /// works directly from sidereal time and right ascension).
    pub equation_of_time_minutes: f64,
}

/// Normalize an angle in degrees to `[0, 360)`.
fn norm_deg_0_360(deg: f64) -> f64 {
    let wrapped = deg % 360.0;
    if wrapped < 0.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

/// Compute the geocentric solar ephemeris at the given Julian Ephemeris Day
/// (`jde`, i.e. Julian Day already corrected for `ΔT = TT - UT1`) and the
/// corresponding raw (UT-based) Julian Day `jd_ut`, which is what apparent
/// sidereal time is conventionally referenced against (Meeus Ch. 12 uses
/// the UT-based Julian Day for the mean sidereal time argument).
pub fn compute(jde: f64, jd_ut: f64) -> Ephemeris {
    let t = (jde - core_constants::J2000_EPOCH_JD) / core_constants::DAYS_PER_JULIAN_CENTURY;

    // --- Geometric mean longitude of the Sun, referred to the mean equinox
    // of the date (Meeus Eq. 25.2), degrees.
    let l0 = norm_deg_0_360(280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t);

    // --- Mean anomaly of the Sun (Meeus Eq. 25.3), degrees.
    let m_deg = norm_deg_0_360(357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t);
    let m = Angle::from_degrees(m_deg);

    // --- Eccentricity of the Earth's orbit (Meeus Eq. 25.4), dimensionless.
    let e = 0.016_708_634 - 0.000_042_037 * t - 0.000_000_126_7 * t * t;

    // --- Equation of the center (Meeus, unnumbered eq. following 25.4), degrees.
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * Angle::from_degrees(2.0 * m_deg).sin()
        + 0.000_289 * Angle::from_degrees(3.0 * m_deg).sin();

    // --- True longitude and true anomaly of the Sun, degrees.
    let true_longitude_deg = l0 + c;
    let true_anomaly = Angle::from_degrees(m_deg + c);

    // --- Radius vector (Earth-Sun distance), AU (Meeus Eq. 25.5, exact
    // Keplerian ellipse equation with the eccentricity above).
    let r_au = 1.000_001_018 * (1.0 - e * e) / (1.0 + e * true_anomaly.cos());

    // --- Longitude of the ascending node of the Moon's mean orbit (Meeus,
    // used both in the apparent-longitude shortcut and the nutation
    // series), degrees.
    let omega_deg = 125.044_52 - 1_934.136_261 * t;
    let omega = Angle::from_degrees(omega_deg);

    // --- Moon's mean longitude (Meeus Ch. 22 note), degrees. Only used as
    // an argument to the reduced nutation series below.
    let moon_mean_longitude_deg = norm_deg_0_360(218.3165 + 481_267.881_3 * t);

    // --- Apparent longitude of the Sun (Meeus Eq. 25.9): true longitude
    // corrected for aberration and (approximately) nutation.
    let apparent_longitude_deg = true_longitude_deg - 0.005_69 - 0.004_78 * omega.sin();
    let apparent_longitude = Angle::from_degrees(apparent_longitude_deg);

    // --- Reduced nutation series (Meeus Eq. 22.1: "sufficient for many
    // purposes"), arcseconds.
    let delta_psi_arcsec = -17.20 * omega.sin()
        - 1.32 * Angle::from_degrees(2.0 * l0).sin()
        - 0.23 * Angle::from_degrees(2.0 * moon_mean_longitude_deg).sin()
        + 0.21 * Angle::from_degrees(2.0 * omega_deg).sin();
    let delta_epsilon_arcsec = 9.20 * omega.cos()
        + 0.57 * Angle::from_degrees(2.0 * l0).cos()
        + 0.10 * Angle::from_degrees(2.0 * moon_mean_longitude_deg).cos()
        - 0.09 * Angle::from_degrees(2.0 * omega_deg).cos();
    let delta_psi_deg = delta_psi_arcsec / 3600.0;
    let delta_epsilon_deg = delta_epsilon_arcsec / 3600.0;

    // --- Mean obliquity of the ecliptic (Meeus Eq. 22.2), arcseconds then
    // converted to degrees.
    let mean_obliquity_arcsec = 84381.448 - 46.8150 * t - 0.00059 * t * t + 0.001_813 * t * t * t;
    let mean_obliquity_deg = mean_obliquity_arcsec / 3600.0;

    // --- True (apparent) obliquity of the ecliptic: mean obliquity plus
    // nutation in obliquity (Meeus Eq. 22.3).
    let true_obliquity = Angle::from_degrees(mean_obliquity_deg + delta_epsilon_deg);

    // --- Geocentric right ascension and declination (Meeus Eq. 25.6, 25.7,
    // with ecliptic latitude of the Sun taken as exactly 0, per Meeus's own
    // note that it never exceeds 1.2 arcsec).
    let right_ascension_rad = f64::atan2(
        true_obliquity.cos() * apparent_longitude.sin(),
        apparent_longitude.cos(),
    );
    let right_ascension = Angle::from_radians(right_ascension_rad).normalized_0_2pi();

    let declination_rad = (true_obliquity.sin() * apparent_longitude.sin()).asin();
    let declination = Angle::from_radians(declination_rad);

    // --- Mean sidereal time at Greenwich (Meeus Eq. 12.4), degrees, using
    // the UT-based Julian day difference from J2000.0.
    let d_ut = jd_ut - core_constants::J2000_EPOCH_JD;
    let nu0_deg = norm_deg_0_360(
        280.460_618_37 + 360.985_647_366_29 * d_ut + 0.000_387_933 * t * t
            - t * t * t / 38_710_000.0,
    );

    // --- Apparent sidereal time at Greenwich: mean sidereal time corrected
    // for nutation in longitude (Meeus, discussion following Eq. 12.4).
    let apparent_sidereal_time_greenwich = Angle::from_degrees(norm_deg_0_360(
        nu0_deg + delta_psi_deg * true_obliquity.cos(),
    ));

    // --- Equation of time (Meeus Eq. 28.3), minutes of time.
    let y = (true_obliquity.radians() / 2.0).tan().powi(2);
    let l0_rad = Angle::from_degrees(l0).radians();
    let e_of_t_rad = y * (2.0 * l0_rad).sin() - 2.0 * e * m.sin()
        + 4.0 * e * y * m.sin() * (2.0 * l0_rad).cos()
        - 0.5 * y * y * (4.0 * l0_rad).sin()
        - 1.25 * e * e * (2.0 * m.radians()).sin();
    let equation_of_time_minutes = 4.0 * e_of_t_rad.to_degrees();

    Ephemeris {
        julian_century_tt: t,
        right_ascension,
        declination,
        earth_sun_distance_au: r_au,
        apparent_sidereal_time_greenwich,
        equation_of_time_minutes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::julian::CalendarDateTime;

    fn jde_for(dt: CalendarDateTime) -> f64 {
        dt.to_julian_day().unwrap()
    }

    #[test]
    fn declination_near_zero_at_march_equinox_2024() {
        // The March equinox 2024 occurred 2024-03-20 ~03:06 UTC (public
        // astronomical almanac fact). Declination should cross zero within
        // a fraction of a degree of that instant.
        let dt = CalendarDateTime {
            year: 2024,
            month: 3,
            day: 20,
            hour: 3,
            minute: 6,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            eph.declination.degrees().abs() < 0.05,
            "declination = {} deg",
            eph.declination.degrees()
        );
    }

    #[test]
    fn declination_near_solstice_extreme_june_2024() {
        // June solstice 2024 occurred 2024-06-20 ~20:51 UTC. Declination
        // should be within a small fraction of a degree of Earth's axial
        // tilt (+23.44 deg), which bounds the Sun's declination.
        let dt = CalendarDateTime {
            year: 2024,
            month: 6,
            day: 20,
            hour: 20,
            minute: 51,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            (eph.declination.degrees() - 23.44).abs() < 0.05,
            "declination = {} deg",
            eph.declination.degrees()
        );
    }

    #[test]
    fn declination_near_solstice_extreme_december_2024() {
        // December solstice 2024 occurred 2024-12-21 ~09:21 UTC.
        let dt = CalendarDateTime {
            year: 2024,
            month: 12,
            day: 21,
            hour: 9,
            minute: 21,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            (eph.declination.degrees() - (-23.44)).abs() < 0.05,
            "declination = {} deg",
            eph.declination.degrees()
        );
    }

    #[test]
    fn earth_sun_distance_near_perihelion_in_early_january() {
        // Earth is near perihelion (~0.983 AU) in early January (public
        // astronomical fact; exact date/distance varies year to year but
        // is always in the range 0.9832-0.9834 AU in the first days of
        // January).
        let dt = CalendarDateTime {
            year: 2024,
            month: 1,
            day: 3,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            (eph.earth_sun_distance_au - 0.9833).abs() < 0.001,
            "R = {} AU",
            eph.earth_sun_distance_au
        );
    }

    #[test]
    fn earth_sun_distance_near_aphelion_in_early_july() {
        // Earth is near aphelion (~1.0167 AU) in early July.
        let dt = CalendarDateTime {
            year: 2024,
            month: 7,
            day: 5,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            (eph.earth_sun_distance_au - 1.0167).abs() < 0.001,
            "R = {} AU",
            eph.earth_sun_distance_au
        );
    }

    #[test]
    fn equation_of_time_minimum_near_mid_february() {
        // The equation of time reaches its most negative extreme (~ -14.2
        // minutes) around February 11 each year (well-established
        // astronomical / analemma fact).
        let dt = CalendarDateTime {
            year: 2024,
            month: 2,
            day: 11,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            (-16.0..=-12.0).contains(&eph.equation_of_time_minutes),
            "EoT = {} min",
            eph.equation_of_time_minutes
        );
    }

    #[test]
    fn equation_of_time_maximum_near_early_november() {
        // The equation of time reaches its most positive extreme (~ +16.4
        // minutes) around November 3 each year.
        let dt = CalendarDateTime {
            year: 2024,
            month: 11,
            day: 3,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let jde = jde_for(dt);
        let eph = compute(jde, jde);
        assert!(
            (14.0..=18.0).contains(&eph.equation_of_time_minutes),
            "EoT = {} min",
            eph.equation_of_time_minutes
        );
    }

    #[test]
    fn right_ascension_and_declination_are_finite_across_a_full_year() {
        // Robustness sweep: no NaN/infinite outputs across a full year of
        // daily samples (guards against domain errors in asin/atan2 near
        // edge-case arguments).
        for day in 0..365 {
            let dt = CalendarDateTime {
                year: 2024,
                month: 1,
                day: 1,
                hour: 12,
                minute: 0,
                second: 0.0,
            };
            let jde = dt.to_julian_day().unwrap() + day as f64;
            let eph = compute(jde, jde);
            assert!(eph.right_ascension.radians().is_finite());
            assert!(eph.declination.radians().is_finite());
            assert!(eph.earth_sun_distance_au.is_finite());
            assert!(eph.declination.degrees().abs() <= 23.45);
        }
    }
}
