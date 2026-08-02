/// Pressure, stored internally in pascals (Pa), the SI coherent derived
/// unit (BIPM SI Brochure, 9th ed., Table 4).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pressure(f64);

impl Pressure {
    /// Construct a pressure from a value in pascals.
    #[inline]
    pub const fn from_pascal(pascal: f64) -> Self {
        Pressure(pascal)
    }

    /// Construct a pressure from a value in hectopascals (equivalently,
    /// millibars), the unit conventionally used for station-level
    /// atmospheric pressure reports.
    #[inline]
    pub fn from_hectopascal(hpa: f64) -> Self {
        Pressure(hpa * 100.0)
    }

    /// The pressure's value in pascals.
    #[inline]
    pub const fn pascal(self) -> f64 {
        self.0
    }

    /// The pressure's value in hectopascals.
    #[inline]
    pub fn hectopascal(self) -> f64 {
        self.0 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hectopascal_roundtrip() {
        let p = Pressure::from_hectopascal(1013.25);
        assert!((p.pascal() - 101_325.0).abs() < 1e-6);
        assert!((p.hectopascal() - 1013.25).abs() < 1e-9);
    }
}
