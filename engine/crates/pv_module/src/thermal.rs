use core_units::TemperatureCoefficient;

use crate::provenance::Measurement;

/// The three temperature coefficients every datasheet publishes, plus the
/// fourth a minority add.
///
/// Without these, no extrapolation away from the 25 °C reference is
/// possible, which would make a module unusable in an hourly simulation.
/// That is why the first three are mandatory.
///
/// **Expected signs** — alpha positive, beta and gamma negative — are
/// checked as *warnings*, never as errors. Rejecting an unexpected sign
/// would be a physical judgement disguised as format validation, and a
/// future technology could legitimately differ.
///
/// **References**: IEC 60891 (procedures for temperature and irradiance
/// corrections), IEC 61215 (measurement of temperature coefficients).
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalCharacteristics {
    /// Temperature coefficient of short-circuit current. Expected
    /// positive.
    pub alpha_i_sc: Measurement<TemperatureCoefficient>,
    /// Temperature coefficient of open-circuit voltage. Expected
    /// negative. Sizes the maximum cold-weather string voltage.
    pub beta_v_oc: Measurement<TemperatureCoefficient>,
    /// Temperature coefficient of maximum power. Expected negative.
    /// Drives the annual thermal loss.
    pub gamma_p_max: Measurement<TemperatureCoefficient>,
    /// Temperature coefficient of maximum-power-point voltage, published
    /// by a minority of manufacturers.
    pub beta_v_mpp: Option<Measurement<TemperatureCoefficient>>,
}

impl ThermalCharacteristics {
    /// Construct thermal characteristics from the three mandatory
    /// coefficients.
    #[inline]
    pub fn new(
        alpha_i_sc: Measurement<TemperatureCoefficient>,
        beta_v_oc: Measurement<TemperatureCoefficient>,
        gamma_p_max: Measurement<TemperatureCoefficient>,
    ) -> Self {
        ThermalCharacteristics {
            alpha_i_sc,
            beta_v_oc,
            gamma_p_max,
            beta_v_mpp: None,
        }
    }

    /// Record the maximum-power-point voltage coefficient.
    #[inline]
    #[must_use]
    pub fn with_beta_v_mpp(mut self, beta_v_mpp: Measurement<TemperatureCoefficient>) -> Self {
        self.beta_v_mpp = Some(beta_v_mpp);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_units::CoefficientBasis;

    fn typical() -> ThermalCharacteristics {
        ThermalCharacteristics::new(
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(0.045)),
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.27)),
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.34)),
        )
    }

    #[test]
    fn the_three_mandatory_coefficients_are_carried() {
        let t = typical();
        assert!(t.alpha_i_sc.get().per_kelvin() > 0.0);
        assert!(t.beta_v_oc.get().per_kelvin() < 0.0);
        assert!(t.gamma_p_max.get().per_kelvin() < 0.0);
        assert!(t.beta_v_mpp.is_none());
    }

    #[test]
    fn the_optional_fourth_coefficient_is_recorded_explicitly() {
        let t = typical().with_beta_v_mpp(Measurement::stated(
            TemperatureCoefficient::from_percent_per_celsius(-0.31),
        ));
        assert!((t.beta_v_mpp.unwrap().get().percent_per_celsius() - (-0.31)).abs() < 1e-12);
    }

    #[test]
    fn mixed_bases_coexist_as_the_cec_database_publishes_them() {
        // The CEC library gives alpha and beta in absolute form and gamma
        // in relative form within the same record. The model must hold
        // both without conversion or guessing.
        let t = ThermalCharacteristics::new(
            Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(0.002_146)),
            Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(-0.159_068)),
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.5072)),
        );
        assert_eq!(t.alpha_i_sc.get().basis(), CoefficientBasis::Absolute);
        assert_eq!(t.beta_v_oc.get().basis(), CoefficientBasis::Absolute);
        assert_eq!(t.gamma_p_max.get().basis(), CoefficientBasis::Relative);
    }
}
