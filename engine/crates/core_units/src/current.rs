/// Electric current, stored internally in amperes (A), an SI base unit
/// (BIPM SI Brochure, 9th ed., Table 1).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Current(f64);

impl Current {
    /// The zero current.
    pub const ZERO: Current = Current(0.0);

    /// Construct a current from a value in amperes.
    #[inline]
    pub const fn from_amperes(amperes: f64) -> Self {
        Current(amperes)
    }

    /// Construct a current from a value in milliamperes.
    #[inline]
    pub fn from_milliamperes(milliamperes: f64) -> Self {
        Current(milliamperes / 1000.0)
    }

    /// The current's value in amperes.
    #[inline]
    pub const fn amperes(self) -> f64 {
        self.0
    }

    /// The current's value in milliamperes.
    #[inline]
    pub fn milliamperes(self) -> f64 {
        self.0 * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milliampere_roundtrip() {
        let i = Current::from_milliamperes(2500.0);
        assert!((i.amperes() - 2.5).abs() < 1e-12);
        assert!((i.milliamperes() - 2500.0).abs() < 1e-9);
    }
}
