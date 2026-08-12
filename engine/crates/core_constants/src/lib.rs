//! # core_constants
//!
//! Referenced physical, astronomical, and geodetic constants used across the
//! HeliosPro scientific engine.
//!
//! ## Design rule
//!
//! Every constant in this crate carries a doc comment citing its primary
//! source (a peer-reviewed publication, an international standard, or a
//! metrology body publication such as CODATA/BIPM). No constant may be added
//! without a citation (see `docs/design/06-algorithmes.md` and
//! `docs/design/07-modeles-physiques.md` in the repository for the modelling
//! context in which each constant is used).
//!
//! All values are expressed in SI base or coherent derived units unless
//! explicitly noted otherwise in the doc comment.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Solar constant: extraterrestrial solar irradiance at the mean Sun-Earth
/// distance of 1 astronomical unit, in W/m².
///
/// **Source**: Duffie, J.A. & Beckman, W.A. (2013). *Solar Engineering of
/// Thermal Processes*, 4th ed., John Wiley & Sons, Section 1.1 (World
/// Radiation Center reference value).
///
/// **Note**: satellite radiometry gives a revised Total Solar Irradiance of
/// 1360.8 ± 0.5 W/m² (Kopp, G. & Lean, J.L. (2011), "A new, lower value of
/// total solar irradiance", *Geophysical Research Letters*, 38, L01706).
/// HeliosPro uses the WRC value of 1367 W/m² because it is the reference
/// value against which the Spencer (1971) Fourier series implemented in
/// `irradiance::extraterrestrial` was originally fitted; mixing the two
/// reference frames would introduce a spurious ~0.5 % bias into the
/// extraterrestrial irradiance model. Revisiting this constant (and
/// re-fitting the Spencer series) is tracked as a controlled engine-wide
/// change, not a local edit.
pub const SOLAR_CONSTANT_W_M2: f64 = 1367.0;

/// Standard gravitational acceleration, m/s².
///
/// **Source**: ISO 80000-3:2019, *Quantities and units — Part 3: Space and
/// time* (defines the standard value `g_n = 9.806 65 m/s²`).
pub const STANDARD_GRAVITY_M_S2: f64 = 9.806_65;

/// Standard atmospheric pressure at sea level, Pa.
///
/// **Source**: ISO 2533:1975, *Standard Atmosphere*.
pub const STANDARD_ATMOSPHERE_PA: f64 = 101_325.0;

/// Stefan-Boltzmann constant, W/(m²·K⁴).
///
/// **Source**: CODATA 2018 recommended values of the fundamental physical
/// constants (exact value under the 2019 SI redefinition).
pub const STEFAN_BOLTZMANN_W_M2_K4: f64 = 5.670_374_419e-8;

/// Boltzmann constant, J/K.
///
/// **Exact by definition** since the 2019 SI redefinition: the kelvin is
/// defined by fixing this value.
///
/// **Source**: BIPM, *The International System of Units (SI)*, 9th edition
/// (2019), Section 2.3.1; CODATA 2018.
pub const BOLTZMANN_J_PER_K: f64 = 1.380_649e-23;

/// Elementary charge, C.
///
/// **Exact by definition** since the 2019 SI redefinition: the ampere is
/// defined by fixing this value.
///
/// **Source**: BIPM, *The International System of Units (SI)*, 9th edition
/// (2019), Section 2.3.1; CODATA 2018.
pub const ELEMENTARY_CHARGE_C: f64 = 1.602_176_634e-19;

/// Boltzmann constant expressed in eV/K, equivalently the thermal voltage
/// per kelvin in V/K.
///
/// **Derived, never entered separately.** Since one electronvolt is by
/// definition [`ELEMENTARY_CHARGE_C`] joules, `k` in eV/K is numerically
/// `k/q`, which is also the thermal voltage `kT/q` per kelvin in V/K. Two
/// independently typed constants would eventually diverge; deriving this
/// one makes that impossible.
///
/// Required by the single-diode model, where the band-gap term
/// `(1/k)(E_g,ref/T_ref − E_g/T_c)` is dimensionless only if `k` and
/// `E_g` share their energy unit.
pub const BOLTZMANN_EV_PER_K: f64 = BOLTZMANN_J_PER_K / ELEMENTARY_CHARGE_C;

/// Offset between the Celsius and Kelvin temperature scales (0 °C in
/// Kelvin).
///
/// **Source**: Bureau International des Poids et Mesures (BIPM), *The
/// International System of Units (SI)*, 9th edition (2019), Section 2.3.1.
pub const CELSIUS_ZERO_KELVIN: f64 = 273.15;

/// Number of seconds in a mean solar day (exact, by definition of the
/// civil day as 24 × 60 × 60 SI seconds).
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// Number of days in a Julian century, used to convert Julian Day intervals
/// into the `T` time argument of astronomical series.
///
/// **Source**: Meeus, J. (1998). *Astronomical Algorithms*, 2nd ed.,
/// Willmann-Bell, Chapter 7 (definition of the Julian century).
pub const DAYS_PER_JULIAN_CENTURY: f64 = 36_525.0;

/// Julian Day Number of the standard epoch J2000.0 (2000-01-01 12:00:00 TT).
///
/// **Source**: Meeus, J. (1998). *Astronomical Algorithms*, 2nd ed.,
/// Willmann-Bell, Chapter 7.
pub const J2000_EPOCH_JD: f64 = 2_451_545.0;

/// WGS84 reference ellipsoid flattening, dimensionless.
///
/// **Source**: National Imagery and Mapping Agency (NIMA), *Department of
/// Defense World Geodetic System 1984*, Technical Report TR8350.2, 3rd ed.
/// (2000), Table 3.1.
pub const WGS84_FLATTENING: f64 = 1.0 / 298.257_223_563;

/// WGS84 reference ellipsoid semi-major axis, meters.
///
/// **Source**: NIMA TR8350.2, 3rd ed. (2000), Table 3.1.
pub const WGS84_SEMI_MAJOR_AXIS_M: f64 = 6_378_137.0;

/// Astronomical Unit, meters (exact, by IAU definition).
///
/// **Source**: International Astronomical Union, Resolution B2 (2012), *Re-
/// definition of the astronomical unit of length*.
pub const ASTRONOMICAL_UNIT_M: f64 = 149_597_870_700.0;

/// Sun's mean equatorial horizontal parallax at a distance of 1 AU,
/// arcseconds. Used as `pi = SOLAR_PARALLAX_ARCSEC_AT_1AU / R(AU)` to obtain
/// the instantaneous parallax for topocentric position corrections.
///
/// **Source**: Meeus, J. (1998). *Astronomical Algorithms*, 2nd ed.,
/// Willmann-Bell, Chapter 40, Equation 40.6.
pub const SOLAR_PARALLAX_ARCSEC_AT_1AU: f64 = 8.794;

/// Mean angular radius of the Sun's disk as seen from 1 AU, degrees. Not
/// used in position calculations (the Sun is treated as a point source for
/// zenith/azimuth purposes) but retained for future irradiance-disk / eclipse
/// related modelling.
///
/// **Source**: Meeus, J. (1998). *Astronomical Algorithms*, 2nd ed.,
/// Willmann-Bell, Chapter 55 (mean value of the solar semi-diameter,
/// 15.99942 arcmin at 1 AU, expressed here in degrees).
pub const SOLAR_MEAN_ANGULAR_RADIUS_DEG: f64 = 15.999_42 / 60.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_zero_matches_bipm_definition() {
        assert!((CELSIUS_ZERO_KELVIN - 273.15).abs() < 1e-12);
    }

    #[test]
    fn wgs84_flattening_is_positive_and_small() {
        // Earth is nearly spherical: flattening must be a small positive
        // number (Earth's polar radius is ~0.335 % shorter than equatorial).
        // Evaluated in a `const` block since both operands are compile-time
        // constants (clippy::assertions_on_constants).
        const { assert!(WGS84_FLATTENING > 0.0) };
        const { assert!(WGS84_FLATTENING < 0.01) };
    }

    #[test]
    fn solar_constant_is_within_accepted_physical_range() {
        // Cross-check against the independently measured TSI (Kopp & Lean
        // 2011) to catch gross transcription errors: the two reference
        // values must agree within their combined uncertainty budget
        // (~0.5 %), even though HeliosPro deliberately uses the WRC value.
        let tsi_kopp_lean_2011 = 1360.8;
        let relative_diff = (SOLAR_CONSTANT_W_M2 - tsi_kopp_lean_2011).abs() / tsi_kopp_lean_2011;
        assert!(relative_diff < 0.005, "relative_diff = {relative_diff}");
    }

    #[test]
    fn astronomical_unit_matches_iau_2012_definition() {
        assert_eq!(ASTRONOMICAL_UNIT_M, 149_597_870_700.0);
    }

    #[test]
    fn the_electronvolt_boltzmann_constant_is_derived_not_transcribed() {
        // The point of deriving it: the identity must hold exactly, which
        // it cannot if two constants are entered independently.
        assert_eq!(BOLTZMANN_EV_PER_K, BOLTZMANN_J_PER_K / ELEMENTARY_CHARGE_C);
        // Cross-check against the value quoted in the literature, to catch
        // a transcription error in either of the two exact constants.
        let quoted = 8.617_333_262e-5;
        let relative = (BOLTZMANN_EV_PER_K - quoted).abs() / quoted;
        assert!(relative < 1e-9, "relative = {relative}");
    }

    #[test]
    fn the_thermal_voltage_at_stc_matches_the_textbook_value() {
        // kT/q at 25 degC = 298.15 K is quoted as ~25.7 mV throughout the
        // photovoltaics literature. An independent check on both constants.
        let v_t = BOLTZMANN_EV_PER_K * (25.0 + CELSIUS_ZERO_KELVIN);
        assert!((v_t - 0.025_693).abs() < 1e-6, "v_t = {v_t}");
    }
}
