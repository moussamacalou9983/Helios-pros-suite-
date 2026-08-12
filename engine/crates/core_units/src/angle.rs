use std::f64::consts::PI;
use std::ops::{Add, Neg, Sub};

/// A plane angle, stored internally in radians (the SI coherent unit for
/// plane angle; see BIPM, *The International System of Units (SI)*, 9th
/// edition, Table 4).
///
/// Construct with [`Angle::from_degrees`] or [`Angle::from_radians`]; read
/// back with [`Angle::degrees`] or [`Angle::radians`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Angle(f64);

impl Angle {
    /// The zero angle.
    pub const ZERO: Angle = Angle(0.0);

    /// Construct an angle from a value in radians.
    #[inline]
    pub const fn from_radians(radians: f64) -> Self {
        Angle(radians)
    }

    /// Construct an angle from a value in degrees.
    #[inline]
    pub fn from_degrees(degrees: f64) -> Self {
        Angle(degrees * PI / 180.0)
    }

    /// The angle's value in radians.
    #[inline]
    pub const fn radians(self) -> f64 {
        self.0
    }

    /// The angle's value in degrees.
    #[inline]
    pub fn degrees(self) -> f64 {
        self.0 * 180.0 / PI
    }

    /// Sine of the angle.
    #[inline]
    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    /// Cosine of the angle.
    #[inline]
    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    /// Tangent of the angle.
    #[inline]
    pub fn tan(self) -> f64 {
        self.0.tan()
    }

    /// Absolute value of the angle.
    #[inline]
    pub fn abs(self) -> Angle {
        Angle(self.0.abs())
    }

    /// Returns an equivalent angle normalized to the half-open interval
    /// `[0, 2*pi)` radians (`[0, 360)` degrees).
    #[inline]
    pub fn normalized_0_2pi(self) -> Angle {
        let two_pi = 2.0 * PI;
        let wrapped = self.0 % two_pi;
        Angle(if wrapped < 0.0 {
            wrapped + two_pi
        } else {
            wrapped
        })
    }

    /// Returns an equivalent angle normalized to the half-open interval
    /// `(-pi, pi]` radians (`(-180, 180]` degrees). Useful for azimuth
    /// differences and hour angles, which are conventionally signed around
    /// solar noon / due south.
    #[inline]
    pub fn normalized_pm_pi(self) -> Angle {
        let two_pi = 2.0 * PI;
        let mut wrapped = (self.0 + PI) % two_pi;
        if wrapped <= 0.0 {
            wrapped += two_pi;
        }
        Angle(wrapped - PI)
    }
}

impl Add for Angle {
    type Output = Angle;
    #[inline]
    fn add(self, rhs: Angle) -> Angle {
        Angle(self.0 + rhs.0)
    }
}

impl Sub for Angle {
    type Output = Angle;
    #[inline]
    fn sub(self, rhs: Angle) -> Angle {
        Angle(self.0 - rhs.0)
    }
}

impl Neg for Angle {
    type Output = Angle;
    #[inline]
    fn neg(self) -> Angle {
        Angle(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_radians_roundtrip() {
        let a = Angle::from_degrees(45.0);
        assert!((a.radians() - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
        assert!((a.degrees() - 45.0).abs() < 1e-12);
    }

    #[test]
    fn trigonometric_functions_match_std() {
        let a = Angle::from_degrees(30.0);
        assert!((a.sin() - (30f64).to_radians().sin()).abs() < 1e-12);
        assert!((a.cos() - (30f64).to_radians().cos()).abs() < 1e-12);
    }

    #[test]
    fn normalized_0_2pi_wraps_negative_angles() {
        let a = Angle::from_degrees(-90.0).normalized_0_2pi();
        assert!((a.degrees() - 270.0).abs() < 1e-9);
    }

    #[test]
    fn normalized_0_2pi_wraps_large_angles() {
        let a = Angle::from_degrees(400.0).normalized_0_2pi();
        assert!((a.degrees() - 40.0).abs() < 1e-9);
    }

    #[test]
    fn normalized_pm_pi_keeps_small_angles_unchanged() {
        let a = Angle::from_degrees(10.0).normalized_pm_pi();
        assert!((a.degrees() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn normalized_pm_pi_wraps_angles_beyond_180() {
        let a = Angle::from_degrees(190.0).normalized_pm_pi();
        assert!((a.degrees() - (-170.0)).abs() < 1e-9);
    }

    #[test]
    fn addition_and_subtraction() {
        let a = Angle::from_degrees(10.0);
        let b = Angle::from_degrees(20.0);
        assert!(((a + b).degrees() - 30.0).abs() < 1e-9);
        assert!(((b - a).degrees() - 10.0).abs() < 1e-9);
        assert!(((-a).degrees() + 10.0).abs() < 1e-9);
    }
}
