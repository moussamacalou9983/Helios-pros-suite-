use core_units::{Current, Irradiance, Power, Temperature, Velocity, Voltage};

use crate::provenance::Measurement;

/// Which temperature a reference condition actually imposes.
///
/// **This distinction is the physics the abstraction exists to carry.**
/// Standard test conditions impose the *cell* temperature at 25 °C. NOCT,
/// NMOT and PTC impose the *ambient* temperature and let the cell
/// temperature settle wherever the module's thermal behaviour puts it.
///
/// An abstraction that did not separate the two would make these
/// conditions falsely comparable, and would let a model read 20 °C as a
/// cell temperature when it is an ambient one — a systematic error in
/// every downstream temperature correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemperatureSpecification {
    /// The cell temperature is imposed (STC, IEC 61853-1 matrix points).
    CellTemperature(Temperature),
    /// The ambient temperature is imposed and the cell temperature
    /// settles (NOCT, NMOT, PTC).
    AmbientTemperature(Temperature),
}

impl TemperatureSpecification {
    /// The imposed temperature, whichever kind it is. Callers that need
    /// to know *which* kind must match on the variant instead.
    #[inline]
    pub const fn temperature(self) -> Temperature {
        match self {
            TemperatureSpecification::CellTemperature(t)
            | TemperatureSpecification::AmbientTemperature(t) => t,
        }
    }

    /// Whether the imposed temperature is the cell's.
    #[inline]
    pub const fn is_cell_temperature(self) -> bool {
        matches!(self, TemperatureSpecification::CellTemperature(_))
    }
}

/// The reference solar spectrum a condition specifies.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Spectrum {
    /// The AM1.5 global reference spectrum of IEC 60904-3.
    Am15Global,
    /// The AM1.5 direct reference spectrum.
    Am15Direct,
    /// A spectrum named by the source but not classified here.
    Other {
        /// The name as the source gave it.
        name: String,
    },
}

/// The electrical state a condition holds the module in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ElectricalState {
    /// Open circuit, as NOCT prescribes.
    OpenCircuit,
    /// Held at the maximum power point, as NMOT prescribes.
    MaximumPowerPoint,
}

/// Which normative reference condition this is.
///
/// `#[non_exhaustive]`: adding a future condition — a revised standard, a
/// regional reference — must be a variant addition, never a structural
/// change. PTC was added exactly this way after direct inspection of the
/// CEC library showed it populated on all 21 535 records
/// (`docs/design/26-validation-empirique-cec-sandia.md` §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OperatingConditionKind {
    /// Standard test conditions: 1000 W/m², cell at 25 °C, AM1.5G.
    Stc,
    /// Nominal operating cell temperature, IEC 61215.
    NoctIec61215,
    /// Nominal module operating temperature, IEC 61853-2.
    NmotIec61853_2,
    /// PVUSA test conditions: 1000 W/m², ambient at 20 °C, wind 1 m/s.
    /// A North American market reference, distinct from STC.
    PtcPvusa,
    /// A point of the IEC 61853-1 performance matrix.
    Iec61853Matrix,
    /// A condition defined by the user.
    UserDefined {
        /// A name for the condition.
        name: String,
    },
}

/// A complete reference condition: what irradiance, what temperature (and
/// of what), what spectrum, what wind, what electrical state.
///
/// Conditions are never mixed: each [`OperatingPoint`] carries its own
/// complete condition rather than relying on a positional convention.
#[derive(Debug, Clone, PartialEq)]
pub struct OperatingCondition {
    /// Which normative condition this is.
    pub kind: OperatingConditionKind,
    /// The irradiance the condition imposes.
    pub irradiance: Irradiance,
    /// The temperature the condition imposes, and of what.
    pub temperature: TemperatureSpecification,
    /// The reference spectrum, if the condition specifies one.
    pub spectrum: Option<Spectrum>,
    /// The wind speed, if the condition specifies one.
    pub wind_speed: Option<Velocity>,
    /// The electrical state, if the condition specifies one.
    pub electrical_state: Option<ElectricalState>,
}

impl OperatingCondition {
    /// Standard test conditions: 1000 W/m², cell temperature 25 °C,
    /// AM1.5G spectrum.
    ///
    /// **References**: IEC 61215 (conditions), IEC 60904-3 (spectrum).
    pub fn stc() -> Self {
        OperatingCondition {
            kind: OperatingConditionKind::Stc,
            irradiance: Irradiance::from_watts_per_m2(1000.0),
            temperature: TemperatureSpecification::CellTemperature(Temperature::from_celsius(25.0)),
            spectrum: Some(Spectrum::Am15Global),
            wind_speed: None,
            electrical_state: None,
        }
    }

    /// Nominal operating cell temperature conditions: 800 W/m², ambient
    /// 20 °C, wind 1 m/s, open circuit.
    ///
    /// **Reference**: IEC 61215.
    pub fn noct() -> Self {
        OperatingCondition {
            kind: OperatingConditionKind::NoctIec61215,
            irradiance: Irradiance::from_watts_per_m2(800.0),
            temperature: TemperatureSpecification::AmbientTemperature(Temperature::from_celsius(
                20.0,
            )),
            spectrum: Some(Spectrum::Am15Global),
            wind_speed: Some(Velocity::from_meters_per_second(1.0)),
            electrical_state: Some(ElectricalState::OpenCircuit),
        }
    }

    /// Nominal module operating temperature conditions: 800 W/m²,
    /// ambient 20 °C, wind 1 m/s, held at the maximum power point.
    ///
    /// **Reference**: IEC 61853-2. Distinct from [`OperatingCondition::noct`]
    /// — treating the two as interchangeable would introduce a systematic
    /// cell-temperature error.
    pub fn nmot() -> Self {
        OperatingCondition {
            kind: OperatingConditionKind::NmotIec61853_2,
            irradiance: Irradiance::from_watts_per_m2(800.0),
            temperature: TemperatureSpecification::AmbientTemperature(Temperature::from_celsius(
                20.0,
            )),
            spectrum: Some(Spectrum::Am15Global),
            wind_speed: Some(Velocity::from_meters_per_second(1.0)),
            electrical_state: Some(ElectricalState::MaximumPowerPoint),
        }
    }

    /// PVUSA test conditions: 1000 W/m², ambient 20 °C, wind 1 m/s.
    ///
    /// A North American market reference populated on every record of the
    /// CEC module library. Note the contrast with STC: the same
    /// irradiance, but an *ambient* temperature specification.
    pub fn ptc() -> Self {
        OperatingCondition {
            kind: OperatingConditionKind::PtcPvusa,
            irradiance: Irradiance::from_watts_per_m2(1000.0),
            temperature: TemperatureSpecification::AmbientTemperature(Temperature::from_celsius(
                20.0,
            )),
            spectrum: Some(Spectrum::Am15Global),
            wind_speed: Some(Velocity::from_meters_per_second(1.0)),
            electrical_state: None,
        }
    }

    /// A point of the IEC 61853-1 performance matrix, at an imposed cell
    /// temperature.
    pub fn iec61853_matrix_point(irradiance: Irradiance, cell_temperature: Temperature) -> Self {
        OperatingCondition {
            kind: OperatingConditionKind::Iec61853Matrix,
            irradiance,
            temperature: TemperatureSpecification::CellTemperature(cell_temperature),
            spectrum: Some(Spectrum::Am15Global),
            wind_speed: None,
            electrical_state: None,
        }
    }

    /// Whether two conditions may meaningfully be compared: they must
    /// impose the same *kind* of temperature.
    ///
    /// Comparing a cell-temperature condition with an
    /// ambient-temperature one is not a difference of value but a
    /// category error, which is what this guard exists to prevent
    /// (`docs/design/27-specification-finale-pv-module.md` §5).
    #[inline]
    pub fn is_comparable_to(&self, other: &OperatingCondition) -> bool {
        self.temperature.is_cell_temperature() == other.temperature.is_cell_temperature()
    }
}

/// A measured operating point: a condition together with whatever
/// electrical quantities were recorded at it.
#[derive(Debug, Clone, PartialEq)]
pub struct OperatingPoint {
    /// The condition this point was measured or declared at.
    pub condition: OperatingCondition,
    /// Maximum power at this condition, if stated.
    pub p_max: Option<Measurement<Power>>,
    /// Open-circuit voltage at this condition, if stated.
    pub v_oc: Option<Measurement<Voltage>>,
    /// Short-circuit current at this condition, if stated.
    pub i_sc: Option<Measurement<Current>>,
    /// Maximum-power-point voltage at this condition, if stated.
    pub v_mpp: Option<Measurement<Voltage>>,
    /// Maximum-power-point current at this condition, if stated.
    pub i_mpp: Option<Measurement<Current>>,
    /// The cell temperature reached, meaningful when the condition
    /// imposes an ambient temperature and lets the cell settle.
    pub cell_temperature: Option<Measurement<Temperature>>,
}

impl OperatingPoint {
    /// An empty operating point at a given condition.
    #[inline]
    pub fn new(condition: OperatingCondition) -> Self {
        OperatingPoint {
            condition,
            p_max: None,
            v_oc: None,
            i_sc: None,
            v_mpp: None,
            i_mpp: None,
            cell_temperature: None,
        }
    }

    /// Record the maximum power.
    #[inline]
    #[must_use]
    pub fn with_p_max(mut self, p_max: Measurement<Power>) -> Self {
        self.p_max = Some(p_max);
        self
    }

    /// Record the open-circuit voltage.
    #[inline]
    #[must_use]
    pub fn with_v_oc(mut self, v_oc: Measurement<Voltage>) -> Self {
        self.v_oc = Some(v_oc);
        self
    }

    /// Record the short-circuit current.
    #[inline]
    #[must_use]
    pub fn with_i_sc(mut self, i_sc: Measurement<Current>) -> Self {
        self.i_sc = Some(i_sc);
        self
    }

    /// Record the maximum-power-point voltage and current.
    #[inline]
    #[must_use]
    pub fn with_mpp(mut self, v_mpp: Measurement<Voltage>, i_mpp: Measurement<Current>) -> Self {
        self.v_mpp = Some(v_mpp);
        self.i_mpp = Some(i_mpp);
        self
    }

    /// Record the cell temperature reached at this condition.
    #[inline]
    #[must_use]
    pub fn with_cell_temperature(mut self, temperature: Measurement<Temperature>) -> Self {
        self.cell_temperature = Some(temperature);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stc_imposes_a_cell_temperature() {
        let c = OperatingCondition::stc();
        assert!(c.temperature.is_cell_temperature());
        assert!((c.temperature.temperature().celsius() - 25.0).abs() < 1e-9);
        assert!((c.irradiance.watts_per_m2() - 1000.0).abs() < 1e-9);
        assert_eq!(c.spectrum, Some(Spectrum::Am15Global));
    }

    #[test]
    fn noct_and_ptc_impose_an_ambient_temperature() {
        for c in [OperatingCondition::noct(), OperatingCondition::ptc()] {
            assert!(!c.temperature.is_cell_temperature());
            assert!((c.temperature.temperature().celsius() - 20.0).abs() < 1e-9);
            assert!((c.wind_speed.unwrap().meters_per_second() - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn ptc_shares_stc_irradiance_but_not_its_temperature_kind() {
        // The exact confusion the abstraction exists to prevent.
        let stc = OperatingCondition::stc();
        let ptc = OperatingCondition::ptc();
        assert!((stc.irradiance.watts_per_m2() - ptc.irradiance.watts_per_m2()).abs() < 1e-9);
        assert!(!stc.is_comparable_to(&ptc));
    }

    #[test]
    fn noct_and_nmot_differ_by_electrical_state() {
        let noct = OperatingCondition::noct();
        let nmot = OperatingCondition::nmot();
        assert_eq!(noct.electrical_state, Some(ElectricalState::OpenCircuit));
        assert_eq!(
            nmot.electrical_state,
            Some(ElectricalState::MaximumPowerPoint)
        );
        assert_ne!(noct.kind, nmot.kind);
    }

    #[test]
    fn conditions_of_the_same_temperature_kind_are_comparable() {
        let stc = OperatingCondition::stc();
        let matrix = OperatingCondition::iec61853_matrix_point(
            Irradiance::from_watts_per_m2(600.0),
            Temperature::from_celsius(50.0),
        );
        assert!(stc.is_comparable_to(&matrix));
        assert!(OperatingCondition::noct().is_comparable_to(&OperatingCondition::nmot()));
    }

    #[test]
    fn an_operating_point_records_only_what_was_stated() {
        let p = OperatingPoint::new(OperatingCondition::noct())
            .with_p_max(Measurement::stated(Power::from_watts(330.0)))
            .with_cell_temperature(Measurement::measured(Temperature::from_celsius(44.0)));
        assert!(p.p_max.is_some());
        assert!(p.v_oc.is_none());
        assert!((p.cell_temperature.unwrap().get().celsius() - 44.0).abs() < 1e-9);
    }

    #[test]
    fn a_user_defined_condition_is_representable() {
        let c = OperatingCondition {
            kind: OperatingConditionKind::UserDefined {
                name: "site design point".to_string(),
            },
            irradiance: Irradiance::from_watts_per_m2(850.0),
            temperature: TemperatureSpecification::CellTemperature(Temperature::from_celsius(45.0)),
            spectrum: None,
            wind_speed: None,
            electrical_state: None,
        };
        assert!(c.temperature.is_cell_temperature());
    }
}
