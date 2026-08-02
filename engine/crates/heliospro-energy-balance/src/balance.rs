use heliospro_math::julian::JulianDayError;
use heliospro_solar::decomposition::erbs_decomposition;
use heliospro_solar::extraterrestrial::{
    extraterrestrial_horizontal_irradiance, extraterrestrial_normal_irradiance,
};
use heliospro_solar::incidence::angle_of_incidence;
use heliospro_solar::position::solar_position;
use heliospro_thermal::{noct, sandia};
use heliospro_transposition::{hay_davies, hdkr, isotropic, perez, TranspositionInputs};

use crate::inputs::{EnergyBalanceInput, EnergyBalanceResult, ThermalModel, TranspositionModel};

/// Compute the complete energy balance for one instant: solar position,
/// extraterrestrial irradiance, GHI/DNI/DHI reconciliation, angle of
/// incidence, plane-of-array irradiance, and cell temperature.
///
/// # Errors
/// Returns [`JulianDayError`] if `input.datetime_utc` is not a valid
/// Gregorian calendar date.
pub fn compute_energy_balance(
    input: EnergyBalanceInput,
) -> Result<EnergyBalanceResult, JulianDayError> {
    let solar_position = solar_position(input.datetime_utc, input.observer, input.atmosphere)?;

    let extraterrestrial_normal =
        extraterrestrial_normal_irradiance(solar_position.earth_sun_distance_au);
    let extraterrestrial_horizontal = extraterrestrial_horizontal_irradiance(
        extraterrestrial_normal,
        solar_position.zenith_true,
    );

    let (dni, dhi, decomposition_used) = match (input.weather.dni, input.weather.dhi) {
        (Some(dni), Some(dhi)) => (dni, dhi, false),
        _ => {
            let decomposed = erbs_decomposition(
                input.weather.ghi,
                extraterrestrial_horizontal,
                solar_position.zenith_true,
            );
            (decomposed.dni, decomposed.dhi, true)
        }
    };

    // Transposition models are validated in the literature against the
    // geometric (airless) zenith angle, not the refraction-corrected
    // apparent zenith, so the true zenith is used consistently throughout
    // the radiometric chain below.
    let angle_of_incidence = angle_of_incidence(
        solar_position.zenith_true,
        solar_position.azimuth,
        input.surface,
    );

    let transposition_inputs = TranspositionInputs {
        dni,
        dhi,
        ghi: input.weather.ghi,
        extraterrestrial_normal,
        solar_zenith: solar_position.zenith_true,
        angle_of_incidence,
        surface: input.surface,
        ground_albedo: input.ground_albedo,
    };

    let poa = match input.transposition_model {
        TranspositionModel::Isotropic => isotropic::plane_of_array_irradiance(&transposition_inputs),
        TranspositionModel::HayDavies => hay_davies::plane_of_array_irradiance(&transposition_inputs),
        TranspositionModel::Hdkr => hdkr::plane_of_array_irradiance(&transposition_inputs),
        TranspositionModel::Perez => perez::plane_of_array_irradiance(&transposition_inputs),
    };

    let cell_temperature = match input.thermal_model {
        ThermalModel::Noct { noct_celsius } => {
            noct::cell_temperature(input.weather.ambient_temperature, poa.global, noct_celsius)
        }
        ThermalModel::Sandia { coefficients } => sandia::cell_temperature_from_ambient(
            poa.global,
            input.weather.wind_speed,
            input.weather.ambient_temperature,
            coefficients,
        ),
    };

    Ok(EnergyBalanceResult {
        solar_position,
        extraterrestrial_normal,
        extraterrestrial_horizontal,
        ghi: input.weather.ghi,
        dni,
        dhi,
        decomposition_used,
        angle_of_incidence,
        poa,
        cell_temperature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use heliospro_math::julian::CalendarDateTime;
    use heliospro_solar::incidence::SurfaceOrientation;
    use heliospro_thermal::sandia::SandiaMountingCoefficients;
    use heliospro_units::{Pressure, Ratio};

    fn base_input(
        datetime_utc: CalendarDateTime,
        transposition_model: TranspositionModel,
    ) -> EnergyBalanceInput {
        EnergyBalanceInput {
            datetime_utc,
            observer: Observer {
                latitude: Angle::from_degrees(45.0),
                longitude: Angle::from_degrees(5.0),
            },
            atmosphere: AtmosphericConditions {
                pressure: Pressure::from_pascal(heliospro_constants::STANDARD_ATMOSPHERE_PA),
                temperature: Temperature::from_celsius(15.0),
            },
            surface: SurfaceOrientation {
                tilt: Angle::from_degrees(35.0),
                azimuth: Angle::from_degrees(180.0),
            },
            ground_albedo: Ratio::new(0.2),
            weather: WeatherInput {
                ghi: Irradiance::from_watts_per_m2(750.0),
                dni: None,
                dhi: None,
                ambient_temperature: Temperature::from_celsius(20.0),
                wind_speed: Velocity::from_meters_per_second(2.0),
            },
            transposition_model,
            thermal_model: ThermalModel::Sandia {
                coefficients: SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS,
            },
        }
    }

    use crate::inputs::WeatherInput;
    use heliospro_solar::position::{AtmosphericConditions, Observer};
    use heliospro_units::{Angle, Irradiance, Temperature, Velocity};

    #[test]
    fn daytime_energy_balance_produces_positive_poa_and_reasonable_cell_temperature() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 6,
            day: 21,
            hour: 11,
            minute: 0,
            second: 0.0,
        };
        let input = base_input(dt, TranspositionModel::Perez);
        let result = compute_energy_balance(input).unwrap();

        assert!(result.solar_position.elevation_true.degrees() > 0.0);
        assert!(result.poa.global.watts_per_m2() > 0.0);
        assert!(result.decomposition_used);
        // Cell temperature should exceed ambient under significant daytime
        // irradiance.
        assert!(result.cell_temperature.celsius() > 20.0);
    }

    #[test]
    fn nighttime_energy_balance_has_zero_irradiance_everywhere() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 1,
            day: 15,
            hour: 2,
            minute: 0,
            second: 0.0,
        };
        let mut input = base_input(dt, TranspositionModel::Perez);
        input.weather.ghi = Irradiance::ZERO;
        let result = compute_energy_balance(input).unwrap();

        assert!(result.solar_position.elevation_true.degrees() < 0.0);
        assert_eq!(result.poa.global.watts_per_m2(), 0.0);
        // At night, cell temperature should equal ambient (no solar
        // heating contribution in either thermal model at G=0).
        assert!((result.cell_temperature.celsius() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn horizontal_surface_poa_global_matches_ghi_across_all_transposition_models() {
        // Top-level regression coverage of the per-model tilt=0 invariant
        // proven and unit-tested in `heliospro-transposition`, exercised
        // here through the full orchestration pipeline with a
        // self-consistent (Erbs-decomposed) DNI/DHI pair.
        let dt = CalendarDateTime {
            year: 2024,
            month: 9,
            day: 22,
            hour: 10,
            minute: 0,
            second: 0.0,
        };
        for model in [
            TranspositionModel::Isotropic,
            TranspositionModel::HayDavies,
            TranspositionModel::Hdkr,
            TranspositionModel::Perez,
        ] {
            let mut input = base_input(dt, model);
            input.surface.tilt = Angle::from_degrees(0.0);
            let result = compute_energy_balance(input).unwrap();
            let relative_diff =
                (result.poa.global.watts_per_m2() - result.ghi.watts_per_m2()).abs()
                    / result.ghi.watts_per_m2();
            assert!(
                relative_diff < 0.005,
                "model={model:?}, poa={}, ghi={}",
                result.poa.global.watts_per_m2(),
                result.ghi.watts_per_m2()
            );
        }
    }

    #[test]
    fn measured_dni_dhi_are_used_directly_without_decomposition() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 6,
            day: 21,
            hour: 11,
            minute: 0,
            second: 0.0,
        };
        let mut input = base_input(dt, TranspositionModel::Isotropic);
        input.weather.dni = Some(Irradiance::from_watts_per_m2(700.0));
        input.weather.dhi = Some(Irradiance::from_watts_per_m2(120.0));
        let result = compute_energy_balance(input).unwrap();

        assert!(!result.decomposition_used);
        assert_eq!(result.dni.watts_per_m2(), 700.0);
        assert_eq!(result.dhi.watts_per_m2(), 120.0);
    }

    #[test]
    fn noct_and_sandia_thermal_models_agree_within_a_reasonable_margin() {
        let dt = CalendarDateTime {
            year: 2024,
            month: 7,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0.0,
        };
        let mut input_noct = base_input(dt, TranspositionModel::Perez);
        input_noct.thermal_model = ThermalModel::Noct { noct_celsius: 45.0 };
        let result_noct = compute_energy_balance(input_noct).unwrap();

        let mut input_sandia = base_input(dt, TranspositionModel::Perez);
        input_sandia.thermal_model = ThermalModel::Sandia {
            coefficients: SandiaMountingCoefficients::OPEN_RACK_GLASS_GLASS,
        };
        let result_sandia = compute_energy_balance(input_sandia).unwrap();

        let diff = (result_noct.cell_temperature.celsius() - result_sandia.cell_temperature.celsius()).abs();
        assert!(diff < 10.0, "NOCT vs Sandia cell temperature diff = {diff} degC");
    }
}
