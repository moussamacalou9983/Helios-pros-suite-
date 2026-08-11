/// Area, stored internally in square metres (m^2), the SI coherent
/// derived unit for area (BIPM SI Brochure, 9th ed., Table 3).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Area(f64);

impl Area {
    /// Construct an area from a value in square metres.
    #[inline]
    pub const fn from_square_meters(square_meters: f64) -> Self {
        Area(square_meters)
    }

    /// Construct an area from a value in square centimetres.
    #[inline]
    pub fn from_square_centimeters(square_centimeters: f64) -> Self {
        Area(square_centimeters / 10_000.0)
    }

    /// Construct an area from a value in square millimetres. Conductor
    /// cross-sections are conventionally quoted in mm^2 (IEC 60228).
    #[inline]
    pub fn from_square_millimeters(square_millimeters: f64) -> Self {
        Area(square_millimeters / 1_000_000.0)
    }

    /// The area's value in square metres.
    #[inline]
    pub const fn square_meters(self) -> f64 {
        self.0
    }

    /// The area's value in square centimetres.
    #[inline]
    pub fn square_centimeters(self) -> f64 {
        self.0 * 10_000.0
    }

    /// The area's value in square millimetres.
    #[inline]
    pub fn square_millimeters(self) -> f64 {
        self.0 * 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_centimeter_roundtrip() {
        let a = Area::from_square_centimeters(24_300.0);
        assert!((a.square_meters() - 2.43).abs() < 1e-12);
    }

    #[test]
    fn square_millimeter_roundtrip() {
        let a = Area::from_square_millimeters(4.0);
        assert!((a.square_meters() - 4e-6).abs() < 1e-18);
        assert!((a.square_millimeters() - 4.0).abs() < 1e-9);
    }
}
