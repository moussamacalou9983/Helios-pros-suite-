/// A calendar date, used for publication dates and conflict-resolution
/// timestamps.
///
/// Deliberately minimal: `pv_module` needs to *record* a date, never to
/// compute with one. Calendar arithmetic lives in `core_math::julian`,
/// which this crate does not depend on (a datasheet publication date has
/// nothing to do with solar geometry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Year (Gregorian).
    pub year: i32,
    /// Month, 1-12.
    pub month: u8,
    /// Day of month, 1-31.
    pub day: u8,
}

impl CalendarDate {
    /// Construct a date. Performs range checking only; it does not verify
    /// that the day exists in the given month, because a malformed date
    /// in a source document is a data-quality observation to record, not
    /// a reason to reject the whole record.
    #[inline]
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
            Some(CalendarDate { year, month, day })
        } else {
            None
        }
    }
}

/// What kind of source a value came from.
///
/// This is the primary determinant of a value's [`crate::DataClass`]: a
/// datasheet yields manufacturer data, a test report yields measured
/// data, user input yields an estimate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceType {
    /// A manufacturer's published datasheet.
    ManufacturerDatasheet,
    /// A laboratory test report following a named standard.
    TestReport {
        /// The standard the test followed, e.g. `"IEC 61853-1"`.
        standard: Option<String>,
    },
    /// A publicly distributed database, such as the CEC or Sandia module
    /// libraries.
    PublicDatabase {
        /// The database's name, e.g. `"CEC"`, `"Sandia"`.
        name: String,
    },
    /// A proprietary or licensed database.
    ProprietaryDatabase {
        /// The database's name.
        name: String,
    },
    /// Entered by a user.
    UserInput,
    /// Computed by HeliosPro from other values.
    Derived,
}

/// How a value was obtained from its source.
///
/// Distinguishing an automated import from a manual transcription matters
/// for auditing: transcription and OCR introduce error modes that a
/// deterministic parser does not.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtractionMethod {
    /// Parsed by a named adapter at a known version.
    AutomatedImport {
        /// The adapter's name, e.g. `"CECAdapter"`.
        adapter: String,
        /// The adapter's version, so a defect can be traced to the code
        /// that produced the value.
        adapter_version: String,
    },
    /// Typed in by a human reading a document.
    ManualTranscription,
    /// Read by optical character recognition.
    OpticalCharacterRecognition,
    /// Computed by the engine from other values.
    ComputedByEngine,
    /// Not recorded.
    Unknown,
}

/// How much confidence is placed in a value, independently of any
/// metrological uncertainty.
///
/// Distinct from [`crate::Uncertainty`]: a value can carry a tight
/// measurement uncertainty and still deserve low confidence if, say, it
/// was transcribed from a poor scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
    /// Not stated.
    Unspecified,
    /// Low confidence.
    Low,
    /// Medium confidence.
    Medium,
    /// High confidence.
    High,
}

/// Identifies the concrete source a value came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIdentifier {
    /// A human-readable name for the source.
    pub name: String,
    /// An optional stable identifier within that source (a database key,
    /// a document number).
    pub identifier: Option<String>,
}

impl SourceIdentifier {
    /// Construct a source identifier from a name alone.
    #[inline]
    pub fn named(name: impl Into<String>) -> Self {
        SourceIdentifier {
            name: name.into(),
            identifier: None,
        }
    }

    /// Attach a stable identifier within the source.
    #[inline]
    #[must_use]
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }
}

/// Where a value came from, in enough detail to audit it later.
///
/// **Immutability.** All fields are private and exposed through
/// accessors. A `Provenance` is built once and thereafter modified only
/// through the explicit `with_*` methods, each of which consumes and
/// returns the value rather than mutating in place. This satisfies the
/// specification's requirement that provenance be immutable after
/// creation except through an explicit update mechanism
/// (`docs/design/27-specification-finale-pv-module.md` §2).
///
/// Only `source_type` and `extraction_method` are mandatory: without
/// them the value's [`crate::DataClass`] could not be determined, which
/// is the minimum needed to keep a measurement distinguishable from an
/// estimate. Everything else is optional and may be filled in later.
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    source: Option<SourceIdentifier>,
    source_type: SourceType,
    source_version: Option<String>,
    publication_date: Option<CalendarDate>,
    manufacturer: Option<String>,
    datasheet_reference: Option<String>,
    page: Option<u32>,
    extraction_method: ExtractionMethod,
    confidence: Option<Confidence>,
}

impl Provenance {
    /// Construct a provenance from the two mandatory elements.
    #[inline]
    pub fn new(source_type: SourceType, extraction_method: ExtractionMethod) -> Self {
        Provenance {
            source: None,
            source_type,
            source_version: None,
            publication_date: None,
            manufacturer: None,
            datasheet_reference: None,
            page: None,
            extraction_method,
            confidence: None,
        }
    }

    /// The concrete source, if recorded.
    #[inline]
    pub fn source(&self) -> Option<&SourceIdentifier> {
        self.source.as_ref()
    }

    /// The kind of source. Always present.
    #[inline]
    pub fn source_type(&self) -> &SourceType {
        &self.source_type
    }

    /// The source's version or vintage, if recorded.
    #[inline]
    pub fn source_version(&self) -> Option<&str> {
        self.source_version.as_deref()
    }

    /// The source's publication date, if recorded.
    #[inline]
    pub fn publication_date(&self) -> Option<CalendarDate> {
        self.publication_date
    }

    /// The declaring manufacturer, if recorded.
    #[inline]
    pub fn manufacturer(&self) -> Option<&str> {
        self.manufacturer.as_deref()
    }

    /// The datasheet or document reference, if recorded.
    #[inline]
    pub fn datasheet_reference(&self) -> Option<&str> {
        self.datasheet_reference.as_deref()
    }

    /// The page within the source document, if recorded.
    #[inline]
    pub fn page(&self) -> Option<u32> {
        self.page
    }

    /// How the value was extracted. Always present.
    #[inline]
    pub fn extraction_method(&self) -> &ExtractionMethod {
        &self.extraction_method
    }

    /// The confidence placed in the value, if recorded.
    #[inline]
    pub fn confidence(&self) -> Option<Confidence> {
        self.confidence
    }

    /// The [`crate::DataClass`] implied by this provenance's source type.
    ///
    /// This is what makes classification a property of the value rather
    /// than of the field: the same quantity classifies differently
    /// depending on where it came from.
    #[inline]
    pub fn implied_class(&self) -> crate::DataClass {
        match self.source_type {
            SourceType::ManufacturerDatasheet => crate::DataClass::ManufacturerData,
            SourceType::TestReport { .. } => crate::DataClass::Measured,
            // A public or proprietary database redistributes values whose
            // original authority is the manufacturer's, unless the record
            // is itself a fitted parameter set -- which is classified at
            // the point of construction, not inferred here.
            SourceType::PublicDatabase { .. } | SourceType::ProprietaryDatabase { .. } => {
                crate::DataClass::ManufacturerData
            }
            SourceType::UserInput => crate::DataClass::Estimated,
            SourceType::Derived => crate::DataClass::Derived,
        }
    }

    /// Record the concrete source.
    #[inline]
    #[must_use]
    pub fn with_source(mut self, source: SourceIdentifier) -> Self {
        self.source = Some(source);
        self
    }

    /// Record the source's version or vintage.
    #[inline]
    #[must_use]
    pub fn with_source_version(mut self, version: impl Into<String>) -> Self {
        self.source_version = Some(version.into());
        self
    }

    /// Record the source's publication date.
    #[inline]
    #[must_use]
    pub fn with_publication_date(mut self, date: CalendarDate) -> Self {
        self.publication_date = Some(date);
        self
    }

    /// Record the declaring manufacturer.
    #[inline]
    #[must_use]
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }

    /// Record the datasheet or document reference.
    #[inline]
    #[must_use]
    pub fn with_datasheet_reference(mut self, reference: impl Into<String>) -> Self {
        self.datasheet_reference = Some(reference.into());
        self
    }

    /// Record the page within the source document.
    #[inline]
    #[must_use]
    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Record the confidence placed in the value.
    #[inline]
    #[must_use]
    pub fn with_confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = Some(confidence);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataClass;

    #[test]
    fn calendar_date_rejects_impossible_month_and_day() {
        assert!(CalendarDate::new(2024, 13, 1).is_none());
        assert!(CalendarDate::new(2024, 0, 1).is_none());
        assert!(CalendarDate::new(2024, 1, 32).is_none());
        assert!(CalendarDate::new(2024, 1, 0).is_none());
        assert!(CalendarDate::new(2024, 6, 21).is_some());
    }

    #[test]
    fn mandatory_fields_are_always_present() {
        let p = Provenance::new(SourceType::UserInput, ExtractionMethod::ManualTranscription);
        assert_eq!(*p.source_type(), SourceType::UserInput);
        assert_eq!(
            *p.extraction_method(),
            ExtractionMethod::ManualTranscription
        );
    }

    #[test]
    fn source_type_determines_the_implied_class() {
        let datasheet = Provenance::new(
            SourceType::ManufacturerDatasheet,
            ExtractionMethod::ManualTranscription,
        );
        assert_eq!(datasheet.implied_class(), DataClass::ManufacturerData);

        let report = Provenance::new(
            SourceType::TestReport {
                standard: Some("IEC 61853-1".to_string()),
            },
            ExtractionMethod::AutomatedImport {
                adapter: "IEC61853Adapter".to_string(),
                adapter_version: "0.2.0".to_string(),
            },
        );
        assert_eq!(report.implied_class(), DataClass::Measured);

        let typed = Provenance::new(SourceType::UserInput, ExtractionMethod::ManualTranscription);
        assert_eq!(typed.implied_class(), DataClass::Estimated);
    }

    #[test]
    fn the_same_quantity_classifies_differently_by_source() {
        // The point of per-value classification: one physical quantity,
        // three authorities.
        let from_datasheet = Provenance::new(
            SourceType::ManufacturerDatasheet,
            ExtractionMethod::ManualTranscription,
        );
        let from_lab = Provenance::new(
            SourceType::TestReport { standard: None },
            ExtractionMethod::AutomatedImport {
                adapter: "a".to_string(),
                adapter_version: "1".to_string(),
            },
        );
        let from_user =
            Provenance::new(SourceType::UserInput, ExtractionMethod::ManualTranscription);

        assert_ne!(from_datasheet.implied_class(), from_lab.implied_class());
        assert_ne!(from_lab.implied_class(), from_user.implied_class());
    }

    #[test]
    fn optional_fields_are_recorded_through_explicit_updates() {
        let p = Provenance::new(
            SourceType::PublicDatabase {
                name: "CEC".to_string(),
            },
            ExtractionMethod::AutomatedImport {
                adapter: "CECAdapter".to_string(),
                adapter_version: "0.2.0".to_string(),
            },
        )
        .with_source(SourceIdentifier::named("CEC module library").with_identifier("row-4211"))
        .with_source_version("2019-03-05")
        .with_publication_date(CalendarDate::new(2019, 3, 5).unwrap())
        .with_manufacturer("Example Manufacturer")
        .with_datasheet_reference("DS-1234 rev C")
        .with_page(2)
        .with_confidence(Confidence::High);

        assert_eq!(p.source().unwrap().name, "CEC module library");
        assert_eq!(p.source().unwrap().identifier.as_deref(), Some("row-4211"));
        assert_eq!(p.source_version(), Some("2019-03-05"));
        assert_eq!(p.publication_date().unwrap().year, 2019);
        assert_eq!(p.manufacturer(), Some("Example Manufacturer"));
        assert_eq!(p.datasheet_reference(), Some("DS-1234 rev C"));
        assert_eq!(p.page(), Some(2));
        assert_eq!(p.confidence(), Some(Confidence::High));
    }

    #[test]
    fn confidence_orders_from_unspecified_to_high() {
        assert!(Confidence::Unspecified < Confidence::Low);
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
    }
}
