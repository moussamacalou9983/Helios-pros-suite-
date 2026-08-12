//! Every invariant, exercised by a declared set of scenarios.
//!
//! ## How this suite is organised, and why
//!
//! The scenarios are **data**, not statements. Each one names the
//! invariant it belongs to, describes its situation, and declares the
//! outcome it expects before anything runs. Three things follow that a
//! conventional suite cannot offer:
//!
//! 1. **The count is measurable.** `the_scenario_census_is_written_to_disk`
//!    reports how many scenarios were actually *executed* per invariant.
//!    Counting `#[test]` functions would say nothing: one function may
//!    assert three cases or thirty.
//! 2. **The floor cannot regress silently.**
//!    `every_invariant_carries_at_least_three_scenarios` fails the build if
//!    any invariant drops below
//!    [`scenario::MINIMUM_SCENARIOS_PER_INVARIANT`], and the CI runs it.
//! 3. **Each invariant stays individually runnable.** One `#[test]` per
//!    invariant is generated below, so `cargo test inv_geo_002` still runs
//!    exactly that invariant's scenarios — as
//!    `docs/design/27-specification-finale-pv-module.md` §11 requires.
//!
//! Every check is called directly rather than through
//! [`pv_module::validation::invariants::validate_module`], so that a
//! finding is attributable to the one quantity the scenario perturbed.
//!
//! A scenario expecting **silence** counts as much as one expecting a
//! finding: an invariant that fires when it should not is exactly as
//! defective as one that stays quiet when it should not.

mod common;
mod scenario;

use common::*;
use core_units::{
    Area, Current, Irradiance, Power, Ratio, Temperature, TemperatureCoefficient, Voltage,
};
use pv_module::validation::invariants::*;
use pv_module::{
    BifacialCharacteristics, CellArchitecture, CellConfiguration, CellDivision, CellTechnology,
    DataClass, ElectricalCharacteristics, ElectricalState, Measurement, ModuleGeometry,
    ModuleTechnology, OperatingCondition, OperatingConditionKind, OperatingPoint,
    RearSideParameters, Spectrum, TemperatureSpecification, ValidationIssue, ValidationReport,
    ValidationStatus,
};
use scenario::{
    InvariantCensus, MINIMUM_SCENARIOS_PER_INVARIANT, Scenario, census, render_table, reported,
    reported_with, silent,
};

use ValidationStatus::{Conflict, Invalid, Valid, Warning};

// =====================================================================
// Situation builders
//
// Free functions so that each scenario's `run` is a plain fn pointer with
// nothing captured: the scenario table stays a value, inspectable and
// countable.
// =====================================================================

fn electrical_report(e: ElectricalCharacteristics) -> ValidationReport {
    report_from(|r| check_electrical_positivity(&e, r))
}

fn ordering_report(e: ElectricalCharacteristics) -> ValidationReport {
    report_from(|r| check_electrical_ordering(&e, r))
}

fn geometry_positivity_report(g: ModuleGeometry) -> ValidationReport {
    report_from(|r| check_geometry_positivity(&g, r))
}

fn geometry_relations_report(g: ModuleGeometry) -> ValidationReport {
    report_from(|r| check_geometry_relations(&g, r))
}

fn configuration_report(c: CellConfiguration) -> ValidationReport {
    report_from(|r| check_configuration_positivity(&c, r))
}

fn thermal_report(t: pv_module::ThermalCharacteristics) -> ValidationReport {
    report_from(|r| check_thermal_coefficients(&t, r))
}

fn efficiency_report(e: ElectricalCharacteristics, g: ModuleGeometry) -> ValidationReport {
    report_from(|r| check_efficiency(&e, &g, r))
}

fn points_report(points: &[OperatingPoint], stc_watts: f64) -> ValidationReport {
    report_from(|r| check_operating_points(points, &coherent_limits(), stc_watts, r))
}

fn points_with_limits_report(limits: pv_module::ElectricalLimits) -> ValidationReport {
    report_from(|r| check_operating_points(&[], &limits, 400.0, r))
}

fn bifacial_report(b: BifacialCharacteristics, t: ModuleTechnology) -> ValidationReport {
    report_from(|r| check_bifacial(&b, &coherent_electrical(), &t, r))
}

fn structure_report(
    c: CellConfiguration,
    t: ModuleTechnology,
    additional_outputs: usize,
) -> ValidationReport {
    report_from(|r| check_structure(&c, &t, additional_outputs, r))
}

fn provenance_report<T>(m: Measurement<T>) -> ValidationReport {
    report_from(|r| check_provenance("v_oc", &m, r))
}

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

fn discrete_config(
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

fn monolithic(series_segments: u32) -> CellConfiguration {
    CellConfiguration::Monolithic { series_segments }
}

fn volts(v: f64) -> Measurement<Voltage> {
    Measurement::stated(Voltage::from_volts(v))
}

fn attributed(m: Measurement<Voltage>) -> Measurement<Voltage> {
    m.with_provenance(datasheet_provenance())
}

// --- Further assertions used by a few scenarios ------------------------

fn names_both_areas(issue: &ValidationIssue) -> Result<(), String> {
    let observed = issue.observed().ok_or("no observed value recorded")?;
    if !observed.contains("0.577") || !observed.contains("1.15") {
        return Err(format!(
            "both candidate areas must be named, found: {observed}"
        ));
    }
    if issue.fields() != ["gross_area", "rectangular_area"] {
        return Err(format!(
            "both fields must be named, found {:?}",
            issue.fields()
        ));
    }
    if !issue.explanation().contains("does not choose") {
        return Err("the explanation must state that the engine does not arbitrate".to_string());
    }
    Ok(())
}

fn names_both_power_figures(issue: &ValidationIssue) -> Result<(), String> {
    let observed = issue.observed().ok_or("no observed value recorded")?;
    if !observed.contains("440.0") || !observed.contains("400.0") {
        return Err(format!(
            "both power figures must be named, found: {observed}"
        ));
    }
    if issue.expected().is_none() {
        return Err("the tolerance that was exceeded must be stated".to_string());
    }
    Ok(())
}

fn names_the_area_used(issue: &ValidationIssue) -> Result<(), String> {
    if !issue.fields().contains(&"gross_area") {
        return Err(format!(
            "the area the efficiency was derived from must be named, found {:?}",
            issue.fields()
        ));
    }
    Ok(())
}

fn names_the_aperture(issue: &ValidationIssue) -> Result<(), String> {
    if !issue.fields().contains(&"aperture_area")
        || !issue
            .observed()
            .unwrap_or_default()
            .contains("aperture_area")
    {
        return Err("the preferred area must be named in the finding".to_string());
    }
    Ok(())
}

// =====================================================================
// The scenario registry
// =====================================================================

/// Every scenario this suite executes, in specification order.
fn all_scenarios() -> Vec<Scenario> {
    let mut s = Vec::new();
    s.extend(positivity_scenarios());
    s.extend(electrical_scenarios());
    s.extend(geometry_scenarios());
    s.extend(efficiency_scenarios());
    s.extend(thermal_scenarios());
    s.extend(condition_scenarios());
    s.extend(bifacial_scenarios());
    s.extend(structure_scenarios());
    s.extend(provenance_scenarios());
    s
}

// --- INV-POS-001 to INV-POS-012 ---------------------------------------

fn positivity_scenarios() -> Vec<Scenario> {
    vec![
        // INV-POS-001 — maximum power
        silent("INV-POS-001", "a coherent 400 W module", || {
            electrical_report(coherent_electrical())
        }),
        reported("INV-POS-001", "zero power", Invalid, || {
            electrical_report(electrical(0.0, 49.5, 10.4, 41.5, 9.64))
        }),
        reported("INV-POS-001", "negative power", Invalid, || {
            electrical_report(electrical(-400.0, 49.5, 10.4, 41.5, 9.64))
        }),
        reported("INV-POS-001", "power is NaN", Invalid, || {
            electrical_report(electrical(f64::NAN, 49.5, 10.4, 41.5, 9.64))
        }),
        reported("INV-POS-001", "power is infinite", Invalid, || {
            electrical_report(electrical(f64::INFINITY, 49.5, 10.4, 41.5, 9.64))
        }),
        // INV-POS-002 — open-circuit voltage
        silent("INV-POS-002", "a coherent 49.5 V module", || {
            electrical_report(coherent_electrical())
        }),
        reported("INV-POS-002", "zero open-circuit voltage", Invalid, || {
            electrical_report(electrical(400.0, 0.0, 10.4, 41.5, 9.64))
        }),
        reported(
            "INV-POS-002",
            "negative open-circuit voltage",
            Invalid,
            || electrical_report(electrical(400.0, -49.5, 10.4, 41.5, 9.64)),
        ),
        reported(
            "INV-POS-002",
            "open-circuit voltage is NaN",
            Invalid,
            || electrical_report(electrical(400.0, f64::NAN, 10.4, 41.5, 9.64)),
        ),
        // INV-POS-003 — short-circuit current
        silent("INV-POS-003", "a coherent 10.4 A module", || {
            electrical_report(coherent_electrical())
        }),
        reported("INV-POS-003", "zero short-circuit current", Invalid, || {
            electrical_report(electrical(400.0, 49.5, 0.0, 41.5, 9.64))
        }),
        reported(
            "INV-POS-003",
            "negative short-circuit current",
            Invalid,
            || electrical_report(electrical(400.0, 49.5, -10.4, 41.5, 9.64)),
        ),
        reported(
            "INV-POS-003",
            "short-circuit current is NaN",
            Invalid,
            || electrical_report(electrical(400.0, 49.5, f64::NAN, 41.5, 9.64)),
        ),
        // INV-POS-004 — maximum-power voltage
        silent(
            "INV-POS-004",
            "a coherent 41.5 V maximum-power point",
            || electrical_report(coherent_electrical()),
        ),
        reported("INV-POS-004", "zero maximum-power voltage", Invalid, || {
            electrical_report(electrical(400.0, 49.5, 10.4, 0.0, 9.64))
        }),
        reported(
            "INV-POS-004",
            "negative maximum-power voltage",
            Invalid,
            || electrical_report(electrical(400.0, 49.5, 10.4, -41.5, 9.64)),
        ),
        reported(
            "INV-POS-004",
            "maximum-power voltage is NaN",
            Invalid,
            || electrical_report(electrical(400.0, 49.5, 10.4, f64::NAN, 9.64)),
        ),
        // INV-POS-005 — maximum-power current
        silent(
            "INV-POS-005",
            "a coherent 9.64 A maximum-power point",
            || electrical_report(coherent_electrical()),
        ),
        reported("INV-POS-005", "zero maximum-power current", Invalid, || {
            electrical_report(electrical(400.0, 49.5, 10.4, 41.5, 0.0))
        }),
        reported(
            "INV-POS-005",
            "negative maximum-power current",
            Invalid,
            || electrical_report(electrical(400.0, 49.5, 10.4, 41.5, -9.64)),
        ),
        reported(
            "INV-POS-005",
            "maximum-power current is NaN",
            Invalid,
            || electrical_report(electrical(400.0, 49.5, 10.4, 41.5, f64::NAN)),
        ),
        // INV-POS-006 — length and width
        silent("INV-POS-006", "a 2 m by 1 m module", || {
            geometry_positivity_report(geometry(2.0, 1.0))
        }),
        reported("INV-POS-006", "zero length", Invalid, || {
            geometry_positivity_report(geometry(0.0, 1.0))
        }),
        reported("INV-POS-006", "zero width", Invalid, || {
            geometry_positivity_report(geometry(2.0, 0.0))
        }),
        reported("INV-POS-006", "negative length", Invalid, || {
            geometry_positivity_report(geometry(-2.0, 1.0))
        }),
        reported("INV-POS-006", "length is NaN", Invalid, || {
            geometry_positivity_report(geometry(f64::NAN, 1.0))
        }),
        // INV-POS-007 — depth
        silent(
            "INV-POS-007",
            "no depth stated (absence is not a defect)",
            || geometry_positivity_report(geometry(2.0, 1.0)),
        ),
        silent("INV-POS-007", "a stated 30 mm depth", || {
            geometry_positivity_report(ModuleGeometry::new(
                dimensions(2.0, 1.0).with_depth(length(0.030)),
            ))
        }),
        reported("INV-POS-007", "zero depth stated", Invalid, || {
            geometry_positivity_report(ModuleGeometry::new(
                dimensions(2.0, 1.0).with_depth(length(0.0)),
            ))
        }),
        reported("INV-POS-007", "negative depth stated", Invalid, || {
            geometry_positivity_report(ModuleGeometry::new(
                dimensions(2.0, 1.0).with_depth(length(-0.030)),
            ))
        }),
        reported("INV-POS-007", "depth is NaN", Invalid, || {
            geometry_positivity_report(ModuleGeometry::new(
                dimensions(2.0, 1.0).with_depth(length(f64::NAN)),
            ))
        }),
        // INV-POS-008 — mass
        silent("INV-POS-008", "no mass stated", || {
            geometry_positivity_report(geometry(2.0, 1.0))
        }),
        silent("INV-POS-008", "a stated 21.5 kg mass", || {
            geometry_positivity_report(geometry(2.0, 1.0).with_mass(mass(21.5)))
        }),
        reported("INV-POS-008", "zero mass stated", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_mass(mass(0.0)))
        }),
        reported("INV-POS-008", "negative mass stated", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_mass(mass(-21.5)))
        }),
        reported("INV-POS-008", "mass is NaN", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_mass(mass(f64::NAN)))
        }),
        // INV-POS-009 — cells in series
        silent("INV-POS-009", "72 cells in series", || {
            configuration_report(coherent_configuration())
        }),
        reported("INV-POS-009", "no cell in series", Invalid, || {
            configuration_report(CellConfiguration::discrete(0))
        }),
        silent(
            "INV-POS-009",
            "a monolithic module (INV-POS-011 governs it)",
            || configuration_report(monolithic(0)),
        ),
        // INV-POS-010 — parallel paths
        silent("INV-POS-010", "one parallel path", || {
            configuration_report(coherent_configuration())
        }),
        silent("INV-POS-010", "two parallel paths (half-cut)", || {
            configuration_report(discrete_config(2, CellDivision::Half, None, None))
        }),
        reported("INV-POS-010", "no parallel path", Invalid, || {
            configuration_report(discrete_config(0, CellDivision::Full, None, None))
        }),
        // INV-POS-011 — series segments
        silent("INV-POS-011", "154 scribed segments", || {
            configuration_report(monolithic(154))
        }),
        reported("INV-POS-011", "no series segment", Invalid, || {
            configuration_report(monolithic(0))
        }),
        silent(
            "INV-POS-011",
            "a discrete module (INV-POS-009 governs it)",
            || configuration_report(CellConfiguration::discrete(0)),
        ),
        // INV-POS-012 — stated areas
        silent("INV-POS-012", "no area stated", || {
            geometry_positivity_report(geometry(2.0, 1.0))
        }),
        silent("INV-POS-012", "a coherent 2 m2 overall area", || {
            geometry_positivity_report(coherent_geometry())
        }),
        reported("INV-POS-012", "zero overall area", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_gross_area(area(0.0)))
        }),
        reported("INV-POS-012", "negative active area", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_active_area(area(-1.0)))
        }),
        reported("INV-POS-012", "aperture area is NaN", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_aperture_area(area(f64::NAN)))
        }),
        reported("INV-POS-012", "zero cell area", Invalid, || {
            geometry_positivity_report(geometry(2.0, 1.0).with_cell_area(area(0.0)))
        }),
    ]
}

// --- INV-ELE-001 to INV-ELE-012 ---------------------------------------

fn electrical_scenarios() -> Vec<Scenario> {
    vec![
        // INV-ELE-001 — v_oc > v_mpp
        silent(
            "INV-ELE-001",
            "49.5 V open circuit against 41.5 V at the knee",
            || ordering_report(coherent_electrical()),
        ),
        reported(
            "INV-ELE-001",
            "equal voltages (no current flows at v_oc)",
            Invalid,
            || ordering_report(electrical(400.0, 49.5, 10.4, 49.5, 9.64)),
        ),
        reported(
            "INV-ELE-001",
            "maximum-power voltage above open circuit",
            Invalid,
            || ordering_report(electrical(400.0, 41.5, 10.4, 49.5, 9.64)),
        ),
        // INV-ELE-002 — i_sc > i_mpp
        silent(
            "INV-ELE-002",
            "10.4 A short circuit against 9.64 A at the knee",
            || ordering_report(coherent_electrical()),
        ),
        reported("INV-ELE-002", "equal currents", Invalid, || {
            ordering_report(electrical(400.0, 49.5, 9.64, 41.5, 9.64))
        }),
        reported(
            "INV-ELE-002",
            "maximum-power current above short circuit",
            Invalid,
            || ordering_report(electrical(400.0, 49.5, 9.0, 41.5, 10.4)),
        ),
        // INV-ELE-003 — voltage ratio
        silent("INV-ELE-003", "ratio 0.838, the ordinary case", || {
            ordering_report(coherent_electrical())
        }),
        reported(
            "INV-ELE-003",
            "ratio 0.606, below the usual range",
            Warning,
            || ordering_report(electrical(289.2, 49.5, 10.4, 30.0, 9.64)),
        ),
        reported(
            "INV-ELE-003",
            "ratio 0.960, above the usual range",
            Warning,
            || ordering_report(electrical(457.9, 49.5, 10.4, 47.5, 9.64)),
        ),
        silent("INV-ELE-003", "ratio just inside the lower bound", || {
            ordering_report(electrical(
                400.0,
                49.5,
                10.4,
                49.5 * (VMPP_VOC_RATIO_RANGE.0 + 0.01),
                9.64,
            ))
        }),
        silent("INV-ELE-003", "ratio just inside the upper bound", || {
            ordering_report(electrical(
                400.0,
                49.5,
                10.4,
                49.5 * (VMPP_VOC_RATIO_RANGE.1 - 0.01),
                9.64,
            ))
        }),
        // INV-ELE-004 — stated against implied power
        silent(
            "INV-ELE-004",
            "400 W stated against 400.06 W implied (rounding)",
            || ordering_report(coherent_electrical()),
        ),
        reported(
            "INV-ELE-004",
            "2 % apart: beyond rounding, both plausible",
            Warning,
            || ordering_report(electrical(408.0, 49.5, 10.4, 41.5, 9.64)),
        ),
        reported_with(
            "INV-ELE-004",
            "10 % apart: a contradiction the engine will not arbitrate",
            Conflict,
            || ordering_report(electrical(440.0, 49.5, 10.4, 41.5, 9.64)),
            names_both_power_figures,
        ),
        // INV-ELE-005 — fill factor range
        silent("INV-ELE-005", "fill factor 0.777", || {
            ordering_report(coherent_electrical())
        }),
        reported(
            "INV-ELE-005",
            "fill factor 0.40, below the commercial range",
            Warning,
            || {
                ordering_report(electrical(
                    49.5 * 10.4 * (FILL_FACTOR_RANGE.0 - 0.1),
                    49.5,
                    10.4,
                    41.5,
                    9.64,
                ))
            },
        ),
        reported(
            "INV-ELE-005",
            "fill factor 0.95, above the commercial range",
            Warning,
            || {
                ordering_report(electrical(
                    49.5 * 10.4 * (FILL_FACTOR_RANGE.1 + 0.05),
                    49.5,
                    10.4,
                    41.5,
                    9.64,
                ))
            },
        ),
        silent(
            "INV-ELE-005",
            "a degenerate module yields NaN, not a finding",
            || ordering_report(electrical(100.0, 0.0, 0.0, 0.0, 0.0)),
        ),
        // INV-ELE-006 — fill factor above 1
        silent("INV-ELE-006", "fill factor 0.777", || {
            ordering_report(coherent_electrical())
        }),
        silent(
            "INV-ELE-006",
            "fill factor exactly 1: the limit, not a breach",
            || ordering_report(electrical(49.5 * 10.4, 49.5, 10.4, 41.5, 9.64)),
        ),
        reported(
            "INV-ELE-006",
            "fill factor 1.17: physically impossible",
            Invalid,
            || ordering_report(electrical(600.0, 49.5, 10.4, 41.5, 9.64)),
        ),
        silent(
            "INV-ELE-005",
            "above 1, INV-ELE-006 takes over exclusively",
            || ordering_report(electrical(600.0, 49.5, 10.4, 41.5, 9.64)),
        ),
        // INV-ELE-010 — volts per junction
        silent("INV-ELE-010", "0.688 V over 72 junctions", || {
            report_from(|r| {
                check_voltage_per_junction(&coherent_electrical(), &coherent_configuration(), r)
            })
        }),
        reported(
            "INV-ELE-010",
            "0.278 V per junction: count or voltage is wrong",
            Warning,
            || {
                report_from(|r| {
                    check_voltage_per_junction(
                        &electrical(154.0, 20.0, 10.4, 16.0, 9.64),
                        &coherent_configuration(),
                        r,
                    )
                })
            },
        ),
        reported(
            "INV-ELE-010",
            "the 144-half-cell trap halves the volts",
            Warning,
            || {
                report_from(|r| {
                    check_voltage_per_junction(
                        &coherent_electrical(),
                        &CellConfiguration::Discrete {
                            cells_in_series: 144,
                            cells_in_parallel: 1,
                            cell_division: CellDivision::Half,
                            substrings: None,
                            bypass_diodes: None,
                        },
                        r,
                    )
                })
            },
        ),
        silent("INV-ELE-010", "no junction to divide by", || {
            report_from(|r| check_voltage_per_junction(&coherent_electrical(), &monolithic(0), r))
        }),
        // INV-ELE-011 — system voltage
        silent(
            "INV-ELE-011",
            "1500 V system against a 49.5 V module",
            || report_from(|r| check_limits(&coherent_electrical(), &coherent_limits(), r)),
        ),
        silent(
            "INV-ELE-011",
            "equal: a string of exactly one module",
            || report_from(|r| check_limits(&coherent_electrical(), &limits(49.5), r)),
        ),
        reported(
            "INV-ELE-011",
            "40 V system below the module's own 49.5 V",
            Invalid,
            || report_from(|r| check_limits(&coherent_electrical(), &limits(40.0), r)),
        ),
        // INV-ELE-012 — fuse rating
        silent("INV-ELE-012", "no fuse rating stated", || {
            report_from(|r| check_limits(&coherent_electrical(), &coherent_limits(), r))
        }),
        silent(
            "INV-ELE-012",
            "20 A fuse against 10.4 A short circuit",
            || {
                report_from(|r| {
                    check_limits(
                        &coherent_electrical(),
                        &coherent_limits()
                            .with_max_series_fuse(Measurement::stated(Current::from_amperes(20.0))),
                        r,
                    )
                })
            },
        ),
        reported(
            "INV-ELE-012",
            "5 A fuse below the short-circuit current",
            Warning,
            || {
                report_from(|r| {
                    check_limits(
                        &coherent_electrical(),
                        &coherent_limits()
                            .with_max_series_fuse(Measurement::stated(Current::from_amperes(5.0))),
                        r,
                    )
                })
            },
        ),
    ]
}

// --- INV-GEO-001 to INV-GEO-005 ---------------------------------------

fn geometry_scenarios() -> Vec<Scenario> {
    vec![
        // INV-GEO-001 — aperture within overall
        silent("INV-GEO-001", "no area stated", || {
            geometry_relations_report(geometry(2.0, 1.0))
        }),
        silent(
            "INV-GEO-001",
            "1.8 m2 aperture inside a 2 m2 overall area",
            || {
                geometry_relations_report(
                    geometry(2.0, 1.0)
                        .with_gross_area(area(2.0))
                        .with_aperture_area(area(1.8)),
                )
            },
        ),
        reported(
            "INV-GEO-001",
            "2.5 m2 aperture on a 2 m2 module",
            Invalid,
            || {
                geometry_relations_report(
                    geometry(2.0, 1.0)
                        .with_gross_area(area(2.0))
                        .with_aperture_area(area(2.5)),
                )
            },
        ),
        // INV-GEO-002 — stated against derived area
        silent(
            "INV-GEO-002",
            "stated and derived areas agree exactly",
            || geometry_relations_report(coherent_geometry()),
        ),
        silent(
            "INV-GEO-002",
            "0.05 % apart: within datasheet rounding",
            || geometry_relations_report(geometry(2.0, 1.0).with_gross_area(area(2.001))),
        ),
        reported(
            "INV-GEO-002",
            "0.5 % apart: beyond rounding",
            Warning,
            || geometry_relations_report(geometry(2.0, 1.0).with_gross_area(area(2.01))),
        ),
        reported_with(
            "INV-GEO-002",
            "the Sharp_ND_72ELUF shape: a factor of two apart",
            Conflict,
            || {
                geometry_relations_report(
                    ModuleGeometry::new(dimensions(1.165, 0.990))
                        .with_gross_area(Measurement::stated(Area::from_square_meters(0.577))),
                )
            },
            names_both_areas,
        ),
        // INV-GEO-003 — area nesting
        silent("INV-GEO-003", "no nested area stated", || {
            geometry_relations_report(geometry(2.0, 1.0))
        }),
        silent("INV-GEO-003", "cell < active < aperture", || {
            geometry_relations_report(
                geometry(2.0, 1.0)
                    .with_cell_area(area(1.5))
                    .with_active_area(area(1.8))
                    .with_aperture_area(area(1.9)),
            )
        }),
        reported(
            "INV-GEO-003",
            "cell area above active area",
            Invalid,
            || {
                geometry_relations_report(
                    geometry(2.0, 1.0)
                        .with_cell_area(area(1.9))
                        .with_active_area(area(1.5)),
                )
            },
        ),
        reported(
            "INV-GEO-003",
            "active area above aperture area",
            Invalid,
            || {
                geometry_relations_report(
                    geometry(2.0, 1.0)
                        .with_active_area(area(1.9))
                        .with_aperture_area(area(1.5)),
                )
            },
        ),
        // INV-GEO-004 — cell format
        silent("INV-GEO-004", "no cell area stated", || {
            report_from(|r| {
                check_cell_area_plausibility(&geometry(2.0, 1.0), &coherent_configuration(), r)
            })
        }),
        silent("INV-GEO-004", "72 cells of 166 mm: 0.0276 m2 each", || {
            report_from(|r| {
                check_cell_area_plausibility(
                    &geometry(2.0, 1.0).with_cell_area(area(72.0 * 0.0276)),
                    &coherent_configuration(),
                    r,
                )
            })
        }),
        reported(
            "INV-GEO-004",
            "0.0069 m2 per junction: below any format",
            Warning,
            || {
                report_from(|r| {
                    check_cell_area_plausibility(
                        &geometry(2.0, 1.0).with_cell_area(area(0.5)),
                        &coherent_configuration(),
                        r,
                    )
                })
            },
        ),
        reported(
            "INV-GEO-004",
            "0.069 m2 per junction: above any format",
            Warning,
            || {
                report_from(|r| {
                    check_cell_area_plausibility(
                        &geometry(2.0, 1.0).with_cell_area(area(5.0)),
                        &coherent_configuration(),
                        r,
                    )
                })
            },
        ),
        silent("INV-GEO-004", "no junction to divide by", || {
            report_from(|r| {
                check_cell_area_plausibility(
                    &geometry(2.0, 1.0).with_cell_area(area(2.0)),
                    &monolithic(0),
                    r,
                )
            })
        }),
        // INV-GEO-005 — depth against lateral dimensions
        silent("INV-GEO-005", "no depth stated", || {
            geometry_relations_report(geometry(2.0, 1.0))
        }),
        silent("INV-GEO-005", "30 mm thick on a 1 m wide module", || {
            geometry_relations_report(ModuleGeometry::new(
                dimensions(2.0, 1.0).with_depth(length(0.030)),
            ))
        }),
        reported(
            "INV-GEO-005",
            "1.5 m thick on a 1 m wide module",
            Warning,
            || {
                geometry_relations_report(ModuleGeometry::new(
                    dimensions(2.0, 1.0).with_depth(length(1.5)),
                ))
            },
        ),
    ]
}

// --- INV-DER-001 to INV-DER-003 ---------------------------------------

fn efficiency_scenarios() -> Vec<Scenario> {
    vec![
        // INV-DER-001 — stated against derived efficiency
        silent("INV-DER-001", "20 % stated against 20 % derived", || {
            efficiency_report(
                coherent_electrical().with_stated_efficiency(Measurement::stated(Ratio::new(0.20))),
                coherent_geometry(),
            )
        }),
        silent("INV-DER-001", "1 % apart: within the threshold", || {
            efficiency_report(
                coherent_electrical()
                    .with_stated_efficiency(Measurement::stated(Ratio::new(0.202))),
                coherent_geometry(),
            )
        }),
        reported_with(
            "INV-DER-001",
            "5 % apart: a different area was used and not stated",
            Conflict,
            || {
                efficiency_report(
                    coherent_electrical()
                        .with_stated_efficiency(Measurement::stated(Ratio::new(0.21))),
                    coherent_geometry(),
                )
            },
            names_the_area_used,
        ),
        silent("INV-DER-001", "no stated efficiency to contradict", || {
            efficiency_report(coherent_electrical(), coherent_geometry())
        }),
        // INV-DER-002 — physical bound
        silent("INV-DER-002", "20 % derived efficiency", || {
            efficiency_report(coherent_electrical(), coherent_geometry())
        }),
        silent(
            "INV-DER-002",
            "exactly at the bound, which is not a breach",
            || {
                efficiency_report(
                    electrical(
                        EFFICIENCY_ABSOLUTE_MAX * 2.0 * 1000.0,
                        49.5,
                        10.4,
                        41.5,
                        9.64,
                    ),
                    coherent_geometry(),
                )
            },
        ),
        reported(
            "INV-DER-002",
            "60 %: power or area must be wrong",
            Invalid,
            || {
                efficiency_report(
                    electrical(1200.0, 49.5, 10.4, 41.5, 9.64),
                    coherent_geometry(),
                )
            },
        ),
        silent(
            "INV-DER-003",
            "above the bound, INV-DER-002 takes over",
            || {
                efficiency_report(
                    electrical(1200.0, 49.5, 10.4, 41.5, 9.64),
                    coherent_geometry(),
                )
            },
        ),
        // INV-DER-003 — commercial range
        silent("INV-DER-003", "20 %, an ordinary commercial module", || {
            efficiency_report(coherent_electrical(), coherent_geometry())
        }),
        reported("INV-DER-003", "2 %: below anything sold", Warning, || {
            efficiency_report(
                electrical(40.0, 49.5, 10.4, 41.5, 9.64),
                coherent_geometry(),
            )
        }),
        reported_with(
            "INV-DER-003",
            "35 %: above any commercial module, below the bound",
            Warning,
            || {
                efficiency_report(
                    electrical(700.0, 49.5, 10.4, 41.5, 9.64),
                    coherent_geometry(),
                )
            },
            names_the_area_used,
        ),
        reported_with(
            "INV-DER-003",
            "the aperture is preferred and the finding says so",
            Warning,
            || {
                efficiency_report(
                    electrical(700.0, 49.5, 10.4, 41.5, 9.64),
                    coherent_geometry().with_aperture_area(area(1.8)),
                )
            },
            names_the_aperture,
        ),
        silent(
            "INV-DER-003",
            "a zero area yields no efficiency at all",
            || efficiency_report(coherent_electrical(), geometry(0.0, 1.0)),
        ),
    ]
}

// --- INV-TMP-001 to INV-TMP-006 ---------------------------------------

fn thermal_scenarios() -> Vec<Scenario> {
    vec![
        // INV-TMP-001 — alpha sign
        silent("INV-TMP-001", "alpha positive, as expected", || {
            thermal_report(coherent_thermal())
        }),
        reported("INV-TMP-001", "alpha negative", Warning, || {
            thermal_report(thermal(-0.045, -0.27, -0.34))
        }),
        reported("INV-TMP-001", "alpha exactly zero", Warning, || {
            thermal_report(thermal(0.0, -0.27, -0.34))
        }),
        // INV-TMP-002 — beta sign
        silent("INV-TMP-002", "beta negative, as expected", || {
            thermal_report(coherent_thermal())
        }),
        reported("INV-TMP-002", "beta positive", Warning, || {
            thermal_report(thermal(0.045, 0.27, -0.34))
        }),
        reported("INV-TMP-002", "beta exactly zero", Warning, || {
            thermal_report(thermal(0.045, 0.0, -0.34))
        }),
        // INV-TMP-003 — gamma sign
        silent("INV-TMP-003", "gamma negative, as expected", || {
            thermal_report(coherent_thermal())
        }),
        reported("INV-TMP-003", "gamma positive", Warning, || {
            thermal_report(thermal(0.045, -0.27, 0.34))
        }),
        reported("INV-TMP-003", "gamma exactly zero", Warning, || {
            thermal_report(thermal(0.045, -0.27, 0.0))
        }),
        // INV-TMP-004 — gamma magnitude
        silent(
            "INV-TMP-004",
            "0.34 %/K, an ordinary silicon module",
            || thermal_report(coherent_thermal()),
        ),
        reported(
            "INV-TMP-004",
            "0.05 %/K: below any known technology",
            Warning,
            || {
                thermal_report(thermal(
                    0.045,
                    -0.02,
                    -GAMMA_MAGNITUDE_RANGE_PERCENT.0 / 2.0,
                ))
            },
        ),
        reported(
            "INV-TMP-004",
            "1.5 %/K: above any known technology",
            Warning,
            || {
                thermal_report(thermal(
                    0.045,
                    -0.27,
                    -GAMMA_MAGNITUDE_RANGE_PERCENT.1 * 1.5,
                ))
            },
        ),
        silent(
            "INV-TMP-004",
            "an absolute coefficient: the range does not apply",
            || {
                thermal_report(pv_module::ThermalCharacteristics::new(
                    coefficient(0.045),
                    coefficient(-0.27),
                    Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(-1.7)),
                ))
            },
        ),
        // INV-TMP-005 — ambiguous basis (raised by an adapter, never here)
        reported_with(
            "INV-TMP-005",
            "a source stating no unit for its coefficient",
            Conflict,
            || {
                let mut report = ValidationReport::new();
                report.push(ambiguous_coefficient_basis("gamma_p_max", "-0.5072"));
                report
            },
            |issue| {
                if issue.observed() != Some("-0.5072") {
                    return Err("the ambiguous value must be quoted verbatim".to_string());
                }
                if !issue
                    .expected()
                    .unwrap_or_default()
                    .contains("explicit unit")
                {
                    return Err("the finding must state what would resolve it".to_string());
                }
                Ok(())
            },
        ),
        silent(
            "INV-TMP-005",
            "coherent coefficients are never ambiguous",
            || thermal_report(coherent_thermal()),
        ),
        silent(
            "INV-TMP-005",
            "an out-of-range magnitude is not an ambiguity",
            || thermal_report(thermal(0.045, -0.27, -1.7)),
        ),
        silent(
            "INV-TMP-005",
            "wrong signs are not an ambiguity either",
            || thermal_report(thermal(-0.045, 0.27, 0.34)),
        ),
        // INV-TMP-006 — gamma dominates beta
        silent("INV-TMP-006", "|gamma| 0.34 above |beta| 0.27", || {
            thermal_report(coherent_thermal())
        }),
        silent("INV-TMP-006", "equal magnitudes: not an inversion", || {
            thermal_report(thermal(0.045, -0.30, -0.30))
        }),
        reported(
            "INV-TMP-006",
            "|gamma| 0.20 below |beta| 0.27: informational",
            Valid,
            || thermal_report(thermal(0.045, -0.27, -0.20)),
        ),
        silent("INV-TMP-006", "an absolute basis is not compared", || {
            thermal_report(pv_module::ThermalCharacteristics::new(
                coefficient(0.045),
                coefficient(-0.27),
                Measurement::stated(TemperatureCoefficient::from_absolute_per_kelvin(-1.7)),
            ))
        }),
    ]
}

// --- INV-CND-001 to INV-CND-006 ---------------------------------------

fn condition_scenarios() -> Vec<Scenario> {
    vec![
        // INV-CND-001 — admissible temperature
        silent("INV-CND-001", "standard test conditions at 25 degC", || {
            points_report(&[OperatingPoint::new(OperatingCondition::stc())], 400.0)
        }),
        reported("INV-CND-001", "500 degC", Invalid, || {
            points_report(&[OperatingPoint::new(condition_at(1000.0, 500.0))], 400.0)
        }),
        reported(
            "INV-CND-001",
            "-200 degC, below absolute zero",
            Invalid,
            || points_report(&[OperatingPoint::new(condition_at(1000.0, -200.0))], 400.0),
        ),
        reported("INV-CND-001", "a temperature that is NaN", Invalid, || {
            points_report(
                &[OperatingPoint::new(condition_at(1000.0, f64::NAN))],
                400.0,
            )
        }),
        silent("INV-CND-001", "exactly at the lower bound", || {
            points_report(
                &[OperatingPoint::new(condition_at(
                    1000.0,
                    TEMPERATURE_RANGE_CELSIUS.0,
                ))],
                400.0,
            )
        }),
        silent("INV-CND-001", "exactly at the upper bound", || {
            points_report(
                &[OperatingPoint::new(condition_at(
                    1000.0,
                    TEMPERATURE_RANGE_CELSIUS.1,
                ))],
                400.0,
            )
        }),
        // INV-CND-002 — operating temperature range
        silent("INV-CND-002", "no range declared", || {
            points_with_limits_report(coherent_limits())
        }),
        silent(
            "INV-CND-002",
            "-40 degC to 85 degC, correctly ordered",
            || {
                points_with_limits_report(
                    coherent_limits()
                        .with_operating_temperature_range(temperature(-40.0), temperature(85.0)),
                )
            },
        ),
        reported("INV-CND-002", "the bounds are inverted", Invalid, || {
            points_with_limits_report(
                coherent_limits()
                    .with_operating_temperature_range(temperature(85.0), temperature(-40.0)),
            )
        }),
        reported(
            "INV-CND-002",
            "a degenerate range, minimum equal to maximum",
            Invalid,
            || {
                points_with_limits_report(
                    coherent_limits()
                        .with_operating_temperature_range(temperature(25.0), temperature(25.0)),
                )
            },
        ),
        // INV-CND-003 — settled cell temperature
        silent("INV-CND-003", "44 degC at the nominal condition", || {
            points_report(
                &[OperatingPoint::new(OperatingCondition::noct())
                    .with_cell_temperature(Measurement::measured(Temperature::from_celsius(44.0)))],
                400.0,
            )
        }),
        reported(
            "INV-CND-003",
            "80 degC: above the usual settled range",
            Warning,
            || {
                points_report(
                    &[
                        OperatingPoint::new(OperatingCondition::noct()).with_cell_temperature(
                            Measurement::measured(Temperature::from_celsius(80.0)),
                        ),
                    ],
                    400.0,
                )
            },
        ),
        reported(
            "INV-CND-003",
            "10 degC: below the usual settled range",
            Warning,
            || {
                points_report(
                    &[
                        OperatingPoint::new(OperatingCondition::noct()).with_cell_temperature(
                            Measurement::measured(Temperature::from_celsius(10.0)),
                        ),
                    ],
                    400.0,
                )
            },
        ),
        reported(
            "INV-CND-003",
            "NMOT lets the cell settle in the same way",
            Warning,
            || {
                points_report(
                    &[
                        OperatingPoint::new(OperatingCondition::nmot()).with_cell_temperature(
                            Measurement::measured(Temperature::from_celsius(80.0)),
                        ),
                    ],
                    400.0,
                )
            },
        ),
        silent(
            "INV-CND-003",
            "STC imposes the cell temperature, so the range does not apply",
            || {
                points_report(
                    &[
                        OperatingPoint::new(OperatingCondition::stc()).with_cell_temperature(
                            Measurement::measured(Temperature::from_celsius(80.0)),
                        ),
                    ],
                    400.0,
                )
            },
        ),
        silent("INV-CND-003", "no settled temperature recorded", || {
            points_report(&[OperatingPoint::new(OperatingCondition::noct())], 400.0)
        }),
        // INV-CND-004 — positive irradiance
        silent(
            "INV-CND-004",
            "1000 W/m2 at standard test conditions",
            || points_report(&[OperatingPoint::new(OperatingCondition::stc())], 400.0),
        ),
        reported("INV-CND-004", "zero irradiance", Invalid, || {
            points_report(&[OperatingPoint::new(condition_at(0.0, 25.0))], 400.0)
        }),
        reported("INV-CND-004", "negative irradiance", Invalid, || {
            points_report(&[OperatingPoint::new(condition_at(-800.0, 25.0))], 400.0)
        }),
        reported("INV-CND-004", "irradiance is NaN", Invalid, || {
            points_report(&[OperatingPoint::new(condition_at(f64::NAN, 25.0))], 400.0)
        }),
        // INV-CND-005 — uniqueness of named conditions
        silent("INV-CND-005", "one nominal condition", || {
            points_report(&[OperatingPoint::new(OperatingCondition::noct())], 400.0)
        }),
        silent("INV-CND-005", "two different conditions", || {
            points_report(
                &[
                    OperatingPoint::new(OperatingCondition::noct()),
                    OperatingPoint::new(OperatingCondition::ptc()),
                ],
                400.0,
            )
        }),
        reported(
            "INV-CND-005",
            "the same condition declared twice",
            Invalid,
            || {
                points_report(
                    &[
                        OperatingPoint::new(OperatingCondition::noct()),
                        OperatingPoint::new(OperatingCondition::noct()),
                    ],
                    400.0,
                )
            },
        ),
        silent(
            "INV-CND-005",
            "five matrix points, many by construction",
            || {
                let matrix: Vec<OperatingPoint> = [1000.0, 800.0, 600.0, 400.0, 200.0]
                    .into_iter()
                    .map(|g| {
                        OperatingPoint::new(OperatingCondition::iec61853_matrix_point(
                            Irradiance::from_watts_per_m2(g),
                            Temperature::from_celsius(25.0),
                        ))
                    })
                    .collect();
                points_report(&matrix, 400.0)
            },
        ),
        // INV-CND-006 — nominal power below standard
        silent("INV-CND-006", "295 W at NOCT against 400 W at STC", || {
            points_report(
                &[OperatingPoint::new(OperatingCondition::noct())
                    .with_p_max(Measurement::stated(Power::from_watts(295.0)))],
                400.0,
            )
        }),
        reported(
            "INV-CND-006",
            "450 W at NOCT, above the STC value",
            Warning,
            || {
                points_report(
                    &[OperatingPoint::new(OperatingCondition::noct())
                        .with_p_max(Measurement::stated(Power::from_watts(450.0)))],
                    400.0,
                )
            },
        ),
        reported(
            "INV-CND-006",
            "equal to the STC value at 800 W/m2",
            Warning,
            || {
                points_report(
                    &[OperatingPoint::new(OperatingCondition::nmot())
                        .with_p_max(Measurement::stated(Power::from_watts(400.0)))],
                    400.0,
                )
            },
        ),
        silent("INV-CND-006", "no nominal power stated to compare", || {
            points_report(&[OperatingPoint::new(OperatingCondition::noct())], 400.0)
        }),
        silent(
            "INV-CND-006",
            "no standard-condition power to compare against",
            || {
                points_report(
                    &[OperatingPoint::new(OperatingCondition::noct())
                        .with_p_max(Measurement::stated(Power::from_watts(450.0)))],
                    0.0,
                )
            },
        ),
    ]
}

// --- INV-BIF-001 to INV-BIF-004 ---------------------------------------

fn bifacial_scenarios() -> Vec<Scenario> {
    vec![
        // INV-BIF-001 — uncharacterised bifaciality
        reported(
            "INV-BIF-001",
            "a CEC-style flag with nothing else",
            Warning,
            || bifacial_report(BifacialCharacteristics::flag_only(), silicon()),
        ),
        silent("INV-BIF-001", "a stated bifaciality factor of 0.70", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70))),
                silicon(),
            )
        }),
        silent("INV-BIF-001", "rear-side power measured", || {
            bifacial_report(
                BifacialCharacteristics::flag_only().with_rear_side_parameters(
                    RearSideParameters {
                        p_max: Some(Measurement::measured(Power::from_watts(280.0))),
                        ..RearSideParameters::empty()
                    },
                ),
                silicon(),
            )
        }),
        reported(
            "INV-BIF-001",
            "an explicitly empty rear-side block",
            Warning,
            || {
                bifacial_report(
                    BifacialCharacteristics::flag_only()
                        .with_rear_side_parameters(RearSideParameters::empty()),
                    silicon(),
                )
            },
        ),
        // INV-BIF-002 — positive bifaciality factor
        silent("INV-BIF-002", "a factor of 0.70", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70))),
                silicon(),
            )
        }),
        silent("INV-BIF-002", "unity: a fully symmetric module", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(1.0))),
                silicon(),
            )
        }),
        reported("INV-BIF-002", "a factor of zero", Invalid, || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.0))),
                silicon(),
            )
        }),
        reported(
            "INV-BIF-002",
            "a negative source value, clamped to zero",
            Invalid,
            || {
                bifacial_report(
                    BifacialCharacteristics::with_factor(Measurement::measured(Ratio::clamped(
                        -0.5,
                    ))),
                    silicon(),
                )
            },
        ),
        // INV-BIF-003 — rear below front
        silent("INV-BIF-003", "no rear-side power stated", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70))),
                silicon(),
            )
        }),
        silent("INV-BIF-003", "280 W rear against 400 W front", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)))
                    .with_rear_side_parameters(RearSideParameters {
                        p_max: Some(Measurement::measured(Power::from_watts(280.0))),
                        ..RearSideParameters::empty()
                    }),
                silicon(),
            )
        }),
        reported(
            "INV-BIF-003",
            "500 W rear against 400 W front",
            Warning,
            || {
                bifacial_report(
                    BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70)))
                        .with_rear_side_parameters(RearSideParameters {
                            p_max: Some(Measurement::measured(Power::from_watts(500.0))),
                            ..RearSideParameters::empty()
                        }),
                    silicon(),
                )
            },
        ),
        // INV-BIF-004 — bifaciality on back-contact architectures
        silent("INV-BIF-004", "a PERC architecture", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70))),
                silicon(),
            )
        }),
        silent("INV-BIF-004", "a heterojunction architecture", || {
            bifacial_report(
                BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70))),
                ModuleTechnology::new(
                    CellTechnology::MonocrystallineSilicon,
                    CellArchitecture::HeterojunctionHJT,
                ),
            )
        }),
        reported(
            "INV-BIF-004",
            "an interdigitated back contact, whose rear carries the contacts",
            Warning,
            || {
                bifacial_report(
                    BifacialCharacteristics::with_factor(Measurement::measured(Ratio::new(0.70))),
                    ModuleTechnology::new(
                        CellTechnology::MonocrystallineSilicon,
                        CellArchitecture::InterdigitatedBackContact,
                    ),
                )
            },
        ),
    ]
}

// --- INV-STR-001 to INV-STR-004 ---------------------------------------

fn structure_scenarios() -> Vec<Scenario> {
    vec![
        // INV-STR-001 — half-cut pairing
        silent("INV-STR-001", "half cells with two parallel paths", || {
            structure_report(
                discrete_config(2, CellDivision::Half, None, None),
                silicon(),
                0,
            )
        }),
        reported(
            "INV-STR-001",
            "half cells with three parallel paths",
            Warning,
            || {
                structure_report(
                    discrete_config(3, CellDivision::Half, None, None),
                    silicon(),
                    0,
                )
            },
        ),
        silent("INV-STR-001", "whole cells impose no pairing", || {
            structure_report(
                discrete_config(3, CellDivision::Full, None, None),
                silicon(),
                0,
            )
        }),
        silent(
            "INV-STR-001",
            "a monolithic module has no cell to cut",
            || structure_report(monolithic(154), cadmium_telluride(), 0),
        ),
        // INV-STR-002 — diodes against sub-strings
        silent(
            "INV-STR-002",
            "neither diodes nor sub-strings stated",
            || {
                structure_report(
                    discrete_config(1, CellDivision::Full, None, None),
                    silicon(),
                    0,
                )
            },
        ),
        silent("INV-STR-002", "three diodes for three sub-strings", || {
            structure_report(
                discrete_config(1, CellDivision::Full, Some(3), Some(3)),
                silicon(),
                0,
            )
        }),
        reported(
            "INV-STR-002",
            "four diodes for three sub-strings",
            Warning,
            || {
                structure_report(
                    discrete_config(1, CellDivision::Full, Some(3), Some(4)),
                    silicon(),
                    0,
                )
            },
        ),
        // INV-STR-003 — additional outputs
        silent("INV-STR-003", "no additional output on silicon", || {
            structure_report(coherent_configuration(), silicon(), 0)
        }),
        silent("INV-STR-003", "two additional outputs on a tandem", || {
            structure_report(coherent_configuration(), tandem(), 2)
        }),
        reported(
            "INV-STR-003",
            "one additional output on silicon",
            Warning,
            || structure_report(coherent_configuration(), silicon(), 1),
        ),
        reported(
            "INV-STR-003",
            "one additional output on cadmium telluride",
            Warning,
            || structure_report(coherent_configuration(), cadmium_telluride(), 1),
        ),
        // INV-STR-004 — thin film against configuration (ADR-0007)
        silent(
            "INV-STR-004",
            "silicon with discrete cells: the ordinary case",
            || structure_report(coherent_configuration(), silicon(), 0),
        ),
        silent(
            "INV-STR-004",
            "cadmium telluride monolithically scribed",
            || structure_report(monolithic(154), cadmium_telluride(), 0),
        ),
        reported(
            "INV-STR-004",
            "cadmium telluride with countable discrete cells",
            Warning,
            || structure_report(coherent_configuration(), cadmium_telluride(), 0),
        ),
        silent(
            "INV-STR-004",
            "silicon monolithic: the invariant is one-directional",
            || structure_report(monolithic(154), silicon(), 0),
        ),
    ]
}

// --- INV-PRV-001 to INV-PRV-003 ---------------------------------------

fn provenance_scenarios() -> Vec<Scenario> {
    vec![
        // INV-PRV-001 — unattributed values
        reported(
            "INV-PRV-001",
            "a datasheet figure with no provenance",
            Warning,
            || provenance_report(volts(49.5)),
        ),
        silent("INV-PRV-001", "the same figure, attributed", || {
            provenance_report(attributed(volts(49.5)))
        }),
        reported(
            "INV-PRV-001",
            "a measurement is as unattributed as any other",
            Warning,
            || provenance_report(Measurement::measured(Voltage::from_volts(49.5))),
        ),
        // INV-PRV-002 — estimates without a rationale
        silent("INV-PRV-002", "an estimate carrying its reason", || {
            provenance_report(
                Measurement::estimated(Voltage::from_volts(49.5), "read from a photograph")
                    .with_provenance(datasheet_provenance()),
            )
        }),
        reported(
            "INV-PRV-002",
            "an estimate with no reason recorded",
            Invalid,
            || {
                provenance_report(attributed(Measurement::new(
                    Voltage::from_volts(49.5),
                    DataClass::Estimated,
                )))
            },
        ),
        reported(
            "INV-PRV-002",
            "an empty string is not a rationale",
            Invalid,
            || {
                provenance_report(attributed(
                    Measurement::new(Voltage::from_volts(49.5), DataClass::Estimated)
                        .with_rationale(""),
                ))
            },
        ),
        silent(
            "INV-PRV-002",
            "manufacturer data needs no rationale",
            || provenance_report(attributed(volts(49.5))),
        ),
        silent("INV-PRV-002", "a measurement needs no rationale", || {
            provenance_report(attributed(Measurement::new(
                Voltage::from_volts(49.5),
                DataClass::Measured,
            )))
        }),
        silent("INV-PRV-002", "a derived value needs no rationale", || {
            provenance_report(attributed(Measurement::new(
                Voltage::from_volts(49.5),
                DataClass::Derived,
            )))
        }),
        // INV-PRV-003 — fitted parameters in physical fields
        silent(
            "INV-PRV-003",
            "a manufacturer figure in a physical field",
            || provenance_report(attributed(volts(49.5))),
        ),
        reported(
            "INV-PRV-003",
            "a De Soto coefficient in a physical field",
            Invalid,
            || {
                provenance_report(attributed(Measurement::new(
                    Voltage::from_volts(49.5),
                    DataClass::Fitted,
                )))
            },
        ),
        silent(
            "INV-PRV-003",
            "a model parameter is not a fitted one",
            || {
                provenance_report(attributed(Measurement::new(
                    Voltage::from_volts(49.5),
                    DataClass::ModelParameter,
                )))
            },
        ),
        silent(
            "INV-PRV-003",
            "a measurement is not a fitted parameter",
            || {
                provenance_report(attributed(Measurement::new(
                    Voltage::from_volts(49.5),
                    DataClass::Measured,
                )))
            },
        ),
    ]
}

// =====================================================================
// The census
// =====================================================================

/// Run every scenario of one invariant, failing with the reason if any
/// did not do what it declared.
#[track_caller]
fn run_scenarios_for(invariant: &str) {
    let all = all_scenarios();
    let mine: Vec<&Scenario> = all.iter().filter(|s| s.invariant == invariant).collect();

    assert!(
        !mine.is_empty(),
        "{invariant} has no scenario at all; it is in the inventory but never exercised"
    );
    assert!(
        mine.len() >= MINIMUM_SCENARIOS_PER_INVARIANT,
        "{invariant} carries {} scenario(s), below the required {MINIMUM_SCENARIOS_PER_INVARIANT}",
        mine.len()
    );

    let mut failures = Vec::new();
    for s in mine {
        if let Err(reason) = s.evaluate() {
            failures.push(format!("  - \"{}\": {reason}", s.case));
        }
    }
    assert!(
        failures.is_empty(),
        "{invariant}: {} scenario(s) did not produce their declared outcome:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Declares one `#[test]` per invariant, so each stays individually
/// runnable — `cargo test inv_geo_002` runs exactly that invariant.
macro_rules! per_invariant_tests {
    ($($name:ident => $id:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                run_scenarios_for($id);
            }
        )*

        /// Every invariant that has a generated test, for the completeness
        /// check below.
        const TESTED_INVARIANTS: &[&str] = &[$($id),*];
    };
}

per_invariant_tests! {
    inv_pos_001 => "INV-POS-001",
    inv_pos_002 => "INV-POS-002",
    inv_pos_003 => "INV-POS-003",
    inv_pos_004 => "INV-POS-004",
    inv_pos_005 => "INV-POS-005",
    inv_pos_006 => "INV-POS-006",
    inv_pos_007 => "INV-POS-007",
    inv_pos_008 => "INV-POS-008",
    inv_pos_009 => "INV-POS-009",
    inv_pos_010 => "INV-POS-010",
    inv_pos_011 => "INV-POS-011",
    inv_pos_012 => "INV-POS-012",
    inv_ele_001 => "INV-ELE-001",
    inv_ele_002 => "INV-ELE-002",
    inv_ele_003 => "INV-ELE-003",
    inv_ele_004 => "INV-ELE-004",
    inv_ele_005 => "INV-ELE-005",
    inv_ele_006 => "INV-ELE-006",
    inv_ele_010 => "INV-ELE-010",
    inv_ele_011 => "INV-ELE-011",
    inv_ele_012 => "INV-ELE-012",
    inv_geo_001 => "INV-GEO-001",
    inv_geo_002 => "INV-GEO-002",
    inv_geo_003 => "INV-GEO-003",
    inv_geo_004 => "INV-GEO-004",
    inv_geo_005 => "INV-GEO-005",
    inv_der_001 => "INV-DER-001",
    inv_der_002 => "INV-DER-002",
    inv_der_003 => "INV-DER-003",
    inv_tmp_001 => "INV-TMP-001",
    inv_tmp_002 => "INV-TMP-002",
    inv_tmp_003 => "INV-TMP-003",
    inv_tmp_004 => "INV-TMP-004",
    inv_tmp_005 => "INV-TMP-005",
    inv_tmp_006 => "INV-TMP-006",
    inv_cnd_001 => "INV-CND-001",
    inv_cnd_002 => "INV-CND-002",
    inv_cnd_003 => "INV-CND-003",
    inv_cnd_004 => "INV-CND-004",
    inv_cnd_005 => "INV-CND-005",
    inv_cnd_006 => "INV-CND-006",
    inv_bif_001 => "INV-BIF-001",
    inv_bif_002 => "INV-BIF-002",
    inv_bif_003 => "INV-BIF-003",
    inv_bif_004 => "INV-BIF-004",
    inv_str_001 => "INV-STR-001",
    inv_str_002 => "INV-STR-002",
    inv_str_003 => "INV-STR-003",
    inv_str_004 => "INV-STR-004",
    inv_prv_001 => "INV-PRV-001",
    inv_prv_002 => "INV-PRV-002",
    inv_prv_003 => "INV-PRV-003",
}

#[test]
fn every_invariant_in_the_inventory_has_a_test_of_its_own() {
    // The generated tests and the inventory must name the same set. An
    // invariant present in one and absent from the other is exactly the
    // silent gap this check exists to prevent.
    for id in ALL_INVARIANT_IDS {
        assert!(
            TESTED_INVARIANTS.contains(&id),
            "{id} is in the inventory but has no test of its own"
        );
    }
    for id in TESTED_INVARIANTS {
        assert!(
            ALL_INVARIANT_IDS.contains(id),
            "{id} has a test but is not in the inventory"
        );
    }
    assert_eq!(TESTED_INVARIANTS.len(), ALL_INVARIANT_IDS.len());
}

#[test]
fn every_invariant_carries_at_least_three_scenarios() {
    let scenarios = all_scenarios();
    let counted = census(&scenarios);

    // No invariant of the inventory may be missing from the census.
    let missing: Vec<&str> = ALL_INVARIANT_IDS
        .iter()
        .filter(|id| !counted.iter().any(|c| c.invariant == **id))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "these invariants have no executed scenario: {missing:?}"
    );

    let deficient: Vec<String> = counted
        .iter()
        .filter(|c| c.scenarios < MINIMUM_SCENARIOS_PER_INVARIANT)
        .map(|c| format!("{} ({} scenario(s))", c.invariant, c.scenarios))
        .collect();
    assert!(
        deficient.is_empty(),
        "{} invariant(s) below the required {MINIMUM_SCENARIOS_PER_INVARIANT} scenarios: {}",
        deficient.len(),
        deficient.join(", ")
    );
}

#[test]
fn every_scenario_produces_the_outcome_it_declared() {
    let scenarios = all_scenarios();
    let counted = census(&scenarios);

    let mut failures = Vec::new();
    for entry in &counted {
        for (case, reason) in &entry.failures {
            failures.push(format!("  {} — \"{case}\": {reason}", entry.invariant));
        }
    }
    assert!(
        failures.is_empty(),
        "{} scenario(s) did not produce their declared outcome:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Writes the census table required by the traceability check.
///
/// Named `zz_` so it runs after the per-invariant tests in the default
/// alphabetical ordering, and so the table it writes reflects a suite that
/// has already been exercised.
#[test]
fn zz_the_scenario_census_is_written_to_disk() {
    let scenarios = all_scenarios();
    let counted = census(&scenarios);
    let table = render_table(&counted, &scenarios);

    // Printed for `cargo test -- --nocapture` and for the CI log.
    println!("\n{table}");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../validation-data/invariant-scenario-matrix.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the validation-data directory must be creatable");
    }
    std::fs::write(&path, &table).expect("the scenario matrix must be writable");

    // The written table is the evidence, so it must be self-consistent.
    let total: usize = counted.iter().map(|c| c.scenarios).sum();
    let passed: usize = counted.iter().map(|c| c.passed).sum();
    let failed: usize = counted.iter().map(InvariantCensus::failed).sum();
    assert_eq!(total, scenarios.len());
    assert_eq!(passed + failed, total);
    assert_eq!(failed, 0, "the matrix records {failed} failing scenario(s)");
    assert_eq!(counted.len(), ALL_INVARIANT_IDS.len());
}

// =====================================================================
// The inventory and the thresholds
// =====================================================================

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
fn the_invariant_inventory_is_complete_and_free_of_duplicates() {
    assert_eq!(ALL_INVARIANT_IDS.len(), 52);

    let mut sorted: Vec<&str> = ALL_INVARIANT_IDS.to_vec();
    sorted.sort_unstable();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(sorted.len(), before, "identifiers must be unique");

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
