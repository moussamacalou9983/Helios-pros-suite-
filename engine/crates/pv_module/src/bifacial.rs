use core_units::{Current, Power, Ratio, TemperatureCoefficient, Voltage};

use crate::provenance::Measurement;

/// A bifacial gain figure as advertised on a datasheet.
///
/// **A commercial claim, not a computable quantity.** The gain a bifacial
/// module actually delivers depends on the site's albedo, the structure
/// height, the row spacing and the installation geometry — none of which
/// is a property of the module. This type exists so the advertised figure
/// remains traceable while being structurally unusable in a calculation.
///
/// The real gain must be computed by the future `bifacial` engine from
/// the bifaciality factor, the site albedo and the installation geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct StatedBifacialGain {
    /// The advertised relative gain.
    pub gain: Ratio,
    /// The albedo the manufacturer assumed, if stated.
    pub reference_albedo: Option<Ratio>,
}

impl StatedBifacialGain {
    /// A note recorded alongside the value, stating plainly that it must
    /// not enter a simulation.
    pub const USAGE_NOTE: &'static str = "commercial figure -- must not be used in simulation; compute the gain from the \
         bifaciality factor, site albedo and installation geometry instead";

    /// Record an advertised gain.
    #[inline]
    pub const fn new(gain: Ratio) -> Self {
        StatedBifacialGain {
            gain,
            reference_albedo: None,
        }
    }

    /// Record the albedo the manufacturer assumed.
    #[inline]
    #[must_use]
    pub const fn with_reference_albedo(mut self, albedo: Ratio) -> Self {
        self.reference_albedo = Some(albedo);
        self
    }
}

/// Electrical characteristics measured on the module's rear face.
///
/// **Reference**: IEC TS 60904-1-2 (measurement of bifacial devices).
#[derive(Debug, Clone, PartialEq)]
pub struct RearSideParameters {
    /// Rear-side maximum power, if stated.
    pub p_max: Option<Measurement<Power>>,
    /// Rear-side open-circuit voltage, if stated.
    pub v_oc: Option<Measurement<Voltage>>,
    /// Rear-side short-circuit current, if stated.
    pub i_sc: Option<Measurement<Current>>,
    /// Rear-side maximum-power-point voltage, if stated.
    pub v_mpp: Option<Measurement<Voltage>>,
    /// Rear-side maximum-power-point current, if stated.
    pub i_mpp: Option<Measurement<Current>>,
    /// Rear-side temperature coefficient of maximum power, if stated.
    pub gamma_p_max: Option<Measurement<TemperatureCoefficient>>,
}

impl RearSideParameters {
    /// An empty set of rear-side parameters.
    #[inline]
    pub const fn empty() -> Self {
        RearSideParameters {
            p_max: None,
            v_oc: None,
            i_sc: None,
            v_mpp: None,
            i_mpp: None,
            gamma_p_max: None,
        }
    }

    /// Whether any rear-side quantity was recorded at all.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.p_max.is_none()
            && self.v_oc.is_none()
            && self.i_sc.is_none()
            && self.v_mpp.is_none()
            && self.i_mpp.is_none()
            && self.gamma_p_max.is_none()
    }
}

/// The module's bifacial properties.
///
/// **Only intrinsic module properties live here.** Rear irradiance
/// models, rear temperature models, albedo, row geometry, row spacing and
/// structure height are deliberately absent: they are properties of the
/// *installation*, not of the module, and belong to the future site and
/// bifacial engines. Putting a model choice inside a physical object's
/// descriptor would be the first breach of the project's founding
/// principle (`docs/design/27-specification-finale-pv-module.md` §6.2).
///
/// `is_bifacial` is retained even though the block's presence already
/// implies it: a CEC import yields a bifacial flag with **no** bifaciality
/// factor and no rear-side data, so without this field such a block would
/// be indistinguishable from an empty one
/// (`docs/design/26-validation-empirique-cec-sandia.md` §2.2).
#[derive(Debug, Clone, PartialEq)]
pub struct BifacialCharacteristics {
    /// Whether the module is bifacial. Always `true` when this block
    /// exists; kept explicit so a source that states only the flag
    /// remains distinguishable.
    pub is_bifacial: bool,
    /// The bifaciality factor, the ratio of rear to front efficiency.
    ///
    /// `Option` because the CEC library marks bifacial modules with a
    /// flag and supplies no factor. Absent means "bifacial, not
    /// characterised", never "monofacial".
    pub bifaciality_factor: Option<Measurement<Ratio>>,
    /// Rear-side electrical characteristics, if measured.
    pub rear_side_parameters: Option<RearSideParameters>,
    /// The fraction of the rear face shaded by the module's own frame and
    /// junction box. An intrinsic property of the module's construction,
    /// unlike the shading caused by the mounting structure.
    pub rear_shading_factor: Option<Measurement<Ratio>>,
    /// The standard the bifacial measurement followed, if stated.
    pub measurement_standard: Option<String>,
    /// The advertised bifacial gain, never usable in a calculation.
    pub stated_gain: Option<StatedBifacialGain>,
}

impl BifacialCharacteristics {
    /// A bifacial module about which nothing more is known than that it
    /// is bifacial — what a CEC import produces.
    #[inline]
    pub const fn flag_only() -> Self {
        BifacialCharacteristics {
            is_bifacial: true,
            bifaciality_factor: None,
            rear_side_parameters: None,
            rear_shading_factor: None,
            measurement_standard: None,
            stated_gain: None,
        }
    }

    /// A bifacial module with a known bifaciality factor.
    #[inline]
    pub fn with_factor(bifaciality_factor: Measurement<Ratio>) -> Self {
        BifacialCharacteristics {
            is_bifacial: true,
            bifaciality_factor: Some(bifaciality_factor),
            rear_side_parameters: None,
            rear_shading_factor: None,
            measurement_standard: None,
            stated_gain: None,
        }
    }

    /// Record rear-side electrical characteristics.
    #[inline]
    #[must_use]
    pub fn with_rear_side_parameters(mut self, parameters: RearSideParameters) -> Self {
        self.rear_side_parameters = Some(parameters);
        self
    }

    /// Record the rear shading caused by the module's own frame.
    #[inline]
    #[must_use]
    pub fn with_rear_shading_factor(mut self, factor: Measurement<Ratio>) -> Self {
        self.rear_shading_factor = Some(factor);
        self
    }

    /// Record the measurement standard followed.
    #[inline]
    #[must_use]
    pub fn with_measurement_standard(mut self, standard: impl Into<String>) -> Self {
        self.measurement_standard = Some(standard.into());
        self
    }

    /// Record the advertised bifacial gain.
    #[inline]
    #[must_use]
    pub fn with_stated_gain(mut self, gain: StatedBifacialGain) -> Self {
        self.stated_gain = Some(gain);
        self
    }

    /// Whether the module is bifacial but carries no quantitative
    /// characterisation — the CEC case, reported by `INV-BIF-001`.
    #[inline]
    pub fn is_uncharacterised(&self) -> bool {
        self.bifaciality_factor.is_none()
            && self
                .rear_side_parameters
                .as_ref()
                .is_none_or(RearSideParameters::is_empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flag_only_block_is_distinguishable_from_a_characterised_one() {
        let cec_style = BifacialCharacteristics::flag_only();
        assert!(cec_style.is_bifacial);
        assert!(cec_style.bifaciality_factor.is_none());
        assert!(cec_style.is_uncharacterised());

        let characterised =
            BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)));
        assert!(!characterised.is_uncharacterised());
        assert!((characterised.bifaciality_factor.unwrap().get().value() - 0.70).abs() < 1e-12);
    }

    #[test]
    fn rear_side_parameters_are_recorded_when_measured() {
        let rear = RearSideParameters {
            p_max: Some(Measurement::measured(Power::from_watts(300.0))),
            ..RearSideParameters::empty()
        };
        assert!(!rear.is_empty());
        let b = BifacialCharacteristics::flag_only().with_rear_side_parameters(rear);
        assert!(!b.is_uncharacterised());
    }

    #[test]
    fn an_empty_rear_side_block_leaves_the_module_uncharacterised() {
        let b = BifacialCharacteristics::flag_only()
            .with_rear_side_parameters(RearSideParameters::empty());
        assert!(b.is_uncharacterised());
    }

    #[test]
    fn the_advertised_gain_carries_its_usage_warning() {
        let g = StatedBifacialGain::new(Ratio::new(0.09)).with_reference_albedo(Ratio::new(0.25));
        assert!((g.gain.value() - 0.09).abs() < 1e-12);
        assert!((g.reference_albedo.unwrap().value() - 0.25).abs() < 1e-12);
        assert!(StatedBifacialGain::USAGE_NOTE.contains("must not be used in simulation"));
    }

    #[test]
    fn frame_shading_is_intrinsic_and_recordable() {
        let b = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.8)))
            .with_rear_shading_factor(Measurement::stated(Ratio::new(0.03)))
            .with_measurement_standard("IEC TS 60904-1-2");
        assert!((b.rear_shading_factor.unwrap().get().value() - 0.03).abs() < 1e-12);
        assert_eq!(b.measurement_standard.as_deref(), Some("IEC TS 60904-1-2"));
    }
}
