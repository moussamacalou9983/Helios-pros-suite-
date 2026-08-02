/// Absolute temperature, stored internally in kelvin (the SI base unit for
/// thermodynamic temperature; BIPM SI Brochure, 9th ed., Table 1).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature(f64);

/// Offset between the Celsius and Kelvin scales.
///
/// Source: BIPM, *The International System of Units (SI)*, 9th edition
/// (2019), Section 2.3.1: `T/K = t/°C + 273.15`.
const CELSIUS_ZERO_KELVIN: f64 = 273.15;

impl Temperature {
    /// Construct a temperature from a value in kelvin.
    #[inline]
    pub const fn from_kelvin(kelvin: f64) -> Self {
        Temperature(kelvin)
    }

    /// Construct a temperature from a value in degrees Celsius.
    #[inline]
    pub fn from_celsius(celsius: f64) -> Self {
        Temperature(celsius + CELSIUS_ZERO_KELVIN)
    }

    /// The temperature's value in kelvin.
    #[inline]
    pub const fn kelvin(self) -> f64 {
        self.0
    }

    /// The temperature's value in degrees Celsius.
    #[inline]
    pub fn celsius(self) -> f64 {
        self.0 - CELSIUS_ZERO_KELVIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_kelvin_roundtrip() {
        let t = Temperature::from_celsius(25.0);
        assert!((t.kelvin() - 298.15).abs() < 1e-9);
        assert!((t.celsius() - 25.0).abs() < 1e-9);
    }

    #[test]
    fn absolute_zero_is_zero_kelvin() {
        let t = Temperature::from_kelvin(0.0);
        assert!((t.celsius() - (-273.15)).abs() < 1e-9);
    }
}
