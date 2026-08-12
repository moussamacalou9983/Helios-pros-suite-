use core_units::{Current, Power, Ratio, Temperature, Voltage};

use crate::provenance::Measurement;

/// The spread between specimens of one product, handled by sorting
/// modules into power bins.
///
/// **Not a measurement uncertainty and not a degradation rate.** These
/// three dispersions apply to different objects — a manufacturing lot, a
/// measured specimen, and the passage of time — and combining them into
/// one figure would produce an uninterpretable number
/// (`docs/design/25-specification-pv-module.md` §8.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerTolerance {
    /// A symmetric relative tolerance, e.g. ±3 %.
    Symmetric {
        /// The relative half-width of the tolerance band.
        relative: Ratio,
    },
    /// An asymmetric absolute tolerance, e.g. `0/+5 W`.
    ///
    /// The prevailing form on modern datasheets, and not representable by
    /// a single scalar — which is why this type has two variants rather
    /// than one number.
    Asymmetric {
        /// Lowest power the bin admits.
        minimum: Power,
        /// Highest power the bin admits.
        maximum: Power,
    },
}

/// The five electrical quantities that define a module's operating point,
/// plus the derived figures a datasheet states alongside them.
///
/// At standard test conditions these are the minimum input any electrical
/// model needs (De Soto et al., 2006).
///
/// **References**: IEC 60904-1 (I-V characteristic measurement),
/// IEC 61215 (STC definition), IEC 60904-3 (AM1.5G reference spectrum).
#[derive(Debug, Clone, PartialEq)]
pub struct ElectricalCharacteristics {
    /// Maximum power.
    pub p_max: Measurement<Power>,
    /// Open-circuit voltage.
    pub v_oc: Measurement<Voltage>,
    /// Short-circuit current.
    pub i_sc: Measurement<Current>,
    /// Voltage at the maximum power point.
    pub v_mpp: Measurement<Voltage>,
    /// Current at the maximum power point.
    pub i_mpp: Measurement<Current>,
    /// The efficiency printed on the datasheet, kept for cross-checking
    /// only.
    ///
    /// The authoritative efficiency is derived from `p_max` and a named
    /// area; this field exists so a disagreement becomes a detectable
    /// data-quality signal rather than an unresolvable ambiguity.
    pub stated_efficiency: Option<Measurement<Ratio>>,
    /// The production tolerance, if stated.
    pub power_tolerance: Option<PowerTolerance>,
}

impl ElectricalCharacteristics {
    /// Construct an electrical characteristic from the five mandatory
    /// quantities.
    #[inline]
    pub fn new(
        p_max: Measurement<Power>,
        v_oc: Measurement<Voltage>,
        i_sc: Measurement<Current>,
        v_mpp: Measurement<Voltage>,
        i_mpp: Measurement<Current>,
    ) -> Self {
        ElectricalCharacteristics {
            p_max,
            v_oc,
            i_sc,
            v_mpp,
            i_mpp,
            stated_efficiency: None,
            power_tolerance: None,
        }
    }

    /// Record the efficiency printed on the datasheet.
    #[inline]
    #[must_use]
    pub fn with_stated_efficiency(mut self, efficiency: Measurement<Ratio>) -> Self {
        self.stated_efficiency = Some(efficiency);
        self
    }

    /// Record the production tolerance.
    #[inline]
    #[must_use]
    pub fn with_power_tolerance(mut self, tolerance: PowerTolerance) -> Self {
        self.power_tolerance = Some(tolerance);
        self
    }

    /// The power implied by the maximum-power-point voltage and current.
    ///
    /// Compared against the stated `p_max` by `INV-ELE-004`. Datasheets
    /// round, so a small discrepancy is expected; a large one signals a
    /// faulty record.
    #[inline]
    pub fn implied_p_max(&self) -> Power {
        self.v_mpp.get() * self.i_mpp.get()
    }

    /// The fill factor, `p_max / (v_oc * i_sc)`.
    ///
    /// Returns a bare `f64` rather than a [`Ratio`] on purpose:
    /// `Ratio::new` panics outside `[0, 1]`, and a fill factor above 1 is
    /// exactly the corrupt-data case `INV-ELE-006` exists to detect. A
    /// clamped or panicking type would destroy the evidence before the
    /// validator could report it.
    #[inline]
    pub fn fill_factor(&self) -> f64 {
        let denominator = self.v_oc.get().volts() * self.i_sc.get().amperes();
        if denominator == 0.0 {
            f64::NAN
        } else {
            self.p_max.get().watts() / denominator
        }
    }
}

/// Safety and protection limits declared by the manufacturer.
///
/// **References**: IEC 61730-1/2 (module safety qualification),
/// IEC 62548 (PV array design requirements).
#[derive(Debug, Clone, PartialEq)]
pub struct ElectricalLimits {
    /// Maximum permissible system voltage. Constrains the longest
    /// admissible string.
    pub max_system_voltage: Measurement<Voltage>,
    /// Maximum series fuse rating, if stated.
    pub max_series_fuse: Option<Measurement<Current>>,
    /// Maximum reverse current the module withstands, if stated.
    pub max_reverse_current: Option<Measurement<Current>>,
    /// Lowest declared operating temperature, if stated.
    pub min_operating_temperature: Option<Measurement<Temperature>>,
    /// Highest declared operating temperature, if stated.
    pub max_operating_temperature: Option<Measurement<Temperature>>,
}

impl ElectricalLimits {
    /// Construct limits from the mandatory maximum system voltage.
    #[inline]
    pub fn new(max_system_voltage: Measurement<Voltage>) -> Self {
        ElectricalLimits {
            max_system_voltage,
            max_series_fuse: None,
            max_reverse_current: None,
            min_operating_temperature: None,
            max_operating_temperature: None,
        }
    }

    /// Record the maximum series fuse rating.
    #[inline]
    #[must_use]
    pub fn with_max_series_fuse(mut self, fuse: Measurement<Current>) -> Self {
        self.max_series_fuse = Some(fuse);
        self
    }

    /// Record the maximum reverse current.
    #[inline]
    #[must_use]
    pub fn with_max_reverse_current(mut self, current: Measurement<Current>) -> Self {
        self.max_reverse_current = Some(current);
        self
    }

    /// Record the declared operating temperature range.
    #[inline]
    #[must_use]
    pub fn with_operating_temperature_range(
        mut self,
        min: Measurement<Temperature>,
        max: Measurement<Temperature>,
    ) -> Self {
        self.min_operating_temperature = Some(min);
        self.max_operating_temperature = Some(max);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn characteristics(
        p: f64,
        voc: f64,
        isc: f64,
        vmpp: f64,
        impp: f64,
    ) -> ElectricalCharacteristics {
        ElectricalCharacteristics::new(
            Measurement::stated(Power::from_watts(p)),
            Measurement::stated(Voltage::from_volts(voc)),
            Measurement::stated(Current::from_amperes(isc)),
            Measurement::stated(Voltage::from_volts(vmpp)),
            Measurement::stated(Current::from_amperes(impp)),
        )
    }

    #[test]
    fn implied_power_is_the_maximum_power_point_product() {
        let e = characteristics(440.0, 49.5, 11.3, 41.5, 10.6);
        assert!((e.implied_p_max().watts() - 439.9).abs() < 1e-9);
    }

    #[test]
    fn fill_factor_is_computed_from_the_three_quantities() {
        let e = characteristics(440.0, 49.5, 11.3, 41.5, 10.6);
        let expected = 440.0 / (49.5 * 11.3);
        assert!((e.fill_factor() - expected).abs() < 1e-12);
    }

    #[test]
    fn fill_factor_above_one_is_representable_not_clamped() {
        // Corrupt data must survive far enough to be reported by
        // INV-ELE-006 rather than panicking inside a clamped type.
        let e = characteristics(1000.0, 10.0, 10.0, 9.0, 9.0);
        assert!(e.fill_factor() > 1.0);
    }

    #[test]
    fn fill_factor_of_a_degenerate_module_is_not_a_panic() {
        let e = characteristics(100.0, 0.0, 0.0, 0.0, 0.0);
        assert!(e.fill_factor().is_nan());
    }

    #[test]
    fn asymmetric_tolerance_keeps_both_bounds() {
        let t = PowerTolerance::Asymmetric {
            minimum: Power::from_watts(440.0),
            maximum: Power::from_watts(445.0),
        };
        match t {
            PowerTolerance::Asymmetric { minimum, maximum } => {
                assert!((minimum.watts() - 440.0).abs() < 1e-12);
                assert!((maximum.watts() - 445.0).abs() < 1e-12);
            }
            PowerTolerance::Symmetric { .. } => panic!("expected an asymmetric tolerance"),
        }
    }

    #[test]
    fn symmetric_tolerance_is_a_relative_band() {
        let t = PowerTolerance::Symmetric {
            relative: Ratio::new(0.03),
        };
        match t {
            PowerTolerance::Symmetric { relative } => {
                assert!((relative.value() - 0.03).abs() < 1e-12);
            }
            PowerTolerance::Asymmetric { .. } => panic!("expected a symmetric tolerance"),
        }
    }

    #[test]
    fn limits_record_optional_protections() {
        let l = ElectricalLimits::new(Measurement::stated(Voltage::from_volts(1500.0)))
            .with_max_series_fuse(Measurement::stated(Current::from_amperes(20.0)))
            .with_operating_temperature_range(
                Measurement::stated(Temperature::from_celsius(-40.0)),
                Measurement::stated(Temperature::from_celsius(85.0)),
            );
        assert!((l.max_system_voltage.get().volts() - 1500.0).abs() < 1e-12);
        assert!((l.max_series_fuse.unwrap().get().amperes() - 20.0).abs() < 1e-12);
        assert!((l.min_operating_temperature.unwrap().get().celsius() - (-40.0)).abs() < 1e-9);
    }
}
