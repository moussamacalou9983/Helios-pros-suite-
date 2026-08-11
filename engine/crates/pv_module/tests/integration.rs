//! End-to-end behaviour of the crate: whole modules built, refused, and
//! decided about.
//!
//! Where `invariants.rs` isolates one rule at a time, these tests exercise
//! the paths a caller actually takes — importing a record shaped like a
//! real database row, meeting a conflict, recording a decision, and
//! reading the result back — and check the properties the specification
//! states about the crate as a whole rather than about any single
//! invariant.

mod common;

use common::*;
use core_units::{
    Area, CoefficientBasis, Current, Irradiance, Length, Power, Ratio, Temperature,
    TemperatureCoefficient, Voltage,
};
use pv_module::{
    BifacialCharacteristics, CalendarDate, CellArchitecture, CellConfiguration, CellTechnology,
    Certification, Confidence, ConflictResolution, DataClass, ExtractionMethod, Measurement,
    ModuleBuildError, ModuleGeometry, ModuleIdentity, ModuleTechnology, MountingForm,
    OperatingCondition, OperatingConditionKind, OperatingPoint, Provenance, PvModuleBuilder,
    ResolutionDecision, SourceIdentifier, SourceType, ThermalCharacteristics, Uncertainty,
    UncertaintyEvaluation, ValidationStatus,
};

// ---------------------------------------------------------------------
// Nominal
// ---------------------------------------------------------------------

#[test]
fn a_fully_described_module_builds_and_exposes_every_block() {
    let module = attributed_builder()
        .with_mounting_form(MountingForm::Rigid {
            frame_material: Some("anodised aluminium".to_string()),
        })
        .with_certification(Certification::Iec61215)
        .with_certification(Certification::Iec61730)
        .with_operating_point(
            OperatingPoint::new(OperatingCondition::noct())
                .with_p_max(Measurement::stated(Power::from_watts(296.0)))
                .with_cell_temperature(Measurement::measured(Temperature::from_celsius(43.0))),
        )
        .build()
        .expect("a coherent module must build");

    assert_eq!(module.identity().manufacturer, "Example Manufacturer");
    assert_eq!(module.technology().architecture, CellArchitecture::PERC);
    assert_eq!(module.configuration().junctions_in_series(), 72);
    assert!(module.mounting_form().is_some());
    assert_eq!(module.certifications().len(), 2);
    assert_eq!(module.operating_points().len(), 1);
    assert!(module.bifacial().is_none());
    assert!(module.additional_outputs().is_empty());
    assert!(module.resolutions().is_empty());

    // The derived figures agree with the inputs.
    let (efficiency, area_name) = module.efficiency().unwrap();
    assert_eq!(area_name, "gross_area");
    assert!((efficiency.value() - 0.20).abs() < 1e-12);
    assert!((module.fill_factor() - 400.0 / (49.5 * 10.4)).abs() < 1e-12);
    assert!((module.rectangular_area().square_meters() - 2.0).abs() < 1e-12);

    // Every value is attributed and every relation holds, so the report is
    // empty — not merely free of blocking findings.
    assert_eq!(module.report().worst_status(), ValidationStatus::Valid);
    assert!(
        module.report().issues().is_empty(),
        "unexpected findings: {:?}",
        module
            .report()
            .issues()
            .iter()
            .map(|i| (i.invariant_id(), i.status()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn an_unattributed_module_is_usable_but_says_so() {
    // The same module with no provenance on its values: it builds, and
    // every unattributed value is named.
    let module = coherent_builder().build().unwrap();
    assert_eq!(module.report().worst_status(), ValidationStatus::Warning);
    assert!(!module.report().worst_status().is_blocking());

    // The eight mandatory physical quantities are each reported once.
    assert_eq!(
        module
            .report()
            .issues()
            .iter()
            .filter(|i| i.invariant_id() == "INV-PRV-001")
            .count(),
        8
    );
}

#[test]
fn validation_is_deterministic() {
    // The same description validated twice yields the same report, in the
    // same order. A validator whose findings depended on iteration order
    // could not be used to gate anything.
    let builder = coherent_builder()
        .with_operating_point(OperatingPoint::new(OperatingCondition::noct()))
        .with_bifacial(BifacialCharacteristics::flag_only());

    let first = builder.validate();
    let second = builder.validate();
    assert_eq!(first, second);
    assert_eq!(
        first
            .issues()
            .iter()
            .map(|i| i.invariant_id())
            .collect::<Vec<_>>(),
        second
            .issues()
            .iter()
            .map(|i| i.invariant_id())
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------
// Units
// ---------------------------------------------------------------------

#[test]
fn units_convert_once_at_the_boundary_and_keep_their_traceability() {
    // A source publishing millimetres and grams: converted where it is
    // read, never inside the model.
    let length_mm = Measurement::stated(1762.0_f64)
        .with_provenance(datasheet())
        .with_confidence(Confidence::High);
    let length = length_mm.map(Length::from_millimeters);

    assert!((length.get().meters() - 1.762).abs() < 1e-12);
    assert_eq!(length.classification(), DataClass::ManufacturerData);
    assert_eq!(length.confidence(), Some(Confidence::High));
    assert!(
        length.provenance().is_some(),
        "a unit conversion must not lose where the value came from"
    );
}

#[test]
fn absolute_and_relative_temperature_coefficients_stay_distinguishable() {
    // The CEC library states alpha and beta in A/K and V/K, and gamma in
    // %/K, within one record. Nothing converts them silently, because the
    // conversion needs a reference value the row does not carry in a form
    // the model can assume.
    let thermal = ThermalCharacteristics::new(
        Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(0.002_146)),
        Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(-0.159_068)),
        Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(-0.5072)),
    );

    assert_eq!(thermal.alpha_i_sc.get().basis(), CoefficientBasis::Absolute);
    assert_eq!(
        thermal.gamma_p_max.get().basis(),
        CoefficientBasis::Relative
    );

    // A relative coefficient converts to an absolute one only against a
    // stated reference; an absolute one has no relative form without one.
    let gamma = thermal.gamma_p_max.get();
    let absolute = gamma.to_absolute_per_kelvin(400.0);
    assert!((absolute - (-0.005_072 * 400.0)).abs() < 1e-9);
    assert!(
        thermal
            .alpha_i_sc
            .get()
            .to_fraction_per_kelvin(0.0)
            .is_none()
    );
}

#[test]
fn a_cell_temperature_is_never_comparable_to_an_ambient_one() {
    // STC and PTC share an irradiance of 1000 W/m2 and differ only in
    // which temperature they impose. Treating them as the same condition
    // is a category error, not a small numerical difference.
    let stc = OperatingCondition::stc();
    let ptc = OperatingCondition::ptc();

    assert!(
        (stc.irradiance.watts_per_m2() - ptc.irradiance.watts_per_m2()).abs() < 1e-9,
        "the two conditions must share their irradiance for this test to mean anything"
    );
    assert!(!stc.is_comparable_to(&ptc));
    assert!(stc.temperature.is_cell_temperature());
    assert!(!ptc.temperature.is_cell_temperature());

    // NOCT and NMOT both impose an ambient temperature, so they are
    // comparable in that one respect while remaining distinct conditions.
    let noct = OperatingCondition::noct();
    let nmot = OperatingCondition::nmot();
    assert!(noct.is_comparable_to(&nmot));
    assert_ne!(noct.kind, nmot.kind);
    assert_ne!(noct.electrical_state, nmot.electrical_state);
}

// ---------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------

fn datasheet() -> Provenance {
    Provenance::new(
        SourceType::ManufacturerDatasheet,
        ExtractionMethod::ManualTranscription,
    )
}

#[test]
fn the_full_provenance_trail_survives_construction() {
    let provenance = Provenance::new(
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
    .with_confidence(Confidence::Medium);

    let mut electrical = coherent_electrical();
    electrical.p_max = Measurement::stated(Power::from_watts(400.0))
        .with_provenance(provenance)
        .with_uncertainty(
            Uncertainty::expanded_relative(0.03, 2.0).with_evaluation(UncertaintyEvaluation::TypeB),
        );

    let module = builder_with_electrical(electrical).build().unwrap();
    let p_max = &module.electrical().p_max;

    let recorded = p_max.provenance().expect("the provenance must survive");
    assert_eq!(
        recorded.source().unwrap().identifier.as_deref(),
        Some("row-4211")
    );
    assert_eq!(recorded.publication_date().unwrap().year, 2019);
    assert_eq!(recorded.page(), Some(2));

    let u = p_max.uncertainty().unwrap();
    assert_eq!(u.coverage_factor(), Some(2.0));
    assert!((u.standard_uncertainty().unwrap() - 0.015).abs() < 1e-12);
    assert_eq!(u.evaluation(), Some(UncertaintyEvaluation::TypeB));

    // The value carrying provenance is not reported as unattributed.
    assert!(
        module
            .report()
            .issues()
            .iter()
            .filter(|i| i.invariant_id() == "INV-PRV-001")
            .count()
            < 8,
        "the attributed value must not be counted among the unattributed ones"
    );
}

#[test]
fn one_quantity_classifies_differently_according_to_its_source() {
    // The founding reason classification travels with the value rather
    // than with the field: the same open-circuit voltage has three
    // different authorities depending on where it was read.
    let volts = Voltage::from_volts(49.5);

    let from_datasheet = Measurement::stated(volts).with_provenance(datasheet());
    let from_lab = Measurement::measured(volts).with_provenance(Provenance::new(
        SourceType::TestReport {
            standard: Some("IEC 61853-1".to_string()),
        },
        ExtractionMethod::AutomatedImport {
            adapter: "IEC61853Adapter".to_string(),
            adapter_version: "0.1.0".to_string(),
        },
    ));
    let from_memory =
        Measurement::estimated(volts, "recalled from a previous project").with_provenance(
            Provenance::new(SourceType::UserInput, ExtractionMethod::ManualTranscription),
        );

    assert_eq!(from_datasheet.classification(), DataClass::ManufacturerData);
    assert_eq!(from_lab.classification(), DataClass::Measured);
    assert_eq!(from_memory.classification(), DataClass::Estimated);

    // The value is identical; only the authority differs.
    assert!((from_datasheet.get().volts() - from_memory.get().volts()).abs() < 1e-12);

    // Each provenance implies the same class its constructor set.
    assert_eq!(
        from_datasheet.provenance().unwrap().implied_class(),
        from_datasheet.classification()
    );
    assert_eq!(
        from_lab.provenance().unwrap().implied_class(),
        from_lab.classification()
    );
    assert_eq!(
        from_memory.provenance().unwrap().implied_class(),
        from_memory.classification()
    );
}

#[test]
fn provenance_cannot_be_altered_in_place_after_construction() {
    let original = Measurement::stated(Voltage::from_volts(49.5));
    let enriched = original.clone().with_provenance(datasheet());

    // The update mechanism consumes and returns; the original is intact.
    assert!(original.provenance().is_none());
    assert!(enriched.provenance().is_some());
    assert_ne!(original, enriched);
}

// ---------------------------------------------------------------------
// Absent data
// ---------------------------------------------------------------------

#[test]
fn absent_optional_data_is_never_replaced_by_a_default() {
    // The minimum a source can state and still describe a module.
    let module = PvModuleBuilder::new(
        ModuleIdentity::new("Example Manufacturer", "EX-400"),
        ModuleTechnology::unknown(),
        coherent_configuration(),
        geometry(2.0, 1.0),
        coherent_electrical(),
        coherent_thermal(),
        coherent_limits(),
    )
    .build()
    .expect("a minimally described module must still build");

    // Nothing was invented to fill the gaps.
    assert!(module.geometry().gross_area.is_none());
    assert!(module.geometry().aperture_area.is_none());
    assert!(module.geometry().mass.is_none());
    assert!(module.geometry().dimensions.depth.is_none());
    assert!(module.limits().max_series_fuse.is_none());
    assert!(module.limits().min_operating_temperature.is_none());
    assert!(module.electrical().stated_efficiency.is_none());
    assert!(module.electrical().power_tolerance.is_none());
    assert!(module.thermal().beta_v_mpp.is_none());
    assert!(module.identity().building_integrated.is_none());
    assert!(module.operating_points().is_empty());

    // An unknown technology is not silently treated as silicon.
    assert_eq!(module.technology().technology, CellTechnology::Unknown);
    assert_eq!(module.technology().architecture, CellArchitecture::Unknown);

    // Efficiency falls back to the derived rectangle and says so.
    assert_eq!(module.efficiency().unwrap().1, "rectangular_area");
}

#[test]
fn an_unstated_flag_stays_distinct_from_a_stated_false_one() {
    let unstated = ModuleIdentity::new("M", "X");
    let stated_false = ModuleIdentity::new("M", "X").with_building_integrated(false);
    assert_eq!(unstated.building_integrated, None);
    assert_eq!(stated_false.building_integrated, Some(false));
    assert_ne!(
        unstated.building_integrated,
        stated_false.building_integrated
    );

    // Likewise for a technology the source did not name versus one it
    // named but this model does not classify.
    assert_ne!(
        CellTechnology::Unknown,
        CellTechnology::Other {
            name: "Thin Film".to_string()
        }
    );
}

// ---------------------------------------------------------------------
// Invalid
// ---------------------------------------------------------------------

#[test]
fn an_impossible_module_is_refused_and_no_decision_can_admit_it() {
    // Maximum power above the product of open-circuit voltage and
    // short-circuit current: a fill factor above 1.
    let builder = builder_with_electrical(electrical(900.0, 49.5, 10.4, 41.5, 9.64));

    let error = builder
        .clone()
        .build()
        .expect_err("a fill factor above 1 must be refused");
    assert!(matches!(error, ModuleBuildError::Invalid { .. }));
    assert!(error.report().contains("INV-ELE-006"));
    assert!(error.to_string().contains("physically invalid"));

    // A resolution is not a way past an impossibility: there is no second
    // candidate value to choose.
    let still_refused = builder
        .resolving(ConflictResolution::new(
            "INV-ELE-006",
            ResolutionDecision::Ignore,
            "operator",
            "attempting to override an impossibility",
        ))
        .build()
        .expect_err("no decision may admit an impossible module");
    assert!(matches!(still_refused, ModuleBuildError::Invalid { .. }));
}

#[test]
fn several_independent_defects_are_all_reported_at_once() {
    // A validator that stopped at the first defect would make correcting a
    // record an exercise in repeated guessing.
    let builder = builder_with(
        silicon(),
        coherent_configuration(),
        coherent_geometry(),
        electrical(400.0, 41.5, 9.0, 49.5, 10.4),
        coherent_thermal(),
        limits(30.0),
    );

    let error = builder
        .build()
        .expect_err("this record has several defects");
    let report = error.report();
    for id in ["INV-ELE-001", "INV-ELE-002", "INV-ELE-011"] {
        assert!(report.contains(id), "{id} should have been reported too");
    }
}

// ---------------------------------------------------------------------
// Conflicts
// ---------------------------------------------------------------------

/// The Sharp_ND_72ELUF shape from the CEC library: a stated area of
/// 0.577 m² against dimensions implying 1.153 m², where direct inspection
/// showed the *derived* figure to be the wrong one
/// (`docs/design/26-validation-empirique-cec-sandia.md` §1).
fn contradictory_area_builder() -> PvModuleBuilder {
    contradictory_area_builder_with(electrical(72.0, 44.6, 2.16, 35.5, 2.03))
}

fn contradictory_area_builder_with(
    electrical: pv_module::ElectricalCharacteristics,
) -> PvModuleBuilder {
    builder_with(
        silicon(),
        CellConfiguration::discrete(72),
        ModuleGeometry::new(dimensions(1.165, 0.990))
            .with_gross_area(Measurement::stated(Area::from_square_meters(0.577))),
        electrical,
        coherent_thermal(),
        coherent_limits(),
    )
}

#[test]
fn a_conflict_blocks_and_names_both_candidates() {
    let builder = contradictory_area_builder();
    let error = builder
        .build()
        .expect_err("a stated area contradicting the dimensions must block");

    match &error {
        ModuleBuildError::UnresolvedConflicts {
            invariant_ids,
            report,
        } => {
            assert!(invariant_ids.contains(&"INV-GEO-002"));
            let issue = report.find("INV-GEO-002").unwrap();
            // Both candidates and the gap between them are stated, which
            // is what makes a decision possible.
            let observed = issue.observed().unwrap();
            assert!(observed.contains("0.577"));
            assert!(observed.contains("1.15"));
            assert!(issue.expected().is_some());
            assert!(!issue.explanation().is_empty());
            // Neither figure was chosen.
            assert!(issue.explanation().contains("does not choose"));
        }
        ModuleBuildError::Invalid { .. } => {
            panic!(
                "both areas are individually plausible; this is a conflict, not an impossibility"
            )
        }
    }
    // The message names the invariant so the caller can act on it.
    assert!(error.to_string().contains("INV-GEO-002"));
    assert!(error.to_string().contains("never arbitrates"));
}

#[test]
fn a_recorded_decision_lifts_the_conflict_and_stays_with_the_module() {
    let module = contradictory_area_builder()
        .resolving(
            ConflictResolution::new(
                "INV-GEO-002",
                ResolutionDecision::UseFirst,
                "metrology team",
                "the stated area is consistent with 72 cells of 125 mm; the dimensions are the \
                 erroneous field",
            )
            .with_date(CalendarDate::new(2026, 8, 11).unwrap())
            .with_reference("docs/design/26-validation-empirique-cec-sandia.md section 1"),
        )
        .build()
        .expect("an explicitly decided conflict must no longer block");

    // The conflict is kept, not erased: a module that hid its own
    // resolved contradictions would not be auditable.
    assert!(module.report().contains("INV-GEO-002"));
    assert_eq!(
        module.report().find("INV-GEO-002").unwrap().status(),
        ValidationStatus::Conflict
    );

    let resolution = &module.resolutions()[0];
    assert_eq!(resolution.invariant_id(), "INV-GEO-002");
    assert_eq!(resolution.decided_by(), "metrology team");
    assert!(resolution.rationale().contains("125 mm"));
    assert_eq!(resolution.decided_on().unwrap().day, 11);
    assert_eq!(resolution.references().len(), 1);
}

#[test]
fn an_anonymous_decision_is_not_representable() {
    // The rule made structural: a resolution without a decider or a
    // rationale cannot be constructed at all, so no code path can produce
    // a silent arbitration.
    assert!(
        ConflictResolution::try_new("INV-GEO-002", ResolutionDecision::UseFirst, "", "a reason")
            .is_none()
    );
    assert!(
        ConflictResolution::try_new(
            "INV-GEO-002",
            ResolutionDecision::UseFirst,
            "an operator",
            "  "
        )
        .is_none()
    );
}

#[test]
fn every_conflict_needs_its_own_decision() {
    // Two independent contradictions: the stated area, and the stated
    // efficiency against the derived one.
    let builder = contradictory_area_builder_with(
        electrical(72.0, 44.6, 2.16, 35.5, 2.03)
            .with_stated_efficiency(Measurement::stated(Ratio::new(0.16))),
    );

    let error = builder
        .clone()
        .build()
        .expect_err("two conflicts must block");
    let ModuleBuildError::UnresolvedConflicts { invariant_ids, .. } = &error else {
        panic!("expected unresolved conflicts, got {error:?}");
    };
    assert!(invariant_ids.contains(&"INV-GEO-002"));
    assert!(invariant_ids.contains(&"INV-DER-001"));

    // Deciding one leaves the other blocking.
    let partial = builder
        .clone()
        .resolving(ConflictResolution::new(
            "INV-GEO-002",
            ResolutionDecision::UseFirst,
            "metrology team",
            "the stated area was re-measured",
        ))
        .build()
        .expect_err("one decision must not lift two conflicts");
    let ModuleBuildError::UnresolvedConflicts { invariant_ids, .. } = &partial else {
        panic!("expected the second conflict to remain");
    };
    assert_eq!(invariant_ids, &["INV-DER-001"]);

    // Deciding both proceeds.
    let module = builder
        .resolving(ConflictResolution::new(
            "INV-GEO-002",
            ResolutionDecision::UseFirst,
            "metrology team",
            "the stated area was re-measured",
        ))
        .resolving(ConflictResolution::new(
            "INV-DER-001",
            ResolutionDecision::UseSecond,
            "metrology team",
            "the datasheet efficiency uses the aperture area, which this record does not state",
        ))
        .build()
        .expect("both decisions recorded, so construction proceeds");
    assert_eq!(module.resolutions().len(), 2);
}

// ---------------------------------------------------------------------
// A database-shaped record, end to end
// ---------------------------------------------------------------------

#[test]
fn a_record_shaped_like_a_cec_row_builds_with_its_gaps_reported() {
    // The shape of a CEC record: no architecture, a bifacial flag with no
    // factor, a PTC condition, and no provenance on the individual
    // values. Not a copy of any row — the shape only.
    let module = PvModuleBuilder::new(
        ModuleIdentity::new("Example Manufacturer", "EX-400-BIFACIAL"),
        ModuleTechnology::new(
            CellTechnology::MonocrystallineSilicon,
            CellArchitecture::Unknown,
        ),
        coherent_configuration(),
        coherent_geometry(),
        coherent_electrical(),
        coherent_thermal(),
        coherent_limits(),
    )
    .with_bifacial(BifacialCharacteristics::flag_only())
    .with_operating_point(
        OperatingPoint::new(OperatingCondition::ptc())
            .with_p_max(Measurement::stated(Power::from_watts(367.0))),
    )
    .build()
    .expect("a record with gaps is not a defective record");

    // The gaps are reported, and none of them blocks.
    assert_eq!(module.report().worst_status(), ValidationStatus::Warning);
    assert!(module.report().contains("INV-BIF-001"));
    assert!(module.report().contains("INV-PRV-001"));

    // "Bifacial, not characterised" stays distinct from "monofacial".
    let bifacial = module.bifacial().unwrap();
    assert!(bifacial.is_bifacial);
    assert!(bifacial.bifaciality_factor.is_none());
    assert!(bifacial.is_uncharacterised());

    // PTC is retrievable and remains an ambient-temperature condition.
    let ptc = module
        .operating_point(&OperatingConditionKind::PtcPvusa)
        .unwrap();
    assert!(!ptc.condition.temperature.is_cell_temperature());
    assert!((ptc.p_max.as_ref().unwrap().get().watts() - 367.0).abs() < 1e-12);
}

#[test]
fn a_performance_matrix_is_held_as_operating_points() {
    // The IEC 61853-1 grid needs no structure of its own: a matrix point
    // is a measured operating point at an imposed cell temperature.
    // The irradiances and temperatures below are placeholders for the
    // grid's own values, which remain to be read from the standard.
    let points: Vec<OperatingPoint> = [1000.0, 800.0, 600.0, 400.0, 200.0, 100.0]
        .into_iter()
        .flat_map(|irradiance| {
            [15.0, 25.0, 50.0, 75.0].into_iter().map(move |celsius| {
                OperatingPoint::new(OperatingCondition::iec61853_matrix_point(
                    Irradiance::from_watts_per_m2(irradiance),
                    Temperature::from_celsius(celsius),
                ))
            })
        })
        .collect();
    let declared = points.len();

    let module = coherent_builder()
        .with_operating_points(points)
        .build()
        .expect("a matrix of points must not trip the uniqueness rule");

    assert_eq!(module.performance_matrix().count(), declared);
    // Every matrix point imposes the cell temperature, so all are
    // mutually comparable.
    let stc = OperatingCondition::stc();
    assert!(
        module
            .performance_matrix()
            .all(|p| p.condition.is_comparable_to(&stc))
    );
}

#[test]
fn a_thin_film_module_is_describable_without_discrete_cells() {
    let module = PvModuleBuilder::new(
        ModuleIdentity::new("Example Manufacturer", "EX-TF-120"),
        cadmium_telluride(),
        CellConfiguration::Monolithic {
            series_segments: 154,
        },
        geometry(1.2, 0.6).with_gross_area(area(0.72)),
        electrical(120.0, 88.0, 1.9, 68.0, 1.77),
        thermal(0.04, -0.28, -0.34),
        limits(1500.0),
    )
    .build()
    .expect("a monolithic module must build");

    assert!(module.configuration().is_monolithic());
    assert_eq!(module.configuration().junctions_in_series(), 154);
    assert_eq!(module.configuration().parallel_paths(), 1);
    assert!(module.technology().is_thin_film());
    assert!(!module.report().contains("INV-STR-004"));
}

#[test]
fn a_flexible_module_carries_a_bend_radius_instead_of_load_ratings() {
    let module = coherent_builder()
        .with_mounting_form(MountingForm::Flexible {
            min_bend_radius: Some(Measurement::stated(Length::from_meters(0.5))),
        })
        .build()
        .unwrap();

    match module.mounting_form().unwrap() {
        MountingForm::Flexible { min_bend_radius } => {
            assert!((min_bend_radius.as_ref().unwrap().get().meters() - 0.5).abs() < 1e-12);
        }
        other => panic!("expected a flexible module, got {other:?}"),
    }
}

#[test]
fn the_advertised_bifacial_gain_is_recorded_but_structurally_unusable() {
    use pv_module::StatedBifacialGain;

    let module = coherent_builder()
        .with_bifacial(
            BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)))
                .with_measurement_standard("IEC TS 60904-1-2")
                .with_stated_gain(
                    StatedBifacialGain::new(Ratio::new(0.09))
                        .with_reference_albedo(Ratio::new(0.25)),
                ),
        )
        .build()
        .unwrap();

    let bifacial = module.bifacial().unwrap();
    let gain = bifacial.stated_gain.as_ref().unwrap();
    assert!((gain.gain.value() - 0.09).abs() < 1e-12);
    assert!((gain.reference_albedo.unwrap().value() - 0.25).abs() < 1e-12);

    // The figure is a commercial claim: the real gain depends on the site,
    // which is not a property of the module. Nothing in this crate
    // computes with it.
    assert!(StatedBifacialGain::USAGE_NOTE.contains("must not be used in simulation"));
    assert!((bifacial.bifaciality_factor.as_ref().unwrap().get().value() - 0.70).abs() < 1e-12);
}

#[test]
fn a_current_measured_in_milliamperes_reaches_the_model_in_amperes() {
    // Conversion belongs at the boundary, so the model never holds a
    // quantity whose unit depends on where it was read.
    let from_source = Measurement::measured(10_400.0_f64).with_provenance(datasheet());
    let i_sc = from_source.map(Current::from_milliamperes);
    assert!((i_sc.get().amperes() - 10.4).abs() < 1e-12);

    let mut electrical = coherent_electrical();
    electrical.i_sc = i_sc;
    let module = builder_with_electrical(electrical).build().unwrap();
    assert!((module.electrical().i_sc.get().amperes() - 10.4).abs() < 1e-12);
    assert_eq!(
        module.electrical().i_sc.classification(),
        DataClass::Measured
    );
}
