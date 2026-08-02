use std::ops::Add;

/// Irradiance (radiant flux received per unit surface area), stored in
/// watts per square metre (W/m²), the SI coherent derived unit.
///
/// Reference: WMO, *Guide to Instruments and Methods of Observation*,
/// WMO-No. 8, Chapter 7 (radiation measurement), uses W/m² throughout as the
/// standard unit for irradiance; ISO 9060:2018 likewise expresses
/// pyranometer/pyrheliometer response in W/m².
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Irradiance(f64);

impl Irradiance {
    /// The zero irradiance.
    pub const ZERO: Irradiance = Irradiance(0.0);

    /// Construct an irradiance from a value in W/m².
    #[inline]
    pub const fn from_watts_per_m2(value: f64) -> Self {
        Irradiance(value)
    }

    /// The irradiance's value in W/m².
    #[inline]
    pub const fn watts_per_m2(self) -> f64 {
        self.0
    }

    /// Returns the irradiance clamped to a physically non-negative value.
    /// Used defensively at model boundaries where a formula could otherwise
    /// yield a small negative artefact near sunrise/sunset due to
    /// floating-point rounding.
    #[inline]
    pub fn non_negative(self) -> Irradiance {
        Irradiance(self.0.max(0.0))
    }
}

impl Add for Irradiance {
    type Output = Irradiance;
    #[inline]
    fn add(self, rhs: Irradiance) -> Irradiance {
        Irradiance(self.0 + rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_accessor_roundtrip() {
        let i = Irradiance::from_watts_per_m2(850.0);
        assert!((i.watts_per_m2() - 850.0).abs() < 1e-12);
    }

    #[test]
    fn non_negative_clamps_small_negative_artifacts() {
        let i = Irradiance::from_watts_per_m2(-0.5).non_negative();
        assert_eq!(i.watts_per_m2(), 0.0);
    }

    #[test]
    fn addition_sums_components() {
        let a = Irradiance::from_watts_per_m2(400.0);
        let b = Irradiance::from_watts_per_m2(120.0);
        assert!(((a + b).watts_per_m2() - 520.0).abs() < 1e-9);
    }
}
