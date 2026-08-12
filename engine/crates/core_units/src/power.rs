/// Power, stored internally in watts (W), the SI coherent derived unit
/// for power (BIPM SI Brochure, 9th ed., Table 4).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f64);

impl Power {
    /// The zero power.
    pub const ZERO: Power = Power(0.0);

    /// Construct a power from a value in watts.
    #[inline]
    pub const fn from_watts(watts: f64) -> Self {
        Power(watts)
    }

    /// Construct a power from a value in kilowatts.
    #[inline]
    pub fn from_kilowatts(kilowatts: f64) -> Self {
        Power(kilowatts * 1000.0)
    }

    /// The power's value in watts.
    #[inline]
    pub const fn watts(self) -> f64 {
        self.0
    }

    /// The power's value in kilowatts.
    #[inline]
    pub fn kilowatts(self) -> f64 {
        self.0 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kilowatt_roundtrip() {
        let p = Power::from_kilowatts(0.4405);
        assert!((p.watts() - 440.5).abs() < 1e-9);
    }
}
