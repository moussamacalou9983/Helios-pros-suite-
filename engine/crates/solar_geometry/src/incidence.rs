//! Angle of incidence of solar radiation on an arbitrarily oriented flat
//! surface.
//!
//! **Source**: Duffie, J.A. & Beckman, W.A. (2013), *Solar Engineering of
//! Thermal Processes*, 4th ed., John Wiley & Sons, Eq. 1.6.3. This is an
//! exact spherical-geometry identity (no series truncation), also
//! reproduced identically in Reda & Andreas (2004), Eq. 3.19 (their
//! "surface incidence angle").

use core_units::Angle;

/// Orientation of a flat collector surface (module plane, tracker plane,
/// etc.).
#[derive(Debug, Clone, Copy)]
pub struct SurfaceOrientation {
    /// Tilt from horizontal: 0° = horizontal, 90° = vertical.
    pub tilt: Angle,
    /// Surface azimuth, measured from geographic North, clockwise (same
    /// convention as `solar_position::SolarPosition::azimuth`); e.g.
    /// 180° for a surface facing due South.
    pub azimuth: Angle,
}

/// Compute the angle of incidence `theta` between the Sun's rays and the
/// surface normal, given the Sun's zenith and azimuth and the surface
/// orientation.
///
/// `cos(theta) = cos(zenith)*cos(tilt) + sin(zenith)*sin(tilt)*cos(solar_azimuth - surface_azimuth)`
///
/// The returned angle is always in `[0°, 180°]`; values above 90° mean the
/// Sun is behind the surface plane (the beam component of irradiance on
/// the surface is then zero, which callers must clamp explicitly — this
/// function reports the raw geometric angle, not the clamped beam
/// contribution).
pub fn angle_of_incidence(
    solar_zenith: Angle,
    solar_azimuth: Angle,
    surface: SurfaceOrientation,
) -> Angle {
    let azimuth_difference = solar_azimuth - surface.azimuth;
    let cos_theta = solar_zenith.cos() * surface.tilt.cos()
        + solar_zenith.sin() * surface.tilt.sin() * azimuth_difference.cos();
    // Clamp to [-1, 1] to guard against floating-point overshoot before
    // calling acos, which would otherwise return NaN.
    Angle::from_radians(cos_theta.clamp(-1.0, 1.0).acos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_surface_incidence_equals_zenith() {
        // A horizontal surface's normal points straight up, so the angle
        // of incidence must equal the solar zenith angle regardless of
        // solar azimuth (tilt = 0 removes all azimuth dependency).
        let surface = SurfaceOrientation {
            tilt: Angle::from_degrees(0.0),
            azimuth: Angle::from_degrees(180.0),
        };
        for zenith_deg in [0.0, 20.0, 45.0, 70.0] {
            for az_deg in [0.0, 90.0, 180.0, 270.0] {
                let theta = angle_of_incidence(
                    Angle::from_degrees(zenith_deg),
                    Angle::from_degrees(az_deg),
                    surface,
                );
                assert!(
                    (theta.degrees() - zenith_deg).abs() < 1e-9,
                    "zenith={zenith_deg}, az={az_deg}, theta={}",
                    theta.degrees()
                );
            }
        }
    }

    #[test]
    fn sun_directly_along_surface_normal_gives_zero_incidence() {
        // A surface tilted 30 deg facing South (azimuth 180 deg), with the
        // Sun at zenith 30 deg and azimuth 180 deg, is exactly aligned with
        // the surface normal: incidence angle must be 0.
        let surface = SurfaceOrientation {
            tilt: Angle::from_degrees(30.0),
            azimuth: Angle::from_degrees(180.0),
        };
        let theta = angle_of_incidence(
            Angle::from_degrees(30.0),
            Angle::from_degrees(180.0),
            surface,
        );
        assert!(theta.degrees() < 1e-6, "theta = {}", theta.degrees());
    }

    #[test]
    fn sun_behind_surface_gives_incidence_above_ninety_degrees() {
        // A vertical wall facing South (tilt 90, azimuth 180) with the Sun
        // near-overhead but tilted slightly towards the North (zenith 30,
        // azimuth 0): the Sun is on the far side of the wall's plane, so
        // the geometric incidence angle must exceed 90 deg.
        let surface = SurfaceOrientation {
            tilt: Angle::from_degrees(90.0),
            azimuth: Angle::from_degrees(180.0),
        };
        let theta =
            angle_of_incidence(Angle::from_degrees(30.0), Angle::from_degrees(0.0), surface);
        assert!(theta.degrees() > 90.0, "theta = {}", theta.degrees());
    }

    #[test]
    fn vertical_south_facing_wall_at_solar_noon_equinox_midlatitude() {
        // At equinox solar noon at 45N, zenith = 45 deg, solar azimuth =
        // 180 deg. A vertical (tilt=90) South-facing (az=180) wall's
        // incidence angle should then equal the zenith angle (both share
        // the same "vertical drop" from normal, since the wall normal is
        // horizontal, pointing South, i.e. exactly at elevation 0 due
        // South -- the angle between the sun vector (elevation 45, due
        // south) and the horizontal south-pointing normal is 45 deg).
        let surface = SurfaceOrientation {
            tilt: Angle::from_degrees(90.0),
            azimuth: Angle::from_degrees(180.0),
        };
        let theta = angle_of_incidence(
            Angle::from_degrees(45.0),
            Angle::from_degrees(180.0),
            surface,
        );
        assert!(
            (theta.degrees() - 45.0).abs() < 1e-9,
            "theta = {}",
            theta.degrees()
        );
    }
}
