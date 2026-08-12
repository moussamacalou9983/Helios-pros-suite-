//! HDKR (Hay-Davies-Klucher-Reindl) anisotropic sky-diffuse model.
//!
//! **Source**: Reindl, D.T., Beckman, W.A., Duffie, J.A. (1990),
//! "Evaluation of hourly tilted surface radiation models", *Solar Energy*,
//! 45(1), 9-17; as reproduced in Duffie & Beckman (2013), Eq. 2.16.3.
//!
//! Extends the Hay-Davies model ([`crate::hay_davies`]) with a horizon-
//! brightening correction term (originally introduced by Klucher (1979)),
//! modulated by a cloudiness index `f = sqrt(Ib / I)` (`Ib` = beam
//! irradiance on the horizontal, `I` = GHI): `f` is close to 1 under clear
//! skies (enhancing horizon brightening) and close to 0 under overcast
//! skies (where horizon brightening is not observed).

use core_units::Irradiance;

use crate::hay_davies::{anisotropy_index, beam_ratio};
use crate::inputs::{beam_component, ground_reflected_component};
use crate::{PlaneOfArrayIrradiance, TranspositionInputs};

/// The horizon-brightening modulating factor `f = sqrt(DNI*cos(zenith) /
/// GHI)`, clamped to `[0, 1]`. Returns 0 when GHI is zero (no radiation to
/// speak of).
pub(crate) fn horizon_brightening_factor(inputs: &TranspositionInputs) -> f64 {
    if inputs.ghi.watts_per_m2() <= 0.0 {
        return 0.0;
    }
    let beam_horizontal = inputs.dni.watts_per_m2() * inputs.solar_zenith.cos().max(0.0);
    (beam_horizontal / inputs.ghi.watts_per_m2())
        .clamp(0.0, 1.0)
        .sqrt()
}

/// Compute plane-of-array irradiance using the HDKR model.
///
/// `sky_diffuse = DHI * [Ai*Rb + (1-Ai) * (1+cos(tilt))/2 * (1 + f*sin³(tilt/2))]`
pub fn plane_of_array_irradiance(inputs: &TranspositionInputs) -> PlaneOfArrayIrradiance {
    let beam = beam_component(inputs);
    let ai = anisotropy_index(inputs.dni, inputs.extraterrestrial_normal);
    let rb = beam_ratio(inputs);
    let f = horizon_brightening_factor(inputs);
    let tilt = inputs.surface.tilt;

    let circumsolar = ai * rb;
    let half_tilt_sin = (tilt.radians() / 2.0).sin();
    let horizon_brightening = 1.0 + f * half_tilt_sin.powi(3);
    let isotropic = (1.0 - ai) * (1.0 + tilt.cos()) / 2.0 * horizon_brightening;

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
    fn tilt_zero_recovers_ghi_hdkr() {
        let zenith_deg: f64 = 35.0;
        let dni = 700.0;
        let dhi = 160.0;
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
    fn horizon_brightening_factor_is_bounded() {
        let inputs = make_inputs(900.0, 850.0, 50.0, 20.0, 30.0);
        let f = horizon_brightening_factor(&inputs);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn hdkr_diffuse_is_at_least_hay_davies_diffuse_under_clear_sky() {
        // The horizon-brightening term is additive and non-negative, so
        // for the same inputs HDKR must predict at least as much sky
        // diffuse as plain Hay-Davies.
        let zenith_deg: f64 = 30.0;
        let dni = 850.0;
        let dhi = 100.0;
        let ghi = dni * zenith_deg.to_radians().cos() + dhi;
        let inputs = make_inputs(ghi, dni, dhi, zenith_deg, 40.0);

        let hdkr_result = plane_of_array_irradiance(&inputs);
        let hd_result = crate::hay_davies::plane_of_array_irradiance(&inputs);
        assert!(
            hdkr_result.sky_diffuse.watts_per_m2() >= hd_result.sky_diffuse.watts_per_m2() - 1e-9
        );
    }
}
