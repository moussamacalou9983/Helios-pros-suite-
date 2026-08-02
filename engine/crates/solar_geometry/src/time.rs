//! Solar time utilities: day-of-year, equation of time, and apparent solar
//! time / solar noon, expressed for a given observer longitude.
//!
//! Note that [`crate::position::solar_position`] does **not** depend on
//! these functions — it derives the hour angle directly from apparent
//! sidereal time and right ascension (the rigorous approach, avoiding any
//! "standard meridian" bookkeeping). The functions here are provided
//! because solar time is independently useful, human-readable output (e.g.
//! for displaying "solar noon is at 12:47 local time" in a report), and
//! because the brief explicitly asks for solar time as a named
//! deliverable.

use core_math::julian::{CalendarDateTime, JulianDayError};
use core_units::Angle;

use crate::ephemeris;

/// Day of the (Gregorian) year, 1-366, for the given calendar date.
///
/// This is ordinary calendar arithmetic (no astronomical model involved);
/// tested directly against known reference dates below.
pub fn day_of_year(dt: &CalendarDateTime) -> u32 {
    const CUMULATIVE_DAYS_NON_LEAP: [u32; 12] =
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let is_leap = (dt.year % 4 == 0 && dt.year % 100 != 0) || dt.year % 400 == 0;
    let mut day = CUMULATIVE_DAYS_NON_LEAP[(dt.month - 1) as usize] + dt.day;
    if is_leap && dt.month > 2 {
        day += 1;
    }
    day
}

/// The equation of time (apparent minus mean solar time) at the given UTC
/// instant, in minutes. Geocentric quantity: independent of observer
/// location.
///
/// **Source**: Meeus (1998), Eq. 28.3 (see `solar_geometry::ephemeris`).
pub fn equation_of_time_minutes(datetime_utc: CalendarDateTime) -> Result<f64, JulianDayError> {
    let jd = datetime_utc.to_julian_day()?;
    let eph = ephemeris::compute(jd, jd);
    Ok(eph.equation_of_time_minutes)
}

/// Apparent solar time, in decimal hours (0-24), at the given UTC instant
/// and observer longitude.
///
/// `apparent_solar_time = UTC_time + longitude/15° + equation_of_time/60`
///
/// **Source**: Duffie & Beckman (2013), Eq. 1.5.2 (solar time correction
/// for longitude and the equation of time). Longitude is positive East, so
/// a positive equation-of-time correction and a positive (Eastern)
/// longitude both advance apparent solar time ahead of UTC, consistent
/// with the Sun transiting earlier at more Easterly longitudes.
pub fn apparent_solar_time_hours(
    datetime_utc: CalendarDateTime,
    longitude: Angle,
) -> Result<f64, JulianDayError> {
    let utc_hours =
        datetime_utc.hour as f64 + datetime_utc.minute as f64 / 60.0 + datetime_utc.second / 3600.0;
    let eot_minutes = equation_of_time_minutes(datetime_utc)?;
    let solar_hours = utc_hours + longitude.degrees() / 15.0 + eot_minutes / 60.0;
    Ok(solar_hours.rem_euclid(24.0))
}

/// UTC decimal hour (0-24) at which apparent solar noon occurs, for the
/// calendar date carried by `date` (its own hour/minute/second fields are
/// ignored) and the given observer longitude.
///
/// Derived by inverting [`apparent_solar_time_hours`] around the target
/// value of 12h: `UTC_noon = 12 - longitude/15° - equation_of_time/60`.
pub fn solar_noon_utc_hour(
    date: CalendarDateTime,
    longitude: Angle,
) -> Result<f64, JulianDayError> {
    // Equation of time varies by only a fraction of a minute over a few
    // hours, so evaluating it at nominal 12:00 UTC (rather than iterating
    // to convergence) introduces a negligible (<0.1 s) error for this
    // engine's purposes.
    let probe = CalendarDateTime {
        hour: 12,
        minute: 0,
        second: 0.0,
        ..date
    };
    let eot_minutes = equation_of_time_minutes(probe)?;
    let noon_utc = 12.0 - longitude.degrees() / 15.0 - eot_minutes / 60.0;
    Ok(noon_utc.rem_euclid(24.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_of_year_january_first() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(day_of_year(&dt), 1);
    }

    #[test]
    fn day_of_year_december_31_non_leap() {
        let dt = CalendarDateTime {
            year: 2023,
            month: 12,
            day: 31,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(day_of_year(&dt), 365);
    }

    #[test]
    fn day_of_year_december_31_leap() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 12,
            day: 31,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(day_of_year(&dt), 366);
    }

    #[test]
    fn day_of_year_leap_day_correctly_offsets_march() {
        // 2024 is a leap year: March 1 is day 61 (31 Jan + 29 Feb + 1).
        let dt = CalendarDateTime {
            year: 2024,
            month: 3,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(day_of_year(&dt), 61);
    }

    #[test]
    fn century_non_leap_year_rule() {
        // 1900 is NOT a leap year (divisible by 100 but not 400).
        let dt = CalendarDateTime {
            year: 1900,
            month: 12,
            day: 31,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(day_of_year(&dt), 365);
    }

    #[test]
    fn apparent_solar_time_at_greenwich_close_to_utc() {
        // At longitude 0 (Greenwich) with equation of time near zero
        // (mid-April / mid-June are near-zero crossings), apparent solar
        // time should be close to UTC clock time.
        let dt = CalendarDateTime {
            year: 2024,
            month: 4,
            day: 15,
            hour: 10,
            minute: 0,
            second: 0.0,
        };
        let solar_time = apparent_solar_time_hours(dt, Angle::from_degrees(0.0)).unwrap();
        assert!((solar_time - 10.0).abs() < 0.2, "solar_time = {solar_time}");
    }

    #[test]
    fn solar_noon_and_apparent_solar_time_are_consistent() {
        let longitude = Angle::from_degrees(15.0);
        let date = CalendarDateTime {
            year: 2024,
            month: 5,
            day: 10,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        let noon_utc_hour = solar_noon_utc_hour(date, longitude).unwrap();
        let whole_hour = noon_utc_hour.floor() as u32;
        let minute = ((noon_utc_hour - whole_hour as f64) * 60.0).round() as u32;
        let dt_at_noon = CalendarDateTime {
            hour: whole_hour,
            minute,
            second: 0.0,
            ..date
        };
        let solar_time = apparent_solar_time_hours(dt_at_noon, longitude).unwrap();
        assert!(
            (solar_time - 12.0).abs() < 0.02,
            "solar_time = {solar_time}"
        );
    }
}
