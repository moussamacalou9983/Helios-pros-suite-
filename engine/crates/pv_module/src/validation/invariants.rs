//! The physical and structural invariants every module is checked
//! against.
//!
//! Each invariant has a stable identifier, a severity level, a mandatory
//! explanation, and a direct correspondence to
//! `docs/design/27-specification-finale-pv-module.md` §11.
//!
//! Every check is a free function taking only what it needs, so that it
//! can be exercised in isolation as well as through
//! [`validate_module`].
//!
//! ## Thresholds
//!
//! The numeric thresholds come from two places: the rounding practice of
//! manufacturer datasheets, and direct measurement of the CEC library
//! (`docs/design/26-validation-empirique-cec-sandia.md`). They are
//! collected here as named constants so they can be revised in one place
//! when the five real datasheets required by the test plan become
//! available.

use core_units::{Area, CoefficientBasis, TemperatureCoefficient};

use super::{ValidationIssue, ValidationReport, ValidationStatus};
use crate::{
    BifacialCharacteristics, CellConfiguration, CellDivision, DataClass, ElectricalCharacteristics,
    ElectricalLimits, Measurement, ModuleGeometry, ModuleTechnology, OperatingPoint,
    ThermalCharacteristics,
};

// --- Thresholds ---------------------------------------------------------

/// Below this relative gap, a stated and a derived area agree.
pub const AREA_AGREEMENT_WARNING: f64 = 0.001;
/// Above this relative gap, a stated and a derived area conflict.
/// Measured: 98.55 % of CEC records agree to better than 0.1 %, and the
/// 29 defective ones exceed 10 %.
pub const AREA_AGREEMENT_CONFLICT: f64 = 0.01;
/// Above this relative gap, stated and implied maximum power are
/// suspect (datasheets round).
pub const POWER_CONSISTENCY_WARNING: f64 = 0.01;
/// Above this relative gap, stated and implied maximum power conflict.
pub const POWER_CONSISTENCY_CONFLICT: f64 = 0.05;
/// Above this relative gap, stated and derived efficiency conflict.
pub const EFFICIENCY_AGREEMENT_CONFLICT: f64 = 0.02;
/// Plausible range for the ratio of maximum-power to open-circuit
/// voltage.
pub const VMPP_VOC_RATIO_RANGE: (f64, f64) = (0.70, 0.90);
/// Plausible range for the fill factor.
pub const FILL_FACTOR_RANGE: (f64, f64) = (0.5, 0.9);
/// Plausible range for volts per series junction.
pub const VOLTS_PER_JUNCTION_RANGE: (f64, f64) = (0.4, 0.8);
/// Hard upper bound on efficiency; above it the data is wrong.
pub const EFFICIENCY_ABSOLUTE_MAX: f64 = 0.50;
/// Commercially plausible efficiency range.
pub const EFFICIENCY_TYPICAL_RANGE: (f64, f64) = (0.05, 0.30);
/// Plausible magnitude range for the relative power temperature
/// coefficient, in percent per kelvin.
pub const GAMMA_MAGNITUDE_RANGE_PERCENT: (f64, f64) = (0.1, 1.0);
/// Physically admissible temperature range, in degrees Celsius.
pub const TEMPERATURE_RANGE_CELSIUS: (f64, f64) = (-100.0, 200.0);
/// Plausible NOCT range, in degrees Celsius.
pub const NOCT_RANGE_CELSIUS: (f64, f64) = (30.0, 60.0);
/// Plausible area per series junction, in square metres. Spans the
/// 125 mm to 210 mm cell formats.
pub const CELL_AREA_RANGE: (f64, f64) = (0.010, 0.060);

/// The module's stated depth in metres, if any. A free function rather
/// than an inherent method because depth is only ever needed by the
/// invariants, not by the domain model itself.
fn depth_value(g: &ModuleGeometry) -> Option<f64> {
    g.dimensions.depth.as_ref().map(|d| d.get().meters())
}

fn relative_gap(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 || !a.is_finite() || !b.is_finite() {
        None
    } else {
        Some(((a - b) / b).abs())
    }
}

fn issue(
    id: &'static str,
    status: ValidationStatus,
    fields: Vec<&'static str>,
    explanation: impl Into<String>,
) -> ValidationIssue {
    ValidationIssue::new(id, status, fields, explanation)
}

// --- Positivity (INV-POS-001..012) -------------------------------------

/// Checks that every mandatory electrical quantity is strictly positive
/// and finite (`INV-POS-001` to `INV-POS-005`).
pub fn check_electrical_positivity(e: &ElectricalCharacteristics, report: &mut ValidationReport) {
    let checks: [(&'static str, &'static str, f64); 5] = [
        ("INV-POS-001", "p_max", e.p_max.get().watts()),
        ("INV-POS-002", "v_oc", e.v_oc.get().volts()),
        ("INV-POS-003", "i_sc", e.i_sc.get().amperes()),
        ("INV-POS-004", "v_mpp", e.v_mpp.get().volts()),
        ("INV-POS-005", "i_mpp", e.i_mpp.get().amperes()),
    ];
    for (id, field, value) in checks {
        if !value.is_finite() {
            report.push(
                issue(
                    id,
                    ValidationStatus::Invalid,
                    vec![field],
                    format!("{field} is not a finite number"),
                )
                .with_observed(format!("{field} = {value}")),
            );
        } else if value <= 0.0 {
            report.push(
                issue(
                    id,
                    ValidationStatus::Invalid,
                    vec![field],
                    format!("{field} must be strictly positive"),
                )
                .with_observed(format!("{field} = {value}"))
                .with_expected("> 0"),
            );
        }
    }
}

/// Checks that the module's dimensions, optional depth, optional mass and
/// any stated area are strictly positive (`INV-POS-006` to `INV-POS-008`,
/// `INV-POS-012`).
pub fn check_geometry_positivity(g: &ModuleGeometry, report: &mut ValidationReport) {
    let length = g.dimensions.length.get().meters();
    let width = g.dimensions.width.get().meters();
    if !(length.is_finite() && width.is_finite()) || length <= 0.0 || width <= 0.0 {
        report.push(
            issue(
                "INV-POS-006",
                ValidationStatus::Invalid,
                vec!["length", "width"],
                "module length and width must both be strictly positive and finite",
            )
            .with_observed(format!("length = {length} m, width = {width} m"))
            .with_expected("> 0"),
        );
    }
    if let Some(depth) = depth_value(g)
        && (!depth.is_finite() || depth <= 0.0)
    {
        report.push(
            issue(
                "INV-POS-007",
                ValidationStatus::Invalid,
                vec!["depth"],
                "a stated depth must be strictly positive",
            )
            .with_observed(format!("depth = {depth} m")),
        );
    }
    if let Some(mass) = g.mass.as_ref().map(|m| m.get().kilograms())
        && (!mass.is_finite() || mass <= 0.0)
    {
        report.push(
            issue(
                "INV-POS-008",
                ValidationStatus::Invalid,
                vec!["mass"],
                "a stated mass must be strictly positive",
            )
            .with_observed(format!("mass = {mass} kg")),
        );
    }
    for (field, area) in named_areas(g) {
        let value = area.square_meters();
        if !value.is_finite() || value <= 0.0 {
            report.push(
                issue(
                    "INV-POS-012",
                    ValidationStatus::Invalid,
                    vec![field],
                    format!("a stated {field} must be strictly positive"),
                )
                .with_observed(format!("{field} = {value} m2")),
            );
        }
    }
}

fn named_areas(g: &ModuleGeometry) -> Vec<(&'static str, Area)> {
    let mut areas = Vec::new();
    if let Some(a) = &g.gross_area {
        areas.push(("gross_area", a.get()));
    }
    if let Some(a) = &g.active_area {
        areas.push(("active_area", a.get()));
    }
    if let Some(a) = &g.aperture_area {
        areas.push(("aperture_area", a.get()));
    }
    if let Some(a) = &g.cell_area {
        areas.push(("cell_area", a.get()));
    }
    areas
}

/// Checks that the cell configuration's counts are non-zero
/// (`INV-POS-009` to `INV-POS-011`).
pub fn check_configuration_positivity(c: &CellConfiguration, report: &mut ValidationReport) {
    match c {
        CellConfiguration::Discrete {
            cells_in_series,
            cells_in_parallel,
            ..
        } => {
            if *cells_in_series == 0 {
                report.push(issue(
                    "INV-POS-009",
                    ValidationStatus::Invalid,
                    vec!["cells_in_series"],
                    "a discrete module must have at least one cell in series",
                ));
            }
            if *cells_in_parallel == 0 {
                report.push(issue(
                    "INV-POS-010",
                    ValidationStatus::Invalid,
                    vec!["cells_in_parallel"],
                    "a discrete module must have at least one parallel path",
                ));
            }
        }
        CellConfiguration::Monolithic { series_segments } => {
            if *series_segments == 0 {
                report.push(issue(
                    "INV-POS-011",
                    ValidationStatus::Invalid,
                    vec!["series_segments"],
                    "a monolithic module must have at least one series segment",
                ));
            }
        }
    }
}

// --- Electrical ordering (INV-ELE-001..012) -----------------------------

/// Checks the electrical ordering, consistency and plausibility
/// relations (`INV-ELE-001` to `INV-ELE-006`).
pub fn check_electrical_ordering(e: &ElectricalCharacteristics, report: &mut ValidationReport) {
    let v_oc = e.v_oc.get().volts();
    let v_mpp = e.v_mpp.get().volts();
    let i_sc = e.i_sc.get().amperes();
    let i_mpp = e.i_mpp.get().amperes();

    if v_oc <= v_mpp {
        report.push(
            issue(
                "INV-ELE-001",
                ValidationStatus::Invalid,
                vec!["v_oc", "v_mpp"],
                "open-circuit voltage must exceed the maximum-power-point voltage",
            )
            .with_observed(format!("v_oc = {v_oc} V, v_mpp = {v_mpp} V"))
            .with_expected("v_oc > v_mpp"),
        );
    }
    if i_sc <= i_mpp {
        report.push(
            issue(
                "INV-ELE-002",
                ValidationStatus::Invalid,
                vec!["i_sc", "i_mpp"],
                "short-circuit current must exceed the maximum-power-point current",
            )
            .with_observed(format!("i_sc = {i_sc} A, i_mpp = {i_mpp} A"))
            .with_expected("i_sc > i_mpp"),
        );
    }

    if v_oc > 0.0 {
        let ratio = v_mpp / v_oc;
        let (low, high) = VMPP_VOC_RATIO_RANGE;
        if ratio.is_finite() && !(low..=high).contains(&ratio) {
            report.push(
                issue(
                    "INV-ELE-003",
                    ValidationStatus::Warning,
                    vec!["v_mpp", "v_oc"],
                    "the ratio of maximum-power to open-circuit voltage is outside the usual range",
                )
                .with_observed(format!("v_mpp / v_oc = {ratio:.4}"))
                .with_expected(format!("{low} to {high}")),
            );
        }
    }

    let stated = e.p_max.get().watts();
    let implied = e.implied_p_max().watts();
    if let Some(gap) = relative_gap(stated, implied) {
        if gap > POWER_CONSISTENCY_CONFLICT {
            report.push(
                issue(
                    "INV-ELE-004",
                    ValidationStatus::Conflict,
                    vec!["p_max", "v_mpp", "i_mpp"],
                    "stated maximum power contradicts the product of maximum-power voltage and \
                     current; the engine cannot determine which is wrong",
                )
                .with_observed(format!(
                    "p_max = {stated:.3} W, v_mpp x i_mpp = {implied:.3} W, gap = {:.2} %",
                    gap * 100.0
                ))
                .with_expected(format!(
                    "gap below {} %",
                    POWER_CONSISTENCY_CONFLICT * 100.0
                )),
            );
        } else if gap > POWER_CONSISTENCY_WARNING {
            report.push(
                issue(
                    "INV-ELE-004",
                    ValidationStatus::Warning,
                    vec!["p_max", "v_mpp", "i_mpp"],
                    "stated maximum power differs from the maximum-power-point product by more \
                     than datasheet rounding explains",
                )
                .with_observed(format!(
                    "p_max = {stated:.3} W, v_mpp x i_mpp = {implied:.3} W, gap = {:.2} %",
                    gap * 100.0
                )),
            );
        }
    }

    let fill = e.fill_factor();
    if fill.is_finite() {
        if fill > 1.0 {
            report.push(
                issue(
                    "INV-ELE-006",
                    ValidationStatus::Invalid,
                    vec!["p_max", "v_oc", "i_sc"],
                    "the fill factor exceeds 1, which is impossible: maximum power cannot exceed \
                     the product of open-circuit voltage and short-circuit current",
                )
                .with_observed(format!("fill factor = {fill:.4}"))
                .with_expected("<= 1"),
            );
        } else {
            let (low, high) = FILL_FACTOR_RANGE;
            if !(low..=high).contains(&fill) {
                report.push(
                    issue(
                        "INV-ELE-005",
                        ValidationStatus::Warning,
                        vec!["p_max", "v_oc", "i_sc"],
                        "the fill factor is outside the range typical of commercial modules",
                    )
                    .with_observed(format!("fill factor = {fill:.4}"))
                    .with_expected(format!("{low} to {high}")),
                );
            }
        }
    }
}

/// Checks the open-circuit voltage against the number of series
/// junctions (`INV-ELE-010`).
pub fn check_voltage_per_junction(
    e: &ElectricalCharacteristics,
    c: &CellConfiguration,
    report: &mut ValidationReport,
) {
    let junctions = c.junctions_in_series();
    if junctions == 0 {
        return;
    }
    let per_junction = e.v_oc.get().volts() / f64::from(junctions);
    let (low, high) = VOLTS_PER_JUNCTION_RANGE;
    if per_junction.is_finite() && !(low..=high).contains(&per_junction) {
        report.push(
            issue(
                "INV-ELE-010",
                ValidationStatus::Warning,
                vec!["v_oc", "cells_in_series"],
                "open-circuit voltage per series junction is outside the range spanned by known \
                 cell technologies; the cell count or the voltage may be wrong",
            )
            .with_observed(format!(
                "{per_junction:.4} V per junction over {junctions} junctions"
            ))
            .with_expected(format!("{low} to {high} V")),
        );
    }
}

/// Checks the declared limits against the module's own ratings
/// (`INV-ELE-011`, `INV-ELE-012`).
pub fn check_limits(
    e: &ElectricalCharacteristics,
    l: &ElectricalLimits,
    report: &mut ValidationReport,
) {
    let v_oc = e.v_oc.get().volts();
    let max_system = l.max_system_voltage.get().volts();
    if max_system < v_oc {
        report.push(
            issue(
                "INV-ELE-011",
                ValidationStatus::Invalid,
                vec!["max_system_voltage", "v_oc"],
                "the maximum system voltage is below the module's own open-circuit voltage, which \
                 no string could satisfy",
            )
            .with_observed(format!(
                "max_system_voltage = {max_system} V, v_oc = {v_oc} V"
            ))
            .with_expected("max_system_voltage >= v_oc"),
        );
    }
    if let Some(fuse) = &l.max_series_fuse {
        let fuse_a = fuse.get().amperes();
        let i_sc = e.i_sc.get().amperes();
        if fuse_a < i_sc {
            report.push(
                issue(
                    "INV-ELE-012",
                    ValidationStatus::Warning,
                    vec!["max_series_fuse", "i_sc"],
                    "the maximum series fuse rating is below the short-circuit current",
                )
                .with_observed(format!("fuse = {fuse_a} A, i_sc = {i_sc} A")),
            );
        }
    }
}

// --- Geometry (INV-GEO-001..005) ---------------------------------------

/// Checks the area relations, including the stated-versus-derived area
/// conflict (`INV-GEO-001` to `INV-GEO-003`, `INV-GEO-005`).
pub fn check_geometry_relations(g: &ModuleGeometry, report: &mut ValidationReport) {
    let rectangular = g.rectangular_area().square_meters();

    if let (Some(aperture), Some(gross)) = (&g.aperture_area, &g.gross_area)
        && aperture.get().square_meters() > gross.get().square_meters()
    {
        report.push(
            issue(
                "INV-GEO-001",
                ValidationStatus::Invalid,
                vec!["aperture_area", "gross_area"],
                "the aperture area exceeds the overall area, which is geometrically impossible",
            )
            .with_observed(format!(
                "aperture = {:.4} m2, gross = {:.4} m2",
                aperture.get().square_meters(),
                gross.get().square_meters()
            )),
        );
    }

    if let Some(gross) = &g.gross_area
        && let Some(gap) = relative_gap(gross.get().square_meters(), rectangular)
    {
        if gap > AREA_AGREEMENT_CONFLICT {
            report.push(
                issue(
                    "INV-GEO-002",
                    ValidationStatus::Conflict,
                    vec!["gross_area", "rectangular_area"],
                    "the stated overall area contradicts the area implied by the module's own \
                     dimensions; either may be the wrong one and the engine does not choose",
                )
                .with_observed(format!(
                    "gross_area = {:.4} m2, rectangular_area = {rectangular:.4} m2, gap = {:.1} %",
                    gross.get().square_meters(),
                    gap * 100.0
                ))
                .with_expected(format!("gap below {} %", AREA_AGREEMENT_CONFLICT * 100.0)),
            );
        } else if gap > AREA_AGREEMENT_WARNING {
            report.push(
                issue(
                    "INV-GEO-002",
                    ValidationStatus::Warning,
                    vec!["gross_area", "rectangular_area"],
                    "the stated overall area differs slightly from the area implied by the \
                     dimensions, beyond what rounding usually explains",
                )
                .with_observed(format!(
                    "gross_area = {:.4} m2, rectangular_area = {rectangular:.4} m2, gap = {:.2} %",
                    gross.get().square_meters(),
                    gap * 100.0
                )),
            );
        }
    }

    let nested: [(&'static str, Option<f64>); 3] = [
        (
            "cell_area",
            g.cell_area.as_ref().map(|a| a.get().square_meters()),
        ),
        (
            "active_area",
            g.active_area.as_ref().map(|a| a.get().square_meters()),
        ),
        (
            "aperture_area",
            g.aperture_area.as_ref().map(|a| a.get().square_meters()),
        ),
    ];
    for window in nested.windows(2) {
        if let (Some(inner), Some(outer)) = (window[0].1, window[1].1)
            && inner > outer
        {
            report.push(
                issue(
                    "INV-GEO-003",
                    ValidationStatus::Invalid,
                    vec![window[0].0, window[1].0],
                    format!(
                        "{} exceeds {}, breaking the physical nesting of module areas",
                        window[0].0, window[1].0
                    ),
                )
                .with_observed(format!("{inner:.4} m2 > {outer:.4} m2")),
            );
        }
    }

    if let Some(depth) = depth_value(g) {
        let smallest = g
            .dimensions
            .length
            .get()
            .meters()
            .min(g.dimensions.width.get().meters());
        if depth > smallest {
            report.push(
                issue(
                    "INV-GEO-005",
                    ValidationStatus::Warning,
                    vec!["depth", "length", "width"],
                    "the module is thicker than its smallest lateral dimension, which is unusual",
                )
                .with_observed(format!(
                    "depth = {depth} m, smallest lateral = {smallest} m"
                )),
            );
        }
    }
}

/// Checks the area per series junction against known cell formats
/// (`INV-GEO-004`).
pub fn check_cell_area_plausibility(
    g: &ModuleGeometry,
    c: &CellConfiguration,
    report: &mut ValidationReport,
) {
    let junctions = c.junctions_in_series();
    let Some(cell_area) = &g.cell_area else {
        return;
    };
    if junctions == 0 {
        return;
    }
    let per_junction = cell_area.get().square_meters() / f64::from(junctions);
    let (low, high) = CELL_AREA_RANGE;
    if per_junction.is_finite() && !(low..=high).contains(&per_junction) {
        report.push(
            issue(
                "INV-GEO-004",
                ValidationStatus::Warning,
                vec!["cell_area", "cells_in_series"],
                "the area per series junction does not match any standard cell format; this is a \
                 data-quality indication only, since large formats postdate the reference data",
            )
            .with_observed(format!("{per_junction:.4} m2 per junction"))
            .with_expected(format!("{low} to {high} m2")),
        );
    }
}

// --- Efficiency (INV-DER-001..003) -------------------------------------

/// Checks the derived efficiency and its agreement with any stated value
/// (`INV-DER-001` to `INV-DER-003`).
pub fn check_efficiency(
    e: &ElectricalCharacteristics,
    g: &ModuleGeometry,
    report: &mut ValidationReport,
) {
    let (area, area_name) = g.preferred_area_for_efficiency();
    let area_m2 = area.square_meters();
    if area_m2 <= 0.0 || !area_m2.is_finite() {
        return;
    }
    let derived = e.p_max.get().watts() / (area_m2 * 1000.0);

    if derived > EFFICIENCY_ABSOLUTE_MAX {
        report.push(
            issue(
                "INV-DER-002",
                ValidationStatus::Invalid,
                vec!["p_max", area_name],
                "the derived efficiency exceeds a very generous physical bound, so the power or \
                 the area must be wrong",
            )
            .with_observed(format!(
                "efficiency = {:.1} % using {area_name}",
                derived * 100.0
            ))
            .with_expected(format!("<= {} %", EFFICIENCY_ABSOLUTE_MAX * 100.0)),
        );
    } else {
        let (low, high) = EFFICIENCY_TYPICAL_RANGE;
        if !(low..=high).contains(&derived) {
            report.push(
                issue(
                    "INV-DER-003",
                    ValidationStatus::Warning,
                    vec!["p_max", area_name],
                    "the derived efficiency is outside the range of commercial modules",
                )
                .with_observed(format!(
                    "efficiency = {:.1} % using {area_name}",
                    derived * 100.0
                ))
                .with_expected(format!("{} % to {} %", low * 100.0, high * 100.0)),
            );
        }
    }

    if let Some(stated) = &e.stated_efficiency
        && let Some(gap) = relative_gap(stated.get().value(), derived)
        && gap > EFFICIENCY_AGREEMENT_CONFLICT
    {
        report.push(
            issue(
                "INV-DER-001",
                ValidationStatus::Conflict,
                vec!["stated_efficiency", "p_max", area_name],
                "the stated efficiency contradicts the one derived from power and area; which \
                 area the manufacturer used is not recorded, so the engine does not choose",
            )
            .with_observed(format!(
                "stated = {:.2} %, derived = {:.2} % using {area_name}, gap = {:.1} %",
                stated.get().value() * 100.0,
                derived * 100.0,
                gap * 100.0
            ))
            .with_expected(format!(
                "gap below {} %",
                EFFICIENCY_AGREEMENT_CONFLICT * 100.0
            )),
        );
    }
}

// --- Thermal (INV-TMP-001..006) ----------------------------------------

fn coefficient_sign_issue(
    id: &'static str,
    field: &'static str,
    coefficient: &Measurement<TemperatureCoefficient>,
    expect_positive: bool,
) -> Option<ValidationIssue> {
    let value = coefficient.get().per_kelvin();
    let wrong = if expect_positive {
        value <= 0.0
    } else {
        value >= 0.0
    };
    if !wrong {
        return None;
    }
    let expectation = if expect_positive {
        "positive"
    } else {
        "negative"
    };
    Some(
        issue(
            id,
            ValidationStatus::Warning,
            vec![field],
            format!(
                "{field} is usually {expectation} for known technologies; an unexpected sign is \
                 reported rather than rejected, since rejecting it would be a physical judgement \
                 disguised as format validation"
            ),
        )
        .with_observed(format!("{field} = {value} per kelvin"))
        .with_expected(expectation),
    )
}

/// Checks the temperature coefficients' signs, magnitude and relative
/// ordering (`INV-TMP-001` to `INV-TMP-004`, `INV-TMP-006`).
pub fn check_thermal_coefficients(t: &ThermalCharacteristics, report: &mut ValidationReport) {
    if let Some(i) = coefficient_sign_issue("INV-TMP-001", "alpha_i_sc", &t.alpha_i_sc, true) {
        report.push(i);
    }
    if let Some(i) = coefficient_sign_issue("INV-TMP-002", "beta_v_oc", &t.beta_v_oc, false) {
        report.push(i);
    }
    if let Some(i) = coefficient_sign_issue("INV-TMP-003", "gamma_p_max", &t.gamma_p_max, false) {
        report.push(i);
    }

    let gamma = t.gamma_p_max.get();
    if gamma.basis() == CoefficientBasis::Relative {
        let magnitude = gamma.percent_per_celsius().abs();
        let (low, high) = GAMMA_MAGNITUDE_RANGE_PERCENT;
        if magnitude.is_finite() && !(low..=high).contains(&magnitude) {
            report.push(
                issue(
                    "INV-TMP-004",
                    ValidationStatus::Warning,
                    vec!["gamma_p_max"],
                    "the magnitude of the power temperature coefficient is outside the range \
                     spanned by known technologies",
                )
                .with_observed(format!("|gamma_p_max| = {magnitude:.3} %/K"))
                .with_expected(format!("{low} to {high} %/K")),
            );
        }

        let beta = t.beta_v_oc.get();
        if beta.basis() == CoefficientBasis::Relative {
            let beta_magnitude = beta.percent_per_celsius().abs();
            if magnitude.is_finite() && beta_magnitude.is_finite() && magnitude < beta_magnitude {
                report.push(
                    issue(
                        "INV-TMP-006",
                        ValidationStatus::Valid,
                        vec!["gamma_p_max", "beta_v_oc"],
                        "the power coefficient is usually at least as large in magnitude as the \
                         voltage coefficient; this is informational, not a defect",
                    )
                    .with_observed(format!(
                        "|gamma| = {magnitude:.3} %/K, |beta| = {beta_magnitude:.3} %/K"
                    )),
                );
            }
        }
    }
}

/// Reports that a coefficient's basis could not be determined by an
/// adapter (`INV-TMP-005`).
///
/// The basis is never inferred from magnitude. An adapter that cannot
/// tell whether its source publishes absolute or relative coefficients
/// must call this and construct no value: guessing would be a silent
/// arbitration, and the CEC library mixes both bases within a single
/// record.
pub fn ambiguous_coefficient_basis(field: &'static str, observed: &str) -> ValidationIssue {
    issue(
        "INV-TMP-005",
        ValidationStatus::Conflict,
        vec![field],
        "the source does not state whether this temperature coefficient is absolute or relative, \
         and the two differ by orders of magnitude; the basis is never guessed from the value",
    )
    .with_observed(observed.to_string())
    .with_expected("an explicit unit in the source")
}

// --- Conditions (INV-CND-001..006) -------------------------------------

/// Checks the operating points' temperatures, irradiances and
/// uniqueness (`INV-CND-001` to `INV-CND-006`).
pub fn check_operating_points(
    points: &[OperatingPoint],
    limits: &ElectricalLimits,
    stc_p_max_watts: f64,
    report: &mut ValidationReport,
) {
    let (low, high) = TEMPERATURE_RANGE_CELSIUS;

    for point in points {
        let celsius = point.condition.temperature.temperature().celsius();
        if !celsius.is_finite() || !(low..=high).contains(&celsius) {
            report.push(
                issue(
                    "INV-CND-001",
                    ValidationStatus::Invalid,
                    vec!["operating_condition.temperature"],
                    "a reference condition's temperature is outside any physically admissible \
                     range",
                )
                .with_observed(format!("{celsius} degC"))
                .with_expected(format!("{low} to {high} degC")),
            );
        }
        let irradiance = point.condition.irradiance.watts_per_m2();
        if !irradiance.is_finite() || irradiance <= 0.0 {
            report.push(
                issue(
                    "INV-CND-004",
                    ValidationStatus::Invalid,
                    vec!["operating_condition.irradiance"],
                    "a reference condition must impose a strictly positive irradiance",
                )
                .with_observed(format!("{irradiance} W/m2")),
            );
        }
    }

    // INV-CND-005: a condition kind must not appear twice, except for
    // matrix points, which are many by construction.
    let mut seen: Vec<&crate::OperatingConditionKind> = Vec::new();
    for point in points {
        let kind = &point.condition.kind;
        if matches!(kind, crate::OperatingConditionKind::Iec61853Matrix) {
            continue;
        }
        if seen.contains(&kind) {
            report.push(
                issue(
                    "INV-CND-005",
                    ValidationStatus::Invalid,
                    vec!["operating_points"],
                    "the same reference condition is declared twice, leaving its definition \
                     ambiguous",
                )
                .with_observed(format!("{kind:?}")),
            );
        } else {
            seen.push(kind);
        }
    }

    if let (Some(min), Some(max)) = (
        limits.min_operating_temperature.as_ref(),
        limits.max_operating_temperature.as_ref(),
    ) && min.get().celsius() >= max.get().celsius()
    {
        report.push(
            issue(
                "INV-CND-002",
                ValidationStatus::Invalid,
                vec!["min_operating_temperature", "max_operating_temperature"],
                "the declared minimum operating temperature is not below the maximum",
            )
            .with_observed(format!(
                "min = {} degC, max = {} degC",
                min.get().celsius(),
                max.get().celsius()
            )),
        );
    }

    for point in points {
        // INV-CND-003 applies to the settled cell temperature of the
        // nominal conditions, which are the only ones that let it settle.
        if matches!(
            point.condition.kind,
            crate::OperatingConditionKind::NoctIec61215
                | crate::OperatingConditionKind::NmotIec61853_2
        ) && let Some(cell) = &point.cell_temperature
        {
            let celsius = cell.get().celsius();
            let (nl, nh) = NOCT_RANGE_CELSIUS;
            if celsius.is_finite() && !(nl..=nh).contains(&celsius) {
                report.push(
                    issue(
                        "INV-CND-003",
                        ValidationStatus::Warning,
                        vec!["cell_temperature"],
                        "the settled cell temperature at the nominal operating condition is \
                         outside the usual range",
                    )
                    .with_observed(format!("{celsius} degC"))
                    .with_expected(format!("{nl} to {nh} degC")),
                );
            }
        }

        if matches!(
            point.condition.kind,
            crate::OperatingConditionKind::NoctIec61215
                | crate::OperatingConditionKind::NmotIec61853_2
        ) && let Some(p) = &point.p_max
            && stc_p_max_watts > 0.0
            && p.get().watts() >= stc_p_max_watts
        {
            report.push(
                issue(
                    "INV-CND-006",
                    ValidationStatus::Warning,
                    vec!["operating_points.p_max", "stc.p_max"],
                    "maximum power at the nominal operating condition is not below its value at \
                     standard test conditions, despite the lower irradiance there",
                )
                .with_observed(format!(
                    "nominal = {} W, STC = {stc_p_max_watts} W",
                    p.get().watts()
                )),
            );
        }
    }
}

// --- Bifacial and structure (INV-BIF-001..004, INV-STR-001..004) -------

/// Checks the bifacial block's internal consistency (`INV-BIF-001` to
/// `INV-BIF-004`).
pub fn check_bifacial(
    b: &BifacialCharacteristics,
    e: &ElectricalCharacteristics,
    technology: &ModuleTechnology,
    report: &mut ValidationReport,
) {
    if b.is_uncharacterised() {
        report.push(issue(
            "INV-BIF-001",
            ValidationStatus::Warning,
            vec!["bifacial"],
            "the module is declared bifacial but carries no bifaciality factor and no rear-side \
             data; this is the normal outcome of a CEC import, not an anomaly",
        ));
    }

    if let Some(factor) = &b.bifaciality_factor {
        let value = factor.get().value();
        if value <= 0.0 {
            report.push(
                issue(
                    "INV-BIF-002",
                    ValidationStatus::Invalid,
                    vec!["bifaciality_factor"],
                    "the bifaciality factor must be strictly positive",
                )
                .with_observed(format!("{value}")),
            );
        }
    }

    if let Some(rear) = &b.rear_side_parameters {
        let front_p = e.p_max.get().watts();
        if let Some(rear_p) = &rear.p_max
            && rear_p.get().watts() > front_p
        {
            report.push(
                issue(
                    "INV-BIF-003",
                    ValidationStatus::Warning,
                    vec!["rear_side_parameters.p_max", "p_max"],
                    "rear-side maximum power exceeds the front-side value, which is unusual",
                )
                .with_observed(format!(
                    "rear = {} W, front = {front_p} W",
                    rear_p.get().watts()
                )),
            );
        }
    }

    if technology.architecture == crate::CellArchitecture::InterdigitatedBackContact {
        report.push(issue(
            "INV-BIF-004",
            ValidationStatus::Warning,
            vec!["bifacial", "architecture"],
            "bifaciality is declared on an interdigitated back-contact architecture, where the \
             rear face carries the contacts; verify the source",
        ));
    }
}

/// Checks structural coherence between configuration, technology and
/// additional outputs (`INV-STR-001` to `INV-STR-004`).
pub fn check_structure(
    c: &CellConfiguration,
    technology: &ModuleTechnology,
    additional_output_count: usize,
    report: &mut ValidationReport,
) {
    if let CellConfiguration::Discrete {
        cells_in_parallel,
        cell_division,
        substrings,
        bypass_diodes,
        ..
    } = c
    {
        if *cell_division == CellDivision::Half && cells_in_parallel % 2 != 0 {
            report.push(
                issue(
                    "INV-STR-001",
                    ValidationStatus::Warning,
                    vec!["cell_division", "cells_in_parallel"],
                    "half-cut modules usually pair their half-modules, so an odd number of \
                     parallel paths is unexpected",
                )
                .with_observed(format!("{cells_in_parallel} parallel paths")),
            );
        }
        if let (Some(diodes), Some(subs)) = (bypass_diodes, substrings)
            && diodes > subs
        {
            report.push(
                issue(
                    "INV-STR-002",
                    ValidationStatus::Warning,
                    vec!["bypass_diodes", "substrings"],
                    "more bypass diodes than sub-strings were declared",
                )
                .with_observed(format!("{diodes} diodes, {subs} sub-strings")),
            );
        }
    }

    if additional_output_count > 0
        && !matches!(technology.technology, crate::CellTechnology::Tandem { .. })
    {
        report.push(issue(
            "INV-STR-003",
            ValidationStatus::Warning,
            vec!["additional_outputs", "technology"],
            "independent additional electrical outputs are declared on a technology other than a \
             tandem, which is the only case known to produce them",
        ));
    }

    if technology.is_thin_film() && !c.is_monolithic() {
        report.push(issue(
            "INV-STR-004",
            ValidationStatus::Warning,
            vec!["technology", "configuration"],
            "a thin-film material is paired with a discrete-cell configuration; thin-film modules \
             are normally monolithically scribed",
        ));
    }
}

// --- Provenance (INV-PRV-001..003) -------------------------------------

/// Checks one value's provenance and classification (`INV-PRV-001` to
/// `INV-PRV-003`).
pub fn check_provenance<T>(field: &'static str, m: &Measurement<T>, report: &mut ValidationReport) {
    if m.provenance().is_none() {
        report.push(issue(
            "INV-PRV-001",
            ValidationStatus::Warning,
            vec![field],
            "no provenance is recorded, so the value's authority cannot be established; it \
             remains usable but unattributed",
        ));
    }

    if m.classification().requires_rationale() && m.rationale().is_none_or(str::is_empty) {
        report.push(issue(
            "INV-PRV-002",
            ValidationStatus::Invalid,
            vec![field],
            "an estimated value carries no rationale; an estimate without a recorded reason is \
             indistinguishable from an invention",
        ));
    }

    if m.classification() == DataClass::Fitted {
        report.push(issue(
            "INV-PRV-003",
            ValidationStatus::Invalid,
            vec![field],
            "a fitted model parameter is held in a field reserved for physical quantities; fitted \
             parameters belong with the model that interprets them, never in the module's \
             physical description",
        ));
    }
}

/// Runs every invariant that applies to the given parts of a module.
///
/// This is what [`crate::PvModuleBuilder`] calls; the individual
/// functions remain public so each invariant can be exercised on its own.
#[allow(clippy::too_many_arguments)]
pub fn validate_module(
    electrical: &ElectricalCharacteristics,
    geometry: &ModuleGeometry,
    configuration: &CellConfiguration,
    thermal: &ThermalCharacteristics,
    limits: &ElectricalLimits,
    technology: &ModuleTechnology,
    operating_points: &[OperatingPoint],
    bifacial: Option<&BifacialCharacteristics>,
    additional_output_count: usize,
) -> ValidationReport {
    let mut report = ValidationReport::new();

    check_electrical_positivity(electrical, &mut report);
    check_geometry_positivity(geometry, &mut report);
    check_configuration_positivity(configuration, &mut report);
    check_electrical_ordering(electrical, &mut report);
    check_voltage_per_junction(electrical, configuration, &mut report);
    check_limits(electrical, limits, &mut report);
    check_geometry_relations(geometry, &mut report);
    check_cell_area_plausibility(geometry, configuration, &mut report);
    check_efficiency(electrical, geometry, &mut report);
    check_thermal_coefficients(thermal, &mut report);
    check_operating_points(
        operating_points,
        limits,
        electrical.p_max.get().watts(),
        &mut report,
    );
    if let Some(b) = bifacial {
        check_bifacial(b, electrical, technology, &mut report);
    }
    check_structure(
        configuration,
        technology,
        additional_output_count,
        &mut report,
    );

    check_provenance("p_max", &electrical.p_max, &mut report);
    check_provenance("v_oc", &electrical.v_oc, &mut report);
    check_provenance("i_sc", &electrical.i_sc, &mut report);
    check_provenance("v_mpp", &electrical.v_mpp, &mut report);
    check_provenance("i_mpp", &electrical.i_mpp, &mut report);
    check_provenance("alpha_i_sc", &thermal.alpha_i_sc, &mut report);
    check_provenance("beta_v_oc", &thermal.beta_v_oc, &mut report);
    check_provenance("gamma_p_max", &thermal.gamma_p_max, &mut report);

    report
}

/// Every invariant identifier this module can emit, in specification
/// order. Used by the test suite to assert that none is forgotten.
///
/// The count is **52**, not the 47 quoted in the prose of
/// `docs/design/27-specification-finale-pv-module.md` §11. The
/// specification's own tables enumerate 51; `INV-STR-004` was added during
/// implementation and is recorded in
/// `docs/adr/0007-invariant-inventory.md`. The array is the authoritative
/// inventory, and `invariant_ids_are_unique_and_complete` in the test
/// suite asserts that the code emits nothing outside it.
pub const ALL_INVARIANT_IDS: [&str; 52] = [
    "INV-POS-001",
    "INV-POS-002",
    "INV-POS-003",
    "INV-POS-004",
    "INV-POS-005",
    "INV-POS-006",
    "INV-POS-007",
    "INV-POS-008",
    "INV-POS-009",
    "INV-POS-010",
    "INV-POS-011",
    "INV-POS-012",
    "INV-ELE-001",
    "INV-ELE-002",
    "INV-ELE-003",
    "INV-ELE-004",
    "INV-ELE-005",
    "INV-ELE-006",
    "INV-ELE-010",
    "INV-ELE-011",
    "INV-ELE-012",
    "INV-GEO-001",
    "INV-GEO-002",
    "INV-GEO-003",
    "INV-GEO-004",
    "INV-GEO-005",
    "INV-DER-001",
    "INV-DER-002",
    "INV-DER-003",
    "INV-TMP-001",
    "INV-TMP-002",
    "INV-TMP-003",
    "INV-TMP-004",
    "INV-TMP-005",
    "INV-TMP-006",
    "INV-CND-001",
    "INV-CND-002",
    "INV-CND-003",
    "INV-CND-004",
    "INV-CND-005",
    "INV-CND-006",
    "INV-BIF-001",
    "INV-BIF-002",
    "INV-BIF-003",
    "INV-BIF-004",
    "INV-STR-001",
    "INV-STR-002",
    "INV-STR-003",
    "INV-STR-004",
    "INV-PRV-001",
    "INV-PRV-002",
    "INV-PRV-003",
];
