//! Exercises the parts of the public API the invariant and integration
//! suites do not reach, so that every published entry point has at least
//! one test asserting what it actually does.

mod common;

use common::*;
use core_units::{Current, Power, Ratio, Voltage};
use pv_module::adapters::{ImportOutcome, ModuleSourceAdapter};
use pv_module::{
    CellConfiguration, DataClass, ElectricalOutput, ExtractionMethod, Measurement,
    OperatingCondition, OperatingPoint, PowerTolerance, Provenance, SourceType, Uncertainty,
    UncertaintyKind,
};

#[test]
fn an_operating_point_records_a_full_current_voltage_summary() {
    // A test report states more than the maximum power at each condition.
    let point = OperatingPoint::new(OperatingCondition::noct())
        .with_p_max(Measurement::measured(Power::from_watts(296.0)))
        .with_v_oc(Measurement::measured(Voltage::from_volts(46.4)))
        .with_i_sc(Measurement::measured(Current::from_amperes(8.35)))
        .with_mpp(
            Measurement::measured(Voltage::from_volts(38.3)),
            Measurement::measured(Current::from_amperes(7.73)),
        );

    assert!((point.v_oc.as_ref().unwrap().get().volts() - 46.4).abs() < 1e-12);
    assert!((point.i_sc.as_ref().unwrap().get().amperes() - 8.35).abs() < 1e-12);
    assert!((point.v_mpp.as_ref().unwrap().get().volts() - 38.3).abs() < 1e-12);
    assert!((point.i_mpp.as_ref().unwrap().get().amperes() - 7.73).abs() < 1e-12);

    // The condition is unchanged by recording measurements at it.
    assert!(!point.condition.temperature.is_cell_temperature());
}

#[test]
fn both_forms_of_production_tolerance_are_representable() {
    // The prevailing modern form is asymmetric, and not expressible as a
    // single number — which is why the type has two variants.
    let asymmetric = coherent_electrical().with_power_tolerance(PowerTolerance::Asymmetric {
        minimum: Power::from_watts(400.0),
        maximum: Power::from_watts(405.0),
    });
    match asymmetric.power_tolerance.unwrap() {
        PowerTolerance::Asymmetric { minimum, maximum } => {
            assert!((minimum.watts() - 400.0).abs() < 1e-12);
            assert!((maximum.watts() - 405.0).abs() < 1e-12);
        }
        other => panic!("expected an asymmetric tolerance, got {other:?}"),
    }

    let symmetric = coherent_electrical().with_power_tolerance(PowerTolerance::Symmetric {
        relative: Ratio::new(0.03),
    });
    match symmetric.power_tolerance.unwrap() {
        PowerTolerance::Symmetric { relative } => {
            assert!((relative.value() - 0.03).abs() < 1e-12);
        }
        other => panic!("expected a symmetric tolerance, got {other:?}"),
    }

    // A production tolerance is not a measurement uncertainty: it applies
    // to a manufacturing lot, not to a measured specimen, and the two are
    // held in different places.
    assert!(coherent_electrical().p_max.uncertainty().is_none());
}

#[test]
fn the_reverse_current_rating_is_recorded_when_stated() {
    let limits = coherent_limits()
        .with_max_reverse_current(Measurement::stated(Current::from_amperes(20.0)));
    assert!((limits.max_reverse_current.as_ref().unwrap().get().amperes() - 20.0).abs() < 1e-12);
    // Unstated stays unstated.
    assert!(coherent_limits().max_reverse_current.is_none());
}

#[test]
fn a_tandem_carries_its_independent_electrical_outputs() {
    // A four-terminal tandem contacts its two sub-cells separately, which
    // is the only construction known to produce additional outputs.
    let top = ElectricalOutput::new(
        "perovskite top cell",
        electrical(180.0, 32.0, 7.4, 26.5, 6.8),
    );
    let bottom = ElectricalOutput::new(
        "silicon bottom cell",
        electrical(220.0, 40.0, 7.1, 33.0, 6.67),
    );

    let module = builder_with(
        tandem(),
        coherent_configuration(),
        coherent_geometry(),
        coherent_electrical(),
        coherent_thermal(),
        coherent_limits(),
    )
    .with_additional_output(top)
    .with_additional_output(bottom)
    .build()
    .expect("a tandem with separate terminals must build");

    assert_eq!(module.additional_outputs().len(), 2);
    assert_eq!(module.additional_outputs()[0].label, "perovskite top cell");
    assert!(
        (module.additional_outputs()[1]
            .characteristics
            .p_max
            .get()
            .watts()
            - 220.0)
            .abs()
            < 1e-12
    );

    // On a tandem, additional outputs are expected rather than reported.
    assert!(!module.report().contains("INV-STR-003"));
}

#[test]
fn a_database_redistributes_the_manufacturers_authority_not_its_own() {
    // A database is a channel, not a new authority: a value it carries was
    // still declared by whoever measured or published it. Only a value the
    // engine itself computed becomes derived.
    for source in [
        SourceType::PublicDatabase {
            name: "CEC".to_string(),
        },
        SourceType::ProprietaryDatabase {
            name: "a licensed library".to_string(),
        },
    ] {
        let provenance = Provenance::new(
            source,
            ExtractionMethod::AutomatedImport {
                adapter: "CECAdapter".to_string(),
                adapter_version: "0.2.0".to_string(),
            },
        );
        assert_eq!(provenance.implied_class(), DataClass::ManufacturerData);
    }

    let computed = Provenance::new(SourceType::Derived, ExtractionMethod::ComputedByEngine);
    assert_eq!(computed.implied_class(), DataClass::Derived);

    // An OCR transcription is distinguishable from a keyed one, because
    // the two have different error modes.
    let scanned = Provenance::new(
        SourceType::ManufacturerDatasheet,
        ExtractionMethod::OpticalCharacterRecognition,
    );
    assert_ne!(
        *scanned.extraction_method(),
        ExtractionMethod::ManualTranscription
    );
    assert_eq!(scanned.implied_class(), DataClass::ManufacturerData);
}

#[test]
fn an_uncertainty_reports_its_magnitude_and_its_form() {
    let absolute = Uncertainty::standard_absolute(0.5);
    assert!((absolute.value() - 0.5).abs() < 1e-12);
    assert_eq!(absolute.kind(), UncertaintyKind::Absolute);

    let relative = Uncertainty::expanded_relative(0.03, 2.0);
    assert!((relative.value() - 0.03).abs() < 1e-12);
    assert_eq!(relative.kind(), UncertaintyKind::Relative);

    // 3 % of 400 W is 12 W: the two forms are not interchangeable without
    // the value they refer to, which is why the form is carried.
    assert!((relative.value() * 400.0 - 12.0).abs() < 1e-12);
}

/// An adapter that hands back whatever it was given, to exercise the
/// outcome type's own API.
struct PassthroughAdapter;

impl ModuleSourceAdapter for PassthroughAdapter {
    type Record = ();

    fn name(&self) -> &'static str {
        "PassthroughAdapter"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn import(&self, _record: &()) -> ImportOutcome {
        ImportOutcome::imported(
            coherent_builder(),
            self.provenance_for(SourceType::ManufacturerDatasheet),
        )
    }
}

#[test]
fn an_import_outcome_yields_its_builder_for_further_assembly() {
    let outcome = PassthroughAdapter.import(&());
    assert!(outcome.is_imported());
    assert_eq!(
        outcome.provenance().implied_class(),
        DataClass::ManufacturerData
    );

    // Taking the builder out lets a caller add what the source did not
    // carry before validating.
    let builder = outcome.into_builder().expect("a builder was expected");
    let module = builder
        .with_certification(pv_module::Certification::Iec61730)
        .build()
        .expect("the imported description must build");
    assert_eq!(module.certifications().len(), 1);
}

#[test]
fn a_discrete_configuration_reports_its_own_shape() {
    let simple = CellConfiguration::discrete(72);
    assert_eq!(simple.junctions_in_series(), 72);
    assert_eq!(simple.parallel_paths(), 1);
    assert!(!simple.is_monolithic());

    match simple {
        CellConfiguration::Discrete {
            cells_in_series,
            cells_in_parallel,
            ..
        } => {
            assert_eq!(cells_in_series, 72);
            assert_eq!(cells_in_parallel, 1);
        }
        other => panic!("expected a discrete configuration, got {other:?}"),
    }
}
