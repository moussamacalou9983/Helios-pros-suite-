//! # solar_position
//!
//! Topocentric solar position (zenith, azimuth, elevation), with a
//! **selectable algorithm**: the full NREL Solar Position Algorithm
//! ([`spa`], Reda & Andreas 2004 — the **default, primary** engine) or a
//! reduced-order Meeus (1998) engine ([`meeus`] — kept only as an
//! independent **secondary engine for cross-validation**, per an explicit
//! project decision documented in `solar_geometry::lib`).
//!
//! ## Choosing an engine
//!
//! Application code that does not need to choose should call
//! [`solar_position`] directly, which uses [`algorithm::Spa`] (identical
//! to calling `spa::solar_position` directly, provided for discoverability
//! and so the "what is the default" question has one obvious answer in the
//! code). Code that needs to compare, benchmark, or explicitly select an
//! engine should use the [`SolarPositionAlgorithm`] trait:
//!
//! ```
//! use core_math::julian::CalendarDateTime;
//! use core_units::{Angle, Length};
//! use solar_position::{AtmosphericConditions, Observer, SolarPositionAlgorithm, Spa, MeeusReduced};
//!
//! let observer = Observer {
//!     latitude: Angle::from_degrees(45.0),
//!     longitude: Angle::from_degrees(5.0),
//!     elevation: Length::from_meters(200.0),
//! };
//! let dt = CalendarDateTime { year: 2024, month: 6, day: 21, hour: 11, minute: 0, second: 0.0 };
//! let atm = AtmosphericConditions::default();
//!
//! let spa_result = Spa.solar_position(dt, observer, atm).unwrap();
//! let meeus_result = MeeusReduced.solar_position(dt, observer, atm).unwrap();
//! ```
//!
//! ## Sign and unit conventions
//!
//! See `solar_geometry::lib` for the conventions binding this crate too
//! (longitude positive east, azimuth from North clockwise, UTC calendar
//! time inputs).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod algorithm;
mod common;
pub mod meeus;
pub mod spa;

pub use algorithm::{MeeusReduced, SolarPositionAlgorithm, Spa};
pub use common::{AtmosphericConditions, Observer, SolarPosition};

use core_math::julian::{CalendarDateTime, JulianDayError};

/// Compute the topocentric solar position using HeliosPro's default engine
/// ([`Spa`], the full NREL Solar Position Algorithm). Equivalent to
/// `Spa.solar_position(datetime_utc, observer, atmosphere)`.
///
/// # Errors
/// Returns [`JulianDayError`] if `datetime_utc` is not a valid Gregorian
/// calendar date.
pub fn solar_position(
    datetime_utc: CalendarDateTime,
    observer: Observer,
    atmosphere: AtmosphericConditions,
) -> Result<SolarPosition, JulianDayError> {
    Spa.solar_position(datetime_utc, observer, atmosphere)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::{Angle, Length};

    #[test]
    fn default_entry_point_matches_explicit_spa_selection() {
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
        let atm = AtmosphericConditions::default();

        let via_default = solar_position(dt, observer, atm).unwrap();
        let via_explicit_spa = Spa.solar_position(dt, observer, atm).unwrap();

        assert_eq!(
            via_default.zenith_apparent.degrees(),
            via_explicit_spa.zenith_apparent.degrees()
        );
    }
}
