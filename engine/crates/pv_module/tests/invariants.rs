//! One test per invariant, each exercising at least three cases: the
//! coherent case in which the invariant must stay silent, and the ways it
//! can be broken.
//!
//! Every check is called directly rather than through
//! [`pv_module::validation::invariants::validate_module`], because the
//! specification requires each invariant to be *individually testable*
//! (`docs/design/27-specification-finale-pv-module.md` §11). Perturbing one
//! quantity and calling one check attributes the finding unambiguously;
//! going through the orchestrator would not.
//!
//! Traceability: the test name carries the invariant identifier, so
//! `cargo test inv_geo_002` runs exactly that invariant's cases.

mod common;

use common::*;
use core_units::{
    Area, Current, Irradiance, Power, Ratio, Temperature, TemperatureCoefficient, Voltage,
};
use pv_module::validation::invariants::*;
use pv_module::{
    BifacialCharacteristics, CellArchitecture, CellConfiguration, CellDivision, CellTechnology,
    DataClass, ElectricalState, Measurement, ModuleGeometry, ModuleTechnology, OperatingCondition,
    OperatingConditionKind, OperatingPoint, RearSideParameters, Spectrum, TemperatureSpecification,
    ValidationStatus,
};

// ---------------------------------------------------------------------
// Positivity: INV-POS-001 to INV-POS-012
// ---------------------------------------------------------------------

/// Each of the five mandatory electrical quantities, checked the same way:
/// the coherent value is silent, and zero, negative and non-finite are all
/// invalid.
fn assert_electrical_positivity(
    id: &str,
    perturb: impl Fn(f64) -> pv_module::ElectricalCharacteristics,
) {
    let coherent = report_from(|r| check_electrical_positivity(&coherent_electrical(), r));
    assert_silent(&coherent, id);

    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let report = report_from(|r| check_electrical_positivity(&perturb(bad), r));
        assert_reported(&report, id, ValidationStatus::Invalid);
    }
}

#[test]
fn inv_pos_001_maximum_power_is_strictly_positive() {
    assert_electrical_positivity("INV-POS-001", |p| electrical(p, 49.5, 10.4, 41.5, 9.64));
}

#[test]
fn inv_pos_002_open_circuit_voltage_is_strictly_positive() {
    assert_electrical_positivity("INV-POS-002", |v| electrical(400.0, v, 10.4, 41.5, 9.64));
}

#[test]
fn inv_pos_003_short_circuit_current_is_strictly_positive() {
    assert_electrical_positivity("INV-POS-003", |i| electrical(400.0, 49.5, i, 41.5, 9.64));
}

#[test]
fn inv_pos_004_maximum_power_voltage_is_strictly_positive() {
    assert_electrical_positivity("INV-POS-004", |v| electrical(400.0, 49.5, 10.4, v, 9.64));
}

#[test]
fn inv_pos_005_maximum_power_current_is_strictly_positive() {
    assert_electrical_positivity("INV-POS-005", |i| electrical(400.0, 49.5, 10.4, 41.5, i));
}

#[test]
fn inv_pos_006_length_and_width_are_strictly_positive() {
    let coherent = report_from(|r| check_geometry_positivity(&geometry(2.0, 1.0), r));
    assert_silent(&coherent, "INV-POS-006");

    for (l, w) in [(0.0, 1.0), (2.0, 0.0), (-2.0, 1.0), (f64::NAN, 1.0)] {
        let report = report_from(|r| check_geometry_positivity(&geometry(l, w), r));
        assert_reported(&report, "INV-POS-006", ValidationStatus::Invalid);
    }
}

#[test]
fn inv_pos_007_a_stated_depth_is_strictly_positive() {
    // Absent is not a defect.
    let absent = report_from(|r| check_geometry_positivity(&geometry(2.0, 1.0), r));
    assert_silent(&absent, "INV-POS-007");

    let stated = ModuleGeometry::new(dimensions(2.0, 1.0).with_depth(length(0.030)));
    let report = report_from(|r| check_geometry_positivity(&stated, r));
    assert_silent(&report, "INV-POS-007");

    for bad in [0.0, -0.030, f64::NAN] {
        let g = ModuleGeometry::new(dimensions(2.0, 1.0).with_depth(length(bad)));
        let report = report_from(|r| check_geometry_positivity(&g, r));
        assert_reported(&report, "INV-POS-007", ValidationStatus::Invalid);
    }
}

#[test]
fn inv_pos_008_a_stated_mass_is_strictly_positive() {
    let absent = report_from(|r| check_geometry_positivity(&geometry(2.0, 1.0), r));
    assert_silent(&absent, "INV-POS-008");

    let stated = geometry(2.0, 1.0).with_mass(mass(21.5));
    let report = report_from(|r| check_geometry_positivity(&stated, r));
    assert_silent(&report, "INV-POS-008");

    for bad in [0.0, -21.5, f64::NAN] {
        let g = geometry(2.0, 1.0).with_mass(mass(bad));
        let report = report_from(|r| check_geometry_positivity(&g, r));
        assert_reported(&report, "INV-POS-008", ValidationStatus::Invalid);
    }
}

#[test]
fn inv_pos_009_a_discrete_module_has_at_least_one_series_cell() {
    let coherent = report_from(|r| check_configuration_positivity(&coherent_configuration(), r));
    assert_silent(&coherent, "INV-POS-009");

    let none = report_from(|r| check_configuration_positivity(&CellConfiguration::discrete(0), r));
    assert_reported(&none, "INV-POS-009", ValidationStatus::Invalid);

    // A monolithic module is governed by INV-POS-011 instead.
    let monolithic = report_from(|r| {
        check_configuration_positivity(&CellConfiguration::Monolithic { series_segments: 0 }, r)
    });
    assert_silent(&monolithic, "INV-POS-009");
}

#[test]
fn inv_pos_010_a_discrete_module_has_at_least_one_parallel_path() {
    let one = report_from(|r| check_configuration_positivity(&coherent_configuration(), r));
    assert_silent(&one, "INV-POS-010");

    let two = CellConfiguration::Discrete {
        cells_in_series: 72,
        cells_in_parallel: 2,
        cell_division: CellDivision::Half,
        substrings: None,
        bypass_diodes: None,
    };
    let report = report_from(|r| check_configuration_positivity(&two, r));
    assert_silent(&report, "INV-POS-010");

    let none = CellConfiguration::Discrete {
        cells_in_series: 72,
        cells_in_parallel: 0,
        cell_division: CellDivision::Full,
        substrings: None,
        bypass_diodes: None,
    };
    let report = report_from(|r| check_configuration_positivity(&none, r));
    assert_reported(&report, "INV-POS-010", ValidationStatus::Invalid);
}

#[test]
fn inv_pos_011_a_monolithic_module_has_at_least_one_segment() {
    let coherent = report_from(|r| {
        check_configuration_positivity(
            &CellConfiguration::Monolithic {
                series_segments: 154,
            },
            r,
        )
    });
    assert_silent(&coherent, "INV-POS-011");

    let none = report_from(|r| {
        check_configuration_positivity(&CellConfiguration::Monolithic { series_segments: 0 }, r)
    });
    assert_reported(&none, "INV-POS-011", ValidationStatus::Invalid);

    // A discrete module is governed by INV-POS-009 instead.
    let discrete =
        report_from(|r| check_configuration_positivity(&CellConfiguration::discrete(0), r));
    assert_silent(&discrete, "INV-POS-011");
}

#[test]
fn inv_pos_012_every_stated_area_is_strictly_positive() {
    let none = report_from(|r| check_geometry_positivity(&geometry(2.0, 1.0), r));
    assert_silent(&none, "INV-POS-012");

    let coherent = report_from(|r| check_geometry_positivity(&coherent_geometry(), r));
    assert_silent(&coherent, "INV-POS-012");

    // Each of the four stated areas is covered.
    let cases: [ModuleGeometry; 4] = [
        geometry(2.0, 1.0).with_gross_area(area(0.0)),
        geometry(2.0, 1.0).with_active_area(area(-1.0)),
        geometry(2.0, 1.0).with_aperture_area(area(f64::NAN)),
        geometry(2.0, 1.0).with_cell_area(area(0.0)),
    ];
    for g in cases {
        let report = report_from(|r| check_geometry_positivity(&g, r));
        assert_reported(&report, "INV-POS-012", ValidationStatus::Invalid);
    }
}

// ---------------------------------------------------------------------
// Electrical ordering: INV-ELE-001 to INV-ELE-012
// ---------------------------------------------------------------------

#[test]
fn inv_ele_001_open_circuit_voltage_exceeds_the_maximum_power_voltage() {
    let coherent = report_from(|r| check_electrical_ordering(&coherent_electrical(), r));
    assert_silent(&coherent, "INV-ELE-001");

    // Equality is already impossible: at v_oc the current is zero.
    let equal =
        report_from(|r| check_electrical_ordering(&electrical(400.0, 49.5, 10.4, 49.5, 9.64), r));
    assert_reported(&equal, "INV-ELE-001", ValidationStatus::Invalid);

    let inverted =
        report_from(|r| check_electrical_ordering(&electrical(400.0, 41.5, 10.4, 49.5, 9.64), r));
    assert_reported(&inverted, "INV-ELE-001", ValidationStatus::Invalid);
}

#[test]
fn inv_ele_002_short_circuit_current_exceeds_the_maximum_power_current() {
    let coherent = report_from(|r| check_electrical_ordering(&coherent_electrical(), r));
    assert_silent(&coherent, "INV-ELE-002");

    let equal =
        report_from(|r| check_electrical_ordering(&electrical(400.0, 49.5, 9.64, 41.5, 9.64), r));
    assert_reported(&equal, "INV-ELE-002", ValidationStatus::Invalid);

    let inverted =
        report_from(|r| check_electrical_ordering(&electrical(400.0, 49.5, 9.0, 41.5, 10.4), r));
    assert_reported(&inverted, "INV-ELE-002", ValidationStatus::Invalid);
}

#[test]
fn inv_ele_003_the_voltage_ratio_stays_in_the_usual_range() {
    // 41.5 / 49.5 = 0.838.
    let coherent = report_from(|r| check_electrical_ordering(&coherent_electrical(), r));
    assert_silent(&coherent, "INV-ELE-003");

    // 30.0 / 49.5 = 0.606, below the range.
    let low =
        report_from(|r| check_electrical_ordering(&electrical(289.2, 49.5, 10.4, 30.0, 9.64), r));
    assert_reported(&low, "INV-ELE-003", ValidationStatus::Warning);

    // 47.5 / 49.5 = 0.960, above the range.
    let high =
        report_from(|r| check_electrical_ordering(&electrical(457.9, 49.5, 10.4, 47.5, 9.64), r));
    assert_reported(&high, "INV-ELE-003", ValidationStatus::Warning);

    // Both ends of the interior are admissible. The bounds themselves are
    // not probed by a multiply-then-divide round trip, which does not
    // land on them exactly in binary floating point; the range's
    // inclusivity is a property of the code, not something a test can
    // establish by constructing an exactly-equal ratio.
    let (low_bound, high_bound) = VMPP_VOC_RATIO_RANGE;
    assert!(low_bound < high_bound);
    for ratio in [low_bound + 0.01, high_bound - 0.01] {
        let inside = electrical(400.0, 49.5, 10.4, 49.5 * ratio, 9.64);
        let report = report_from(|r| check_electrical_ordering(&inside, r));
        assert_silent(&report, "INV-ELE-003");
    }
}

#[test]
fn inv_ele_004_stated_power_agrees_with_the_maximum_power_product() {
    // 41.5 V x 9.64 A = 400.06 W against a stated 400 W: datasheet rounding.
    let coherent = report_from(|r| check_electrical_ordering(&coherent_electrical(), r));
    assert_silent(&coherent, "INV-ELE-004");

    // 2 % apart: beyond rounding, but both figures remain plausible.
    let warned =
        report_from(|r| check_electrical_ordering(&electrical(408.0, 49.5, 10.4, 41.5, 9.64), r));
    assert_reported(&warned, "INV-ELE-004", ValidationStatus::Warning);

    // 10 % apart: a contradiction the engine will not arbitrate.
    let conflict =
        report_from(|r| check_electrical_ordering(&electrical(440.0, 49.5, 10.4, 41.5, 9.64), r));
    assert_reported(&conflict, "INV-ELE-004", ValidationStatus::Conflict);
    let issue = conflict.find("INV-ELE-004").unwrap();
    assert!(issue.observed().unwrap().contains("400.0"));
    assert!(issue.expected().is_some());
}

#[test]
fn inv_ele_005_the_fill_factor_stays_in_the_commercial_range() {
    // 400 / (49.5 x 10.4) = 0.777.
    let coherent = report_from(|r| check_electrical_ordering(&coherent_electrical(), r));
    assert_silent(&coherent, "INV-ELE-005");

    let (low, high) = FILL_FACTOR_RANGE;
    let denominator = 49.5 * 10.4;

    let too_low = electrical(denominator * (low - 0.1), 49.5, 10.4, 41.5, 9.64);
    let report = report_from(|r| check_electrical_ordering(&too_low, r));
    assert_reported(&report, "INV-ELE-005", ValidationStatus::Warning);

    let too_high = electrical(denominator * (high + 0.05), 49.5, 10.4, 41.5, 9.64);
    let report = report_from(|r| check_electrical_ordering(&too_high, r));
    assert_reported(&report, "INV-ELE-005", ValidationStatus::Warning);

    // A degenerate module yields a non-finite fill factor and no finding
    // rather than a panic.
    let degenerate =
        report_from(|r| check_electrical_ordering(&electrical(100.0, 0.0, 0.0, 0.0, 0.0), r));
    assert_silent(&degenerate, "INV-ELE-005");
}

#[test]
fn inv_ele_006_the_fill_factor_never_exceeds_one() {
    let coherent = report_from(|r| check_electrical_ordering(&coherent_electrical(), r));
    assert_silent(&coherent, "INV-ELE-006");

    // Exactly 1 is the physical limit, not a violation of it.
    let at_limit = electrical(49.5 * 10.4, 49.5, 10.4, 41.5, 9.64);
    let report = report_from(|r| check_electrical_ordering(&at_limit, r));
    assert_silent(&report, "INV-ELE-006");

    // Above it, the maximum power exceeds v_oc x i_sc, which is impossible.
    let impossible = electrical(600.0, 49.5, 10.4, 41.5, 9.64);
    let report = report_from(|r| check_electrical_ordering(&impossible, r));
    assert_reported(&report, "INV-ELE-006", ValidationStatus::Invalid);
    // The two fill-factor invariants are mutually exclusive by construction.
    assert_silent(&report, "INV-ELE-005");
}

#[test]
fn inv_ele_010_the_voltage_per_series_junction_is_plausible() {
    // 49.5 V over 72 junctions = 0.688 V.
    let coherent = report_from(|r| {
        check_voltage_per_junction(&coherent_electrical(), &coherent_configuration(), r)
    });
    assert_silent(&coherent, "INV-ELE-010");

    // 20 V over 72 junctions = 0.278 V: either the count or the voltage is
    // wrong, and the invariant does not say which.
    let low = report_from(|r| {
        check_voltage_per_junction(
            &electrical(154.0, 20.0, 10.4, 16.0, 9.64),
            &coherent_configuration(),
            r,
        )
    });
    assert_reported(&low, "INV-ELE-010", ValidationStatus::Warning);

    // The 144-half-cell trap: reading "144" as the series count halves the
    // volts per junction.
    let doubled = CellConfiguration::Discrete {
        cells_in_series: 144,
        cells_in_parallel: 1,
        cell_division: CellDivision::Half,
        substrings: None,
        bypass_diodes: None,
    };
    let report = report_from(|r| check_voltage_per_junction(&coherent_electrical(), &doubled, r));
    assert_reported(&report, "INV-ELE-010", ValidationStatus::Warning);

    // A configuration with no junctions cannot be divided by.
    let empty = report_from(|r| {
        check_voltage_per_junction(
            &coherent_electrical(),
            &CellConfiguration::Monolithic { series_segments: 0 },
            r,
        )
    });
    assert_silent(&empty, "INV-ELE-010");
}

#[test]
fn inv_ele_011_the_system_voltage_admits_the_module_itself() {
    let coherent = report_from(|r| check_limits(&coherent_electrical(), &coherent_limits(), r));
    assert_silent(&coherent, "INV-ELE-011");

    // Equal is admissible: a string of exactly one module.
    let equal = report_from(|r| check_limits(&coherent_electrical(), &limits(49.5), r));
    assert_silent(&equal, "INV-ELE-011");

    let below = report_from(|r| check_limits(&coherent_electrical(), &limits(40.0), r));
    assert_reported(&below, "INV-ELE-011", ValidationStatus::Invalid);
}

#[test]
fn inv_ele_012_the_fuse_rating_covers_the_short_circuit_current() {
    // Absent is not a defect.
    let absent = report_from(|r| check_limits(&coherent_electrical(), &coherent_limits(), r));
    assert_silent(&absent, "INV-ELE-012");

    let adequate =
        coherent_limits().with_max_series_fuse(Measurement::stated(Current::from_amperes(20.0)));
    let report = report_from(|r| check_limits(&coherent_electrical(), &adequate, r));
    assert_silent(&report, "INV-ELE-012");

    let inadequate =
        coherent_limits().with_max_series_fuse(Measurement::stated(Current::from_amperes(5.0)));
    let report = report_from(|r| check_limits(&coherent_electrical(), &inadequate, r));
    assert_reported(&report, "INV-ELE-012", ValidationStatus::Warning);
}

// ---------------------------------------------------------------------
// Geometry: INV-GEO-001 to INV-GEO-005
// ---------------------------------------------------------------------

#[test]
fn inv_geo_001_the_aperture_never_exceeds_the_overall_area() {
    let absent = report_from(|r| check_geometry_relations(&geometry(2.0, 1.0), r));
    assert_silent(&absent, "INV-GEO-001");

    let nested = geometry(2.0, 1.0)
        .with_gross_area(area(2.0))
        .with_aperture_area(area(1.8));
    let report = report_from(|r| check_geometry_relations(&nested, r));
    assert_silent(&report, "INV-GEO-001");

    let inverted = geometry(2.0, 1.0)
        .with_gross_area(area(2.0))
        .with_aperture_area(area(2.5));
    let report = report_from(|r| check_geometry_relations(&inverted, r));
    assert_reported(&report, "INV-GEO-001", ValidationStatus::Invalid);
}

#[test]
fn inv_geo_002_a_stated_area_contradicting_the_dimensions_is_a_conflict() {
    // Agreement.
    let coherent = report_from(|r| check_geometry_relations(&coherent_geometry(), r));
    assert_silent(&coherent, "INV-GEO-002");

    // Within rounding: 2.001 m2 against 2.000 m2 is 0.05 %.
    let rounded = geometry(2.0, 1.0).with_gross_area(area(2.001));
    let report = report_from(|r| check_geometry_relations(&rounded, r));
    assert_silent(&report, "INV-GEO-002");

    // Beyond rounding but below the conflict threshold: 0.5 %.
    let drifted = geometry(2.0, 1.0).with_gross_area(area(2.01));
    let report = report_from(|r| check_geometry_relations(&drifted, r));
    assert_reported(&report, "INV-GEO-002", ValidationStatus::Warning);

    // The Sharp_ND_72ELUF shape from the CEC library: a factor of two, and
    // the *derived* figure is the wrong one
    // (docs/design/26-validation-empirique-cec-sandia.md section 1).
    let contradictory = ModuleGeometry::new(dimensions(1.165, 0.990))
        .with_gross_area(Measurement::stated(Area::from_square_meters(0.577)));
    let report = report_from(|r| check_geometry_relations(&contradictory, r));
    assert_reported(&report, "INV-GEO-002", ValidationStatus::Conflict);

    // The conflict names both candidates, so a decision can be made.
    let issue = report.find("INV-GEO-002").unwrap();
    let observed = issue.observed().unwrap();
    assert!(observed.contains("0.577"), "observed was {observed}");
    assert!(observed.contains("1.15"), "observed was {observed}");
    assert_eq!(issue.fields(), &["gross_area", "rectangular_area"]);
}

#[test]
fn inv_geo_003_the_areas_nest_from_cell_to_aperture() {
    let absent = report_from(|r| check_geometry_relations(&geometry(2.0, 1.0), r));
    assert_silent(&absent, "INV-GEO-003");

    let nested = geometry(2.0, 1.0)
        .with_cell_area(area(1.5))
        .with_active_area(area(1.8))
        .with_aperture_area(area(1.9));
    let report = report_from(|r| check_geometry_relations(&nested, r));
    assert_silent(&report, "INV-GEO-003");

    // Cell area above active area.
    let broken_inner = geometry(2.0, 1.0)
        .with_cell_area(area(1.9))
        .with_active_area(area(1.5));
    let report = report_from(|r| check_geometry_relations(&broken_inner, r));
    assert_reported(&report, "INV-GEO-003", ValidationStatus::Invalid);

    // Active area above aperture area.
    let broken_outer = geometry(2.0, 1.0)
        .with_active_area(area(1.9))
        .with_aperture_area(area(1.5));
    let report = report_from(|r| check_geometry_relations(&broken_outer, r));
    assert_reported(&report, "INV-GEO-003", ValidationStatus::Invalid);
}

#[test]
fn inv_geo_004_the_area_per_series_junction_matches_a_known_cell_format() {
    // Absent is not a defect.
    let absent = report_from(|r| {
        check_cell_area_plausibility(&geometry(2.0, 1.0), &coherent_configuration(), r)
    });
    assert_silent(&absent, "INV-GEO-004");

    // 72 junctions of 166 mm square cells: 0.0276 m2 each.
    let plausible = geometry(2.0, 1.0).with_cell_area(area(72.0 * 0.0276));
    let report =
        report_from(|r| check_cell_area_plausibility(&plausible, &coherent_configuration(), r));
    assert_silent(&report, "INV-GEO-004");

    let too_small = geometry(2.0, 1.0).with_cell_area(area(0.5));
    let report =
        report_from(|r| check_cell_area_plausibility(&too_small, &coherent_configuration(), r));
    assert_reported(&report, "INV-GEO-004", ValidationStatus::Warning);

    let too_large = geometry(2.0, 1.0).with_cell_area(area(5.0));
    let report =
        report_from(|r| check_cell_area_plausibility(&too_large, &coherent_configuration(), r));
    assert_reported(&report, "INV-GEO-004", ValidationStatus::Warning);

    // No junctions means nothing to divide by.
    let report = report_from(|r| {
        check_cell_area_plausibility(
            &plausible,
            &CellConfiguration::Monolithic { series_segments: 0 },
            r,
        )
    });
    assert_silent(&report, "INV-GEO-004");
}

#[test]
fn inv_geo_005_the_module_is_thinner_than_its_smallest_lateral_dimension() {
    let absent = report_from(|r| check_geometry_relations(&geometry(2.0, 1.0), r));
    assert_silent(&absent, "INV-GEO-005");

    let normal = ModuleGeometry::new(dimensions(2.0, 1.0).with_depth(length(0.030)));
    let report = report_from(|r| check_geometry_relations(&normal, r));
    assert_silent(&report, "INV-GEO-005");

    // A metre-thick module: possible to state, never seen in a product.
    let thick = ModuleGeometry::new(dimensions(2.0, 1.0).with_depth(length(1.5)));
    let report = report_from(|r| check_geometry_relations(&thick, r));
    assert_reported(&report, "INV-GEO-005", ValidationStatus::Warning);
}

// ---------------------------------------------------------------------
// Derived efficiency: INV-DER-001 to INV-DER-003
// ---------------------------------------------------------------------

#[test]
fn inv_der_001_a_stated_efficiency_contradicting_the_derived_one_is_a_conflict() {
    // 400 W over 2 m2 at 1000 W/m2 is 20 %.
    let agreeing =
        coherent_electrical().with_stated_efficiency(Measurement::stated(Ratio::new(0.20)));
    let report = report_from(|r| check_efficiency(&agreeing, &coherent_geometry(), r));
    assert_silent(&report, "INV-DER-001");

    // 1 % apart: within the threshold, because a datasheet rounds.
    let rounded =
        coherent_electrical().with_stated_efficiency(Measurement::stated(Ratio::new(0.202)));
    let report = report_from(|r| check_efficiency(&rounded, &coherent_geometry(), r));
    assert_silent(&report, "INV-DER-001");

    // 5 % apart: the manufacturer used a different area and did not say
    // which, so the engine reports rather than chooses.
    let contradicting =
        coherent_electrical().with_stated_efficiency(Measurement::stated(Ratio::new(0.21)));
    let report = report_from(|r| check_efficiency(&contradicting, &coherent_geometry(), r));
    assert_reported(&report, "INV-DER-001", ValidationStatus::Conflict);
    assert!(
        report
            .find("INV-DER-001")
            .unwrap()
            .fields()
            .contains(&"gross_area"),
        "the conflict must name the area it used"
    );
}

#[test]
fn inv_der_002_the_derived_efficiency_stays_below_the_physical_bound() {
    let coherent =
        report_from(|r| check_efficiency(&coherent_electrical(), &coherent_geometry(), r));
    assert_silent(&coherent, "INV-DER-002");

    // Exactly at the bound is not a violation of it.
    let at_bound = electrical(
        EFFICIENCY_ABSOLUTE_MAX * 2.0 * 1000.0,
        49.5,
        10.4,
        41.5,
        9.64,
    );
    let report = report_from(|r| check_efficiency(&at_bound, &coherent_geometry(), r));
    assert_silent(&report, "INV-DER-002");

    // 60 %: either the power or the area is wrong.
    let impossible = electrical(1200.0, 49.5, 10.4, 41.5, 9.64);
    let report = report_from(|r| check_efficiency(&impossible, &coherent_geometry(), r));
    assert_reported(&report, "INV-DER-002", ValidationStatus::Invalid);
    // The two derived-efficiency invariants are mutually exclusive.
    assert_silent(&report, "INV-DER-003");
}

#[test]
fn inv_der_003_the_derived_efficiency_stays_in_the_commercial_range() {
    let coherent =
        report_from(|r| check_efficiency(&coherent_electrical(), &coherent_geometry(), r));
    assert_silent(&coherent, "INV-DER-003");

    // 2 %: below anything sold.
    let low = report_from(|r| {
        check_efficiency(
            &electrical(40.0, 49.5, 10.4, 41.5, 9.64),
            &coherent_geometry(),
            r,
        )
    });
    assert_reported(&low, "INV-DER-003", ValidationStatus::Warning);

    // 35 %: above any commercial module, below the physical bound.
    let high = report_from(|r| {
        check_efficiency(
            &electrical(700.0, 49.5, 10.4, 41.5, 9.64),
            &coherent_geometry(),
            r,
        )
    });
    assert_reported(&high, "INV-DER-003", ValidationStatus::Warning);

    // A zero area yields no efficiency and therefore no finding.
    let degenerate =
        report_from(|r| check_efficiency(&coherent_electrical(), &geometry(0.0, 1.0), r));
    assert_silent(&degenerate, "INV-DER-003");
}

#[test]
fn the_efficiency_invariants_name_the_area_they_used() {
    // Aperture is preferred over the stated overall area, and the finding
    // says so, because an efficiency without its reference area is
    // ambiguous by several points.
    let with_aperture = coherent_geometry().with_aperture_area(area(1.8));
    let report = report_from(|r| {
        check_efficiency(
            &electrical(700.0, 49.5, 10.4, 41.5, 9.64),
            &with_aperture,
            r,
        )
    });
    let issue = report.find("INV-DER-003").unwrap();
    assert!(issue.fields().contains(&"aperture_area"));
    assert!(issue.observed().unwrap().contains("aperture_area"));

    // With no stated area at all, the derived rectangle is used and named.
    let report = report_from(|r| {
        check_efficiency(
            &electrical(700.0, 49.5, 10.4, 41.5, 9.64),
            &geometry(2.0, 1.0),
            r,
        )
    });
    assert!(
        report
            .find("INV-DER-003")
            .unwrap()
            .fields()
            .contains(&"rectangular_area")
    );
}

// ---------------------------------------------------------------------
// Thermal: INV-TMP-001 to INV-TMP-006
// ---------------------------------------------------------------------

#[test]
fn inv_tmp_001_the_current_coefficient_is_usually_positive() {
    let coherent = report_from(|r| check_thermal_coefficients(&coherent_thermal(), r));
    assert_silent(&coherent, "INV-TMP-001");

    let negative = report_from(|r| check_thermal_coefficients(&thermal(-0.045, -0.27, -0.34), r));
    assert_reported(&negative, "INV-TMP-001", ValidationStatus::Warning);

    let zero = report_from(|r| check_thermal_coefficients(&thermal(0.0, -0.27, -0.34), r));
    assert_reported(&zero, "INV-TMP-001", ValidationStatus::Warning);

    // A warning, never a rejection: an unexpected sign is a physical
    // judgement, not a format error.
    assert!(!negative.has_invalid());
}

#[test]
fn inv_tmp_002_the_voltage_coefficient_is_usually_negative() {
    let coherent = report_from(|r| check_thermal_coefficients(&coherent_thermal(), r));
    assert_silent(&coherent, "INV-TMP-002");

    let positive = report_from(|r| check_thermal_coefficients(&thermal(0.045, 0.27, -0.34), r));
    assert_reported(&positive, "INV-TMP-002", ValidationStatus::Warning);

    let zero = report_from(|r| check_thermal_coefficients(&thermal(0.045, 0.0, -0.34), r));
    assert_reported(&zero, "INV-TMP-002", ValidationStatus::Warning);
}

#[test]
fn inv_tmp_003_the_power_coefficient_is_usually_negative() {
    let coherent = report_from(|r| check_thermal_coefficients(&coherent_thermal(), r));
    assert_silent(&coherent, "INV-TMP-003");

    let positive = report_from(|r| check_thermal_coefficients(&thermal(0.045, -0.27, 0.34), r));
    assert_reported(&positive, "INV-TMP-003", ValidationStatus::Warning);

    let zero = report_from(|r| check_thermal_coefficients(&thermal(0.045, -0.27, 0.0), r));
    assert_reported(&zero, "INV-TMP-003", ValidationStatus::Warning);
}

#[test]
fn inv_tmp_004_the_power_coefficient_magnitude_is_plausible() {
    let coherent = report_from(|r| check_thermal_coefficients(&coherent_thermal(), r));
    assert_silent(&coherent, "INV-TMP-004");

    let (low, high) = GAMMA_MAGNITUDE_RANGE_PERCENT;

    let too_small =
        report_from(|r| check_thermal_coefficients(&thermal(0.045, -0.02, -low / 2.0), r));
    assert_reported(&too_small, "INV-TMP-004", ValidationStatus::Warning);

    let too_large =
        report_from(|r| check_thermal_coefficients(&thermal(0.045, -0.27, -high * 1.5), r));
    assert_reported(&too_large, "INV-TMP-004", ValidationStatus::Warning);

    // An absolute coefficient is in W/K, not %/K: the magnitude range does
    // not apply and the invariant stays silent rather than guessing.
    let absolute = pv_module::ThermalCharacteristics::new(
        coefficient(0.045),
        coefficient(-0.27),
        Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(-1.7)),
    );
    let report = report_from(|r| check_thermal_coefficients(&absolute, r));
    assert_silent(&report, "INV-TMP-004");
}

#[test]
fn inv_tmp_005_an_ambiguous_coefficient_basis_is_a_conflict_never_a_guess() {
    // The CEC library states alpha and beta in A/K and V/K but gamma in
    // %/K, within one record. An adapter that cannot tell which it has
    // must report, not infer from the magnitude.
    let issue = ambiguous_coefficient_basis("gamma_p_max", "-0.5072");
    assert_eq!(issue.invariant_id(), "INV-TMP-005");
    assert_eq!(issue.status(), ValidationStatus::Conflict);
    assert_eq!(issue.observed(), Some("-0.5072"));
    assert!(issue.expected().unwrap().contains("explicit unit"));
    assert!(issue.fields().contains(&"gamma_p_max"));

    // It is never emitted by the coefficient checks themselves: only an
    // adapter, which alone knows its source, can raise it.
    for t in [
        coherent_thermal(),
        thermal(0.045, -0.27, -1.7),
        thermal(-0.045, 0.27, 0.34),
    ] {
        let report = report_from(|r| check_thermal_coefficients(&t, r));
        assert_silent(&report, "INV-TMP-005");
    }
}

#[test]
fn inv_tmp_006_the_power_coefficient_usually_dominates_the_voltage_one() {
    // |gamma| = 0.34 >= |beta| = 0.27: nothing to say.
    let coherent = report_from(|r| check_thermal_coefficients(&coherent_thermal(), r));
    assert_silent(&coherent, "INV-TMP-006");

    // Equality is not an inversion.
    let equal = report_from(|r| check_thermal_coefficients(&thermal(0.045, -0.30, -0.30), r));
    assert_silent(&equal, "INV-TMP-006");

    // |gamma| = 0.20 < |beta| = 0.27: recorded as informational, since a
    // technology could legitimately behave this way.
    let inverted = report_from(|r| check_thermal_coefficients(&thermal(0.045, -0.27, -0.20), r));
    assert_reported(&inverted, "INV-TMP-006", ValidationStatus::Valid);
    assert!(!inverted.worst_status().is_blocking());
}

// ---------------------------------------------------------------------
// Reference conditions: INV-CND-001 to INV-CND-006
// ---------------------------------------------------------------------

fn condition_at(irradiance: f64, celsius: f64) -> OperatingCondition {
    OperatingCondition {
        kind: OperatingConditionKind::UserDefined {
            name: "test".to_string(),
        },
        irradiance: Irradiance::from_watts_per_m2(irradiance),
        temperature: TemperatureSpecification::CellTemperature(Temperature::from_celsius(celsius)),
        spectrum: Some(Spectrum::Am15Global),
        wind_speed: None,
        electrical_state: Some(ElectricalState::MaximumPowerPoint),
    }
}

#[test]
fn inv_cnd_001_a_reference_temperature_is_physically_admissible() {
    let stc = [OperatingPoint::new(OperatingCondition::stc())];
    let report = report_from(|r| check_operating_points(&stc, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-001");

    for celsius in [500.0, -200.0, f64::NAN] {
        let points = [OperatingPoint::new(condition_at(1000.0, celsius))];
        let report = report_from(|r| check_operating_points(&points, &coherent_limits(), 400.0, r));
        assert_reported(&report, "INV-CND-001", ValidationStatus::Invalid);
    }

    // The bounds themselves are admissible.
    let (low, high) = TEMPERATURE_RANGE_CELSIUS;
    for celsius in [low, high] {
        let points = [OperatingPoint::new(condition_at(1000.0, celsius))];
        let report = report_from(|r| check_operating_points(&points, &coherent_limits(), 400.0, r));
        assert_silent(&report, "INV-CND-001");
    }
}

#[test]
fn inv_cnd_002_the_operating_temperature_range_is_ordered() {
    let absent = report_from(|r| check_operating_points(&[], &coherent_limits(), 400.0, r));
    assert_silent(&absent, "INV-CND-002");

    let ordered =
        coherent_limits().with_operating_temperature_range(temperature(-40.0), temperature(85.0));
    let report = report_from(|r| check_operating_points(&[], &ordered, 400.0, r));
    assert_silent(&report, "INV-CND-002");

    let inverted =
        coherent_limits().with_operating_temperature_range(temperature(85.0), temperature(-40.0));
    let report = report_from(|r| check_operating_points(&[], &inverted, 400.0, r));
    assert_reported(&report, "INV-CND-002", ValidationStatus::Invalid);

    let degenerate =
        coherent_limits().with_operating_temperature_range(temperature(25.0), temperature(25.0));
    let report = report_from(|r| check_operating_points(&[], &degenerate, 400.0, r));
    assert_reported(&report, "INV-CND-002", ValidationStatus::Invalid);
}

#[test]
fn inv_cnd_003_the_settled_cell_temperature_is_plausible() {
    let typical = [OperatingPoint::new(OperatingCondition::noct())
        .with_cell_temperature(Measurement::measured(Temperature::from_celsius(44.0)))];
    let report = report_from(|r| check_operating_points(&typical, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-003");

    for celsius in [80.0, 10.0] {
        let points = [OperatingPoint::new(OperatingCondition::noct())
            .with_cell_temperature(Measurement::measured(Temperature::from_celsius(celsius)))];
        let report = report_from(|r| check_operating_points(&points, &coherent_limits(), 400.0, r));
        assert_reported(&report, "INV-CND-003", ValidationStatus::Warning);
    }

    // NMOT lets the cell settle in the same way, so the range applies.
    let nmot = [OperatingPoint::new(OperatingCondition::nmot())
        .with_cell_temperature(Measurement::measured(Temperature::from_celsius(80.0)))];
    let report = report_from(|r| check_operating_points(&nmot, &coherent_limits(), 400.0, r));
    assert_reported(&report, "INV-CND-003", ValidationStatus::Warning);

    // STC imposes the cell temperature rather than letting it settle, so
    // the nominal range does not apply there.
    let stc = [OperatingPoint::new(OperatingCondition::stc())
        .with_cell_temperature(Measurement::measured(Temperature::from_celsius(80.0)))];
    let report = report_from(|r| check_operating_points(&stc, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-003");
}

#[test]
fn inv_cnd_004_a_reference_condition_imposes_a_positive_irradiance() {
    let stc = [OperatingPoint::new(OperatingCondition::stc())];
    let report = report_from(|r| check_operating_points(&stc, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-004");

    for irradiance in [0.0, -800.0, f64::NAN] {
        let points = [OperatingPoint::new(condition_at(irradiance, 25.0))];
        let report = report_from(|r| check_operating_points(&points, &coherent_limits(), 400.0, r));
        assert_reported(&report, "INV-CND-004", ValidationStatus::Invalid);
    }
}

#[test]
fn inv_cnd_005_a_named_reference_condition_is_declared_only_once() {
    let single = [OperatingPoint::new(OperatingCondition::noct())];
    let report = report_from(|r| check_operating_points(&single, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-005");

    // Two different conditions are not a duplicate.
    let distinct = [
        OperatingPoint::new(OperatingCondition::noct()),
        OperatingPoint::new(OperatingCondition::ptc()),
    ];
    let report = report_from(|r| check_operating_points(&distinct, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-005");

    let duplicated = [
        OperatingPoint::new(OperatingCondition::noct()),
        OperatingPoint::new(OperatingCondition::noct()),
    ];
    let report = report_from(|r| check_operating_points(&duplicated, &coherent_limits(), 400.0, r));
    assert_reported(&report, "INV-CND-005", ValidationStatus::Invalid);

    // Matrix points are many by construction and exempt.
    let matrix: Vec<OperatingPoint> = [1000.0, 800.0, 600.0, 400.0, 200.0]
        .into_iter()
        .map(|g| {
            OperatingPoint::new(OperatingCondition::iec61853_matrix_point(
                Irradiance::from_watts_per_m2(g),
                Temperature::from_celsius(25.0),
            ))
        })
        .collect();
    let report = report_from(|r| check_operating_points(&matrix, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-005");
}

#[test]
fn inv_cnd_006_nominal_power_falls_below_its_standard_test_value() {
    // 800 W/m2 and a hotter cell: the power must be lower.
    let typical = [OperatingPoint::new(OperatingCondition::noct())
        .with_p_max(Measurement::stated(Power::from_watts(295.0)))];
    let report = report_from(|r| check_operating_points(&typical, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-006");

    let higher = [OperatingPoint::new(OperatingCondition::noct())
        .with_p_max(Measurement::stated(Power::from_watts(450.0)))];
    let report = report_from(|r| check_operating_points(&higher, &coherent_limits(), 400.0, r));
    assert_reported(&report, "INV-CND-006", ValidationStatus::Warning);

    let equal = [OperatingPoint::new(OperatingCondition::nmot())
        .with_p_max(Measurement::stated(Power::from_watts(400.0)))];
    let report = report_from(|r| check_operating_points(&equal, &coherent_limits(), 400.0, r));
    assert_reported(&report, "INV-CND-006", ValidationStatus::Warning);

    // With no stated power there is nothing to compare.
    let absent = [OperatingPoint::new(OperatingCondition::noct())];
    let report = report_from(|r| check_operating_points(&absent, &coherent_limits(), 400.0, r));
    assert_silent(&report, "INV-CND-006");
}

// ---------------------------------------------------------------------
// Bifaciality: INV-BIF-001 to INV-BIF-004
// ---------------------------------------------------------------------

#[test]
fn inv_bif_001_a_bifacial_flag_without_characterisation_is_reported() {
    // What every CEC import yields: a flag and nothing else.
    let flag_only = BifacialCharacteristics::flag_only();
    let report = report_from(|r| check_bifacial(&flag_only, &coherent_electrical(), &silicon(), r));
    assert_reported(&report, "INV-BIF-001", ValidationStatus::Warning);
    // Reported, never blocking: the module is usable monofacially.
    assert!(!report.worst_status().is_blocking());

    let with_factor = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)));
    let report =
        report_from(|r| check_bifacial(&with_factor, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-001");

    let with_rear =
        BifacialCharacteristics::flag_only().with_rear_side_parameters(RearSideParameters {
            p_max: Some(Measurement::measured(Power::from_watts(280.0))),
            ..RearSideParameters::empty()
        });
    let report = report_from(|r| check_bifacial(&with_rear, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-001");

    // An explicitly empty rear-side block characterises nothing.
    let empty_rear =
        BifacialCharacteristics::flag_only().with_rear_side_parameters(RearSideParameters::empty());
    let report =
        report_from(|r| check_bifacial(&empty_rear, &coherent_electrical(), &silicon(), r));
    assert_reported(&report, "INV-BIF-001", ValidationStatus::Warning);
}

#[test]
fn inv_bif_002_the_bifaciality_factor_is_strictly_positive() {
    let typical = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)));
    let report = report_from(|r| check_bifacial(&typical, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-002");

    let zero = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.0)));
    let report = report_from(|r| check_bifacial(&zero, &coherent_electrical(), &silicon(), r));
    assert_reported(&report, "INV-BIF-002", ValidationStatus::Invalid);

    // A negative source value clamps to zero, which is still not positive.
    let clamped = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::clamped(-0.5)));
    let report = report_from(|r| check_bifacial(&clamped, &coherent_electrical(), &silicon(), r));
    assert_reported(&report, "INV-BIF-002", ValidationStatus::Invalid);

    // Unity is admissible: a fully symmetric module.
    let unity = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(1.0)));
    let report = report_from(|r| check_bifacial(&unity, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-002");
}

#[test]
fn inv_bif_003_rear_power_stays_below_front_power() {
    let none = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)));
    let report = report_from(|r| check_bifacial(&none, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-003");

    let lower = none.clone().with_rear_side_parameters(RearSideParameters {
        p_max: Some(Measurement::measured(Power::from_watts(280.0))),
        ..RearSideParameters::empty()
    });
    let report = report_from(|r| check_bifacial(&lower, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-003");

    let higher = none.with_rear_side_parameters(RearSideParameters {
        p_max: Some(Measurement::measured(Power::from_watts(500.0))),
        ..RearSideParameters::empty()
    });
    let report = report_from(|r| check_bifacial(&higher, &coherent_electrical(), &silicon(), r));
    assert_reported(&report, "INV-BIF-003", ValidationStatus::Warning);
}

#[test]
fn inv_bif_004_bifaciality_on_a_back_contact_architecture_is_reported() {
    let b = BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)));

    let report = report_from(|r| check_bifacial(&b, &coherent_electrical(), &silicon(), r));
    assert_silent(&report, "INV-BIF-004");

    let hjt = ModuleTechnology::new(
        CellTechnology::MonocrystallineSilicon,
        CellArchitecture::HeterojunctionHJT,
    );
    let report = report_from(|r| check_bifacial(&b, &coherent_electrical(), &hjt, r));
    assert_silent(&report, "INV-BIF-004");

    // The rear face of an IBC cell carries the contacts.
    let ibc = ModuleTechnology::new(
        CellTechnology::MonocrystallineSilicon,
        CellArchitecture::InterdigitatedBackContact,
    );
    let report = report_from(|r| check_bifacial(&b, &coherent_electrical(), &ibc, r));
    assert_reported(&report, "INV-BIF-004", ValidationStatus::Warning);
}

// ---------------------------------------------------------------------
// Structure: INV-STR-001 to INV-STR-004
// ---------------------------------------------------------------------

fn discrete(
    cells_in_parallel: u32,
    cell_division: CellDivision,
    substrings: Option<u32>,
    bypass_diodes: Option<u32>,
) -> CellConfiguration {
    CellConfiguration::Discrete {
        cells_in_series: 72,
        cells_in_parallel,
        cell_division,
        substrings,
        bypass_diodes,
    }
}

#[test]
fn inv_str_001_half_cut_modules_pair_their_parallel_paths() {
    let paired = discrete(2, CellDivision::Half, None, None);
    let report = report_from(|r| check_structure(&paired, &silicon(), 0, r));
    assert_silent(&report, "INV-STR-001");

    let odd = discrete(3, CellDivision::Half, None, None);
    let report = report_from(|r| check_structure(&odd, &silicon(), 0, r));
    assert_reported(&report, "INV-STR-001", ValidationStatus::Warning);

    // Whole cells impose no such pairing.
    let whole = discrete(3, CellDivision::Full, None, None);
    let report = report_from(|r| check_structure(&whole, &silicon(), 0, r));
    assert_silent(&report, "INV-STR-001");

    // A monolithic module has no cells to cut.
    let monolithic = CellConfiguration::Monolithic {
        series_segments: 154,
    };
    let report = report_from(|r| check_structure(&monolithic, &cadmium_telluride(), 0, r));
    assert_silent(&report, "INV-STR-001");
}

#[test]
fn inv_str_002_bypass_diodes_do_not_outnumber_the_substrings() {
    let unstated = discrete(1, CellDivision::Full, None, None);
    let report = report_from(|r| check_structure(&unstated, &silicon(), 0, r));
    assert_silent(&report, "INV-STR-002");

    let matched = discrete(1, CellDivision::Full, Some(3), Some(3));
    let report = report_from(|r| check_structure(&matched, &silicon(), 0, r));
    assert_silent(&report, "INV-STR-002");

    let excess = discrete(1, CellDivision::Full, Some(3), Some(4));
    let report = report_from(|r| check_structure(&excess, &silicon(), 0, r));
    assert_reported(&report, "INV-STR-002", ValidationStatus::Warning);
}

#[test]
fn inv_str_003_additional_outputs_belong_to_tandems() {
    let report = report_from(|r| check_structure(&coherent_configuration(), &silicon(), 0, r));
    assert_silent(&report, "INV-STR-003");

    let report = report_from(|r| check_structure(&coherent_configuration(), &tandem(), 2, r));
    assert_silent(&report, "INV-STR-003");

    let report = report_from(|r| check_structure(&coherent_configuration(), &silicon(), 1, r));
    assert_reported(&report, "INV-STR-003", ValidationStatus::Warning);

    let report =
        report_from(|r| check_structure(&coherent_configuration(), &cadmium_telluride(), 1, r));
    assert_reported(&report, "INV-STR-003", ValidationStatus::Warning);
}

#[test]
fn inv_str_004_thin_film_materials_pair_with_monolithic_configurations() {
    // Silicon with discrete cells: the ordinary case.
    let report = report_from(|r| check_structure(&coherent_configuration(), &silicon(), 0, r));
    assert_silent(&report, "INV-STR-004");

    // Cadmium telluride, monolithically scribed: also ordinary.
    let monolithic = CellConfiguration::Monolithic {
        series_segments: 154,
    };
    let report = report_from(|r| check_structure(&monolithic, &cadmium_telluride(), 0, r));
    assert_silent(&report, "INV-STR-004");

    // Cadmium telluride with countable discrete cells: not a known
    // construction, so the pairing is reported.
    let report =
        report_from(|r| check_structure(&coherent_configuration(), &cadmium_telluride(), 0, r));
    assert_reported(&report, "INV-STR-004", ValidationStatus::Warning);

    // Silicon on a monolithic configuration is not this invariant's
    // concern: it warns only in the thin-film direction.
    let report = report_from(|r| check_structure(&monolithic, &silicon(), 0, r));
    assert_silent(&report, "INV-STR-004");
}

// ---------------------------------------------------------------------
// Provenance: INV-PRV-001 to INV-PRV-003
// ---------------------------------------------------------------------

fn datasheet_provenance() -> pv_module::Provenance {
    pv_module::Provenance::new(
        pv_module::SourceType::ManufacturerDatasheet,
        pv_module::ExtractionMethod::ManualTranscription,
    )
}

#[test]
fn inv_prv_001_a_value_without_provenance_is_reported_but_usable() {
    let bare = Measurement::stated(Voltage::from_volts(49.5));
    let report = report_from(|r| check_provenance("v_oc", &bare, r));
    assert_reported(&report, "INV-PRV-001", ValidationStatus::Warning);
    assert!(!report.worst_status().is_blocking());

    let attributed = bare.clone().with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &attributed, r));
    assert_silent(&report, "INV-PRV-001");

    // The class makes no difference: a measurement is as unattributed as a
    // datasheet figure when nothing says where it came from.
    let measured = Measurement::measured(Voltage::from_volts(49.5));
    let report = report_from(|r| check_provenance("v_oc", &measured, r));
    assert_reported(&report, "INV-PRV-001", ValidationStatus::Warning);
}

#[test]
fn inv_prv_002_an_estimate_without_a_rationale_is_refused() {
    let justified = Measurement::estimated(
        Voltage::from_volts(49.5),
        "read from a photograph of the label",
    )
    .with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &justified, r));
    assert_silent(&report, "INV-PRV-002");

    // An estimate with no reason recorded is indistinguishable from an
    // invention, which is what the transparency principle forbids.
    let unjustified = Measurement::new(Voltage::from_volts(49.5), DataClass::Estimated)
        .with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &unjustified, r));
    assert_reported(&report, "INV-PRV-002", ValidationStatus::Invalid);

    // An empty string is not a rationale.
    let empty = Measurement::new(Voltage::from_volts(49.5), DataClass::Estimated)
        .with_rationale("")
        .with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &empty, r));
    assert_reported(&report, "INV-PRV-002", ValidationStatus::Invalid);

    // No other class requires one.
    for class in [
        DataClass::ManufacturerData,
        DataClass::Measured,
        DataClass::Normative,
        DataClass::Derived,
    ] {
        let m = Measurement::new(Voltage::from_volts(49.5), class)
            .with_provenance(datasheet_provenance());
        let report = report_from(|r| check_provenance("v_oc", &m, r));
        assert_silent(&report, "INV-PRV-002");
    }
}

#[test]
fn inv_prv_003_a_fitted_parameter_never_occupies_a_physical_field() {
    let physical =
        Measurement::stated(Voltage::from_volts(49.5)).with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &physical, r));
    assert_silent(&report, "INV-PRV-003");

    // A De Soto coefficient is meaningless outside the model that produced
    // it, and is not a property of the module.
    let fitted = Measurement::new(Voltage::from_volts(49.5), DataClass::Fitted)
        .with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &fitted, r));
    assert_reported(&report, "INV-PRV-003", ValidationStatus::Invalid);

    // A model parameter is not a fitted one, and is not caught here.
    let model = Measurement::new(Voltage::from_volts(49.5), DataClass::ModelParameter)
        .with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &model, r));
    assert_silent(&report, "INV-PRV-003");

    let measured =
        Measurement::measured(Voltage::from_volts(49.5)).with_provenance(datasheet_provenance());
    let report = report_from(|r| check_provenance("v_oc", &measured, r));
    assert_silent(&report, "INV-PRV-003");
}

// ---------------------------------------------------------------------
// The inventory itself
// ---------------------------------------------------------------------

#[test]
fn the_invariant_inventory_is_complete_and_free_of_duplicates() {
    assert_eq!(ALL_INVARIANT_IDS.len(), 52);

    let mut sorted = ALL_INVARIANT_IDS;
    sorted.sort_unstable();
    let mut deduplicated = sorted;
    let unique = {
        let mut v: Vec<&str> = deduplicated.to_vec();
        v.dedup();
        v.len()
    };
    assert_eq!(
        unique,
        ALL_INVARIANT_IDS.len(),
        "identifiers must be unique"
    );
    deduplicated.sort_unstable();

    for id in ALL_INVARIANT_IDS {
        assert!(id.starts_with("INV-"), "{id} is malformed");
        assert_eq!(id.len(), 11, "{id} is malformed");
        assert!(pv_module::is_known_invariant(id));
    }
}

#[test]
fn every_family_is_represented_in_the_inventory() {
    // The eight families of the specification, with the count each
    // contributes. Their sum is the inventory's length, so a family added
    // without updating this test fails here rather than silently.
    let families: [(&str, usize); 8] = [
        ("INV-POS-", 12),
        ("INV-ELE-", 9),
        ("INV-GEO-", 5),
        ("INV-DER-", 3),
        ("INV-TMP-", 6),
        ("INV-CND-", 6),
        ("INV-BIF-", 4),
        ("INV-STR-", 4),
    ];
    let mut counted = 3; // the three provenance invariants
    for (prefix, expected) in families {
        let found = ALL_INVARIANT_IDS
            .iter()
            .filter(|id| id.starts_with(prefix))
            .count();
        assert_eq!(
            found, expected,
            "{prefix} has {found} members, expected {expected}"
        );
        counted += found;
    }
    assert_eq!(counted, ALL_INVARIANT_IDS.len());
}

/// A warning threshold above its conflict threshold would make the warning
/// unreachable. Checked in a `const` block so a bad revision fails the
/// build rather than a test run.
const _: () = {
    assert!(AREA_AGREEMENT_WARNING < AREA_AGREEMENT_CONFLICT);
    assert!(POWER_CONSISTENCY_WARNING < POWER_CONSISTENCY_CONFLICT);
    assert!(EFFICIENCY_AGREEMENT_CONFLICT > 0.0);
    assert!(EFFICIENCY_TYPICAL_RANGE.1 < EFFICIENCY_ABSOLUTE_MAX);
    // A metre-square cell is not a cell.
    assert!(CELL_AREA_RANGE.1 < 1.0);
};

#[test]
fn thresholds_are_ordered_and_within_their_own_ranges() {
    for (low, high) in [
        VMPP_VOC_RATIO_RANGE,
        FILL_FACTOR_RANGE,
        VOLTS_PER_JUNCTION_RANGE,
        EFFICIENCY_TYPICAL_RANGE,
        GAMMA_MAGNITUDE_RANGE_PERCENT,
        TEMPERATURE_RANGE_CELSIUS,
        NOCT_RANGE_CELSIUS,
        CELL_AREA_RANGE,
    ] {
        assert!(low < high, "range ({low}, {high}) is inverted");
    }

    // The nominal operating range is inside the admissible one, so
    // INV-CND-003 can only fire where INV-CND-001 stayed silent.
    assert!(NOCT_RANGE_CELSIUS.0 > TEMPERATURE_RANGE_CELSIUS.0);
    assert!(NOCT_RANGE_CELSIUS.1 < TEMPERATURE_RANGE_CELSIUS.1);
}
