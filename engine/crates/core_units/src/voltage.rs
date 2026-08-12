use std::ops::Mul;

use crate::current::Current;
use crate::power::Power;

/// Electric potential difference, stored internally in volts (V), the SI
/// coherent derived unit for electric potential (BIPM SI Brochure, 9th
/// ed., Table 4).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Voltage(f64);

impl Voltage {
    /// The zero voltage.
    pub const ZERO: Voltage = Voltage(0.0);

    /// Construct a voltage from a value in volts.
    #[inline]
    pub const fn from_volts(volts: f64) -> Self {
        Voltage(volts)
    }

    /// Construct a voltage from a value in millivolts.
    #[inline]
    pub fn from_millivolts(millivolts: f64) -> Self {
        Voltage(millivolts / 1000.0)
    }

    /// The voltage's value in volts.
    #[inline]
    pub const fn volts(self) -> f64 {
        self.0
    }

    /// The voltage's value in millivolts.
    #[inline]
    pub fn millivolts(self) -> f64 {
        self.0 * 1000.0
    }
}

/// `V x I = P`, the defining relation of electrical power. Provided as a
/// typed operation so that a voltage-current product cannot silently be
/// mistaken for any other quantity.
impl Mul<Current> for Voltage {
    type Output = Power;

    #[inline]
    fn mul(self, current: Current) -> Power {
        Power::from_watts(self.0 * current.amperes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millivolt_roundtrip() {
        let v = Voltage::from_millivolts(1500.0);
        assert!((v.volts() - 1.5).abs() < 1e-12);
        assert!((v.millivolts() - 1500.0).abs() < 1e-9);
    }

    #[test]
    fn voltage_times_current_is_power() {
        let p = Voltage::from_volts(40.0) * Current::from_amperes(9.5);
        assert!((p.watts() - 380.0).abs() < 1e-12);
    }
}
