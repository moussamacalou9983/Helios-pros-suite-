/// Whether a temperature coefficient expresses a *relative* change (a
/// fraction of a reference value, per kelvin) or an *absolute* change (the
/// quantity's own unit, per kelvin).
///
/// This distinction is not cosmetic. The CEC module database publishes
/// `alpha_sc` and `beta_oc` in absolute form (A/K, V/K) and `gamma_r` in
/// relative form (%/K) **within the same record** — verified by direct
/// inspection of 21 535 records, see
/// `docs/design/26-validation-empirique-cec-sandia.md` §2.4. A coefficient
/// of `-0.159` is therefore either -0.159 V/K or -0.159 %/K depending on
/// which field it came from, two quantities differing by more than two
/// orders of magnitude for a typical module.
///
/// The basis is consequently carried by the value and never inferred from
/// its magnitude: guessing would be a silent arbitration, which the
/// project's data-conflict policy forbids
/// (`docs/design/27-specification-finale-pv-module.md` §3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoefficientBasis {
    /// A fraction of a reference value, per kelvin. Datasheets publish
    /// this form as a percentage per degree Celsius.
    Relative,
    /// The quantity's own unit, per kelvin (A/K for a current
    /// coefficient, V/K for a voltage coefficient, W/K for a power
    /// coefficient).
    Absolute,
}

/// A temperature coefficient: the rate of change of a physical quantity
/// with temperature, stored internally per kelvin and carrying its own
/// [`CoefficientBasis`].
///
/// **Signed and unbounded** by design. Two of the three coefficients a PV
/// datasheet publishes are negative (beta on open-circuit voltage, gamma
/// on maximum power), which is why [`crate::Ratio`] — clamped to `[0, 1]`
/// — cannot represent them.
///
/// **Unit note**: a coefficient acts on a temperature *difference*, and
/// the Celsius and Kelvin scales differ only by a constant offset, so a
/// value per degree Celsius and the same value per kelvin are numerically
/// identical. Both accessors are provided; no conversion occurs between
/// them.
///
/// **References**: IEC 60891 (procedures for temperature and irradiance
/// corrections to measured I-V characteristics); IEC 61215 (measurement
/// of temperature coefficients).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemperatureCoefficient {
    per_kelvin: f64,
    basis: CoefficientBasis,
}

impl TemperatureCoefficient {
    /// Construct a *relative* coefficient from a percentage per degree
    /// Celsius, the form used by essentially every manufacturer
    /// datasheet (for example `-0.29` for a typical gamma on maximum
    /// power).
    #[inline]
    pub fn from_percent_per_celsius(percent_per_celsius: f64) -> Self {
        TemperatureCoefficient {
            per_kelvin: percent_per_celsius / 100.0,
            basis: CoefficientBasis::Relative,
        }
    }

    /// Construct a *relative* coefficient from a dimensionless fraction
    /// per kelvin.
    #[inline]
    pub const fn from_fraction_per_kelvin(fraction_per_kelvin: f64) -> Self {
        TemperatureCoefficient {
            per_kelvin: fraction_per_kelvin,
            basis: CoefficientBasis::Relative,
        }
    }

    /// Construct an *absolute* coefficient from a value in the quantity's
    /// own unit per kelvin (A/K, V/K, W/K).
    #[inline]
    pub const fn from_absolute_per_kelvin(absolute_per_kelvin: f64) -> Self {
        TemperatureCoefficient {
            per_kelvin: absolute_per_kelvin,
            basis: CoefficientBasis::Absolute,
        }
    }

    /// The coefficient's stored value per kelvin, in whichever basis it
    /// was constructed with. Always inspect [`TemperatureCoefficient::basis`]
    /// before interpreting this number.
    #[inline]
    pub const fn per_kelvin(self) -> f64 {
        self.per_kelvin
    }

    /// The coefficient's value per degree Celsius, numerically identical
    /// to [`TemperatureCoefficient::per_kelvin`] (see the unit note on
    /// this type).
    #[inline]
    pub const fn per_celsius(self) -> f64 {
        self.per_kelvin
    }

    /// The coefficient expressed as a percentage per degree Celsius.
    /// Meaningful only for a [`CoefficientBasis::Relative`] coefficient;
    /// for an absolute one this is simply the stored value times 100 and
    /// carries no percentage meaning.
    #[inline]
    pub fn percent_per_celsius(self) -> f64 {
        self.per_kelvin * 100.0
    }

    /// Whether this coefficient is relative or absolute.
    #[inline]
    pub const fn basis(self) -> CoefficientBasis {
        self.basis
    }

    /// Convert to an absolute rate (the quantity's unit per kelvin) given
    /// the reference value the relative coefficient applies to.
    ///
    /// Returns the stored value unchanged when the coefficient is already
    /// absolute, so that `reference` is ignored rather than misapplied.
    #[inline]
    pub fn to_absolute_per_kelvin(self, reference_value: f64) -> f64 {
        match self.basis {
            CoefficientBasis::Relative => self.per_kelvin * reference_value,
            CoefficientBasis::Absolute => self.per_kelvin,
        }
    }

    /// Convert to a relative rate (fraction per kelvin) given the
    /// reference value the absolute coefficient applies to.
    ///
    /// Returns `None` when the coefficient is absolute and
    /// `reference_value` is zero, since the relative form is then
    /// undefined rather than infinite.
    #[inline]
    pub fn to_fraction_per_kelvin(self, reference_value: f64) -> Option<f64> {
        match self.basis {
            CoefficientBasis::Relative => Some(self.per_kelvin),
            CoefficientBasis::Absolute => {
                if reference_value == 0.0 {
                    None
                } else {
                    Some(self.per_kelvin / reference_value)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_per_celsius_roundtrip() {
        let g = TemperatureCoefficient::from_percent_per_celsius(-0.29);
        assert!((g.percent_per_celsius() - (-0.29)).abs() < 1e-12);
        assert!((g.per_kelvin() - (-0.0029)).abs() < 1e-12);
        assert_eq!(g.basis(), CoefficientBasis::Relative);
    }

    #[test]
    fn negative_values_are_representable() {
        // The reason Ratio cannot be used for these quantities.
        let b = TemperatureCoefficient::from_percent_per_celsius(-0.35);
        assert!(b.per_kelvin() < 0.0);
    }

    #[test]
    fn relative_to_absolute_uses_the_reference() {
        // beta_oc = -0.35 %/K on a 44 V module -> -0.154 V/K.
        let b = TemperatureCoefficient::from_percent_per_celsius(-0.35);
        assert!((b.to_absolute_per_kelvin(44.0) - (-0.154)).abs() < 1e-12);
    }

    #[test]
    fn absolute_to_relative_uses_the_reference() {
        // The CEC form: beta_oc = -0.159068 V/K on a 43.99 V module.
        let b = TemperatureCoefficient::from_absolute_per_kelvin(-0.159_068);
        let relative = b.to_fraction_per_kelvin(43.99).unwrap();
        assert!((relative * 100.0 - (-0.361_6)).abs() < 1e-3);
    }

    #[test]
    fn absolute_coefficient_ignores_reference_when_asked_for_absolute() {
        let b = TemperatureCoefficient::from_absolute_per_kelvin(-0.159);
        assert!((b.to_absolute_per_kelvin(999.0) - (-0.159)).abs() < 1e-12);
    }

    #[test]
    fn relative_form_is_undefined_for_a_zero_reference() {
        let b = TemperatureCoefficient::from_absolute_per_kelvin(-0.159);
        assert_eq!(b.to_fraction_per_kelvin(0.0), None);
    }

    #[test]
    fn basis_is_carried_not_inferred_from_magnitude() {
        // Identical magnitudes, different bases: the type keeps them apart.
        let relative = TemperatureCoefficient::from_fraction_per_kelvin(-0.159);
        let absolute = TemperatureCoefficient::from_absolute_per_kelvin(-0.159);
        assert_ne!(relative.basis(), absolute.basis());
        assert_ne!(relative, absolute);
    }
}
