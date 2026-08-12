/// Wavelength, stored internally in metres (m). Spectral response data is
/// conventionally tabulated in nanometres (IEC 60904-8), hence the
/// explicit nanometre constructor and accessor.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Wavelength(f64);

impl Wavelength {
    /// Construct a wavelength from a value in metres.
    #[inline]
    pub const fn from_meters(meters: f64) -> Self {
        Wavelength(meters)
    }

    /// Construct a wavelength from a value in nanometres.
    #[inline]
    pub fn from_nanometers(nanometers: f64) -> Self {
        Wavelength(nanometers * 1e-9)
    }

    /// The wavelength's value in metres.
    #[inline]
    pub const fn meters(self) -> f64 {
        self.0
    }

    /// The wavelength's value in nanometres.
    #[inline]
    pub fn nanometers(self) -> f64 {
        self.0 * 1e9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nanometer_roundtrip() {
        let w = Wavelength::from_nanometers(550.0);
        assert!((w.meters() - 5.5e-7).abs() < 1e-18);
        assert!((w.nanometers() - 550.0).abs() < 1e-9);
    }
}
