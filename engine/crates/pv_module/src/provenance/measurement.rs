use super::{Confidence, DataClass, Provenance};

/// Whether an uncertainty is expressed in the measured quantity's own
/// unit or as a fraction of the measured value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncertaintyKind {
    /// Expressed in the quantity's own unit (volts, watts, metres).
    Absolute,
    /// Expressed as a dimensionless fraction of the value (0.02 = 2 %).
    Relative,
}

/// How an uncertainty component was evaluated, per the GUM's two
/// categories.
///
/// **Reference**: JCGM 100:2008 (GUM), §4.2 (Type A) and §4.3 (Type B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncertaintyEvaluation {
    /// Type A — by statistical analysis of repeated observations.
    TypeA,
    /// Type B — by any other means: calibration certificate,
    /// manufacturer specification, informed judgement.
    TypeB,
    /// A combination of Type A and Type B components.
    Combined,
}

/// A measurement uncertainty, expressed per the GUM.
///
/// **Reference**: JCGM 100:2008, *Evaluation of measurement data — Guide
/// to the expression of uncertainty in measurement* (BIPM/ISO/IEC).
/// Standard uncertainty `u`; expanded uncertainty `U = k·u` where `k` is
/// the coverage factor, conventionally 2 for roughly 95 % coverage under
/// an approximately normal distribution (GUM §6.2).
///
/// **This is not a production tolerance and not a degradation rate.**
/// Those are separate dispersions applying to different objects (a
/// manufacturing lot, and time) and are held elsewhere in the model. See
/// `docs/design/25-specification-pv-module.md` §8.1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uncertainty {
    value: f64,
    kind: UncertaintyKind,
    coverage_factor: Option<f64>,
    evaluation: Option<UncertaintyEvaluation>,
}

impl Uncertainty {
    /// A standard uncertainty `u` (coverage factor 1, left unstated) in
    /// the quantity's own unit.
    #[inline]
    pub const fn standard_absolute(value: f64) -> Self {
        Uncertainty {
            value,
            kind: UncertaintyKind::Absolute,
            coverage_factor: None,
            evaluation: None,
        }
    }

    /// A standard uncertainty `u` as a fraction of the measured value.
    #[inline]
    pub const fn standard_relative(fraction: f64) -> Self {
        Uncertainty {
            value: fraction,
            kind: UncertaintyKind::Relative,
            coverage_factor: None,
            evaluation: None,
        }
    }

    /// An expanded uncertainty `U = k·u` in the quantity's own unit.
    #[inline]
    pub const fn expanded_absolute(value: f64, coverage_factor: f64) -> Self {
        Uncertainty {
            value,
            kind: UncertaintyKind::Absolute,
            coverage_factor: Some(coverage_factor),
            evaluation: None,
        }
    }

    /// An expanded uncertainty `U = k·u` as a fraction of the measured
    /// value.
    #[inline]
    pub const fn expanded_relative(fraction: f64, coverage_factor: f64) -> Self {
        Uncertainty {
            value: fraction,
            kind: UncertaintyKind::Relative,
            coverage_factor: Some(coverage_factor),
            evaluation: None,
        }
    }

    /// Record how the uncertainty was evaluated.
    #[inline]
    #[must_use]
    pub const fn with_evaluation(mut self, evaluation: UncertaintyEvaluation) -> Self {
        self.evaluation = Some(evaluation);
        self
    }

    /// The uncertainty's magnitude, in whichever form
    /// [`Uncertainty::kind`] indicates.
    #[inline]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Whether the magnitude is absolute or relative.
    #[inline]
    pub const fn kind(self) -> UncertaintyKind {
        self.kind
    }

    /// The coverage factor `k`, if this is an expanded uncertainty.
    /// `None` means a standard uncertainty (`k = 1`).
    #[inline]
    pub const fn coverage_factor(self) -> Option<f64> {
        self.coverage_factor
    }

    /// How the uncertainty was evaluated, if recorded.
    #[inline]
    pub const fn evaluation(self) -> Option<UncertaintyEvaluation> {
        self.evaluation
    }

    /// The standard uncertainty `u`, obtained by dividing an expanded
    /// uncertainty by its coverage factor.
    ///
    /// Returns `None` if the coverage factor is zero, since `u` is then
    /// undefined rather than infinite.
    #[inline]
    pub fn standard_uncertainty(self) -> Option<f64> {
        match self.coverage_factor {
            None => Some(self.value),
            Some(k) if k != 0.0 => Some(self.value / k),
            Some(_) => None,
        }
    }
}

/// A physical value together with everything needed to judge how much to
/// trust it: its epistemic class, where it came from, its measurement
/// uncertainty, and the confidence placed in it.
///
/// **Why a wrapper rather than parallel tables.** Classification is not a
/// property of the field but of the value — the same open-circuit voltage
/// is manufacturer data from a datasheet, measured data from a test
/// report, and an estimate when typed in by hand. Provenance is likewise
/// heterogeneous within one module: its STC point may come from a
/// datasheet while its performance matrix comes from a laboratory. Three
/// blocks keyed by field name could not be checked by the compiler and
/// would fall out of step the first time a field was added. See
/// `docs/adr/0006-measurement-provenance-conflicts.md`, Decision 1, which
/// supersedes the opposite choice recorded in ADR-0005.
///
/// **Scope.** Only physical quantities coming from an external source are
/// wrapped. Metadata (class H), derived quantities (class D, whose class
/// is known and whose provenance is that of their inputs) and
/// classification enums stay bare.
///
/// **Immutability.** Fields are private; the `with_*` methods consume and
/// return the value rather than mutating in place, so a measurement's
/// provenance cannot be altered in place after construction.
/// **Why the provenance is boxed.** A [`Provenance`] holds nine fields,
/// six of them heap-allocated strings, and comes to roughly 230 bytes.
/// Inlining it would make every `Measurement<T>` about 300 bytes wide
/// regardless of `T`, and a module holds around twenty of them. The box
/// costs one pointer indirection on a field that is read when auditing,
/// never in a computation loop, and is invisible at the API boundary
/// because [`Measurement::provenance`] already hands back a reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement<T> {
    value: T,
    classification: DataClass,
    provenance: Option<Box<Provenance>>,
    uncertainty: Option<Uncertainty>,
    confidence: Option<Confidence>,
    rationale: Option<String>,
}

impl<T> Measurement<T> {
    /// Construct a measurement with an explicit classification and no
    /// provenance yet.
    #[inline]
    pub const fn new(value: T, classification: DataClass) -> Self {
        Measurement {
            value,
            classification,
            provenance: None,
            uncertainty: None,
            confidence: None,
            rationale: None,
        }
    }

    /// A value taken from a manufacturer datasheet (class A) — the most
    /// common case, hence the short name.
    #[inline]
    pub const fn stated(value: T) -> Self {
        Measurement::new(value, DataClass::ManufacturerData)
    }

    /// A value obtained by measurement under a normative method
    /// (class B).
    #[inline]
    pub const fn measured(value: T) -> Self {
        Measurement::new(value, DataClass::Measured)
    }

    /// A value fixed by a standard (class C).
    #[inline]
    pub const fn normative(value: T) -> Self {
        Measurement::new(value, DataClass::Normative)
    }

    /// A value computed from other fields (class D).
    #[inline]
    pub const fn derived(value: T) -> Self {
        Measurement::new(value, DataClass::Derived)
    }

    /// A value supplied by judgement (class E). A rationale is mandatory:
    /// an estimate without a recorded reason is what `INV-PRV-002`
    /// rejects.
    #[inline]
    pub fn estimated(value: T, rationale: impl Into<String>) -> Self {
        Measurement {
            value,
            classification: DataClass::Estimated,
            provenance: None,
            uncertainty: None,
            confidence: None,
            rationale: Some(rationale.into()),
        }
    }

    /// Borrow the underlying value.
    #[inline]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consume the measurement, yielding the underlying value.
    #[inline]
    pub fn into_value(self) -> T {
        self.value
    }

    /// The value's epistemic class.
    #[inline]
    pub const fn classification(&self) -> DataClass {
        self.classification
    }

    /// Where the value came from, if recorded.
    #[inline]
    pub fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_deref()
    }

    /// The value's measurement uncertainty, if recorded.
    #[inline]
    pub const fn uncertainty(&self) -> Option<Uncertainty> {
        self.uncertainty
    }

    /// The confidence placed in the value, if recorded.
    #[inline]
    pub const fn confidence(&self) -> Option<Confidence> {
        self.confidence
    }

    /// The recorded rationale, mandatory for an estimated value.
    #[inline]
    pub fn rationale(&self) -> Option<&str> {
        self.rationale.as_deref()
    }

    /// Attach a provenance. This is the explicit update mechanism the
    /// specification requires: it consumes and returns the measurement
    /// rather than mutating it.
    #[inline]
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(Box::new(provenance));
        self
    }

    /// Attach a measurement uncertainty.
    #[inline]
    #[must_use]
    pub fn with_uncertainty(mut self, uncertainty: Uncertainty) -> Self {
        self.uncertainty = Some(uncertainty);
        self
    }

    /// Attach a confidence level.
    #[inline]
    #[must_use]
    pub fn with_confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Attach a rationale.
    #[inline]
    #[must_use]
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }

    /// Apply a function to the wrapped value, preserving every
    /// traceability attribute. Used when converting units at an adapter
    /// boundary without losing where the value came from.
    #[inline]
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Measurement<U> {
        Measurement {
            value: f(self.value),
            classification: self.classification,
            provenance: self.provenance,
            uncertainty: self.uncertainty,
            confidence: self.confidence,
            rationale: self.rationale,
        }
    }
}

impl<T: Copy> Measurement<T> {
    /// Copy the underlying value out. Available for the `Copy` unit types
    /// of `core_units`, where borrowing would be needless friction.
    #[inline]
    pub const fn get(&self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{ExtractionMethod, SourceType};
    use core_units::Voltage;

    #[test]
    fn constructors_set_the_matching_class() {
        assert_eq!(
            Measurement::stated(Voltage::from_volts(44.0)).classification(),
            DataClass::ManufacturerData
        );
        assert_eq!(
            Measurement::measured(Voltage::from_volts(44.0)).classification(),
            DataClass::Measured
        );
        assert_eq!(
            Measurement::normative(Voltage::from_volts(44.0)).classification(),
            DataClass::Normative
        );
        assert_eq!(
            Measurement::derived(Voltage::from_volts(44.0)).classification(),
            DataClass::Derived
        );
    }

    #[test]
    fn an_estimate_carries_its_rationale() {
        let m = Measurement::estimated(Voltage::from_volts(44.0), "read from a photograph");
        assert_eq!(m.classification(), DataClass::Estimated);
        assert_eq!(m.rationale(), Some("read from a photograph"));
    }

    #[test]
    fn value_is_preserved_and_accessible() {
        let m = Measurement::stated(Voltage::from_volts(44.0));
        assert!((m.get().volts() - 44.0).abs() < 1e-12);
        assert!((m.value().volts() - 44.0).abs() < 1e-12);
        assert!((m.into_value().volts() - 44.0).abs() < 1e-12);
    }

    #[test]
    fn attributes_attach_without_mutation_in_place() {
        let base = Measurement::stated(Voltage::from_volts(44.0));
        let enriched = base
            .clone()
            .with_uncertainty(Uncertainty::expanded_relative(0.03, 2.0))
            .with_confidence(Confidence::High)
            .with_provenance(Provenance::new(
                SourceType::ManufacturerDatasheet,
                ExtractionMethod::ManualTranscription,
            ));

        // The original is untouched: the update mechanism is explicit.
        assert!(base.uncertainty().is_none());
        assert!(base.provenance().is_none());
        assert!(enriched.uncertainty().is_some());
        assert!(enriched.provenance().is_some());
        assert_eq!(enriched.confidence(), Some(Confidence::High));
    }

    #[test]
    fn map_preserves_traceability_across_a_unit_conversion() {
        let millivolts = Measurement::measured(44_000.0f64)
            .with_confidence(Confidence::Medium)
            .with_uncertainty(Uncertainty::standard_relative(0.01));
        let volts = millivolts.map(Voltage::from_millivolts);

        assert!((volts.get().volts() - 44.0).abs() < 1e-9);
        assert_eq!(volts.classification(), DataClass::Measured);
        assert_eq!(volts.confidence(), Some(Confidence::Medium));
        assert!(volts.uncertainty().is_some());
    }

    #[test]
    fn expanded_uncertainty_divides_by_its_coverage_factor() {
        let u = Uncertainty::expanded_relative(0.03, 2.0);
        assert!((u.standard_uncertainty().unwrap() - 0.015).abs() < 1e-12);
        assert_eq!(u.coverage_factor(), Some(2.0));
        assert_eq!(u.kind(), UncertaintyKind::Relative);
    }

    #[test]
    fn a_standard_uncertainty_has_no_coverage_factor() {
        let u = Uncertainty::standard_absolute(0.5);
        assert_eq!(u.coverage_factor(), None);
        assert!((u.standard_uncertainty().unwrap() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn a_zero_coverage_factor_leaves_the_standard_uncertainty_undefined() {
        let u = Uncertainty::expanded_absolute(0.5, 0.0);
        assert_eq!(u.standard_uncertainty(), None);
    }

    #[test]
    fn boxing_the_provenance_keeps_a_measurement_small() {
        // Guards the decision documented on `Measurement`: a module holds
        // around twenty of these, so inlining `Provenance` would cost
        // roughly 6 kB per module for a field read only when auditing.
        assert!(
            core::mem::size_of::<Provenance>() > 128,
            "if Provenance ever became small, the box would be unjustified"
        );
        assert!(
            core::mem::size_of::<Measurement<Voltage>>() < 128,
            "a Measurement is {} bytes wide",
            core::mem::size_of::<Measurement<Voltage>>()
        );
    }

    #[test]
    fn evaluation_type_is_recorded_when_stated() {
        let u = Uncertainty::standard_absolute(0.2).with_evaluation(UncertaintyEvaluation::TypeA);
        assert_eq!(u.evaluation(), Some(UncertaintyEvaluation::TypeA));
    }
}
