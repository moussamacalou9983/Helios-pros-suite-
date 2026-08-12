//! The boundary between external module sources and this crate's domain
//! model.
//!
//! ## What lives here, and what deliberately does not
//!
//! This module defines **interfaces only**. It contains no parser, no file
//! format, no field mapping and no data. That is a deliberate boundary,
//! for three reasons stated in
//! `docs/design/27-specification-finale-pv-module.md` §12:
//!
//! - **The domain model must not know its sources.** If
//!   [`crate::PvModule`] were shaped by the CEC library's columns, every
//!   new source would deform it. The dependency runs one way: an adapter
//!   knows both its source and the domain model; the domain model knows
//!   neither.
//! - **No source is truth.** An adapter produces values *with their
//!   provenance*, so that a figure redistributed by a database keeps the
//!   authority of whoever originally measured or declared it.
//! - **Nothing is fetched.** No adapter may scrape a website or call a
//!   remote service. Sources are files or user input, handed to the
//!   adapter by the caller.
//!
//! This crate also takes no dependency on pvlib, on PVsyst, or on any
//! other suite. pvlib serves as an *external validation oracle* elsewhere
//! in the project; it is never a dependency of the engine.
//!
//! ## The five sources
//!
//! [`ADAPTER_CATALOGUE`] names them with their status. Four are specified
//! and awaiting real data; the fifth, PVsyst's `.PAN`, is **blocked** —
//! see [`AdapterStatus::Blocked`].
//!
//! ## Real data remains open
//!
//! The five manufacturer datasheets and the IEC 61853-1 grid required by
//! `docs/design/28-plan-tests-pv-module.md` are **not** supplied here, and
//! must not be invented: a fabricated datasheet would be an estimate
//! presented as manufacturer data, which is exactly what
//! [`crate::DataClass`] exists to prevent. These interfaces are what will
//! let that data be integrated when it is obtained.

use crate::provenance::{ExtractionMethod, Provenance, SourceType};
use crate::validation::{ValidationIssue, ValidationReport};
use crate::{ModuleBuildError, PvModule, PvModuleBuilder};

/// Whether an adapter may be implemented, and if not, why.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AdapterStatus {
    /// The source format is documented or directly inspectable, and the
    /// field mapping has been verified against real records.
    Specified,
    /// The field mapping is drafted but not yet verified against real
    /// records.
    AwaitingReferenceData,
    /// Implementation is forbidden until the stated condition is lifted.
    Blocked {
        /// Why, in terms a reviewer can act on.
        reason: &'static str,
    },
}

impl AdapterStatus {
    /// Whether an implementation may proceed.
    #[inline]
    pub const fn is_implementable(&self) -> bool {
        !matches!(self, AdapterStatus::Blocked { .. })
    }
}

/// A source this crate intends to read, with its status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterDescriptor {
    /// The adapter's name, recorded verbatim in every
    /// [`ExtractionMethod::AutomatedImport`] it produces.
    pub name: &'static str,
    /// What the source is.
    pub source: &'static str,
    /// Whether it may be implemented.
    pub status: AdapterStatus,
    /// Where the field mapping or the blocking decision is documented.
    pub reference: &'static str,
}

/// The five sources named by the specification, with their status.
///
/// `PANAdapter` is blocked: `.PAN` is PVsyst's format and is not, to our
/// knowledge, publicly specified as an open format. No implementation will
/// be undertaken before its legal status is established **and** a
/// dedicated ADR is written. This is consistent with the project's
/// founding rule forbidding any reuse of PVsyst, PV\*SOL or Archelios
/// Calc (`docs/design/27-specification-finale-pv-module.md` §12.4).
pub const ADAPTER_CATALOGUE: [AdapterDescriptor; 5] = [
    AdapterDescriptor {
        name: "CECAdapter",
        source: "California Energy Commission module library (CSV)",
        status: AdapterStatus::Specified,
        reference: "docs/design/26-validation-empirique-cec-sandia.md section 2",
    },
    AdapterDescriptor {
        name: "SandiaAdapter",
        source: "Sandia National Laboratories module database (CSV)",
        status: AdapterStatus::Specified,
        reference: "docs/design/26-validation-empirique-cec-sandia.md section 3",
    },
    AdapterDescriptor {
        name: "IEC61853Adapter",
        source: "IEC 61853-1 / -2 laboratory test report",
        status: AdapterStatus::AwaitingReferenceData,
        reference: "docs/design/28-plan-tests-pv-module.md; the grid is to be read from the standard",
    },
    AdapterDescriptor {
        name: "ManufacturerDatasheetAdapter",
        source: "manufacturer datasheet, transcribed or read by OCR",
        status: AdapterStatus::AwaitingReferenceData,
        reference: "docs/design/27-specification-finale-pv-module.md section 12",
    },
    AdapterDescriptor {
        name: "PANAdapter",
        source: "PVsyst .PAN file",
        status: AdapterStatus::Blocked {
            reason: "the .PAN format is not, to our knowledge, publicly specified as an open \
                     format; implementation requires its legal status to be established and a \
                     dedicated ADR",
        },
        reference: "docs/design/27-specification-finale-pv-module.md section 12.4",
    },
];

/// The descriptor for a named adapter, if it is one of the five.
pub fn describe_adapter(name: &str) -> Option<&'static AdapterDescriptor> {
    ADAPTER_CATALOGUE.iter().find(|d| d.name == name)
}

/// What an import produced: a module description if enough was found, and
/// every issue met along the way.
///
/// An adapter **never** silently repairs what it reads. A missing
/// mandatory field leaves [`ImportOutcome::builder`] empty rather than
/// yielding a default; an ambiguous one is reported through an issue —
/// see [`crate::validation::invariants::ambiguous_coefficient_basis`],
/// which exists because the CEC library mixes absolute and relative
/// temperature coefficients within one record and the basis must never be
/// guessed from a magnitude.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportOutcome {
    builder: Option<PvModuleBuilder>,
    issues: ValidationReport,
    provenance: Provenance,
}

impl ImportOutcome {
    /// An import that produced a module description.
    #[inline]
    pub fn imported(builder: PvModuleBuilder, provenance: Provenance) -> Self {
        ImportOutcome {
            builder: Some(builder),
            issues: ValidationReport::new(),
            provenance,
        }
    }

    /// An import that could not produce one, for the recorded reasons.
    #[inline]
    pub fn rejected(provenance: Provenance) -> Self {
        ImportOutcome {
            builder: None,
            issues: ValidationReport::new(),
            provenance,
        }
    }

    /// Record an issue met while reading the source.
    #[inline]
    #[must_use]
    pub fn with_issue(mut self, issue: ValidationIssue) -> Self {
        self.issues.push(issue);
        self
    }

    /// The module description, if the source carried enough to build one.
    #[inline]
    pub const fn builder(&self) -> Option<&PvModuleBuilder> {
        self.builder.as_ref()
    }

    /// Take the module description out of the outcome.
    #[inline]
    pub fn into_builder(self) -> Option<PvModuleBuilder> {
        self.builder
    }

    /// Everything the adapter noticed about the source record.
    #[inline]
    pub const fn issues(&self) -> &ValidationReport {
        &self.issues
    }

    /// Where the record came from. Always present: an import without
    /// provenance would produce values of unattributable authority.
    #[inline]
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Whether the source carried enough to attempt a build.
    #[inline]
    pub const fn is_imported(&self) -> bool {
        self.builder.is_some()
    }

    /// Validate and build the imported description, if there is one.
    ///
    /// The import issues are *not* merged into the module's own report:
    /// they concern the reading of the source, while the module's report
    /// concerns the module's physics. Callers that need both keep this
    /// outcome alongside the built module.
    ///
    /// # Errors
    ///
    /// Propagates [`ModuleBuildError`] from [`PvModuleBuilder::build`].
    /// Returns `Ok(None)` when the source carried too little to build.
    pub fn build(self) -> Result<Option<PvModule>, ModuleBuildError> {
        match self.builder {
            Some(builder) => builder.build().map(Some),
            None => Ok(None),
        }
    }
}

/// Reads modules from one external source into this crate's domain model.
///
/// The associated `Record` type is the adapter's own representation of one
/// source record — a parsed CSV row, a transcribed datasheet. It stays the
/// adapter's business: **the domain model must never be parameterised by
/// it**, which is why it appears here and nowhere in [`PvModule`].
///
/// Implementors must guarantee that every value they produce carries a
/// [`Provenance`] naming this adapter and its version, so that a defect
/// can be traced back to the code that produced the value.
pub trait ModuleSourceAdapter {
    /// The adapter's own representation of one source record.
    type Record;

    /// The adapter's name, matching its [`AdapterDescriptor::name`].
    fn name(&self) -> &'static str;

    /// The adapter's version, recorded in every value it produces so a
    /// defect is traceable to the code that introduced it.
    fn version(&self) -> &'static str;

    /// Convert one source record into a module description.
    ///
    /// Must not repair, default, or infer. What the source does not state,
    /// the outcome does not carry.
    fn import(&self, record: &Self::Record) -> ImportOutcome;

    /// The provenance every value from this adapter carries.
    ///
    /// Provided rather than required: the shape is the same for every
    /// adapter, and only the source type varies.
    fn provenance_for(&self, source_type: SourceType) -> Provenance {
        Provenance::new(
            source_type,
            ExtractionMethod::AutomatedImport {
                adapter: self.name().to_string(),
                adapter_version: self.version().to_string(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::Measurement;
    use crate::validation::{ValidationStatus, invariants};
    use crate::{
        CellArchitecture, CellConfiguration, CellTechnology, ElectricalCharacteristics,
        ElectricalLimits, ModuleDimensions, ModuleGeometry, ModuleIdentity, ModuleTechnology,
        ThermalCharacteristics,
    };
    use core_units::{Current, Length, Power, TemperatureCoefficient, Voltage};

    /// A minimal adapter over an already-parsed record, standing in for a
    /// real one. It reads nothing from disk and invents no values: the
    /// record is handed to it whole.
    struct StubAdapter;

    struct StubRecord {
        model: Option<&'static str>,
    }

    impl ModuleSourceAdapter for StubAdapter {
        type Record = StubRecord;

        fn name(&self) -> &'static str {
            "StubAdapter"
        }

        fn version(&self) -> &'static str {
            "0.1.0"
        }

        fn import(&self, record: &StubRecord) -> ImportOutcome {
            let provenance = self.provenance_for(SourceType::PublicDatabase {
                name: "stub".to_string(),
            });
            let Some(model) = record.model else {
                // The mandatory field is absent, so nothing is built. No
                // placeholder is substituted.
                return ImportOutcome::rejected(provenance).with_issue(ValidationIssue::new(
                    "INV-PRV-001",
                    ValidationStatus::Unknown,
                    vec!["model_name"],
                    "the record states no model designation, so no module can be identified",
                ));
            };
            let builder = PvModuleBuilder::new(
                ModuleIdentity::new("Stub", model),
                ModuleTechnology::new(
                    CellTechnology::MonocrystallineSilicon,
                    CellArchitecture::Unknown,
                ),
                CellConfiguration::discrete(72),
                ModuleGeometry::new(ModuleDimensions::new(
                    Measurement::stated(Length::from_meters(2.0)),
                    Measurement::stated(Length::from_meters(1.0)),
                )),
                ElectricalCharacteristics::new(
                    Measurement::stated(Power::from_watts(400.0)),
                    Measurement::stated(Voltage::from_volts(49.5)),
                    Measurement::stated(Current::from_amperes(10.4)),
                    Measurement::stated(Voltage::from_volts(41.5)),
                    Measurement::stated(Current::from_amperes(9.64)),
                ),
                ThermalCharacteristics::new(
                    Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(0.045)),
                    Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.27)),
                    Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.34)),
                ),
                ElectricalLimits::new(Measurement::stated(Voltage::from_volts(1500.0))),
            );
            ImportOutcome::imported(builder, provenance)
        }
    }

    #[test]
    fn the_catalogue_names_the_five_specified_sources() {
        assert_eq!(ADAPTER_CATALOGUE.len(), 5);
        for name in [
            "CECAdapter",
            "SandiaAdapter",
            "IEC61853Adapter",
            "ManufacturerDatasheetAdapter",
            "PANAdapter",
        ] {
            assert!(
                describe_adapter(name).is_some(),
                "{name} must be catalogued"
            );
        }
        assert!(describe_adapter("ScraperAdapter").is_none());
    }

    #[test]
    fn the_pan_adapter_is_blocked_with_a_stated_reason() {
        let pan = describe_adapter("PANAdapter").unwrap();
        assert!(!pan.status.is_implementable());
        match &pan.status {
            AdapterStatus::Blocked { reason } => assert!(reason.contains("legal status")),
            other => panic!("PAN must be blocked, found {other:?}"),
        }
        // Every other adapter may proceed.
        for d in ADAPTER_CATALOGUE.iter().filter(|d| d.name != "PANAdapter") {
            assert!(
                d.status.is_implementable(),
                "{} must be implementable",
                d.name
            );
        }
    }

    #[test]
    fn an_import_records_the_adapter_and_its_version_in_the_provenance() {
        let outcome = StubAdapter.import(&StubRecord {
            model: Some("EX-400"),
        });
        match outcome.provenance().extraction_method() {
            ExtractionMethod::AutomatedImport {
                adapter,
                adapter_version,
            } => {
                assert_eq!(adapter, "StubAdapter");
                assert_eq!(adapter_version, "0.1.0");
            }
            other => panic!("an automated import was expected, found {other:?}"),
        }
        assert!(outcome.is_imported());
    }

    #[test]
    fn a_missing_mandatory_field_yields_no_module_and_no_placeholder() {
        let outcome = StubAdapter.import(&StubRecord { model: None });
        assert!(!outcome.is_imported());
        assert!(outcome.builder().is_none());
        assert_eq!(outcome.issues().worst_status(), ValidationStatus::Unknown);
        assert!(outcome.clone().build().unwrap().is_none());
    }

    #[test]
    fn an_imported_record_builds_through_the_same_validation_gate() {
        let module = StubAdapter
            .import(&StubRecord {
                model: Some("EX-400"),
            })
            .build()
            .expect("a coherent record must build")
            .expect("a module was expected");
        assert_eq!(module.identity().model_name, "EX-400");
        assert!(!module.report().has_invalid());
    }

    #[test]
    fn an_ambiguous_coefficient_basis_is_a_conflict_not_a_guess() {
        // The behaviour an adapter must exhibit when its source does not
        // state whether a coefficient is absolute or relative.
        let issue = invariants::ambiguous_coefficient_basis("gamma_p_max", "-0.5072");
        assert_eq!(issue.status(), ValidationStatus::Conflict);
        assert_eq!(issue.invariant_id(), "INV-TMP-005");

        let outcome = ImportOutcome::rejected(Provenance::new(
            SourceType::PublicDatabase {
                name: "CEC".to_string(),
            },
            ExtractionMethod::AutomatedImport {
                adapter: "CECAdapter".to_string(),
                adapter_version: "0.0.0".to_string(),
            },
        ))
        .with_issue(issue);
        assert!(outcome.issues().contains("INV-TMP-005"));
        assert!(!outcome.is_imported());
    }
}
