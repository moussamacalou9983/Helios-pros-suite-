//! Numerical stability of the solar-position engines
//! ([`solar_position::Spa`], [`solar_position::MeeusReduced`]) at
//! deliberately degenerate/extreme inputs.
//!
//! This is distinct from [`validation`]'s external accuracy campaigns
//! (`engine/crates/validation/src/report.rs`,
//! `engine/crates/validation/src/global_campaign/mod.rs`, which already
//! assert finiteness incidentally across ~2900 realistic cases): the tests
//! here deliberately target the exact synthetic edge cases where a naive
//! implementation of the underlying trigonometry is most likely to produce
//! `NaN`, `+-inf`, or a panic -- very small angles, angles at/near 90 deg,
//! angles at/near 180 deg, the geographic poles (+-90 deg latitude, where
//! `tan(latitude)` is nominally undefined), years far outside any claimed
//! accuracy window, and UTC midnight/day-boundary crossings -- rather than
//! relying on realistic cases to happen to exercise them. No case here
//! makes an accuracy claim (several are far outside either engine's
//! documented validity range, e.g. year 9999); every case makes only a
//! **stability** claim: finite output, no panic.

use core_math::julian::CalendarDateTime;
use core_units::{Angle, Length};
use solar_position::{
    AtmosphericConditions, MeeusReduced, Observer, SolarPosition, SolarPositionAlgorithm, Spa,
};

fn observer(lat_deg: f64, lon_deg: f64, elev_m: f64) -> Observer {
    Observer {
        latitude: Angle::from_degrees(lat_deg),
        longitude: Angle::from_degrees(lon_deg),
        elevation: Length::from_meters(elev_m),
    }
}

fn dt(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: f64) -> CalendarDateTime {
    CalendarDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }
}

/// Every scalar quantity in [`SolarPosition`] must be finite: `NaN` or
/// `+-inf` anywhere here would silently propagate into every downstream
/// engine module (irradiance, transposition, thermal, and eventually
/// electrical yield), so this is checked exhaustively, not just on the
/// headline zenith/azimuth.
fn assert_finite(label: &str, engine_name: &str, pos: &SolarPosition) {
    let fields: [(&str, f64); 9] = [
        ("zenith_true", pos.zenith_true.degrees()),
        ("zenith_apparent", pos.zenith_apparent.degrees()),
        ("elevation_true", pos.elevation_true.degrees()),
        ("elevation_apparent", pos.elevation_apparent.degrees()),
        ("azimuth", pos.azimuth.degrees()),
        ("declination", pos.declination.degrees()),
        ("right_ascension", pos.right_ascension.degrees()),
        ("hour_angle", pos.hour_angle.degrees()),
        ("earth_sun_distance_au", pos.earth_sun_distance_au),
    ];
    for (name, value) in fields {
        assert!(
            value.is_finite(),
            "{engine_name} produced a non-finite {name} = {value} for case '{label}'"
        );
    }
    assert!(
        pos.equation_of_time_minutes.is_finite(),
        "{engine_name} produced a non-finite equation_of_time_minutes = {} for case '{label}'",
        pos.equation_of_time_minutes
    );
}

/// Runs `case` through both engines and asserts finiteness of both.
/// Returns silently (without asserting) if the date is calendar-invalid
/// (e.g. pre-Gregorian) -- that is a documented, deliberate `Err`, not an
/// instability, and is exercised separately by `core_math::julian`'s own
/// tests.
fn check_both_engines(label: &str, datetime_utc: CalendarDateTime, obs: Observer) {
    let atm = AtmosphericConditions::default();
    if let Ok(pos) = Spa.solar_position(datetime_utc, obs, atm) {
        assert_finite(label, "SPA", &pos);
    }
    if let Ok(pos) = MeeusReduced.solar_position(datetime_utc, obs, atm) {
        assert_finite(label, "Meeus", &pos);
    }
}

#[test]
fn very_small_angles() {
    // Latitude, longitude, and elevation all at (or a hair off) zero;
    // hour angle near zero (local solar noon at the Greenwich meridian);
    // declination near zero (close to an equinox).
    check_both_engines(
        "equator_prime_meridian_solar_noon",
        dt(2024, 3, 20, 12, 0, 0.0),
        observer(0.0, 0.0, 0.0),
    );
    check_both_engines(
        "equator_prime_meridian_tiny_offset",
        dt(2024, 3, 20, 12, 0, 0.0),
        observer(1e-9, 1e-9, 0.0),
    );
    check_both_engines(
        "declination_exactly_at_equinox_instant",
        // 2024 March equinox is close to 2024-03-20 03:06 UTC; the exact
        // instant is not required for this stability check, only
        // proximity to declination = 0.
        dt(2024, 3, 20, 3, 6, 0.0),
        observer(45.0, 5.0, 200.0),
    );
}

#[test]
fn angles_close_to_90_degrees() {
    // Latitude a hair short of the poles (the poles themselves are
    // covered separately in `extreme_latitudes_at_the_poles`, since that
    // is the more degenerate case: `tan(90 deg)`).
    check_both_engines(
        "latitude_just_short_of_north_pole",
        dt(2024, 6, 21, 12, 0, 0.0),
        observer(89.999999, 10.0, 100.0),
    );
    check_both_engines(
        "latitude_just_short_of_south_pole",
        dt(2024, 12, 21, 12, 0, 0.0),
        observer(-89.999999, 10.0, 100.0),
    );
    // Subsolar point: observer at the latitude equal to the declination
    // at local solar noon puts true elevation at (near) exactly 90 deg,
    // the azimuth-formula coordinate singularity documented in
    // `validation::global_campaign`.
    check_both_engines(
        "near_subsolar_point_june_solstice",
        dt(2024, 6, 21, 12, 0, 0.0),
        observer(23.436, 0.0, 0.0),
    );
}

#[test]
fn angles_close_to_180_degrees() {
    // Hour angle near 180 deg: local solar midnight (antisolar meridian
    // transit) at the Greenwich meridian.
    check_both_engines(
        "hour_angle_near_180_local_solar_midnight",
        dt(2024, 6, 21, 0, 0, 0.0),
        observer(45.0, 0.0, 200.0),
    );
    // Azimuth near 180 deg (due South, the everyday case for a Northern
    // Hemisphere solar-noon transit) plus, at high latitude in winter, an
    // elevation near the near-horizon refraction-cutoff band documented
    // in `validation::global_campaign`.
    check_both_engines(
        "azimuth_near_180_high_latitude_winter_noon",
        dt(2024, 12, 21, 12, 0, 0.0),
        observer(65.0, 0.0, 50.0),
    );
}

#[test]
fn extreme_latitudes_at_the_poles() {
    // Exactly +-90 deg: `tan(latitude)` is nominally undefined at the
    // true pole. In `f64`, `(pi/2).tan()` (the *representable* f64
    // nearest pi/2, not the true irrational pi/2) evaluates to a large
    // but finite value, not NaN/inf -- this test exists to pin that this
    // remains true through every downstream step (the `atan`/`atan2`
    // calls that consume it), not just at the `tan()` call site itself.
    for lat in [90.0, -90.0] {
        for month in [3, 6, 9, 12] {
            for hour in [0, 6, 12, 18] {
                check_both_engines(
                    &format!("pole_lat{lat}_month{month}_hour{hour}"),
                    dt(2024, month, 15, hour, 0, 0.0),
                    observer(lat, 30.0, 0.0),
                );
            }
        }
    }
}

#[test]
fn very_distant_years() {
    // 1582: the earliest year `core_math::julian::CalendarDateTime`
    // accepts at all (the Gregorian calendar reform boundary).
    check_both_engines(
        "earliest_supported_gregorian_year",
        dt(1582, 10, 15, 12, 0, 0.0),
        observer(45.0, 5.0, 200.0),
    );
    // 6000: the upper bound of Reda & Andreas (2004)'s own claimed
    // +-0.0003 deg accuracy window (-2000 to 6000) -- still an accuracy
    // claim the paper makes, so worth a dedicated stability check at
    // exactly that boundary.
    check_both_engines(
        "spa_claimed_upper_accuracy_boundary_year_6000",
        dt(6000, 6, 21, 12, 0, 0.0),
        observer(45.0, 5.0, 200.0),
    );
    // 9999: far beyond any accuracy claim either engine makes (this is a
    // pure stability check -- the computed position is not expected to be
    // astronomically meaningful, only finite and panic-free).
    check_both_engines(
        "far_future_year_9999_no_accuracy_claim",
        dt(9999, 6, 21, 12, 0, 0.0),
        observer(45.0, 5.0, 200.0),
    );
    // A large negative-offset date near the minimum representable `i32`
    // year is rejected by `CalendarDateTime::validate` (pre-Gregorian),
    // which `check_both_engines` already treats as a non-assertion case
    // (a documented `Err`, not an instability) -- see
    // `core_math::julian::tests::rejects_pre_gregorian_dates`.
}

#[test]
fn hours_close_to_midnight() {
    check_both_engines(
        "exact_utc_midnight",
        dt(2024, 6, 21, 0, 0, 0.0),
        observer(45.0, 5.0, 200.0),
    );
    check_both_engines(
        "one_microsecond_before_midnight",
        dt(2024, 6, 21, 23, 59, 59.999_999),
        observer(45.0, 5.0, 200.0),
    );
    // Crossing a year boundary at midnight: the Julian Day arithmetic
    // must roll over cleanly (no discontinuity/NaN at the year-end
    // month/day/hour boundary all changing simultaneously).
    check_both_engines(
        "new_year_eve_just_before_midnight",
        dt(2024, 12, 31, 23, 59, 59.999_999),
        observer(45.0, 5.0, 200.0),
    );
    check_both_engines(
        "new_year_day_just_after_midnight",
        dt(2025, 1, 1, 0, 0, 0.000_001),
        observer(45.0, 5.0, 200.0),
    );
    // Crossing a leap-day boundary (2024 is a leap year) at midnight.
    check_both_engines(
        "leap_day_just_before_midnight",
        dt(2024, 2, 29, 23, 59, 59.999_999),
        observer(45.0, 5.0, 200.0),
    );
    check_both_engines(
        "day_after_leap_day_just_after_midnight",
        dt(2024, 3, 1, 0, 0, 0.000_001),
        observer(45.0, 5.0, 200.0),
    );
}
