/// Velocity, stored internally in metres per second (m/s), the SI coherent
/// derived unit. Used in the engine for wind speed inputs to cell-temperature
/// models.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Velocity(f64);

impl Velocity {
    /// Construct a velocity from a value in m/s.
    #[inline]
    pub const fn from_meters_per_second(value: f64) -> Self {
        Velocity(value)
    }

    /// The velocity's value in m/s.
    #[inline]
    pub const fn meters_per_second(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_roundtrip() {
        let v = Velocity::from_meters_per_second(3.5);
        assert!((v.meters_per_second() - 3.5).abs() < 1e-12);
    }
}
