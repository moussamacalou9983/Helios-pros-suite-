//! Perez anisotropic sky-diffuse model.
//!
//! **Source**: Perez, R., Ineichen, P., Seals, R., Michalsky, J., Stewart,
//! R. (1990), "Modeling daylight availability and irradiance components
//! from direct and global irradiance", *Solar Energy*, 44(5), 271-289. The
//! empirical coefficient table (`PEREZ_COEFFICIENTS`) is Table 6 of that
//! paper, reproduced identically in Duffie & Beckman (2013), *Solar
//! Engineering of Thermal Processes*, 4th ed., Table 2.16.1.
//!
//! The sky is modelled as three components: an isotropic background, a
//! circumsolar brightening (radiation concentrated near the solar disk),
//! and a horizon brightening band, whose relative weights `F1`
//! (circumsolar) and `F2` (horizon) are empirical functions of the sky's
//! clearness `epsilon` and brightness `Delta`, evaluated by binning
//! `epsilon` into eight ranges and looking up that bin's coefficients.
//!
//! Relative air mass uses the Kasten & Young (1989) formula: Kasten, F. &
//! Young, A.T. (1989), "Revised optical air mass tables and approximation
//! formula", *Applied Optics*, 28(22), 4735-4738.

use core_units::{Angle, Irradiance};

use crate::inputs::{beam_component, ground_reflected_component};
use crate::{PlaneOfArrayIrradiance, TranspositionInputs};

/// One row of the Perez (1990) empirical coefficient table: the upper
/// bound of the `epsilon` (clearness) bin it applies to, and the six
/// `f11, f12, f13, f21, f22, f23` coefficients.
struct PerezBin {
    epsilon_upper_bound: f64,
    f11: f64,
    f12: f64,
    f13: f64,
    f21: f64,
    f22: f64,
    f23: f64,
}

/// Perez (1990) Table 6 / Duffie & Beckman (2013) Table 2.16.1.
const PEREZ_COEFFICIENTS: [PerezBin; 8] = [
    PerezBin {
        epsilon_upper_bound: 1.065,
        f11: -0.008,
        f12: 0.588,
        f13: -0.062,
        f21: -0.060,
        f22: 0.072,
        f23: -0.022,
    },
    PerezBin {
        epsilon_upper_bound: 1.230,
        f11: 0.130,
        f12: 0.683,
        f13: -0.151,
        f21: -0.019,
        f22: 0.066,
        f23: -0.029,
    },
    PerezBin {
        epsilon_upper_bound: 1.500,
        f11: 0.330,
        f12: 0.487,
        f13: -0.221,
        f21: 0.055,
        f22: -0.064,
        f23: -0.026,
    },
    PerezBin {
        epsilon_upper_bound: 1.950,
        f11: 0.568,
        f12: 0.187,
        f13: -0.295,
        f21: 0.109,
        f22: -0.152,
        f23: -0.014,
    },
    PerezBin {
        epsilon_upper_bound: 2.800,
        f11: 0.873,
        f12: -0.392,
        f13: -0.362,
        f21: 0.226,
        f22: -0.462,
        f23: 0.001,
    },
    PerezBin {
        epsilon_upper_bound: 4.500,
        f11: 1.132,
        f12: -1.237,
        f13: -0.412,
        f21: 0.288,
        f22: -0.823,
        f23: 0.056,
    },
    PerezBin {
        epsilon_upper_bound: 6.200,
        f11: 1.060,
        f12: -1.600,
        f13: -0.359,
        f21: 0.264,
        f22: -1.127,
        f23: 0.131,
    },
    PerezBin {
        epsilon_upper_bound: f64::INFINITY,
        f11: 0.678,
        f12: -0.327,
        f13: -0.250,
        f21: 0.156,
        f22: -1.377,
        f23: 0.251,
    },
];

/// Empirical constant in the clearness parameter `epsilon` formula
/// (Perez et al. 1990, Eq. 1).
const KAPPA: f64 = 1.041;

/// Relative (unitless) optical air mass via the Kasten & Young (1989)
/// formula, valid across the full range of zenith angles from 0 to 90°
/// without the singularity `1/cos(Z)` develops near the horizon.
///
/// `m = 1 / (cos(Z) + 0.50572 * (96.07995 - Z_deg)^-1.6364)`
pub(crate) fn relative_air_mass(solar_zenith: Angle) -> f64 {
    let z_deg = solar_zenith.degrees().min(90.0);
    1.0 / (solar_zenith.cos().max(0.0) + 0.50572 * (96.07995 - z_deg).powf(-1.6364))
}

/// Perez sky clearness parameter `epsilon` (Perez et al. 1990, Eq. 1).
/// Larger values indicate a clearer sky. Returns the upper-bin sentinel
/// value when DHI is (near) zero, since `epsilon` is undefined there and a
/// vanishing diffuse component is, physically, associated with the
/// clearest-sky regime the model distinguishes.
pub(crate) fn clearness_epsilon(dni: Irradiance, dhi: Irradiance, solar_zenith: Angle) -> f64 {
    if dhi.watts_per_m2() < 1e-6 {
        return f64::INFINITY;
    }
    let z_rad = solar_zenith.radians();
    let numerator =
        (dhi.watts_per_m2() + dni.watts_per_m2()) / dhi.watts_per_m2() + KAPPA * z_rad.powi(3);
    numerator / (1.0 + KAPPA * z_rad.powi(3))
}

/// Perez sky brightness parameter `Delta = DHI * air_mass / G0n` (Perez et
/// al. 1990, Eq. 2).
pub(crate) fn brightness_delta(
    dhi: Irradiance,
    air_mass: f64,
    extraterrestrial_normal: Irradiance,
) -> f64 {
    if extraterrestrial_normal.watts_per_m2() <= 0.0 {
        return 0.0;
    }
    dhi.watts_per_m2() * air_mass / extraterrestrial_normal.watts_per_m2()
}

fn bin_for_epsilon(epsilon: f64) -> &'static PerezBin {
    PEREZ_COEFFICIENTS
        .iter()
        .find(|bin| epsilon < bin.epsilon_upper_bound)
        .unwrap_or(&PEREZ_COEFFICIENTS[7])
}

/// Compute plane-of-array irradiance using the Perez (1990) model.
pub fn plane_of_array_irradiance(inputs: &TranspositionInputs) -> PlaneOfArrayIrradiance {
    let beam = beam_component(inputs);

    let air_mass = relative_air_mass(inputs.solar_zenith);
    let epsilon = clearness_epsilon(inputs.dni, inputs.dhi, inputs.solar_zenith);
    let delta = brightness_delta(inputs.dhi, air_mass, inputs.extraterrestrial_normal);
    let bin = bin_for_epsilon(epsilon);

    let z_rad = inputs.solar_zenith.radians();
    let f1 = (bin.f11 + bin.f12 * delta + bin.f13 * z_rad).max(0.0);
    let f2 = bin.f21 + bin.f22 * delta + bin.f23 * z_rad;

    let a = inputs.angle_of_incidence.cos().max(0.0);
    let cos_85 = Angle::from_degrees(85.0).cos();
    let b = inputs.solar_zenith.cos().max(cos_85);

    let tilt = inputs.surface.tilt;
    let sky_diffuse_factor = (1.0 - f1) * (1.0 + tilt.cos()) / 2.0 + f1 * (a / b) + f2 * tilt.sin();
    let sky_diffuse = Irradiance::from_watts_per_m2(inputs.dhi.watts_per_m2() * sky_diffuse_factor);

    let ground_reflected = ground_reflected_component(inputs);
    PlaneOfArrayIrradiance::from_components(beam, sky_diffuse, ground_reflected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::Ratio;
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
    fn tilt_zero_recovers_ghi_perez() {
        let zenith_deg: f64 = 30.0;
        let dni = 800.0;
        let dhi = 150.0;
        let ghi = dni * zenith_deg.to_radians().cos() + dhi;
        let inputs = make_inputs(ghi, dni, dhi, zenith_deg, 0.0);
        let poa = plane_of_array_irradiance(&inputs);
        assert!(
            (poa.global.watts_per_m2() - ghi).abs() < 1e-3,
            "poa.global = {}, ghi = {ghi}",
            poa.global.watts_per_m2()
        );
    }

    #[test]
    fn relative_air_mass_is_one_at_zenith() {
        let m = relative_air_mass(Angle::from_degrees(0.0));
        assert!((m - 1.0).abs() < 1e-3, "m = {m}");
    }

    #[test]
    fn relative_air_mass_increases_monotonically_with_zenith() {
        let mut previous = 0.0;
        for z in [0.0, 15.0, 30.0, 45.0, 60.0, 75.0, 85.0] {
            let m = relative_air_mass(Angle::from_degrees(z));
            assert!(m > previous, "air mass should increase with zenith angle");
            previous = m;
        }
    }

    #[test]
    fn clearness_epsilon_is_low_for_overcast_sky() {
        // Overcast: essentially no beam radiation (DNI near 0), all
        // diffuse. Epsilon should sit near its minimum (1.0).
        let epsilon = clearness_epsilon(
            Irradiance::from_watts_per_m2(0.0),
            Irradiance::from_watts_per_m2(100.0),
            Angle::from_degrees(30.0),
        );
        assert!((epsilon - 1.0).abs() < 0.05, "epsilon = {epsilon}");
    }

    #[test]
    fn clearness_epsilon_is_high_for_clear_sky_dominant_beam() {
        let epsilon = clearness_epsilon(
            Irradiance::from_watts_per_m2(900.0),
            Irradiance::from_watts_per_m2(60.0),
            Angle::from_degrees(20.0),
        );
        assert!(epsilon > 6.0, "epsilon = {epsilon}");
    }

    #[test]
    fn bin_lookup_covers_full_epsilon_range_without_panicking() {
        for epsilon in [0.5, 1.0, 1.1, 1.3, 1.6, 2.0, 3.0, 5.0, 7.0, 100.0] {
            let bin = bin_for_epsilon(epsilon);
            assert!(bin.epsilon_upper_bound > 0.0);
        }
    }

    #[test]
    fn perez_diffuse_is_non_negative_across_a_range_of_sky_conditions() {
        for zenith_deg in [0.0f64, 15.0, 30.0, 45.0, 60.0, 75.0, 85.0] {
            for kt_like_dni in [0.0, 200.0, 500.0, 800.0, 1000.0] {
                let dhi = 150.0;
                let inputs = make_inputs(
                    kt_like_dni * zenith_deg.to_radians().cos() + dhi,
                    kt_like_dni,
                    dhi,
                    zenith_deg,
                    30.0,
                );
                let poa = plane_of_array_irradiance(&inputs);
                assert!(
                    poa.sky_diffuse.watts_per_m2() >= 0.0,
                    "negative diffuse at zenith={zenith_deg}, dni={kt_like_dni}"
                );
                assert!(poa.global.watts_per_m2().is_finite());
            }
        }
    }
}
