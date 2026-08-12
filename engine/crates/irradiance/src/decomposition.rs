//! Relationships between Global Horizontal Irradiance (GHI), Direct Normal
//! Irradiance (DNI), and Diffuse Horizontal Irradiance (DHI): the exact
//! geometric closure relation, and the Erbs et al. (1982) correlation used
//! to estimate DNI/DHI when only GHI is measured (a common situation for
//! ground weather stations and satellite-derived irradiance products).

use core_units::{Angle, Irradiance};

/// The exact geometric relationship between the three irradiance
/// components on a horizontal surface: `GHI = DNI * cos(zenith) + DHI`.
///
/// **Source**: definitional identity relating beam and diffuse components
/// on a horizontal plane; see Duffie & Beckman (2013), Section 2.15
/// ("distinction between beam and diffuse radiation").
pub fn ghi_from_components(dni: Irradiance, dhi: Irradiance, solar_zenith: Angle) -> Irradiance {
    Irradiance::from_watts_per_m2(dni.watts_per_m2() * solar_zenith.cos() + dhi.watts_per_m2())
        .non_negative()
}

/// Residual of the GHI closure equation: `computed_ghi - (dni*cos(zenith) +
/// dhi)`, in W/m². A well-formed, mutually consistent (measured or
/// modelled) irradiance triplet should have a residual close to the
/// combined measurement uncertainty of the instruments (typically a few
/// W/m² to a few percent of GHI for calibrated pyranometers/pyrheliometers,
/// per ISO 9060:2018 classification).
pub fn closure_residual(
    ghi: Irradiance,
    dni: Irradiance,
    dhi: Irradiance,
    solar_zenith: Angle,
) -> f64 {
    ghi.watts_per_m2() - ghi_from_components(dni, dhi, solar_zenith).watts_per_m2()
}

/// Result of decomposing GHI into its DNI and DHI components.
#[derive(Debug, Clone, Copy)]
pub struct DecomposedIrradiance {
    /// Estimated Direct Normal Irradiance.
    pub dni: Irradiance,
    /// Estimated Diffuse Horizontal Irradiance.
    pub dhi: Irradiance,
    /// The diffuse fraction `kd = DHI / GHI` used to derive the split.
    pub diffuse_fraction: f64,
    /// The clearness index `kt = GHI / G0` used as the correlation input.
    pub clearness_index: f64,
}

/// Estimate DNI and DHI from GHI using the Erbs et al. (1982) correlation
/// between the clearness index `kt = GHI / G0` (`G0` = extraterrestrial
/// horizontal irradiance) and the diffuse fraction `kd = DHI / GHI`.
///
/// **Source**: Erbs, D.G., Klein, S.A., Duffie, J.A. (1982), "Estimation of
/// the diffuse radiation fraction for hourly, daily and monthly-average
/// global radiation", *Solar Energy*, 28(4), 293-302; as reproduced in
/// Duffie & Beckman (2013), Eq. 2.10.1:
///
/// - `kt <= 0.22`: `kd = 1 - 0.09*kt`
/// - `0.22 < kt <= 0.80`: `kd = 0.9511 - 0.1604*kt + 4.388*kt² - 16.638*kt³ + 12.336*kt⁴`
/// - `kt > 0.80`: `kd = 0.165`
///
/// Returns `dni = 0`, `dhi = ghi` (fully diffuse) when the Sun is at or
/// below the horizon or `g0_horizontal` is zero, since the clearness index
/// is undefined there.
pub fn erbs_decomposition(
    ghi: Irradiance,
    g0_horizontal: Irradiance,
    solar_zenith: Angle,
) -> DecomposedIrradiance {
    let cos_z = solar_zenith.cos();
    if g0_horizontal.watts_per_m2() <= 0.0 || cos_z <= 0.0 || ghi.watts_per_m2() <= 0.0 {
        return DecomposedIrradiance {
            dni: Irradiance::ZERO,
            dhi: ghi.non_negative(),
            diffuse_fraction: 1.0,
            clearness_index: 0.0,
        };
    }

    let kt = (ghi.watts_per_m2() / g0_horizontal.watts_per_m2()).clamp(0.0, 1.0);

    let kd = if kt <= 0.22 {
        1.0 - 0.09 * kt
    } else if kt <= 0.80 {
        0.9511 - 0.1604 * kt + 4.388 * kt.powi(2) - 16.638 * kt.powi(3) + 12.336 * kt.powi(4)
    } else {
        0.165
    }
    .clamp(0.0, 1.0);

    let dhi = Irradiance::from_watts_per_m2(kd * ghi.watts_per_m2());
    let beam_horizontal = (ghi.watts_per_m2() - dhi.watts_per_m2()).max(0.0);
    let dni = Irradiance::from_watts_per_m2(beam_horizontal / cos_z);

    DecomposedIrradiance {
        dni,
        dhi,
        diffuse_fraction: kd,
        clearness_index: kt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_relation_is_exact_by_construction() {
        let dni = Irradiance::from_watts_per_m2(800.0);
        let dhi = Irradiance::from_watts_per_m2(120.0);
        let zenith = Angle::from_degrees(30.0);
        let ghi = ghi_from_components(dni, dhi, zenith);
        let residual = closure_residual(ghi, dni, dhi, zenith);
        assert!(residual.abs() < 1e-9);
    }

    #[test]
    fn erbs_overcast_sky_is_fully_diffuse() {
        // Very low clearness index (kt near 0): the sky is overcast and
        // essentially all radiation should be diffuse (kd near 1).
        let ghi = Irradiance::from_watts_per_m2(50.0);
        let g0 = Irradiance::from_watts_per_m2(1000.0);
        let zenith = Angle::from_degrees(30.0);
        let result = erbs_decomposition(ghi, g0, zenith);
        assert!(
            result.diffuse_fraction > 0.95,
            "kd = {}",
            result.diffuse_fraction
        );
    }

    #[test]
    fn erbs_clear_sky_has_low_diffuse_fraction() {
        // High clearness index (kt > 0.8): clear sky, diffuse fraction
        // should be at its minimum plateau value of 0.165.
        let g0 = Irradiance::from_watts_per_m2(1000.0);
        let ghi = Irradiance::from_watts_per_m2(900.0); // kt = 0.9
        let zenith = Angle::from_degrees(20.0);
        let result = erbs_decomposition(ghi, g0, zenith);
        assert!((result.diffuse_fraction - 0.165).abs() < 1e-9);
    }

    #[test]
    fn erbs_decomposition_satisfies_closure_relation() {
        // The DNI/DHI pair returned by the decomposition must, by
        // construction, reconstruct the original GHI via the exact
        // closure relation.
        let g0 = Irradiance::from_watts_per_m2(1100.0);
        let zenith = Angle::from_degrees(25.0);
        for ghi_value in [50.0, 200.0, 400.0, 600.0, 850.0] {
            let ghi = Irradiance::from_watts_per_m2(ghi_value);
            let result = erbs_decomposition(ghi, g0, zenith);
            let residual = closure_residual(ghi, result.dni, result.dhi, zenith);
            assert!(
                residual.abs() < 1e-6,
                "ghi={ghi_value}, residual={residual}"
            );
        }
    }

    #[test]
    fn erbs_below_horizon_returns_fully_diffuse_zero_beam() {
        let ghi = Irradiance::from_watts_per_m2(0.0);
        let g0 = Irradiance::from_watts_per_m2(0.0);
        let zenith = Angle::from_degrees(95.0);
        let result = erbs_decomposition(ghi, g0, zenith);
        assert_eq!(result.dni.watts_per_m2(), 0.0);
    }
}
