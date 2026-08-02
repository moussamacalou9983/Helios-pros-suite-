//! The selectable solar-position algorithm abstraction: [`Spa`] (primary,
//! default) and [`MeeusReduced`] (secondary, validation-only), both
//! implementing [`SolarPositionAlgorithm`] so application code can pick an
//! engine explicitly, or rely on [`crate::solar_position`] to use the
//! default.

use core_math::julian::{CalendarDateTime, JulianDayError};

use crate::common::{AtmosphericConditions, Observer, SolarPosition};
use crate::{meeus, spa};

/// A solar-position computation engine. Implemented by [`Spa`] (primary)
/// and [`MeeusReduced`] (secondary/validation), both producing the exact
/// same [`SolarPosition`] output type from the exact same input types,
/// which is what makes them interchangeable at this trait boundary —
/// callers depending on `dyn SolarPositionAlgorithm` or generic over `A:
/// SolarPositionAlgorithm` do not need to know which engine they are
/// using.
pub trait SolarPositionAlgorithm {
    /// Compute the topocentric solar position for a UTC instant and
    /// observer location.
    ///
    /// # Errors
    /// Returns [`JulianDayError`] if `datetime_utc` is not a valid
    /// Gregorian calendar date.
    fn solar_position(
        &self,
        datetime_utc: CalendarDateTime,
        observer: Observer,
        atmosphere: AtmosphericConditions,
    ) -> Result<SolarPosition, JulianDayError>;

    /// A short, human-readable identification of the engine and its
    /// primary literature source, suitable for inclusion in generated
    /// reports and logs.
    fn name(&self) -> &'static str;
}

/// The full NREL Solar Position Algorithm (Reda & Andreas, 2004,
/// NREL/TP-560-34302) — HeliosPro's **default, primary** solar-position
/// engine.
#[derive(Debug, Clone, Copy, Default)]
pub struct Spa;

impl SolarPositionAlgorithm for Spa {
    fn solar_position(
        &self,
        datetime_utc: CalendarDateTime,
        observer: Observer,
        atmosphere: AtmosphericConditions,
    ) -> Result<SolarPosition, JulianDayError> {
        spa::solar_position(datetime_utc, observer, atmosphere)
    }

    fn name(&self) -> &'static str {
        "SPA (Reda & Andreas, 2004, NREL/TP-560-34302) -- primary engine"
    }
}

/// The reduced-order Meeus (1998) engine — kept **only** as a secondary,
/// independently-sourced engine for cross-validation against [`Spa`]. See
/// `solar_geometry::meeus` for the full rationale.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeeusReduced;

impl SolarPositionAlgorithm for MeeusReduced {
    fn solar_position(
        &self,
        datetime_utc: CalendarDateTime,
        observer: Observer,
        atmosphere: AtmosphericConditions,
    ) -> Result<SolarPosition, JulianDayError> {
        meeus::solar_position(datetime_utc, observer, atmosphere)
    }

    fn name(&self) -> &'static str {
        "Meeus reduced-order (Astronomical Algorithms, 1998) -- secondary validation engine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::{Angle, Length};

    fn observer() -> Observer {
        Observer {
            latitude: Angle::from_degrees(45.0),
            longitude: Angle::from_degrees(5.0),
            elevation: Length::from_meters(200.0),
        }
    }

    fn noon() -> CalendarDateTime {
        CalendarDateTime {
            year: 2024,
            month: 6,
            day: 21,
            hour: 11,
            minute: 0,
            second: 0.0,
        }
    }

    #[test]
    fn spa_and_meeus_agree_within_their_documented_accuracy_budget() {
        // The two independently-sourced engines should never disagree by
        // more than a small fraction of a degree (see `solar_geometry`
        // module docs for the ~0.01-0.02 deg accuracy floor of the
        // secondary Meeus engine); a much larger disagreement would
        // indicate a bug in one of the two implementations.
        let spa_result = Spa
            .solar_position(noon(), observer(), AtmosphericConditions::default())
            .unwrap();
        let meeus_result = MeeusReduced
            .solar_position(noon(), observer(), AtmosphericConditions::default())
            .unwrap();

        let zenith_diff =
            (spa_result.zenith_true.degrees() - meeus_result.zenith_true.degrees()).abs();
        let azimuth_diff = (spa_result.azimuth.degrees() - meeus_result.azimuth.degrees()).abs();
        assert!(zenith_diff < 0.05, "zenith diff = {zenith_diff} deg");
        assert!(azimuth_diff < 0.05, "azimuth diff = {azimuth_diff} deg");
    }

    #[test]
    fn algorithm_names_identify_the_engine_and_its_role() {
        assert!(Spa.name().contains("primary"));
        assert!(MeeusReduced.name().contains("secondary"));
    }
}
