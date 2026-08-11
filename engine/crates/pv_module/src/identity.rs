use crate::provenance::Provenance;

/// A standard the module is certified against.
///
/// Metadata (class H): an attestation, not a physical quantity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Certification {
    /// IEC 61215 — design qualification and type approval.
    Iec61215,
    /// IEC 61730 — module safety qualification.
    Iec61730,
    /// IEC 61701 — salt mist corrosion testing.
    Iec61701,
    /// IEC 62716 — ammonia corrosion testing.
    Iec62716,
    /// IEC 61853 — performance testing and energy rating.
    Iec61853,
    /// A certification named by the source but not classified here.
    Other {
        /// The name as the source gave it.
        name: String,
    },
}

/// Who made the module, which model it is, and where that information
/// came from.
///
/// Every field here is metadata (class H) except `provenance`, which
/// records the authority of the identification itself.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleIdentity {
    /// The manufacturer's name. Two manufacturers may use the same model
    /// designation, so this is part of the identity, not decoration.
    pub manufacturer: String,
    /// The commercial model designation.
    pub model_name: String,
    /// An exact part number, if stated. Some databases index by it.
    pub part_number: Option<String>,
    /// The datasheet revision, if stated. Two revisions of one model
    /// carry different values.
    pub datasheet_revision: Option<String>,
    /// Whether the module is intended for building integration.
    ///
    /// `Option` rather than `bool` so that "the source does not say"
    /// stays distinct from "not building-integrated". The CEC library
    /// carries this flag on 164 of its 21 535 records
    /// (`docs/design/26-validation-empirique-cec-sandia.md` §2.5).
    pub building_integrated: Option<bool>,
    /// Where the identification came from.
    pub provenance: Option<Provenance>,
}

impl ModuleIdentity {
    /// Construct an identity from the two mandatory fields.
    #[inline]
    pub fn new(manufacturer: impl Into<String>, model_name: impl Into<String>) -> Self {
        ModuleIdentity {
            manufacturer: manufacturer.into(),
            model_name: model_name.into(),
            part_number: None,
            datasheet_revision: None,
            building_integrated: None,
            provenance: None,
        }
    }

    /// Record the exact part number.
    #[inline]
    #[must_use]
    pub fn with_part_number(mut self, part_number: impl Into<String>) -> Self {
        self.part_number = Some(part_number.into());
        self
    }

    /// Record the datasheet revision.
    #[inline]
    #[must_use]
    pub fn with_datasheet_revision(mut self, revision: impl Into<String>) -> Self {
        self.datasheet_revision = Some(revision.into());
        self
    }

    /// Record whether the module is building-integrated.
    #[inline]
    #[must_use]
    pub const fn with_building_integrated(mut self, building_integrated: bool) -> Self {
        self.building_integrated = Some(building_integrated);
        self
    }

    /// Record where the identification came from.
    #[inline]
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{ExtractionMethod, SourceType};

    #[test]
    fn the_two_mandatory_fields_are_always_present() {
        let id = ModuleIdentity::new("Example Manufacturer", "EX-440");
        assert_eq!(id.manufacturer, "Example Manufacturer");
        assert_eq!(id.model_name, "EX-440");
        assert!(id.part_number.is_none());
    }

    #[test]
    fn building_integration_distinguishes_unstated_from_false() {
        let unstated = ModuleIdentity::new("M", "X");
        assert_eq!(unstated.building_integrated, None);

        let stated_false = ModuleIdentity::new("M", "X").with_building_integrated(false);
        assert_eq!(stated_false.building_integrated, Some(false));

        let stated_true = ModuleIdentity::new("M", "X").with_building_integrated(true);
        assert_eq!(stated_true.building_integrated, Some(true));
    }

    #[test]
    fn optional_identification_is_recorded_explicitly() {
        let id = ModuleIdentity::new("M", "X")
            .with_part_number("PN-001")
            .with_datasheet_revision("rev C")
            .with_provenance(Provenance::new(
                SourceType::ManufacturerDatasheet,
                ExtractionMethod::ManualTranscription,
            ));
        assert_eq!(id.part_number.as_deref(), Some("PN-001"));
        assert_eq!(id.datasheet_revision.as_deref(), Some("rev C"));
        assert!(id.provenance.is_some());
    }

    #[test]
    fn certifications_admit_unclassified_names() {
        let other = Certification::Other {
            name: "UL 61730".to_string(),
        };
        assert_ne!(other, Certification::Iec61730);
    }
}
