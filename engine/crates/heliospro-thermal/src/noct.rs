//! NOCT (Nominal Operating Cell Temperature) cell temperature model.
//!
//! **Source**: Ross, R.G. (1976), "Interface design considerations for
//! terrestrial solar cell modules", *Conference Record, 12th IEEE
//! Photovoltaic Specialists Conference*, 801-806; as reproduced in Duffie &
//! Beckman (2013), *Solar Engineering of Thermal Processes*, 4th ed.,
//! Eq. 23.2.4.
//!
//! NOCT is the cell temperature (°C) a module reaches under a standard
//! reference condition defined by IEC 61215: plane-of-array irradiance
//! 800 W/m², ambient temperature 20°C, wind speed 1 m/s, open-circuit
//! operation. Manufacturers publish this single number on datasheets,
//! making this the simplest cell-temperature model to apply, at the cost
//! of not capturing wind-speed variability explicitly (see [`crate::sandia`]
//! for a model that does).

use heliospro_units::{Irradiance, Temperature};

/// Reference irradiance at which NOCT is defined, W/m² (IEC 61215).
const NOCT_REFERENCE_IRRADIANCE_W_M2: f64 = 800.0;
/// Reference ambient temperature at which NOCT is defined, °C (IEC 61215).
const NOCT_REFERENCE_AMBIENT_CELSIUS: f64 = 20.0;

/// Estimate cell temperature from ambient temperature and plane-of-array
/// irradiance, using the module's datasheet NOCT value.
///
/// `T_cell = T_ambient + (NOCT - 20) / 800 * G_POA`
pub fn cell_temperature(
    ambient: Temperature,
    poa_global: Irradiance,
    noct_celsius: f64,
) -> Temperature {
    let temperature_rise = (noct_celsius - NOCT_REFERENCE_AMBIENT_CELSIUS)
        / NOCT_REFERENCE_IRRADIANCE_W_M2
        * poa_global.watts_per_m2();
    Temperature::from_celsius(ambient.celsius() + temperature_rise)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defining_condition_recovers_noct_exactly() {
        // At exactly the IEC 61215 reference condition (G=800 W/m2,
        // Tamb=20 degC) the model must return the NOCT value itself, by
        // definition of what NOCT means.
        let noct = 45.0;
        let t_cell = cell_temperature(
            Temperature::from_celsius(20.0),
            Irradiance::from_watts_per_m2(800.0),
            noct,
        );
        assert!((t_cell.celsius() - noct).abs() < 1e-9, "T_cell = {}", t_cell.celsius());
    }

    #[test]
    fn zero_irradiance_gives_cell_temperature_equal_to_ambient() {
        let t_cell = cell_temperature(
            Temperature::from_celsius(15.0),
            Irradiance::from_watts_per_m2(0.0),
            45.0,
        );
        assert!((t_cell.celsius() - 15.0).abs() < 1e-9);
    }

    #[test]
    fn higher_irradiance_gives_higher_cell_temperature() {
        let ambient = Temperature::from_celsius(25.0);
        let low = cell_temperature(ambient, Irradiance::from_watts_per_m2(300.0), 45.0);
        let high = cell_temperature(ambient, Irradiance::from_watts_per_m2(1000.0), 45.0);
        assert!(high.celsius() > low.celsius());
    }

    #[test]
    fn full_sun_temperature_rise_is_in_a_physically_reasonable_range() {
        // At G=1000 W/m2 with a typical NOCT of 45 degC, the temperature
        // rise above ambient should be on the order of 25-35 degC (a very
        // widely cited rule of thumb for crystalline-silicon PV modules
        // under full sun in low wind).
        let ambient = Temperature::from_celsius(25.0);
        let t_cell = cell_temperature(ambient, Irradiance::from_watts_per_m2(1000.0), 45.0);
        let rise = t_cell.celsius() - ambient.celsius();
        assert!((20.0..35.0).contains(&rise), "rise = {rise} degC");
    }
}
