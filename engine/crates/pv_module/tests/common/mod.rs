//! Fixtures shared by the invariant and integration suites.
//!
//! **These are not real products.** Every figure here is constructed to be
//! internally coherent so that a test can perturb exactly one quantity and
//! attribute the resulting finding to that perturbation. The five real
//! manufacturer datasheets and the IEC 61853-1 grid required by
//! `docs/design/28-plan-tests-pv-module.md` remain open items and are
//! deliberately **not** simulated here: a fabricated datasheet would be an
//! estimate wearing the authority of manufacturer data, which is the exact
//! confusion `DataClass` exists to prevent.

#![allow(dead_code)]

use core_units::{
    Area, Current, Length, Mass, Power, Temperature, TemperatureCoefficient, Voltage,
};
use pv_module::{
    CellArchitecture, CellConfiguration, CellTechnology, ElectricalCharacteristics,
    ElectricalLimits, ExtractionMethod, Measurement, ModuleDimensions, ModuleGeometry,
    ModuleIdentity, ModuleTechnology, Provenance, PvModuleBuilder, SourceType,
    ThermalCharacteristics, ValidationReport, ValidationStatus,
};

/// Electrical characteristics from bare SI magnitudes.
pub fn electrical(
    p: f64,
    v_oc: f64,
    i_sc: f64,
    v_mpp: f64,
    i_mpp: f64,
) -> ElectricalCharacteristics {
    ElectricalCharacteristics::new(
        Measurement::stated(Power::from_watts(p)),
        Measurement::stated(Voltage::from_volts(v_oc)),
        Measurement::stated(Current::from_amperes(i_sc)),
        Measurement::stated(Voltage::from_volts(v_mpp)),
        Measurement::stated(Current::from_amperes(i_mpp)),
    )
}

/// A coherent 400 W module: 72 junctions, 2 m², fill factor 0.777,
/// efficiency 20 %.
pub fn coherent_electrical() -> ElectricalCharacteristics {
    electrical(400.0, 49.5, 10.4, 41.5, 9.64)
}

/// Dimensions in metres.
pub fn dimensions(length: f64, width: f64) -> ModuleDimensions {
    ModuleDimensions::new(
        Measurement::stated(Length::from_meters(length)),
        Measurement::stated(Length::from_meters(width)),
    )
}

/// A 2 m x 1 m geometry with no stated areas.
pub fn geometry(length: f64, width: f64) -> ModuleGeometry {
    ModuleGeometry::new(dimensions(length, width))
}

/// The coherent baseline geometry: 2 m², stated and derived in agreement.
pub fn coherent_geometry() -> ModuleGeometry {
    geometry(2.0, 1.0).with_gross_area(area(2.0))
}

/// A stated area in square metres.
pub fn area(square_meters: f64) -> Measurement<Area> {
    Measurement::stated(Area::from_square_meters(square_meters))
}

/// A stated mass in kilograms.
pub fn mass(kilograms: f64) -> Measurement<Mass> {
    Measurement::stated(Mass::from_kilograms(kilograms))
}

/// A stated length in metres.
pub fn length(meters: f64) -> Measurement<Length> {
    Measurement::stated(Length::from_meters(meters))
}

/// A stated temperature in degrees Celsius.
pub fn temperature(celsius: f64) -> Measurement<Temperature> {
    Measurement::stated(Temperature::from_celsius(celsius))
}

/// A relative temperature coefficient in percent per kelvin.
pub fn coefficient(percent_per_kelvin: f64) -> Measurement<TemperatureCoefficient> {
    Measurement::stated(TemperatureCoefficient::from_percent_per_celsius(
        percent_per_kelvin,
    ))
}

/// Thermal characteristics from three relative coefficients, in %/K.
pub fn thermal(alpha: f64, beta: f64, gamma: f64) -> ThermalCharacteristics {
    ThermalCharacteristics::new(coefficient(alpha), coefficient(beta), coefficient(gamma))
}

/// The coherent baseline coefficients: alpha positive, beta and gamma
/// negative, |gamma| > |beta|.
pub fn coherent_thermal() -> ThermalCharacteristics {
    thermal(0.045, -0.27, -0.34)
}

/// A transcription from a manufacturer datasheet.
pub fn datasheet_provenance() -> Provenance {
    Provenance::new(
        SourceType::ManufacturerDatasheet,
        ExtractionMethod::ManualTranscription,
    )
    .with_manufacturer("Example Manufacturer")
    .with_datasheet_reference("DS-EX-400 rev A")
}

/// The coherent electrical block with every value attributed, so that
/// `INV-PRV-001` has nothing to report.
pub fn attributed_electrical() -> ElectricalCharacteristics {
    let mut e = coherent_electrical();
    e.p_max = e.p_max.with_provenance(datasheet_provenance());
    e.v_oc = e.v_oc.with_provenance(datasheet_provenance());
    e.i_sc = e.i_sc.with_provenance(datasheet_provenance());
    e.v_mpp = e.v_mpp.with_provenance(datasheet_provenance());
    e.i_mpp = e.i_mpp.with_provenance(datasheet_provenance());
    e
}

/// The coherent thermal block with every coefficient attributed.
pub fn attributed_thermal() -> ThermalCharacteristics {
    let mut t = coherent_thermal();
    t.alpha_i_sc = t.alpha_i_sc.with_provenance(datasheet_provenance());
    t.beta_v_oc = t.beta_v_oc.with_provenance(datasheet_provenance());
    t.gamma_p_max = t.gamma_p_max.with_provenance(datasheet_provenance());
    t
}

/// A builder in which every physical value carries its provenance, so the
/// report is empty rather than merely free of blocking findings.
pub fn attributed_builder() -> PvModuleBuilder {
    builder_with(
        silicon(),
        coherent_configuration(),
        coherent_geometry(),
        attributed_electrical(),
        attributed_thermal(),
        coherent_limits(),
    )
}

/// Limits carrying only the mandatory maximum system voltage.
pub fn limits(max_system_volts: f64) -> ElectricalLimits {
    ElectricalLimits::new(Measurement::stated(Voltage::from_volts(max_system_volts)))
}

/// The coherent baseline limits: 1500 V, comfortably above `v_oc`.
pub fn coherent_limits() -> ElectricalLimits {
    limits(1500.0)
}

/// Monocrystalline silicon with a PERC architecture.
pub fn silicon() -> ModuleTechnology {
    ModuleTechnology::new(
        CellTechnology::MonocrystallineSilicon,
        CellArchitecture::PERC,
    )
}

/// Cadmium telluride on a monolithically scribed stack.
pub fn cadmium_telluride() -> ModuleTechnology {
    ModuleTechnology::new(
        CellTechnology::CadmiumTelluride,
        CellArchitecture::MonolithicThinFilm,
    )
}

/// A perovskite-on-silicon tandem.
pub fn tandem() -> ModuleTechnology {
    ModuleTechnology::new(
        CellTechnology::Tandem {
            top: Box::new(CellTechnology::Perovskite),
            bottom: Box::new(CellTechnology::MonocrystallineSilicon),
        },
        CellArchitecture::Unknown,
    )
}

/// The coherent baseline configuration: 72 whole cells in one string.
pub fn coherent_configuration() -> CellConfiguration {
    CellConfiguration::discrete(72)
}

/// A builder whose every block is coherent, for tests that perturb one.
pub fn coherent_builder() -> PvModuleBuilder {
    builder_with(
        silicon(),
        coherent_configuration(),
        coherent_geometry(),
        coherent_electrical(),
        coherent_thermal(),
        coherent_limits(),
    )
}

/// A builder assembled from named blocks, with the coherent identity.
///
/// The builder's fields are private, so a test that needs a different
/// block supplies it here rather than mutating a built one — which is
/// also how a caller outside the crate must do it.
pub fn builder_with(
    technology: ModuleTechnology,
    configuration: CellConfiguration,
    geometry: ModuleGeometry,
    electrical: ElectricalCharacteristics,
    thermal: ThermalCharacteristics,
    limits: ElectricalLimits,
) -> PvModuleBuilder {
    PvModuleBuilder::new(
        ModuleIdentity::new("Example Manufacturer", "EX-400"),
        technology,
        configuration,
        geometry,
        electrical,
        thermal,
        limits,
    )
}

/// A builder differing from the coherent baseline only in its electrical
/// block.
pub fn builder_with_electrical(electrical: ElectricalCharacteristics) -> PvModuleBuilder {
    builder_with(
        silicon(),
        coherent_configuration(),
        coherent_geometry(),
        electrical,
        coherent_thermal(),
        coherent_limits(),
    )
}

/// A builder differing from the coherent baseline only in its geometry.
pub fn builder_with_geometry(geometry: ModuleGeometry) -> PvModuleBuilder {
    builder_with(
        silicon(),
        coherent_configuration(),
        geometry,
        coherent_electrical(),
        coherent_thermal(),
        coherent_limits(),
    )
}

/// Build an empty report, hand it to `f`, and return what `f` recorded.
pub fn report_from(f: impl FnOnce(&mut ValidationReport)) -> ValidationReport {
    let mut report = ValidationReport::new();
    f(&mut report);
    report
}

/// The status recorded for `id`, or `None` if the invariant said nothing.
pub fn status_of(report: &ValidationReport, id: &str) -> Option<ValidationStatus> {
    report.find(id).map(|i| i.status())
}

/// Assert that `id` was reported at exactly `status`, with a usable
/// explanation.
#[track_caller]
pub fn assert_reported(report: &ValidationReport, id: &str, status: ValidationStatus) {
    let issue = report
        .find(id)
        .unwrap_or_else(|| panic!("{id} was expected at {status:?} but was not reported"));
    assert_eq!(
        issue.status(),
        status,
        "{id} was reported at {:?}, expected {status:?}",
        issue.status()
    );
    assert!(
        !issue.explanation().is_empty(),
        "{id} must carry a non-empty explanation"
    );
    assert!(
        !issue.fields().is_empty(),
        "{id} must name the fields it concerns"
    );
}

/// Assert that `id` said nothing at all.
#[track_caller]
pub fn assert_silent(report: &ValidationReport, id: &str) {
    assert!(
        !report.contains(id),
        "{id} was expected to be silent but reported {:?}",
        status_of(report, id)
    );
}
