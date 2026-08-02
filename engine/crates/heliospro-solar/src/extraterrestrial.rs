//! Extraterrestrial solar irradiance: the irradiance that would be
//! received in the absence of the atmosphere, used as the normalizing
//! reference for clearness-index and transposition calculations.

use heliospro_units::{Angle, Irradiance};
use std::f64::consts::PI;

/// Extraterrestrial irradiance on a plane normal to the Sun's rays,
/// computed from the instantaneous Earth-Sun distance via the inverse-
/// square law.
///
/// `G0n = SOLAR_CONSTANT_W_M2 / R²`, `R` in astronomical units.
///
/// **Source**: basic radiometric inverse-square law applied to the solar
/// constant (Duffie & Beckman (2013), Eq. 1.4.1a); `R` itself should come
/// from [`crate::ephemeris::Ephemeris::earth_sun_distance_au`] (Meeus
/// (1998) Eq. 25.5), which is more accurate than the classical Spencer
/// (1971) Fourier fit for the same quantity — see
/// [`extraterrestrial_normal_irradiance_spencer1971`] for a cross-check
/// against that independent, widely cited approximation.
pub fn extraterrestrial_normal_irradiance(earth_sun_distance_au: f64) -> Irradiance {
    let g0n = heliospro_constants::SOLAR_CONSTANT_W_M2 / (earth_sun_distance_au * earth_sun_distance_au);
    Irradiance::from_watts_per_m2(g0n)
}

/// Extraterrestrial irradiance on a horizontal plane at the given solar
/// zenith angle: `G0 = G0n * cos(zenith)`, clamped to zero when the Sun is
/// below the horizon.
///
/// **Source**: Duffie & Beckman (2013), Eq. 1.10.1.
pub fn extraterrestrial_horizontal_irradiance(
    normal_irradiance: Irradiance,
    solar_zenith: Angle,
) -> Irradiance {
    Irradiance::from_watts_per_m2(normal_irradiance.watts_per_m2() * solar_zenith.cos()).non_negative()
}

/// Extraterrestrial normal irradiance from a Fourier-series approximation
/// of the Earth's orbit, independent of the Keplerian-ellipse `R`
/// computation used by [`extraterrestrial_normal_irradiance`]. Provided as
/// a literature-standard cross-check (see the `spencer_and_kepler_r_agree`
/// test) and as a convenient day-of-year-only entry point when a full
/// ephemeris is not otherwise being computed.
///
/// **Source**: Spencer, J.W. (1971), "Fourier series representation of the
/// position of the sun", *Search*, 2(5), 172, as reproduced in Duffie &
/// Beckman (2013), Eq. 1.4.1a:
///
/// `G0n = Gsc * (1.000110 + 0.034221 cos(B) + 0.001280 sin(B)
///        + 0.000719 cos(2B) + 0.000077 sin(2B))`
///
/// where `B = 2*pi*(n-1)/365` and `n` is the day of the year (1-366).
pub fn extraterrestrial_normal_irradiance_spencer1971(day_of_year: u32) -> Irradiance {
    let b = 2.0 * PI * (day_of_year as f64 - 1.0) / 365.0;
    let factor = 1.000_110
        + 0.034_221 * b.cos()
        + 0.001_280 * b.sin()
        + 0.000_719 * (2.0 * b).cos()
        + 0.000_077 * (2.0 * b).sin();
    Irradiance::from_watts_per_m2(heliospro_constants::SOLAR_CONSTANT_W_M2 * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_irradiance_equals_solar_constant_at_one_au() {
        let g0n = extraterrestrial_normal_irradiance(1.0);
        assert!((g0n.watts_per_m2() - heliospro_constants::SOLAR_CONSTANT_W_M2).abs() < 1e-9);
    }

    #[test]
    fn normal_irradiance_is_higher_at_perihelion_than_aphelion() {
        let perihelion = extraterrestrial_normal_irradiance(0.9833);
        let aphelion = extraterrestrial_normal_irradiance(1.0167);
        assert!(perihelion.watts_per_m2() > aphelion.watts_per_m2());
        // The annual variation is approximately +/-3.3% about the mean,
        // i.e. about 6.9% peak-to-peak (well-established fact, following
        // directly from the ~3.3% eccentricity-driven distance variation
        // and the inverse-square law: (1.0167/0.9833)^2 - 1 ~= 6.9%).
        let ratio = perihelion.watts_per_m2() / aphelion.watts_per_m2();
        assert!((ratio - 1.069).abs() < 0.002, "ratio = {ratio}");
    }

    #[test]
    fn horizontal_irradiance_is_zero_below_horizon() {
        let g0n = Irradiance::from_watts_per_m2(1361.0);
        let g0 = extraterrestrial_horizontal_irradiance(g0n, Angle::from_degrees(95.0));
        assert_eq!(g0.watts_per_m2(), 0.0);
    }

    #[test]
    fn horizontal_irradiance_at_overhead_sun_equals_normal_irradiance() {
        let g0n = Irradiance::from_watts_per_m2(1361.0);
        let g0 = extraterrestrial_horizontal_irradiance(g0n, Angle::from_degrees(0.0));
        assert!((g0.watts_per_m2() - 1361.0).abs() < 1e-6);
    }

    /// Cross-validation: the Keplerian-ellipse based distance (via
    /// `heliospro-solar::ephemeris`, Meeus 1998 Eq. 25.5) and the
    /// independently-sourced Spencer (1971) Fourier fit are two different
    /// published approximations of the same physical quantity (Earth-Sun
    /// distance through the year). Close agreement between the two is
    /// strong evidence that neither contains a gross transcription error.
    #[test]
    fn spencer_and_kepler_r_agree_across_the_year() {
        use crate::ephemeris;
        use heliospro_math::julian::CalendarDateTime;

        let mut max_relative_diff = 0.0_f64;
        for day in 1..=365u32 {
            let dt = CalendarDateTime {
                year: 2024,
                month: 1,
                day: 1,
                hour: 12,
                minute: 0,
                second: 0.0,
            };
            let jde = dt.to_julian_day().unwrap() + (day - 1) as f64;
            let eph = ephemeris::compute(jde, jde);
            let g0n_kepler = extraterrestrial_normal_irradiance(eph.earth_sun_distance_au);
            let g0n_spencer = extraterrestrial_normal_irradiance_spencer1971(day);
            let diff = (g0n_kepler.watts_per_m2() - g0n_spencer.watts_per_m2()).abs()
                / g0n_spencer.watts_per_m2();
            max_relative_diff = max_relative_diff.max(diff);
        }
        assert!(
            max_relative_diff < 0.001,
            "max relative difference between Kepler-R and Spencer(1971) G0n = {:.4}%",
            max_relative_diff * 100.0
        );
    }
}
