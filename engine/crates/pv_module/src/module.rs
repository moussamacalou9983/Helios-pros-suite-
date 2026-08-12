use core_units::{Area, Ratio};

use crate::validation::invariants::{self, ALL_INVARIANT_IDS};
use crate::{
    BifacialCharacteristics, CellConfiguration, Certification, ConflictResolution,
    ElectricalCharacteristics, ElectricalLimits, ModuleGeometry, ModuleIdentity, ModuleTechnology,
    MountingForm, OperatingCondition, OperatingConditionKind, OperatingPoint,
    ThermalCharacteristics, ValidationIssue, ValidationReport, ValidationStatus,
};

/// An independent pair of electrical terminals beyond the module's main
/// output.
///
/// The only construction known to produce these is the four-terminal
/// tandem, whose two sub-cells are contacted separately. `INV-STR-003`
/// therefore warns when additional outputs appear on any other technology
/// (`docs/design/27-specification-finale-pv-module.md` §11, §13.1).
#[derive(Debug, Clone, PartialEq)]
pub struct ElectricalOutput {
    /// What this output is, e.g. `"perovskite top cell"`.
    pub label: String,
    /// The output's own electrical characteristics.
    pub characteristics: ElectricalCharacteristics,
}

impl ElectricalOutput {
    /// Name an additional electrical output and give its characteristics.
    #[inline]
    pub fn new(label: impl Into<String>, characteristics: ElectricalCharacteristics) -> Self {
        ElectricalOutput {
            label: label.into(),
            characteristics,
        }
    }
}

/// Why a module could not be built.
///
/// The two variants are not interchangeable, and the distinction is the
/// whole point of the five-status system: an [`ModuleBuildError::Invalid`]
/// record is beyond repair, while
/// [`ModuleBuildError::UnresolvedConflicts`] names contradictions that a
/// recorded human decision can lift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleBuildError {
    /// At least one invariant found something physically impossible. No
    /// resolution can lift this: there is no value to choose between.
    Invalid {
        /// The full report, so every finding stays inspectable.
        report: ValidationReport,
    },
    /// At least one conflict has no matching [`ConflictResolution`].
    ///
    /// **The engine does not arbitrate.** Both candidate values are named
    /// in the report; supply a resolution through
    /// [`PvModuleBuilder::resolving`] to proceed.
    UnresolvedConflicts {
        /// The invariant identifiers still unresolved.
        invariant_ids: Vec<&'static str>,
        /// The full report.
        report: ValidationReport,
    },
}

impl ModuleBuildError {
    /// The report, whichever variant this is.
    #[inline]
    pub const fn report(&self) -> &ValidationReport {
        match self {
            ModuleBuildError::Invalid { report }
            | ModuleBuildError::UnresolvedConflicts { report, .. } => report,
        }
    }
}

impl core::fmt::Display for ModuleBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModuleBuildError::Invalid { report } => {
                write!(
                    f,
                    "the module description is physically invalid ({} finding(s))",
                    report.issues_at(ValidationStatus::Invalid).count()
                )
            }
            ModuleBuildError::UnresolvedConflicts { invariant_ids, .. } => {
                write!(
                    f,
                    "{} unresolved conflict(s): {}; each requires an explicit recorded decision, \
                     as the engine never arbitrates between two plausible values",
                    invariant_ids.len(),
                    invariant_ids.join(", ")
                )
            }
        }
    }
}

impl core::error::Error for ModuleBuildError {}

/// A validated photovoltaic module.
///
/// **Construction is the validation gate.** A `PvModule` can only be
/// obtained through [`PvModuleBuilder::build`], which runs every invariant
/// and refuses anything invalid or carrying an unresolved conflict. Once
/// built, a module is therefore known to be internally coherent to the
/// extent the invariants can establish.
///
/// Warnings do **not** block: they are recorded in [`PvModule::report`] and
/// travel with the module so a downstream engine can weigh them. A module
/// that a warning silently suppressed would be worse than one carrying a
/// visible caveat.
///
/// ## Scope
///
/// The performance matrix of IEC 61853-1 is held as
/// [`OperatingPoint`]s carrying
/// [`OperatingConditionKind::Iec61853Matrix`], not as a separate
/// structure: a matrix point *is* a measured operating point, and giving
/// it its own type would duplicate the condition machinery. The blocks
/// sketched in `docs/design/27-specification-finale-pv-module.md` §13.1
/// for angular response, spectral response, mechanical limits, warranty
/// and construction are **deliberately not implemented in this sub-phase**
/// — each needs its own measurement standard and its own invariants, and
/// none is required by the exit criteria for the domain model.
#[derive(Debug, Clone, PartialEq)]
pub struct PvModule {
    identity: ModuleIdentity,
    technology: ModuleTechnology,
    configuration: CellConfiguration,
    geometry: ModuleGeometry,
    mounting_form: Option<MountingForm>,
    electrical: ElectricalCharacteristics,
    thermal: ThermalCharacteristics,
    limits: ElectricalLimits,
    operating_points: Vec<OperatingPoint>,
    bifacial: Option<BifacialCharacteristics>,
    certifications: Vec<Certification>,
    additional_outputs: Vec<ElectricalOutput>,
    report: ValidationReport,
    resolutions: Vec<ConflictResolution>,
}

impl PvModule {
    /// Who made it and which model it is.
    #[inline]
    pub const fn identity(&self) -> &ModuleIdentity {
        &self.identity
    }

    /// The cell material and contact architecture.
    #[inline]
    pub const fn technology(&self) -> &ModuleTechnology {
        &self.technology
    }

    /// How the photovoltaic conversion is physically organised.
    #[inline]
    pub const fn configuration(&self) -> &CellConfiguration {
        &self.configuration
    }

    /// Dimensions, mass and the five areas.
    #[inline]
    pub const fn geometry(&self) -> &ModuleGeometry {
        &self.geometry
    }

    /// How the module is built to be mounted, if stated.
    #[inline]
    pub const fn mounting_form(&self) -> Option<&MountingForm> {
        self.mounting_form.as_ref()
    }

    /// The electrical characteristics at standard test conditions.
    #[inline]
    pub const fn electrical(&self) -> &ElectricalCharacteristics {
        &self.electrical
    }

    /// The temperature coefficients.
    #[inline]
    pub const fn thermal(&self) -> &ThermalCharacteristics {
        &self.thermal
    }

    /// The declared safety and protection limits.
    #[inline]
    pub const fn limits(&self) -> &ElectricalLimits {
        &self.limits
    }

    /// Every operating point declared beyond standard test conditions.
    #[inline]
    pub fn operating_points(&self) -> &[OperatingPoint] {
        &self.operating_points
    }

    /// The bifacial block, if the module is bifacial.
    #[inline]
    pub const fn bifacial(&self) -> Option<&BifacialCharacteristics> {
        self.bifacial.as_ref()
    }

    /// The declared certifications.
    #[inline]
    pub fn certifications(&self) -> &[Certification] {
        &self.certifications
    }

    /// Additional independent electrical outputs, if any.
    #[inline]
    pub fn additional_outputs(&self) -> &[ElectricalOutput] {
        &self.additional_outputs
    }

    /// Every finding the invariants produced, warnings included.
    #[inline]
    pub const fn report(&self) -> &ValidationReport {
        &self.report
    }

    /// The recorded decisions that lifted this module's conflicts.
    ///
    /// Kept with the module because a resolution is provenance: knowing
    /// that a value was chosen by a named person for a stated reason is
    /// part of knowing how much to trust the module.
    #[inline]
    pub fn resolutions(&self) -> &[ConflictResolution] {
        &self.resolutions
    }

    /// The bounding rectangle's area, `length x width`.
    ///
    /// A *derived* quantity, distinct from any stated area. See
    /// [`crate::ModuleDimensions::rectangular_area`].
    #[inline]
    pub fn rectangular_area(&self) -> Area {
        self.geometry.rectangular_area()
    }

    /// The module's efficiency at standard test conditions, and the name
    /// of the area it was computed from.
    ///
    /// **The area is returned alongside the value, never assumed.** An
    /// efficiency without its reference area is ambiguous by up to several
    /// points, and reporting which area was used is what keeps the figure
    /// interpretable (`docs/design/27-specification-finale-pv-module.md`
    /// §8.2).
    ///
    /// Returns `None` when the reference irradiance of the module's own
    /// standard-test condition is not positive, which cannot happen for a
    /// built module but keeps the function total.
    pub fn efficiency(&self) -> Option<(Ratio, &'static str)> {
        let (area, area_name) = self.geometry.preferred_area_for_efficiency();
        let denominator =
            area.square_meters() * OperatingCondition::stc().irradiance.watts_per_m2();
        if denominator <= 0.0 || !denominator.is_finite() {
            return None;
        }
        let value = self.electrical.p_max.get().watts() / denominator;
        // `INV-DER-002` refuses construction above 0.50, so the value is
        // in range here; `clamped` rather than `new` keeps this total
        // instead of trading an impossible case for a panic.
        Some((Ratio::clamped(value), area_name))
    }

    /// The fill factor, `p_max / (v_oc * i_sc)`.
    ///
    /// A built module has passed `INV-ELE-006`, so the value is at most 1.
    #[inline]
    pub fn fill_factor(&self) -> f64 {
        self.electrical.fill_factor()
    }

    /// The operating point declared at a given kind of reference
    /// condition, if there is one.
    pub fn operating_point(&self, kind: &OperatingConditionKind) -> Option<&OperatingPoint> {
        self.operating_points
            .iter()
            .find(|p| p.condition.kind == *kind)
    }

    /// Every IEC 61853-1 matrix point, in declaration order.
    pub fn performance_matrix(&self) -> impl Iterator<Item = &OperatingPoint> {
        self.operating_points
            .iter()
            .filter(|p| p.condition.kind == OperatingConditionKind::Iec61853Matrix)
    }
}

/// Builds a [`PvModule`], running every invariant at [`PvModuleBuilder::build`].
///
/// The seven blocks passed to [`PvModuleBuilder::new`] are the ones without
/// which no electrical model could run; everything else is optional and
/// added through a `with_*` method.
#[derive(Debug, Clone, PartialEq)]
pub struct PvModuleBuilder {
    identity: ModuleIdentity,
    technology: ModuleTechnology,
    configuration: CellConfiguration,
    geometry: ModuleGeometry,
    mounting_form: Option<MountingForm>,
    electrical: ElectricalCharacteristics,
    thermal: ThermalCharacteristics,
    limits: ElectricalLimits,
    operating_points: Vec<OperatingPoint>,
    bifacial: Option<BifacialCharacteristics>,
    certifications: Vec<Certification>,
    additional_outputs: Vec<ElectricalOutput>,
    resolutions: Vec<ConflictResolution>,
}

impl PvModuleBuilder {
    /// Start from the mandatory blocks.
    pub fn new(
        identity: ModuleIdentity,
        technology: ModuleTechnology,
        configuration: CellConfiguration,
        geometry: ModuleGeometry,
        electrical: ElectricalCharacteristics,
        thermal: ThermalCharacteristics,
        limits: ElectricalLimits,
    ) -> Self {
        PvModuleBuilder {
            identity,
            technology,
            configuration,
            geometry,
            mounting_form: None,
            electrical,
            thermal,
            limits,
            operating_points: Vec::new(),
            bifacial: None,
            certifications: Vec::new(),
            additional_outputs: Vec::new(),
            resolutions: Vec::new(),
        }
    }

    /// Record how the module is built to be mounted.
    #[must_use]
    pub fn with_mounting_form(mut self, form: MountingForm) -> Self {
        self.mounting_form = Some(form);
        self
    }

    /// Add one operating point.
    #[must_use]
    pub fn with_operating_point(mut self, point: OperatingPoint) -> Self {
        self.operating_points.push(point);
        self
    }

    /// Add several operating points, such as a performance matrix.
    #[must_use]
    pub fn with_operating_points(
        mut self,
        points: impl IntoIterator<Item = OperatingPoint>,
    ) -> Self {
        self.operating_points.extend(points);
        self
    }

    /// Declare the module bifacial and describe what is known of it.
    #[must_use]
    pub fn with_bifacial(mut self, bifacial: BifacialCharacteristics) -> Self {
        self.bifacial = Some(bifacial);
        self
    }

    /// Record a certification.
    #[must_use]
    pub fn with_certification(mut self, certification: Certification) -> Self {
        self.certifications.push(certification);
        self
    }

    /// Record an additional independent electrical output.
    #[must_use]
    pub fn with_additional_output(mut self, output: ElectricalOutput) -> Self {
        self.additional_outputs.push(output);
        self
    }

    /// Supply an explicit decision lifting one conflict.
    ///
    /// **This is the only way past a conflict.** The resolution names who
    /// decided and why; both are mandatory in
    /// [`ConflictResolution`], so an anonymous arbitration is not
    /// representable.
    #[must_use]
    pub fn resolving(mut self, resolution: ConflictResolution) -> Self {
        self.resolutions.push(resolution);
        self
    }

    /// Run every invariant without building, for inspection.
    ///
    /// Useful for showing a user what is wrong with a record before asking
    /// them to decide, and for exercising the invariants in tests.
    pub fn validate(&self) -> ValidationReport {
        invariants::validate_module(
            &self.electrical,
            &self.geometry,
            &self.configuration,
            &self.thermal,
            &self.limits,
            &self.technology,
            &self.operating_points,
            self.bifacial.as_ref(),
            self.additional_outputs.len(),
        )
    }

    /// Validate and build.
    ///
    /// # Errors
    ///
    /// - [`ModuleBuildError::Invalid`] if any invariant found something
    ///   physically impossible.
    /// - [`ModuleBuildError::UnresolvedConflicts`] if any conflict lacks a
    ///   matching [`ConflictResolution`].
    ///
    /// Warnings never block; they are carried in [`PvModule::report`].
    pub fn build(self) -> Result<PvModule, ModuleBuildError> {
        let report = self.validate();

        if report.has_invalid() {
            return Err(ModuleBuildError::Invalid { report });
        }

        let unresolved: Vec<&'static str> = report
            .conflicts()
            .map(ValidationIssue::invariant_id)
            .filter(|id| !self.resolutions.iter().any(|r| r.invariant_id() == *id))
            .collect();

        if !unresolved.is_empty() {
            return Err(ModuleBuildError::UnresolvedConflicts {
                invariant_ids: unresolved,
                report,
            });
        }

        Ok(PvModule {
            identity: self.identity,
            technology: self.technology,
            configuration: self.configuration,
            geometry: self.geometry,
            mounting_form: self.mounting_form,
            electrical: self.electrical,
            thermal: self.thermal,
            limits: self.limits,
            operating_points: self.operating_points,
            bifacial: self.bifacial,
            certifications: self.certifications,
            additional_outputs: self.additional_outputs,
            report,
            resolutions: self.resolutions,
        })
    }
}

/// Whether an identifier belongs to the crate's invariant inventory.
///
/// Exposed so that adapters and downstream tools can check an identifier
/// they were handed rather than matching on strings blindly.
#[inline]
pub fn is_known_invariant(id: &str) -> bool {
    ALL_INVARIANT_IDS.contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::Measurement;
    use crate::{CellArchitecture, CellTechnology, ModuleDimensions, ResolutionDecision};
    use core_units::{Area, Current, Length, Power, Temperature, TemperatureCoefficient, Voltage};

    /// A coherent module, used as the baseline every test perturbs.
    ///
    /// The values are internally consistent by construction and are not
    /// taken from any real product: the five real datasheets remain an
    /// open item of the test plan and must not be simulated
    /// (`docs/design/28-plan-tests-pv-module.md`).
    fn coherent_builder() -> PvModuleBuilder {
        let electrical = ElectricalCharacteristics::new(
            Measurement::stated(Power::from_watts(400.0)),
            Measurement::stated(Voltage::from_volts(49.5)),
            Measurement::stated(Current::from_amperes(10.4)),
            Measurement::stated(Voltage::from_volts(41.5)),
            Measurement::stated(Current::from_amperes(9.64)),
        );
        let geometry = ModuleGeometry::new(ModuleDimensions::new(
            Measurement::stated(Length::from_meters(2.0)),
            Measurement::stated(Length::from_meters(1.0)),
        ))
        .with_gross_area(Measurement::stated(Area::from_square_meters(2.0)));
        let thermal = ThermalCharacteristics::new(
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(0.045)),
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.27)),
            Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.34)),
        );
        PvModuleBuilder::new(
            ModuleIdentity::new("Example Manufacturer", "EX-400"),
            ModuleTechnology::new(
                CellTechnology::MonocrystallineSilicon,
                CellArchitecture::PERC,
            ),
            CellConfiguration::discrete(72),
            geometry,
            electrical,
            thermal,
            ElectricalLimits::new(Measurement::stated(Voltage::from_volts(1500.0))),
        )
    }

    #[test]
    fn a_coherent_module_builds_without_a_blocking_finding() {
        let module = coherent_builder().build().expect("the baseline must build");
        assert!(!module.report().has_invalid());
        assert!(module.report().conflicts().next().is_none());
        assert_eq!(module.identity().model_name, "EX-400");
    }

    #[test]
    fn efficiency_names_the_area_it_used() {
        let module = coherent_builder().build().unwrap();
        let (efficiency, area_name) = module.efficiency().unwrap();
        assert_eq!(area_name, "gross_area");
        assert!((efficiency.value() - 0.20).abs() < 1e-12);
        assert!((module.rectangular_area().square_meters() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn an_impossible_voltage_ordering_blocks_construction_outright() {
        let mut builder = coherent_builder();
        builder.electrical.v_mpp = Measurement::stated(Voltage::from_volts(60.0));
        let error = builder
            .build()
            .expect_err("v_mpp above v_oc must be refused");
        assert!(matches!(error, ModuleBuildError::Invalid { .. }));
        assert!(error.report().contains("INV-ELE-001"));
    }

    #[test]
    fn a_conflict_blocks_construction_until_it_is_decided() {
        // The Sharp_ND_72ELUF shape: a stated area contradicting the
        // module's own dimensions by a factor of two.
        let mut builder = coherent_builder();
        builder.geometry = builder
            .geometry
            .with_gross_area(Measurement::stated(Area::from_square_meters(1.0)));

        let error = builder
            .clone()
            .build()
            .expect_err("a stated area contradicting the dimensions must block");
        match &error {
            ModuleBuildError::UnresolvedConflicts { invariant_ids, .. } => {
                assert!(invariant_ids.contains(&"INV-GEO-002"));
            }
            ModuleBuildError::Invalid { .. } => panic!("a conflict is not an invalid record"),
        }
        assert!(error.to_string().contains("INV-GEO-002"));

        // The same builder, with a recorded decision, proceeds.
        let module = builder
            .resolving(ConflictResolution::new(
                "INV-GEO-002",
                ResolutionDecision::UseFirst,
                "metrology team",
                "the stated area was re-measured on a specimen",
            ))
            .build()
            .expect("an explicitly decided conflict must no longer block");

        // The conflict is kept, not erased: the module remains auditable.
        assert!(module.report().contains("INV-GEO-002"));
        assert_eq!(module.resolutions().len(), 1);
        assert_eq!(module.resolutions()[0].decided_by(), "metrology team");
    }

    #[test]
    fn a_resolution_for_another_invariant_does_not_lift_this_conflict() {
        let mut builder = coherent_builder();
        builder.geometry = builder
            .geometry
            .with_gross_area(Measurement::stated(Area::from_square_meters(1.0)));
        let error = builder
            .resolving(ConflictResolution::new(
                "INV-DER-001",
                ResolutionDecision::UseFirst,
                "operator",
                "unrelated",
            ))
            .build()
            .expect_err("a mismatched resolution must not lift the conflict");
        assert!(matches!(
            error,
            ModuleBuildError::UnresolvedConflicts { .. }
        ));
    }

    #[test]
    fn a_warning_is_recorded_and_does_not_block() {
        let mut builder = coherent_builder();
        // 72 junctions at 20 V open circuit is 0.28 V per junction, below
        // the range any known cell chemistry spans.
        builder.electrical.v_oc = Measurement::stated(Voltage::from_volts(20.0));
        builder.electrical.v_mpp = Measurement::stated(Voltage::from_volts(16.0));
        builder.electrical.p_max = Measurement::stated(Power::from_watts(154.24));
        let module = builder.build().expect("a warning must not block");
        assert!(module.report().contains("INV-ELE-010"));
        assert_eq!(module.report().worst_status(), ValidationStatus::Warning);
    }

    #[test]
    fn matrix_points_are_reachable_and_separate_from_named_conditions() {
        use core_units::Irradiance;

        let module = coherent_builder()
            .with_operating_point(
                OperatingPoint::new(OperatingCondition::noct())
                    .with_cell_temperature(Measurement::measured(Temperature::from_celsius(44.0))),
            )
            .with_operating_points([
                OperatingPoint::new(OperatingCondition::iec61853_matrix_point(
                    Irradiance::from_watts_per_m2(600.0),
                    Temperature::from_celsius(25.0),
                )),
                OperatingPoint::new(OperatingCondition::iec61853_matrix_point(
                    Irradiance::from_watts_per_m2(400.0),
                    Temperature::from_celsius(25.0),
                )),
            ])
            .build()
            .unwrap();

        assert_eq!(module.performance_matrix().count(), 2);
        assert!(
            module
                .operating_point(&OperatingConditionKind::NoctIec61215)
                .is_some()
        );
        assert!(
            module
                .operating_point(&OperatingConditionKind::PtcPvusa)
                .is_none()
        );
    }

    #[test]
    fn optional_blocks_are_carried_through_construction() {
        let module = coherent_builder()
            .with_mounting_form(MountingForm::rigid())
            .with_certification(Certification::Iec61215)
            .with_bifacial(BifacialCharacteristics::with_factor(Measurement::measured(
                Ratio::new(0.70),
            )))
            .build()
            .unwrap();

        assert!(module.mounting_form().is_some());
        assert_eq!(module.certifications(), &[Certification::Iec61215]);
        assert!(module.bifacial().is_some());
        assert!(module.additional_outputs().is_empty());
    }

    #[test]
    fn validate_inspects_without_consuming_the_builder() {
        let builder = coherent_builder();
        let first = builder.validate();
        let second = builder.validate();
        assert_eq!(first, second);
        assert!(builder.build().is_ok());
    }

    #[test]
    fn known_invariant_identifiers_are_recognised() {
        assert!(is_known_invariant("INV-GEO-002"));
        assert!(is_known_invariant("INV-STR-004"));
        assert!(!is_known_invariant("INV-XXX-999"));
    }
}
