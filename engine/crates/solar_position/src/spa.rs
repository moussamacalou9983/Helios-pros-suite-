//! Full-precision topocentric solar position: the primary, default engine.
//!
//! **Source**: Reda, I. & Andreas, A. (2004), "Solar Position Algorithm for
//! Solar Radiation Applications", *Solar Energy*, 76(5), 577-589
//! (NREL/TP-560-34302). Section numbers below refer to that paper. The
//! geocentric stage (right ascension, declination, Earth radius vector,
//! apparent sidereal time, equation of time) is computed by
//! [`solar_geometry::spa`]; this module implements the topocentric stage
//! (parallax correction, refraction, final zenith/azimuth) that stage
//! feeds into.
//!
//! Unlike [`crate::meeus`], this engine performs the full topocentric
//! parallax correction (Section 3.12-3.13), which requires the observer's
//! elevation in addition to latitude/longitude.

use core_math::julian::{CalendarDateTime, JulianDayError};
use core_units::Angle;

use solar_geometry::delta_t;
use solar_geometry::spa::{self as spa_geometry, SpaGeocentricPosition};

use crate::common::{AtmosphericConditions, Observer, SolarPosition};

/// Standard atmospheric refraction at the horizon used by the SPA
/// algorithm's refraction-correction switch, degrees.
///
/// **Source**: Reda & Andreas (2004), Section 3.14 (the paper's own
/// example and pvlib's default both use this value; it corresponds to the
/// typically observed refraction at the astronomical horizon).
pub const DEFAULT_ATMOSPHERIC_REFRACTION_AT_HORIZON_DEG: f64 = 0.5667;

/// Earth's equatorial radius, meters, as published in Reda & Andreas
/// (2004), Eq. 3.12.3. This is the paper's own rounded constant (used for
/// byte-for-byte reproducibility of the published algorithm), distinct
/// from the more precise modern WGS84 value
/// ([`core_constants::WGS84_SEMI_MAJOR_AXIS_M`] = 6,378,137 m) used
/// elsewhere in this project; the ~3 m difference is many orders of
/// magnitude below the parallax correction's own sensitivity.
const SPA_EARTH_EQUATORIAL_RADIUS_M: f64 = 6_378_140.0;

/// Earth ellipsoid flattening factor `1 - f` used by the SPA algorithm's
/// observer geocentric-latitude correction, Eq. 3.12.1.
///
/// **Source**: Reda & Andreas (2004), Eq. 3.12.1. Equal to `1 -
/// 1/298.257223563` (the WGS84 flattening) rounded to the paper's own
/// published precision.
const SPA_ELLIPSOID_ONE_MINUS_FLATTENING: f64 = 0.99664719;

/// Full result of the topocentric SPA computation, including every
/// intermediate quantity from Reda & Andreas (2004) Sections 3.12-3.14 not
/// otherwise exposed in the engine-agnostic [`SolarPosition`] struct. Used
/// by the `validation` crate to compare each individual stage against the
/// pvlib reference dataset.
#[derive(Debug, Clone, Copy)]
pub struct SpaDetailedResult {
    /// The geocentric stage result this topocentric computation is based on.
    pub geocentric: SpaGeocentricPosition,
    /// Equatorial horizontal parallax, xi, degrees (Eq. 3.12.2).
    pub equatorial_horizontal_parallax_deg: f64,
    /// Parallax in right ascension, delta_alpha, degrees (Eq. 3.12.4).
    pub parallax_right_ascension_deg: f64,
    /// Topocentric declination, delta', degrees (Eq. 3.12.5).
    pub topocentric_declination_deg: f64,
    /// Topocentric local hour angle, H', degrees (Eq. 3.12.6).
    pub topocentric_local_hour_angle_deg: f64,
    /// Final topocentric solar position.
    pub position: SolarPosition,
}

/// Compute the full topocentric solar position for a UTC instant and
/// observer location, using the standard 67 s/current-era estimated
/// `delta_t` and the standard `atmos_refract` = 0.5667°. This is the
/// convenience entry point for application code; see
/// [`solar_position_precise`] for full control over `delta_t` and
/// `atmos_refract` (needed to reproduce published reference test cases
/// exactly).
///
/// # Errors
/// Returns [`JulianDayError`] if `datetime_utc` is not a valid Gregorian
/// calendar date.
pub fn solar_position(
    datetime_utc: CalendarDateTime,
    observer: Observer,
    atmosphere: AtmosphericConditions,
) -> Result<SolarPosition, JulianDayError> {
    Ok(solar_position_detailed(datetime_utc, observer, atmosphere)?.position)
}

/// As [`solar_position`], but returning every intermediate topocentric
/// quantity (see [`SpaDetailedResult`]).
///
/// # Errors
/// Returns [`JulianDayError`] if `datetime_utc` is not a valid Gregorian
/// calendar date.
pub fn solar_position_detailed(
    datetime_utc: CalendarDateTime,
    observer: Observer,
    atmosphere: AtmosphericConditions,
) -> Result<SpaDetailedResult, JulianDayError> {
    let jd_ut = datetime_utc.to_julian_day()?;
    let delta_t_seconds = delta_t::estimate_delta_t_seconds(datetime_utc.year, datetime_utc.month);
    Ok(solar_position_precise(
        jd_ut,
        delta_t_seconds,
        observer,
        atmosphere.pressure.hectopascal(),
        atmosphere.temperature.celsius(),
        DEFAULT_ATMOSPHERIC_REFRACTION_AT_HORIZON_DEG,
    ))
}

/// Compute the full topocentric solar position with every input specified
/// explicitly and no estimation performed internally: this is a pure,
/// deterministic function of its arguments, used both by the convenience
/// wrappers above and directly by the validation test suite to reproduce
/// published reference cases (which specify `delta_t` and `atmos_refract`
/// exactly rather than leaving them to an estimator).
///
/// **Source**: Reda & Andreas (2004), Sections 3.12 ("Topocentric
/// coordinates"), 3.13 ("Topocentric local hour angle"), 3.14
/// ("Topocentric zenith angle").
#[allow(clippy::too_many_arguments)]
pub fn solar_position_precise(
    jd_ut: f64,
    delta_t_seconds: f64,
    observer: Observer,
    pressure_hpa: f64,
    temperature_celsius: f64,
    atmos_refract_deg: f64,
) -> SpaDetailedResult {
    let jde = jd_ut + delta_t_seconds / core_constants::SECONDS_PER_DAY;
    let geocentric = spa_geometry::compute(jde, jd_ut);

    // --- Local hour angle (geocentric), Eq. 3.10.1.
    let hour_angle_deg = (geocentric.apparent_sidereal_time_deg + observer.longitude.degrees()
        - geocentric.right_ascension.degrees())
    .rem_euclid(360.0);

    // --- Equatorial horizontal parallax, Eq. 3.12.2.
    let xi_deg =
        core_constants::SOLAR_PARALLAX_ARCSEC_AT_1AU / (3600.0 * geocentric.earth_radius_vector_au);

    // --- Observer's geocentric latitude correction terms u, x, y,
    // Eq. 3.12.1, 3.12.3.
    let lat_rad = observer.latitude.radians();
    let u = (SPA_ELLIPSOID_ONE_MINUS_FLATTENING * lat_rad.tan()).atan();
    let elevation_over_radius = observer.elevation.meters() / SPA_EARTH_EQUATORIAL_RADIUS_M;
    let x = u.cos() + elevation_over_radius * lat_rad.cos();
    let y = SPA_ELLIPSOID_ONE_MINUS_FLATTENING * u.sin() + elevation_over_radius * lat_rad.sin();

    // --- Parallax in right ascension, Eq. 3.12.4.
    let xi_rad = xi_deg.to_radians();
    let h_rad = hour_angle_deg.to_radians();
    let delta_rad = geocentric.declination.radians();
    let parallax_ra_num = -x * xi_rad.sin() * h_rad.sin();
    let parallax_ra_denom = delta_rad.cos() - x * xi_rad.sin() * h_rad.cos();
    let parallax_right_ascension_deg = parallax_ra_num.atan2(parallax_ra_denom).to_degrees();

    // --- Topocentric declination, Eq. 3.12.5.
    let delta_alpha_rad = parallax_right_ascension_deg.to_radians();
    let topo_dec_num = (delta_rad.sin() - y * xi_rad.sin()) * delta_alpha_rad.cos();
    let topo_dec_denom = delta_rad.cos() - x * xi_rad.sin() * h_rad.cos();
    let topocentric_declination_deg = topo_dec_num.atan2(topo_dec_denom).to_degrees();

    // --- Topocentric local hour angle, Eq. 3.12.6.
    let topocentric_local_hour_angle_deg = hour_angle_deg - parallax_right_ascension_deg;

    // --- Topocentric elevation angle without atmosphere, Eq. 3.14.1
    // (uses the observer's *geographic* latitude, not the geocentric-
    // corrected u/x/y quantities, per the paper).
    let topo_dec_rad = topocentric_declination_deg.to_radians();
    let topo_h_rad = topocentric_local_hour_angle_deg.to_radians();
    let elevation_true_rad = (lat_rad.sin() * topo_dec_rad.sin()
        + lat_rad.cos() * topo_dec_rad.cos() * topo_h_rad.cos())
    .asin();
    let elevation_true = Angle::from_radians(elevation_true_rad);

    // --- Atmospheric refraction correction, Eq. 3.14.2 (with the switch
    // that zeroes the correction once the Sun is far enough below the
    // horizon that the small-angle refraction formula is no longer valid).
    let e0_deg = elevation_true.degrees();
    let refraction_deg = if e0_deg >= -(0.26667 + atmos_refract_deg) {
        (pressure_hpa / 1010.0) * (283.0 / (273.0 + temperature_celsius)) * 1.02
            / (60.0 * (e0_deg + 10.3 / (e0_deg + 5.11)).to_radians().tan())
    } else {
        0.0
    };
    let elevation_apparent = Angle::from_degrees(e0_deg + refraction_deg);

    let ninety = Angle::from_degrees(90.0);
    let zenith_true = ninety - elevation_true;
    let zenith_apparent = ninety - elevation_apparent;

    // --- Topocentric astronomers azimuth then converted to from-North,
    // Eq. 3.14.3-3.14.4.
    let astronomers_azimuth_rad = topo_h_rad
        .sin()
        .atan2(topo_h_rad.cos() * lat_rad.sin() - topo_dec_rad.tan() * lat_rad.cos());
    let azimuth = (Angle::from_radians(astronomers_azimuth_rad) + Angle::from_degrees(180.0))
        .normalized_0_2pi();

    let position = SolarPosition {
        zenith_true,
        zenith_apparent,
        elevation_true,
        elevation_apparent,
        azimuth,
        declination: geocentric.declination,
        right_ascension: geocentric.right_ascension,
        hour_angle: Angle::from_degrees(hour_angle_deg).normalized_pm_pi(),
        earth_sun_distance_au: geocentric.earth_radius_vector_au,
        equation_of_time_minutes: geocentric.equation_of_time_minutes,
    };

    SpaDetailedResult {
        geocentric,
        equatorial_horizontal_parallax_deg: xi_deg,
        parallax_right_ascension_deg,
        topocentric_declination_deg,
        topocentric_local_hour_angle_deg,
        position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::{Length, Pressure, Temperature};

    /// The Reda & Andreas (2004) paper's own published worked example
    /// (Golden, Colorado), independently reproduced via pvlib-python
    /// 0.15.2 (BSD-3-Clause) -- see
    /// `engine/validation-data/README.md`. This is the primary
    /// correctness test for the entire SPA implementation: it exercises
    /// every stage (geocentric ephemeris, parallax, refraction, azimuth)
    /// against the algorithm's own canonical reference case, matching to
    /// six decimal degrees.
    #[test]
    fn reda_andreas_2004_worked_example_full_topocentric() {
        let dt = CalendarDateTime {
            year: 2003,
            month: 10,
            day: 17,
            hour: 19,
            minute: 30,
            second: 30.0,
        };
        let jd_ut = dt.to_julian_day().unwrap();
        let observer = Observer {
            latitude: Angle::from_degrees(39.742476),
            longitude: Angle::from_degrees(-105.1786),
            elevation: Length::from_meters(1830.14),
        };

        let result = solar_position_precise(jd_ut, 67.0, observer, 820.0, 11.0, 0.5667);

        assert!(
            (result.position.elevation_true.degrees() - 39.872045903839926).abs() < 1e-6,
            "elevation_true = {}",
            result.position.elevation_true.degrees()
        );
        assert!(
            (result.position.elevation_apparent.degrees() - 39.88837797596303).abs() < 1e-6,
            "elevation_apparent = {}",
            result.position.elevation_apparent.degrees()
        );
        assert!(
            (result.position.zenith_apparent.degrees() - 50.11162202403697).abs() < 1e-6,
            "zenith_apparent = {}",
            result.position.zenith_apparent.degrees()
        );
        assert!(
            (result.position.azimuth.degrees() - 194.34024051024002).abs() < 1e-6,
            "azimuth = {}",
            result.position.azimuth.degrees()
        );
        assert!(
            (result.topocentric_local_hour_angle_deg - 11.106270548935946).abs() < 1e-6,
            "H' = {}",
            result.topocentric_local_hour_angle_deg
        );
        assert!(
            (result.topocentric_declination_deg - (-9.316178699714861)).abs() < 1e-6,
            "delta' = {}",
            result.topocentric_declination_deg
        );
    }

    #[test]
    fn parallax_correction_is_negligible_but_nonzero() {
        // Sanity check that the topocentric correction is actually being
        // applied (this engine's whole point of difference from
        // `crate::meeus`) and that its magnitude matches the physically
        // expected order (parallax amplitude <= xi ~ 8.8 arcsec ~ 0.0025 deg).
        let dt = CalendarDateTime {
            year: 2024,
            month: 6,
            day: 21,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let jd_ut = dt.to_julian_day().unwrap();
        let observer = Observer {
            latitude: Angle::from_degrees(45.0),
            longitude: Angle::from_degrees(0.0),
            elevation: Length::from_meters(0.0),
        };
        let result = solar_position_precise(jd_ut, 69.0, observer, 1013.25, 15.0, 0.5667);
        assert!(result.parallax_right_ascension_deg.abs() > 0.0);
        assert!(result.parallax_right_ascension_deg.abs() < 0.01);
    }

    #[test]
    fn night_time_sun_is_below_horizon() {
        let observer = Observer {
            latitude: Angle::from_degrees(48.85),
            longitude: Angle::from_degrees(2.35),
            elevation: Length::from_meters(35.0),
        };
        let dt = CalendarDateTime {
            year: 2024,
            month: 1,
            day: 15,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        let pos = solar_position(dt, observer, AtmosphericConditions::default()).unwrap();
        assert!(pos.elevation_true.degrees() < 0.0);
    }

    #[test]
    fn zenith_and_elevation_are_complementary() {
        let observer = Observer {
            latitude: Angle::from_degrees(48.85),
            longitude: Angle::from_degrees(2.35),
            elevation: Length::from_meters(35.0),
        };
        let dt = CalendarDateTime {
            year: 2024,
            month: 6,
            day: 21,
            hour: 10,
            minute: 30,
            second: 0.0,
        };
        let pos = solar_position(dt, observer, AtmosphericConditions::default()).unwrap();
        assert!((pos.zenith_true.degrees() + pos.elevation_true.degrees() - 90.0).abs() < 1e-9);
        assert!(
            (pos.zenith_apparent.degrees() + pos.elevation_apparent.degrees() - 90.0).abs() < 1e-9
        );
    }

    #[test]
    fn default_pressure_and_temperature_helper_matches_precise_call() {
        let observer = Observer {
            latitude: Angle::from_degrees(39.742476),
            longitude: Angle::from_degrees(-105.1786),
            elevation: Length::from_meters(1830.14),
        };
        let dt = CalendarDateTime {
            year: 2024,
            month: 5,
            day: 1,
            hour: 15,
            minute: 0,
            second: 0.0,
        };
        let atm = AtmosphericConditions {
            pressure: Pressure::from_hectopascal(1013.25),
            temperature: Temperature::from_celsius(20.0),
        };
        let via_convenience = solar_position(dt, observer, atm).unwrap();

        let jd_ut = dt.to_julian_day().unwrap();
        let delta_t_seconds = delta_t::estimate_delta_t_seconds(dt.year, dt.month);
        let via_precise = solar_position_precise(
            jd_ut,
            delta_t_seconds,
            observer,
            1013.25,
            20.0,
            DEFAULT_ATMOSPHERIC_REFRACTION_AT_HORIZON_DEG,
        )
        .position;

        assert!(
            (via_convenience.zenith_apparent.degrees() - via_precise.zenith_apparent.degrees())
                .abs()
                < 1e-12
        );
    }
}
