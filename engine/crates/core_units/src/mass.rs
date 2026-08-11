/// Mass, stored internally in kilograms (kg), an SI base unit (BIPM SI
/// Brochure, 9th ed., Table 1).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(f64);

impl Mass {
    /// Construct a mass from a value in kilograms.
    #[inline]
    pub const fn from_kilograms(kilograms: f64) -> Self {
        Mass(kilograms)
    }

    /// Construct a mass from a value in grams.
    #[inline]
    pub fn from_grams(grams: f64) -> Self {
        Mass(grams / 1000.0)
    }

    /// The mass's value in kilograms.
    #[inline]
    pub const fn kilograms(self) -> f64 {
        self.0
    }

    /// The mass's value in grams.
    #[inline]
    pub fn grams(self) -> f64 {
        self.0 * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gram_roundtrip() {
        let m = Mass::from_grams(21500.0);
        assert!((m.kilograms() - 21.5).abs() < 1e-12);
    }
}
