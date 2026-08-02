//! Types shared by every solar-position algorithm implementation in this
//! crate ([`crate::spa`], [`crate::meeus`]): the observer's location and
//! local atmosphere (inputs), and the resulting topocentric position
//! (output). Keeping these engine-agnostic is what makes the
//! [`crate::SolarPositionAlgorithm`] abstraction possible: both engines
//! consume and produce exactly the same types.

use core_units::{Angle, Length, Pressure, Temperature};

/// Geographic position of the observer on Earth's surface.
#[derive(Debug, Clone, Copy)]
pub struct Observer {
    /// Geographic latitude, positive north.
    pub latitude: Angle,
    /// Geographic longitude, positive **east** of Greenwich (see the
    /// crate-level convention note in `lib.rs`).
    pub longitude: Angle,
    /// Elevation above the reference ellipsoid (approximately, above mean
    /// sea level). Used by [`crate::spa`]'s topocentric parallax
    /// correction; **not** used by [`crate::meeus`], which omits parallax
    /// entirely (see that module's documentation for why).
    pub elevation: Length,
}

/// Local atmospheric conditions used only for the (small) atmospheric
/// refraction correction near the horizon.
#[derive(Debug, Clone, Copy)]
pub struct AtmosphericConditions {
    /// Station-level atmospheric pressure.
    pub pressure: Pressure,
    /// Ambient air temperature.
    pub temperature: Temperature,
}

impl Default for AtmosphericConditions {
    /// ISO 2533:1975 standard atmosphere sea-level pressure and a nominal
    /// 15 °C temperature. Override with site-measured values when
    /// available; the refraction correction this feeds into is at most a
    /// few arcminutes, so the sensitivity to this default is small away
    /// from the horizon.
    fn default() -> Self {
        AtmosphericConditions {
            pressure: Pressure::from_pascal(core_constants::STANDARD_ATMOSPHERE_PA),
            temperature: Temperature::from_celsius(15.0),
        }
    }
}

/// Complete topocentric solar position at a given instant and location.
/// Produced identically by every [`crate::SolarPositionAlgorithm`]
/// implementation (only the internal computation differs).
#[derive(Debug, Clone, Copy)]
pub struct SolarPosition {
    /// Geometric (airless) solar zenith angle: 0° = overhead, 90° =
    /// horizon.
    pub zenith_true: Angle,
    /// Apparent solar zenith angle, including atmospheric refraction.
    pub zenith_apparent: Angle,
    /// Geometric solar elevation angle (`90° - zenith_true`).
    pub elevation_true: Angle,
    /// Apparent solar elevation angle (`90° - zenith_apparent`).
    pub elevation_apparent: Angle,
    /// Solar azimuth, measured from geographic North, clockwise (0°=N,
    /// 90°=E, 180°=S, 270°=W).
    pub azimuth: Angle,
    /// Geocentric declination of the Sun at this instant (topocentric
    /// parallax, where applied, is a sub-0.003° correction not separately
    /// exposed here — see [`crate::spa::SpaDetailedResult`] for the full
    /// topocentric-corrected intermediate values).
    pub declination: Angle,
    /// Geocentric right ascension of the Sun at this instant.
    pub right_ascension: Angle,
    /// Local (geometric, geocentric) hour angle of the Sun, normalized to
    /// `(-180°, 180°]`; negative before local solar noon, positive after.
    pub hour_angle: Angle,
    /// Instantaneous Earth-Sun distance, astronomical units.
    pub earth_sun_distance_au: f64,
    /// Equation of time (apparent minus mean solar time), minutes.
    pub equation_of_time_minutes: f64,
}
