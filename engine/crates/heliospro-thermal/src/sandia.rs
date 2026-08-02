//! Sandia (SAPM) cell temperature model.
//!
//! **Source**: King, D.L., Boyson, W.E., Kratochvil, J.A. (2004),
//! *Photovoltaic Array Performance Model*, SAND2004-3535, Sandia National
//! Laboratories, Section on "Temperature Model", Eq. 11-12.
//!
//! A two-stage empirical model: first the back-of-module (or open-rack cell,
//! depending on mounting) temperature is estimated from irradiance and wind
//! speed, then the actual cell temperature is obtained by adding a
//! small irradiance-dependent offset representing the module's internal
//! (laminate) temperature gradient. Unlike [`crate::noct`], this model
//! explicitly accounts for convective cooling by wind.

use heliospro_units::{Irradiance, Temperature, Velocity};

/// Empirical mounting-configuration-dependent coefficients for the Sandia
/// thermal model.
///
/// **Source**: King et al. (2004), SAND2004-3535, Table 1. HeliosPro
/// currently provides the two configurations the project could verify with
/// high confidence in this development session (see the associated
/// constants' documentation); callers with manufacturer- or
/// installation-specific coefficients (from IEC 61853 outdoor testing, for
/// instance) should construct `SandiaMountingCoefficients` directly.
#[derive(Debug, Clone, Copy)]
pub struct SandiaMountingCoefficients {
    /// Coefficient `a` in the module-temperature exponential (dimensionless).
    pub a: f64,
    /// Coefficient `b` in the module-temperature exponential (s/m).
    pub b: f64,
    /// Temperature difference between module back-surface and cell at
    /// 1000 W/m², °C.
    pub delta_t_celsius: f64,
}

impl SandiaMountingCoefficients {
    /// Glass/cell/glass modules on an open rack (free-standing, ground- or
    /// roof-mounted with an open air gap). The single most commonly cited
    /// default coefficient set for open-rack crystalline-silicon PV.
    ///
    /// **Source**: King et al. (2004), SAND2004-3535, Table 1.
    pub const OPEN_RACK_GLASS_GLASS: Self = Self {
        a: -3.56,
        b: -0.075,
        delta_t_celsius: 3.0,
    };

    /// Glass/cell/polymer-sheet modules in an insulated (close) roof mount
    /// (typical residential rooftop installation with little rear
    /// ventilation).
    ///
    /// **Source**: King et al. (2004), SAND2004-3535, Table 1.
    pub const INSULATED_ROOF_GLASS_POLYMER: Self = Self {
        a: -2.81,
        b: -0.0455,
        delta_t_celsius: 0.0,
    };
}

/// Estimate module back-surface temperature from plane-of-array irradiance,
/// wind speed and ambient temperature.
///
/// `T_module = G_POA * exp(a + b * WS) + T_ambient`
pub fn module_temperature(
    poa_global: Irradiance,
    wind_speed: Velocity,
    ambient: Temperature,
    coefficients: SandiaMountingCoefficients,
) -> Temperature {
    let exponent = coefficients.a + coefficients.b * wind_speed.meters_per_second();
    let temperature_rise = poa_global.watts_per_m2() * exponent.exp();
    Temperature::from_celsius(ambient.celsius() + temperature_rise)
}

/// Estimate cell temperature from module back-surface temperature and
/// plane-of-array irradiance.
///
/// `T_cell = T_module + (G_POA / 1000) * delta_T`
pub fn cell_temperature(
    module_temp: Temperature,
    poa_global: Irradiance,
    coefficients: SandiaMountingCoefficients,
) -> Temperature {
    let offset = (poa_global.watts_per_m2() / 1000.0) * coefficients.delta_t_celsius;
    Temperature::from_celsius(module_temp.celsius() + offset)
}

/// Convenience function computing cell temperature directly from ambient
/// conditions in a single call (composes [`module_temperature`] and
/// [`cell_temperature`]).
pub fn cell_temperature_from_ambient(
    poa_global: Irradiance,
    wind_speed: Velocity,
    ambient: Temperature,
    coefficients: SandiaMountingCoefficients,
) -> Temperature {
    let t_module = module_temperature(poa_global, wind_speed, ambient, coefficients);
    cell_temperature(t_module, poa_global, coefficients)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_irradiance_module_temperature_equals_ambient() {
        let t = module_temperature(
            Irradiance::from_watts_per_m2(0.0),
            Velocity::from_meters_per_second(1.0),
            Temperature::from_celsius(20.0),
            SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS,
        );
        assert!((t.celsius() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn full_sun_open_rack_temperature_rise_is_physically_reasonable() {
        // Open-rack crystalline modules at 1000 W/m2 and light wind (1 m/s)
        // typically run ~25-30 degC above ambient (widely cited rule of
        // thumb, consistent with the NOCT-model test in `crate::noct`).
        let t = module_temperature(
            Irradiance::from_watts_per_m2(1000.0),
            Velocity::from_meters_per_second(1.0),
            Temperature::from_celsius(25.0),
            SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS,
        );
        let rise = t.celsius() - 25.0;
        assert!((15.0..35.0).contains(&rise), "rise = {rise} degC");
    }

    #[test]
    fn higher_wind_speed_reduces_module_temperature() {
        let poa = Irradiance::from_watts_per_m2(900.0);
        let ambient = Temperature::from_celsius(25.0);
        let coeffs = SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS;
        let calm = module_temperature(poa, Velocity::from_meters_per_second(0.5), ambient, coeffs);
        let windy = module_temperature(poa, Velocity::from_meters_per_second(8.0), ambient, coeffs);
        assert!(windy.celsius() < calm.celsius());
    }

    #[test]
    fn cell_temperature_exceeds_or_equals_module_temperature() {
        // The cell (inside the laminate) runs at or above the back-surface
        // module temperature under illumination, since delta_T >= 0 for
        // both provided mounting configurations.
        let poa = Irradiance::from_watts_per_m2(1000.0);
        let module_temp = Temperature::from_celsius(50.0);
        let cell_temp = cell_temperature(module_temp, poa, SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS);
        assert!(cell_temp.celsius() >= module_temp.celsius());
    }

    #[test]
    fn insulated_roof_mount_runs_hotter_than_open_rack_at_same_conditions() {
        // Physical expectation: an insulated close-roof mount has worse
        // convective cooling than an open rack, so it should reach a
        // higher module temperature under the same irradiance and wind.
        let poa = Irradiance::from_watts_per_m2(900.0);
        let wind = Velocity::from_meters_per_second(2.0);
        let ambient = Temperature::from_celsius(25.0);
        let open_rack = module_temperature(poa, wind, ambient, SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS);
        let insulated_roof = module_temperature(
            poa,
            wind,
            ambient,
            SandiaMountingCoefficients::INSULATED_ROOF_GLASS_POLYMER,
        );
        assert!(insulated_roof.celsius() > open_rack.celsius());
    }

    #[test]
    fn cell_temperature_from_ambient_matches_manual_composition() {
        let poa = Irradiance::from_watts_per_m2(800.0);
        let wind = Velocity::from_meters_per_second(3.0);
        let ambient = Temperature::from_celsius(22.0);
        let coeffs = SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS;

        let composed = cell_temperature_from_ambient(poa, wind, ambient, coeffs);
        let manual_module = module_temperature(poa, wind, ambient, coeffs);
        let manual_cell = cell_temperature(manual_module, poa, coeffs);

        assert!((composed.celsius() - manual_cell.celsius()).abs() < 1e-12);
    }
}
