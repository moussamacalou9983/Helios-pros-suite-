/// A dimensionless ratio clamped to the physically valid range `[0, 1]`,
/// used for quantities such as ground albedo, reflectance, and efficiency
/// that are dimensionless by definition (ISO 80000-1:2022, Annex C) but
/// bounded in a way a bare `f64` does not enforce.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ratio(f64);

impl Ratio {
    /// Construct a ratio from a value expected to already lie in `[0, 1]`.
    ///
    /// # Panics
    ///
    /// Panics if `value` is outside `[0, 1]` or is NaN. Use
    /// [`Ratio::clamped`] at data-ingestion boundaries where out-of-range
    /// input must be tolerated rather than rejected.
    #[inline]
    pub fn new(value: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "Ratio value {value} outside valid range [0, 1]"
        );
        Ratio(value)
    }

    /// Construct a ratio by clamping `value` into `[0, 1]`.
    #[inline]
    pub fn clamped(value: f64) -> Self {
        Ratio(value.clamp(0.0, 1.0))
    }

    /// The ratio's dimensionless value.
    #[inline]
    pub const fn value(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamped_bounds_values() {
        assert_eq!(Ratio::clamped(1.5).value(), 1.0);
        assert_eq!(Ratio::clamped(-0.2).value(), 0.0);
        assert_eq!(Ratio::clamped(0.42).value(), 0.42);
    }

    #[test]
    #[should_panic]
    fn new_panics_out_of_range() {
        let _ = Ratio::new(1.2);
    }
}
