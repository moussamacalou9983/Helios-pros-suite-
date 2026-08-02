//! Hay-Davies anisotropic sky-diffuse model.
//!
//! **Source**: Hay, J.E. & Davies, J.A. (1980), "Calculation of the solar
//! radiation incident on an inclined surface", *Proc. First Canadian Solar
//! Radiation Data Workshop*, Toronto; as reproduced in Duffie & Beckman
//! (2013), *Solar Engineering of Thermal Processes*, 4th ed., Eq. 2.16.1.
//!
//! Splits the diffuse sky radiation into an isotropic part and a
//! circumsolar part (radiation scattered in the vicinity of the solar
//! disk, treated as if it came directly from the Sun's direction), weighted
//! by the anisotropy index `Ai = DNI / G0n`.

use core_units::Irradiance;

use crate::inputs::{beam_component, ground_reflected_component};
use crate::{PlaneOfArrayIrradiance, TranspositionInputs};

/// Cosine of the zenith angle below which the beam ratio `Rb =
/// cos(theta)/cos(zenith)` is evaluated using this floor instead of the
/// true `cos(zenith)`, to avoid numerical blow-up as the Sun approaches
/// the horizon (`zenith -> 90°`). This mirrors the `b = max(cos 85°,
/// cos(zenith))` safeguard used explicitly in the Perez (1990) model (see
/// `crate::perez`); it is a HeliosPro numerical-robustness choice, not part
/// of Hay & Davies's (1980) original published formula, which did not
/// need to address this corner case for the hourly-averaged, non-negative
/// beam data it was validated against.
const MIN_COS_ZENITH: f64 = 0.087; // cos(85 deg)

/// The Hay-Davies anisotropy index `Ai = DNI / G0n`, clamped to `[0, 1]`
/// (physically, `Ai` represents the fraction of extraterrestrial beam
/// irradiance actually reaching the surface as DNI, which cannot exceed
/// 1 for a well-formed input).
pub(crate) fn anisotropy_index(dni: Irradiance, extraterrestrial_normal: Irradiance) -> f64 {
    if extraterrestrial_normal.watts_per_m2() <= 0.0 {
        return 0.0;
    }
    (dni.watts_per_m2() / extraterrestrial_normal.watts_per_m2()).clamp(0.0, 1.0)
}

/// The beam ratio `Rb = cos(angle_of_incidence) / cos(zenith)`, used to
/// project the circumsolar diffuse component from the horizontal to the
/// tilted plane in the same way as the direct beam.
pub(crate) fn beam_ratio(inputs: &TranspositionInputs) -> f64 {
    let cos_theta = inputs.angle_of_incidence.cos().max(0.0);
    let cos_zenith = inputs.solar_zenith.cos().max(MIN_COS_ZENITH);
    cos_theta / cos_zenith
}

/// Compute plane-of-array irradiance using the Hay-Davies anisotropic sky
/// model.
///
/// `sky_diffuse = DHI * [Ai * Rb + (1 - Ai) * (1 + cos(tilt)) / 2]`
pub fn plane_of_array_irradiance(inputs: &TranspositionInputs) -> PlaneOfArrayIrradiance {
    let beam = beam_component(inputs);
    let ai = anisotropy_index(inputs.dni, inputs.extraterrestrial_normal);
    let rb = beam_ratio(inputs);
    let tilt = inputs.surface.tilt;

    let circumsolar = ai * rb;
    let isotropic = (1.0 - ai) * (1.0 + tilt.cos()) / 2.0;
    let sky_diffuse =
        Irradiance::from_watts_per_m2(inputs.dhi.watts_per_m2() * (circumsolar + isotropic));

    let ground_reflected = ground_reflected_component(inputs);
    PlaneOfArrayIrradiance::from_components(beam, sky_diffuse, ground_reflected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::{Angle, Ratio};
    use solar_geometry::incidence::{SurfaceOrientation, angle_of_incidence};

    fn make_inputs(
        ghi: f64,
        dni: f64,
        dhi: f64,
        zenith_deg: f64,
        tilt_deg: f64,
    ) -> TranspositionInputs {
        let solar_zenith = Angle::from_degrees(zenith_deg);
        let solar_azimuth = Angle::from_degrees(180.0);
        let surface = SurfaceOrientation {
            tilt: Angle::from_degrees(tilt_deg),
            azimuth: Angle::from_degrees(180.0),
        };
        TranspositionInputs {
            dni: Irradiance::from_watts_per_m2(dni),
            dhi: Irradiance::from_watts_per_m2(dhi),
            ghi: Irradiance::from_watts_per_m2(ghi),
            extraterrestrial_normal: Irradiance::from_watts_per_m2(1361.0),
            solar_zenith,
            angle_of_incidence: angle_of_incidence(solar_zenith, solar_azimuth, surface),
            surface,
            ground_albedo: Ratio::new(0.2),
        }
    }

    #[test]
    fn tilt_zero_recovers_ghi_hay_davies() {
        let zenith_deg: f64 = 25.0;
        let dni = 750.0;
        let dhi = 130.0;
        let ghi = dni * zenith_deg.to_radians().cos() + dhi;
        let inputs = make_inputs(ghi, dni, dhi, zenith_deg, 0.0);
        let poa = plane_of_array_irradiance(&inputs);
        assert!(
            (poa.global.watts_per_m2() - ghi).abs() < 1e-6,
            "poa.global = {}, ghi = {ghi}",
            poa.global.watts_per_m2()
        );
    }

    #[test]
    fn anisotropy_index_is_zero_for_fully_diffuse_sky() {
        assert_eq!(
            anisotropy_index(Irradiance::ZERO, Irradiance::from_watts_per_m2(1361.0)),
            0.0
        );
    }

    #[test]
    fn anisotropy_index_is_clamped_to_one() {
        // A (physically implausible) DNI exceeding G0n must still clamp.
        let ai = anisotropy_index(
            Irradiance::from_watts_per_m2(1500.0),
            Irradiance::from_watts_per_m2(1361.0),
        );
        assert_eq!(ai, 1.0);
    }

    #[test]
    fn clear_sky_hay_davies_exceeds_isotropic_diffuse_on_a_south_facing_tilt() {
        // Under a clear sky (high Ai), the circumsolar term concentrates
        // diffuse radiation towards the Sun's direction; for a surface
        // facing the Sun, Hay-Davies should predict at least as much sky
        // diffuse as the plain isotropic model.
        let zenith_deg: f64 = 30.0;
        let dni = 850.0;
        let dhi = 100.0;
        let ghi = dni * zenith_deg.to_radians().cos() + dhi;
        let inputs = make_inputs(ghi, dni, dhi, zenith_deg, 30.0);

        let hd = plane_of_array_irradiance(&inputs);
        let iso = crate::isotropic::plane_of_array_irradiance(&inputs);
        assert!(hd.sky_diffuse.watts_per_m2() >= iso.sky_diffuse.watts_per_m2() - 1e-9);
    }
}
