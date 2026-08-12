/// The epistemic nature of a value: where its authority comes from, and
/// therefore how much weight a downstream model may put on it.
///
/// **Why this exists.** A PV module's open-circuit voltage may come from
/// a manufacturer datasheet (a nominal figure for a product type), from
/// an IEC 61853 test report (a measurement on one specimen, with a
/// quantifiable uncertainty), or from a user typing a remembered value.
/// These are the same physical quantity with radically different
/// authority. Collapsing them into a bare `f64` would let an estimate be
/// presented as a measurement.
///
/// **Fixed versus variable classification.** Some fields have a class
/// fixed at design time: a manufacturer name is always
/// [`DataClass::Metadata`], a derived area is always
/// [`DataClass::Derived`], a De Soto coefficient is always
/// [`DataClass::Fitted`]. Others — most electrical, geometric and thermal
/// quantities — take their class from the provenance of the particular
/// value. That is why the class travels with the value rather than living
/// in a static table.
///
/// **Reference**: `docs/design/27-specification-finale-pv-module.md` §1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DataClass {
    /// **A** — published by the manufacturer on a datasheet. A *nominal*
    /// value for a product type, not a measurement of one specimen.
    /// Contractually binding, commercially motivated.
    ManufacturerData,
    /// **B** — the result of a test on a physical specimen following a
    /// normative method (IEC 60904-1, IEC 61853). Carries a quantifiable
    /// metrological uncertainty.
    Measured,
    /// **C** — fixed by a standard rather than measured: 1000 W/m²,
    /// 25 °C, AM1.5G. Exact by definition.
    Normative,
    /// **D** — computed exactly from other fields of the model. Has no
    /// authority of its own; it inherits that of its inputs.
    Derived,
    /// **E** — supplied by judgement, without measurement or
    /// publication. Requires a recorded rationale.
    Estimated,
    /// **F** — obtained by regression against measurements, specific to
    /// one model (De Soto's five parameters, the SAPM coefficients).
    /// Meaningless outside that model, and never a physical property of
    /// the module.
    Fitted,
    /// **G** — a constant or coefficient defining a model itself, not
    /// specific to any module.
    ModelParameter,
    /// **H** — not a physical quantity at all: names, references, dates,
    /// certifications.
    Metadata,
}

impl DataClass {
    /// The single-letter code used in the specification's tables
    /// (`docs/design/27-specification-finale-pv-module.md` §1.1).
    #[inline]
    pub const fn code(self) -> char {
        match self {
            DataClass::ManufacturerData => 'A',
            DataClass::Measured => 'B',
            DataClass::Normative => 'C',
            DataClass::Derived => 'D',
            DataClass::Estimated => 'E',
            DataClass::Fitted => 'F',
            DataClass::ModelParameter => 'G',
            DataClass::Metadata => 'H',
        }
    }

    /// Whether this class denotes a quantity obtained from a physical
    /// measurement, as opposed to one declared, derived, fitted or
    /// guessed. Used by `INV-PRV-003` to prevent a fitted parameter from
    /// being presented as a measurement.
    #[inline]
    pub const fn is_measurement(self) -> bool {
        matches!(self, DataClass::Measured)
    }

    /// Whether a recorded rationale is mandatory for this class. Only
    /// [`DataClass::Estimated`] requires one: an estimate without a
    /// stated reason is exactly what the project's transparency
    /// principle forbids.
    #[inline]
    pub const fn requires_rationale(self) -> bool {
        matches!(self, DataClass::Estimated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_the_specification_letters() {
        assert_eq!(DataClass::ManufacturerData.code(), 'A');
        assert_eq!(DataClass::Measured.code(), 'B');
        assert_eq!(DataClass::Normative.code(), 'C');
        assert_eq!(DataClass::Derived.code(), 'D');
        assert_eq!(DataClass::Estimated.code(), 'E');
        assert_eq!(DataClass::Fitted.code(), 'F');
        assert_eq!(DataClass::ModelParameter.code(), 'G');
        assert_eq!(DataClass::Metadata.code(), 'H');
    }

    #[test]
    fn only_measured_data_counts_as_a_measurement() {
        assert!(DataClass::Measured.is_measurement());
        for other in [
            DataClass::ManufacturerData,
            DataClass::Normative,
            DataClass::Derived,
            DataClass::Estimated,
            DataClass::Fitted,
            DataClass::ModelParameter,
            DataClass::Metadata,
        ] {
            assert!(
                !other.is_measurement(),
                "{other:?} must not pass as a measurement"
            );
        }
    }

    #[test]
    fn only_estimates_require_a_rationale() {
        assert!(DataClass::Estimated.requires_rationale());
        assert!(!DataClass::ManufacturerData.requires_rationale());
        assert!(!DataClass::Measured.requires_rationale());
    }
}
