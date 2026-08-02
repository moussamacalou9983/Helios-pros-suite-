//! Calendar-to-Julian-Day conversion.
//!
//! **Source**: Meeus, J. (1998). *Astronomical Algorithms*, 2nd ed.,
//! Willmann-Bell, Chapter 7, "Julian Day". The algorithm below is the
//! standard Gregorian-calendar Julian Day Number formula reproduced in
//! every modern positional-astronomy reference (also found, in equivalent
//! form, in the *Explanatory Supplement to the Astronomical Almanac*).
//!
//! Valid for Gregorian calendar dates on or after 1582-10-15. HeliosPro's
//! scientific engine only ever needs dates in the industrial/renewable-
//! energy era (project lifetimes, historical weather records), so the
//! Julian-calendar branch used for earlier dates is deliberately not
//! implemented (`year < 1582` is rejected, see [`JulianDayError`]).
//!
//! ## Architecture decision: engine time is always UTC
//!
//! [`CalendarDateTime`] carries no timezone information. All HeliosPro
//! engine crates work exclusively in UTC; conversion from a local civil
//! timezone (with DST rules, etc.) to UTC is an application-layer /
//! ingestion-layer responsibility, deliberately kept out of the scientific
//! core so that the engine's calculations remain simple, pure, and
//! independent of any timezone database.

/// A UTC calendar date and time, decomposed into civil calendar fields.
///
/// # Fields
/// - `year`: astronomical year numbering (no year 0 ambiguity issue arises
///   for the supported range, since only `year >= 1582` is accepted).
/// - `month`: 1 (January) through 12 (December).
/// - `day`: 1 through the number of days in `month`.
/// - `hour`: 0 through 23.
/// - `minute`: 0 through 59.
/// - `second`: 0.0 through (exclusive) 61.0, to tolerate leap seconds; not
///   otherwise validated (leap-second bookkeeping is out of scope for
///   engineering-grade solar position accuracy, whose sensitivity to a
///   ~1-second time error is far below other model uncertainties).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalendarDateTime {
    /// Astronomical (proleptic Gregorian) year.
    pub year: i32,
    /// Month, 1-12.
    pub month: u32,
    /// Day of month, 1-31.
    pub day: u32,
    /// Hour of day, UTC, 0-23.
    pub hour: u32,
    /// Minute, 0-59.
    pub minute: u32,
    /// Second, 0.0-61.0 (fractional seconds allowed).
    pub second: f64,
}

/// Error returned when a [`CalendarDateTime`] cannot be converted to a
/// Julian Day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JulianDayError {
    /// The supplied year predates the Gregorian calendar reform
    /// (1582-10-15), which this implementation does not model.
    PreGregorianDate,
    /// `month` was outside the valid range 1-12.
    InvalidMonth,
    /// `day` was outside the valid range 1-31.
    InvalidDay,
    /// `hour` was outside the valid range 0-23.
    InvalidHour,
    /// `minute` was outside the valid range 0-59.
    InvalidMinute,
}

impl CalendarDateTime {
    fn validate(&self) -> Result<(), JulianDayError> {
        if self.year < 1582 {
            return Err(JulianDayError::PreGregorianDate);
        }
        if !(1..=12).contains(&self.month) {
            return Err(JulianDayError::InvalidMonth);
        }
        if !(1..=31).contains(&self.day) {
            return Err(JulianDayError::InvalidDay);
        }
        if self.hour > 23 {
            return Err(JulianDayError::InvalidHour);
        }
        if self.minute > 59 {
            return Err(JulianDayError::InvalidMinute);
        }
        Ok(())
    }

    /// Convert to the Julian Day Number (JD), a continuous count of days
    /// (and fraction thereof) since noon UTC on 4713 BCE January 1
    /// (proleptic Julian calendar).
    ///
    /// **Source**: Meeus (1998), Chapter 7, Eq. 7.1.
    pub fn to_julian_day(&self) -> Result<f64, JulianDayError> {
        self.validate()?;

        let day_fraction = self.day as f64
            + (self.hour as f64 + self.minute as f64 / 60.0 + self.second / 3600.0) / 24.0;

        // Meeus's algorithm treats January and February as months 13 and 14
        // of the *previous* year.
        let (y, m) = if self.month <= 2 {
            (self.year - 1, self.month + 12)
        } else {
            (self.year, self.month)
        };

        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();

        let jd = (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + day_fraction
            + b
            - 1524.5;

        Ok(jd)
    }

    /// Convert to Julian centuries elapsed since the J2000.0 epoch
    /// (2000-01-01 12:00 TT, JD 2451545.0), the time argument `T` used by
    /// the low-order solar position series in `heliospro-solar`.
    ///
    /// **Source**: Meeus (1998), Chapter 22 (definition of `T`).
    ///
    /// This function does not apply a TT-UTC correction (`ΔT`); callers
    /// requiring Terrestrial Time precision should add `ΔT` (seconds) to
    /// the Julian Day before calling, as documented in
    /// `heliospro-solar::time`.
    pub fn to_julian_century(&self) -> Result<f64, JulianDayError> {
        const J2000_EPOCH_JD: f64 = 2_451_545.0;
        const DAYS_PER_JULIAN_CENTURY: f64 = 36_525.0;
        let jd = self.to_julian_day()?;
        Ok((jd - J2000_EPOCH_JD) / DAYS_PER_JULIAN_CENTURY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// J2000.0 epoch, by definition JD = 2451545.0.
    /// Source: Meeus (1998), Chapter 7.
    #[test]
    fn j2000_epoch() {
        let dt = CalendarDateTime {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let jd = dt.to_julian_day().unwrap();
        assert!((jd - 2_451_545.0).abs() < 1e-9, "jd = {jd}");
    }

    /// Meeus (1998), Chapter 7, Example 7.a: 1957 October 4.81 -> JD =
    /// 2436116.31 (the Sputnik 1 launch date, the canonical worked example
    /// for this algorithm).
    #[test]
    fn meeus_example_7a_sputnik_launch() {
        // 4.81 days = day 4 + 0.81*24h = 4d 19h 26m 24s.
        let dt = CalendarDateTime {
            year: 1957,
            month: 10,
            day: 4,
            hour: 19,
            minute: 26,
            second: 24.0,
        };
        let jd = dt.to_julian_day().unwrap();
        assert!((jd - 2_436_116.31).abs() < 1e-6, "jd = {jd}");
    }

    #[test]
    fn julian_century_is_zero_at_j2000() {
        let dt = CalendarDateTime {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let t = dt.to_julian_century().unwrap();
        assert!(t.abs() < 1e-12, "T = {t}");
    }

    #[test]
    fn julian_century_one_century_later() {
        let dt = CalendarDateTime {
            year: 2100,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let t = dt.to_julian_century().unwrap();
        assert!((t - 1.0).abs() < 1e-6, "T = {t}");
    }

    #[test]
    fn rejects_pre_gregorian_dates() {
        let dt = CalendarDateTime {
            year: 1500,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(dt.to_julian_day(), Err(JulianDayError::PreGregorianDate));
    }

    #[test]
    fn rejects_invalid_month() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 13,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0.0,
        };
        assert_eq!(dt.to_julian_day(), Err(JulianDayError::InvalidMonth));
    }
}
