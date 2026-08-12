//! Reduced-order topocentric solar position, built on the secondary Meeus
//! (1998) geocentric engine ([`solar_geometry::meeus`]).
//!
//! ## Status: secondary validation engine (not the default)
//!
//! See `solar_geometry::meeus` and the crate-level documentation in
//! `lib.rs` for why this engine exists and is deliberately kept distinct
//! from [`crate::spa`] (the primary engine): a genuinely independent
//! second implementation is valuable evidence against shared bugs in a
//! way that comparing the primary engine against itself cannot be.
//!
//! Unlike [`crate::spa`], this engine omits topocentric parallax
//! correction entirely (its maximum amplitude, ~8.8 arcsec, is below this
//! engine's own ~0.01-0.02° accuracy floor set by the reduced Meeus
//! longitude series — see `solar_geometry::meeus` for the precision
//! discussion), so [`Observer::elevation`](crate::common::Observer) is
//! accepted (for API uniformity with [`crate::spa`]) but not used here.
//!
//! **Sources**:
//! - Overall multi-step procedure: Reda, I. & Andreas, A. (2004), "Solar
//!   Position Algorithm for Solar Radiation Applications", *Solar Energy*,
//!   76(5), 577-589 (structure only; this module's geocentric stage uses
//!   Meeus's reduced series, not SPA's tables).
//! - Horizontal-coordinate (elevation/azimuth) transform: Meeus, J. (1998),
//!   *Astronomical Algorithms*, 2nd ed., Willmann-Bell, Chapter 13
//!   ("Transformation of Coordinates"), Eq. 13.5-13.6.
//! - Atmospheric refraction: Sæmundsson's formula as reproduced in Meeus
//!   (1998), Chapter 16, Eq. 16.4, with the standard pressure/temperature
//!   scaling also used in Reda & Andreas (2004), Eq. 3.19.

use core_math::julian::{CalendarDateTime, JulianDayError};
use core_units::Angle;

use solar_geometry::{delta_t, meeus as meeus_geometry};

use crate::common::{AtmosphericConditions, Observer, SolarPosition};

/// Pure spherical-astronomy transform from (latitude, declination, hour
/// angle) to (geometric elevation, azimuth-from-North). Kept separate from
/// [`solar_position`] so the trigonometry itself can be validated against
/// exact geometric invariants (equinox sunrise/sunset azimuth, equinox
/// transit elevation, etc.) independent of the ephemeris/time machinery.
///
/// **Source**: Meeus (1998), Eq. 13.5-13.6 (azimuth measured from South in
/// Meeus's convention; converted here to the from-North, clockwise
/// convention used throughout HeliosPro).
pub(crate) fn topocentric_from_hour_angle(
    latitude: Angle,
    declination: Angle,
    hour_angle: Angle,
) -> (Angle, Angle) {
    let elevation_rad = (latitude.sin() * declination.sin()
        + latitude.cos() * declination.cos() * hour_angle.cos())
    .asin();
    let elevation = Angle::from_radians(elevation_rad);

    let azimuth_from_south_rad = f64::atan2(
        hour_angle.sin(),
        hour_angle.cos() * latitude.sin() - declination.tan() * latitude.cos(),
    );
    let azimuth = (Angle::from_radians(azimuth_from_south_rad) + Angle::from_degrees(180.0))
        .normalized_0_2pi();

    (elevation, azimuth)
}

/// Sæmundsson's atmospheric refraction correction (degrees to *add* to the
/// true/geometric elevation to obtain the apparent elevation).
///
/// **Source**: Meeus (1998), Eq. 16.4: `R = 1.02 / tan(h + 10.3/(h+5.11))`
/// (`R` in arcminutes, `h` = true altitude in degrees), with the standard
/// pressure/temperature scaling `(P/1010) * (283/(273+T))` also used in
/// Reda & Andreas (2004), Eq. 3.19.
///
/// Returns 0 for true elevations below -1°, where the formula's small-angle
/// behaviour becomes unreliable and the Sun's irradiance contribution is in
/// any case zero.
pub(crate) fn refraction_correction_degrees(
    true_elevation: Angle,
    atmosphere: AtmosphericConditions,
) -> f64 {
    let h_deg = true_elevation.degrees();
    if h_deg < -1.0 {
        return 0.0;
    }
    let r_arcmin = 1.02 / Angle::from_degrees(h_deg + 10.3 / (h_deg + 5.11)).tan();
    let pressure_scale = atmosphere.pressure.hectopascal() / 1010.0;
    let temperature_scale = 283.0 / (273.0 + atmosphere.temperature.celsius());
    (r_arcmin * pressure_scale * temperature_scale) / 60.0
}

/// Compute the full topocentric solar position for a UTC instant and
/// observer location, using the secondary Meeus (1998) engine.
///
/// # Errors
/// Returns [`JulianDayError`] if `datetime_utc` is not a valid Gregorian
/// calendar date (see [`CalendarDateTime::to_julian_day`]).
pub fn solar_position(
    datetime_utc: CalendarDateTime,
    observer: Observer,
    atmosphere: AtmosphericConditions,
) -> Result<SolarPosition, JulianDayError> {
    let jd_ut = datetime_utc.to_julian_day()?;
    let delta_t_seconds = delta_t::estimate_delta_t_seconds(datetime_utc.year, datetime_utc.month);
    Ok(solar_position_precise(
        jd_ut,
        delta_t_seconds,
        observer,
        atmosphere,
    ))
}

/// As [`solar_position`], but with `delta_t` specified explicitly instead
/// of estimated from the calendar year/month, and taking a UT-based Julian
/// Day directly rather than calendar fields (so this function is a pure,
/// infallible, deterministic function of its arguments — there is no
/// calendar validation left to perform). Used to compare this engine
/// against external reference data across a wide historical span (e.g.
/// 1900-2100): [`delta_t::estimate_delta_t_seconds`] is only documented as
/// valid for 2005-2050, so validation campaigns covering a wider year
/// range must supply a known/reference `delta_t` directly rather than rely
/// on that estimator's extrapolation. Mirrors
/// `solar_position::spa::solar_position_precise`, which exists for the
/// same reason and has the same signature shape.
pub fn solar_position_precise(
    jd_ut: f64,
    delta_t_seconds: f64,
    observer: Observer,
    atmosphere: AtmosphericConditions,
) -> SolarPosition {
    let jde = jd_ut + delta_t_seconds / core_constants::SECONDS_PER_DAY;

    let eph = meeus_geometry::compute(jde, jd_ut);

    let hour_angle = (eph.apparent_sidereal_time_greenwich + observer.longitude
        - eph.right_ascension)
        .normalized_pm_pi();

    let (elevation_true, azimuth) =
        topocentric_from_hour_angle(observer.latitude, eph.declination, hour_angle);

    let refraction_deg = refraction_correction_degrees(elevation_true, atmosphere);
    let elevation_apparent = Angle::from_degrees(elevation_true.degrees() + refraction_deg);

    let ninety = Angle::from_degrees(90.0);

    SolarPosition {
        zenith_true: ninety - elevation_true,
        zenith_apparent: ninety - elevation_apparent,
        elevation_true,
        elevation_apparent,
        azimuth,
        declination: eph.declination,
        right_ascension: eph.right_ascension,
        hour_angle,
        earth_sun_distance_au: eph.earth_sun_distance_au,
        equation_of_time_minutes: eph.equation_of_time_minutes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::Length;

    fn test_observer(latitude_deg: f64, longitude_deg: f64) -> Observer {
        Observer {
            latitude: Angle::from_degrees(latitude_deg),
            longitude: Angle::from_degrees(longitude_deg),
            elevation: Length::from_meters(0.0),
        }
    }

    #[test]
    fn equinox_noon_elevation_equals_ninety_minus_latitude() {
        // At the equinox (declination = 0) and solar noon (H = 0), the
        // Sun's elevation equals 90 deg minus the observer's latitude —
        // an exact, well-known geometric identity, independent of any
        // ephemeris precision question.
        let lat = Angle::from_degrees(45.0);
        let dec = Angle::from_degrees(0.0);
        let h = Angle::from_degrees(0.0);
        let (elevation, azimuth) = topocentric_from_hour_angle(lat, dec, h);
        assert!((elevation.degrees() - 45.0).abs() < 1e-9);
        assert!(
            (azimuth.degrees() - 180.0).abs() < 1e-9,
            "azimuth = {}",
            azimuth.degrees()
        );
    }

    #[test]
    fn equator_overhead_sun_at_equinox_noon() {
        let lat = Angle::from_degrees(0.0);
        let dec = Angle::from_degrees(0.0);
        let h = Angle::from_degrees(0.0);
        let (elevation, _azimuth) = topocentric_from_hour_angle(lat, dec, h);
        assert!((elevation.degrees() - 90.0).abs() < 1e-9);
    }

    #[test]
    fn equinox_sunrise_is_due_east_for_any_latitude() {
        // At equinox, sunrise (H = -90 deg) occurs at elevation = 0 and
        // azimuth = 90 deg (due East) for any latitude.
        let dec = Angle::from_degrees(0.0);
        let h = Angle::from_degrees(-90.0);
        for lat_deg in [-60.0, -23.44, 0.0, 23.44, 60.0] {
            let lat = Angle::from_degrees(lat_deg);
            let (elevation, azimuth) = topocentric_from_hour_angle(lat, dec, h);
            assert!(elevation.degrees().abs() < 1e-9, "lat={lat_deg}");
            assert!(
                (azimuth.degrees() - 90.0).abs() < 1e-9,
                "lat={lat_deg}, az={}",
                azimuth.degrees()
            );
        }
    }

    #[test]
    fn equinox_sunset_is_due_west_for_any_latitude() {
        let dec = Angle::from_degrees(0.0);
        let h = Angle::from_degrees(90.0);
        for lat_deg in [-60.0, -23.44, 0.0, 23.44, 60.0] {
            let lat = Angle::from_degrees(lat_deg);
            let (elevation, azimuth) = topocentric_from_hour_angle(lat, dec, h);
            assert!(elevation.degrees().abs() < 1e-9, "lat={lat_deg}");
            assert!(
                (azimuth.degrees() - 270.0).abs() < 1e-9,
                "lat={lat_deg}, az={}",
                azimuth.degrees()
            );
        }
    }

    #[test]
    fn refraction_is_positive_near_horizon_and_negligible_overhead() {
        let atm = AtmosphericConditions::default();
        let near_horizon = refraction_correction_degrees(Angle::from_degrees(0.0), atm);
        let overhead = refraction_correction_degrees(Angle::from_degrees(89.0), atm);
        assert!(near_horizon > 0.2, "near_horizon = {near_horizon}");
        assert!(overhead < 0.02, "overhead = {overhead}");
        assert!(near_horizon > overhead);
    }

    #[test]
    fn full_pipeline_produces_sun_to_the_south_at_local_solar_noon_paris() {
        // Paris (48.85 deg N, 2.35 deg E), a mid-latitude site where the
        // Sun is always well clear of the zenith (so azimuth is
        // well-conditioned, unlike near-equatorial/near-zenith cases where
        // azimuth changes very rapidly with hour angle). Use the crate's
        // own `solar_noon_utc_hour` to find the precise instant of solar
        // noon, then verify the full pipeline places the Sun almost
        // exactly due South with a near-zero hour angle.
        let observer = test_observer(48.85, 2.35);
        let date = CalendarDateTime {
            year: 2024,
            month: 3,
            day: 20,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        let noon_utc_hour =
            solar_geometry::time::solar_noon_utc_hour(date, observer.longitude).unwrap();
        let hour = noon_utc_hour.floor() as u32;
        let minute = ((noon_utc_hour - hour as f64) * 60.0).round() as u32;
        let dt = CalendarDateTime {
            hour,
            minute,
            second: 0.0,
            ..date
        };

        let pos = solar_position(dt, observer, AtmosphericConditions::default()).unwrap();
        assert!(
            pos.hour_angle.degrees().abs() < 0.3,
            "hour angle = {} deg",
            pos.hour_angle.degrees()
        );
        assert!(
            (pos.azimuth.degrees() - 180.0).abs() < 0.5,
            "azimuth = {} deg",
            pos.azimuth.degrees()
        );
        // At the equinox, transit elevation = 90 - latitude = 41.15 deg.
        assert!(
            (pos.elevation_true.degrees() - 41.15).abs() < 0.5,
            "elevation = {} deg",
            pos.elevation_true.degrees()
        );
    }

    #[test]
    fn azimuth_sweeps_rapidly_near_zenith_passage_at_low_latitude() {
        // Physical sanity check for a well-known low-latitude phenomenon:
        // near the equator around the equinox, the Sun passes very close
        // to the zenith at transit, and azimuth is then ill-conditioned
        // (it can change by several degrees for a hour-angle change of
        // only a degree or two), unlike the mid-latitude case above. This
        // test documents and pins down that behaviour rather than
        // asserting a tight azimuth tolerance, which would be physically
        // meaningless this close to zenith passage.
        let lat = Angle::from_degrees(12.1);
        let dec = Angle::from_degrees(0.0);
        let (elevation_at_transit, azimuth_at_transit) =
            topocentric_from_hour_angle(lat, dec, Angle::from_degrees(0.0));
        let (_elevation_off_transit, azimuth_off_transit) =
            topocentric_from_hour_angle(lat, dec, Angle::from_degrees(5.0));
        assert!((elevation_at_transit.degrees() - 77.9).abs() < 0.1);
        assert!((azimuth_at_transit.degrees() - 180.0).abs() < 1e-9);
        // A 5 deg hour-angle change swings azimuth by several tens of
        // degrees this close to the zenith -- confirming the effect is
        // real and not a bug in the earlier (now removed) tight-tolerance
        // assertion.
        assert!((azimuth_off_transit.degrees() - azimuth_at_transit.degrees()).abs() > 5.0);
    }

    #[test]
    fn zenith_and_elevation_are_complementary() {
        let observer = test_observer(48.85, 2.35);
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
    fn night_time_sun_is_below_horizon() {
        // Paris, near local midnight UTC in winter: Sun should be well
        // below the horizon (negative elevation).
        let observer = test_observer(48.85, 2.35);
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
}
