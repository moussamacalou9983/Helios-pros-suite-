/// Length, stored internally in metres (m), the SI base unit for length.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length(f64);

impl Length {
    /// Construct a length from a value in metres.
    #[inline]
    pub const fn from_meters(meters: f64) -> Self {
        Length(meters)
    }

    /// Construct a length from a value in kilometres.
    #[inline]
    pub fn from_kilometers(km: f64) -> Self {
        Length(km * 1000.0)
    }

    /// Construct a length from a value in millimetres.
    ///
    /// Module datasheets state overall dimensions and glass thickness in
    /// millimetres, so an adapter reading one converts here, once, at the
    /// boundary rather than carrying the source's unit inward.
    #[inline]
    pub fn from_millimeters(millimeters: f64) -> Self {
        Length(millimeters / 1000.0)
    }

    /// The length's value in metres.
    #[inline]
    pub const fn meters(self) -> f64 {
        self.0
    }

    /// The length's value in kilometres.
    #[inline]
    pub fn kilometers(self) -> f64 {
        self.0 / 1000.0
    }

    /// The length's value in millimetres.
    #[inline]
    pub fn millimeters(self) -> f64 {
        self.0 * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kilometer_roundtrip() {
        let l = Length::from_kilometers(1.83014);
        assert!((l.meters() - 1830.14).abs() < 1e-6);
    }

    #[test]
    fn millimeter_roundtrip() {
        // The dimensions a module datasheet publishes.
        let l = Length::from_millimeters(1762.0);
        assert!((l.meters() - 1.762).abs() < 1e-12);
        assert!((l.millimeters() - 1762.0).abs() < 1e-9);
    }
}
