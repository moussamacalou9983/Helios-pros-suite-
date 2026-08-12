//! Isotropic sky-diffuse model.
//!
//! **Source**: Liu, B.Y. & Jordan, R.C. (1963), "The interrelationship and
//! characteristic distribution of direct, diffuse, and total solar
//! radiation", *Solar Energy*, 4(3), 1-19.
//!
//! Treats the diffuse sky radiation as uniformly distributed over the sky
//! dome (no circumsolar or horizon brightening). It is the simplest of the
//! four models and, being a physical lower bound in most practical
//! configurations, is retained both as a first-order estimate and as a
//! cross-check baseline for the more elaborate anisotropic models.

use core_units::Irradiance;

use crate::inputs::{beam_component, ground_reflected_component};
use crate::{PlaneOfArrayIrradiance, TranspositionInputs};

/// Compute plane-of-array irradiance using the isotropic sky model.
///
/// `sky_diffuse = DHI * (1 + cos(tilt)) / 2`
pub fn plane_of_array_irradiance(inputs: &TranspositionInputs) -> PlaneOfArrayIrradiance {
    let beam = beam_component(inputs);
    let tilt = inputs.surface.tilt;
    let sky_diffuse =
        Irradiance::from_watts_per_m2(inputs.dhi.watts_per_m2() * (1.0 + tilt.cos()) / 2.0);
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
    fn tilt_zero_recovers_ghi_isotropic() {
        // Self-consistent closure: GHI = DNI*cos(Z) + DHI.
        let zenith_deg: f64 = 30.0;
        let dni = 800.0;
        let dhi = 150.0;
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
    fn ground_reflected_is_zero_for_horizontal_surface() {
        let inputs = make_inputs(900.0, 700.0, 200.0, 20.0, 0.0);
        let poa = plane_of_array_irradiance(&inputs);
        assert_eq!(poa.ground_reflected.watts_per_m2(), 0.0);
    }

    #[test]
    fn vertical_surface_receives_half_the_diffuse_sky_and_half_the_ground_reflection() {
        // At tilt = 90 deg, (1+cos90)/2 = 0.5 and (1-cos90)/2 = 0.5.
        let ghi = 900.0;
        let dhi = 200.0;
        let albedo = 0.2;
        let inputs = make_inputs(ghi, 0.0, dhi, 90.0, 90.0);
        let poa = plane_of_array_irradiance(&inputs);
        assert!((poa.sky_diffuse.watts_per_m2() - dhi * 0.5).abs() < 1e-6);
        assert!((poa.ground_reflected.watts_per_m2() - ghi * albedo * 0.5).abs() < 1e-6);
    }

    #[test]
    fn beam_is_zero_when_sun_is_behind_the_surface() {
        let inputs = make_inputs(50.0, 0.0, 50.0, 85.0, 30.0);
        // Sun azimuth 180 (south), surface azimuth 180 too but Sun placed
        // behind via a large zenith and DNI=0 for simplicity; explicitly
        // verify the clamp logic using a case with nonzero DNI but a
        // surface facing away from the Sun.
        let mut behind = inputs;
        behind.dni = Irradiance::from_watts_per_m2(800.0);
        behind.surface = SurfaceOrientation {
            tilt: Angle::from_degrees(90.0),
            azimuth: Angle::from_degrees(0.0), // facing North
        };
        behind.angle_of_incidence = angle_of_incidence(
            behind.solar_zenith,
            Angle::from_degrees(180.0), // Sun in the South
            behind.surface,
        );
        let poa = plane_of_array_irradiance(&behind);
        assert_eq!(poa.beam.watts_per_m2(), 0.0);
    }
}
