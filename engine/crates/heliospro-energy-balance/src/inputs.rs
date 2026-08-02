use heliospro_math::julian::CalendarDateTime;
use heliospro_solar::incidence::SurfaceOrientation;
use heliospro_solar::position::{AtmosphericConditions, Observer, SolarPosition};
use heliospro_thermal::sandia::SandiaMountingCoefficients;
use heliospro_transposition::PlaneOfArrayIrradiance;
use heliospro_units::{Angle, Irradiance, Ratio, Temperature, Velocity};

/// Measured (or partially measured) weather inputs for one instant.
#[derive(Debug, Clone, Copy)]
pub struct WeatherInput {
    /// Global Horizontal Irradiance (always required).
    pub ghi: Irradiance,
    /// Direct Normal Irradiance, if measured. When `None` (together with
    /// `dhi: None`), it is estimated from `ghi` via the Erbs et al. (1982)
    /// correlation (see [`heliospro_solar::decomposition`]).
    pub dni: Option<Irradiance>,
    /// Diffuse Horizontal Irradiance, if measured. See `dni` above.
    pub dhi: Option<Irradiance>,
    /// Ambient (dry-bulb) air temperature.
    pub ambient_temperature: Temperature,
    /// Wind speed at module height (only used by [`ThermalModel::Sandia`]).
    pub wind_speed: Velocity,
}

/// Which sky-diffuse transposition model to use (see
/// `heliospro_transposition` for the physical basis of each).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranspositionModel {
    /// Liu & Jordan (1963).
    Isotropic,
    /// Hay & Davies (1980).
    HayDavies,
    /// Reindl, Beckman & Duffie (1990).
    Hdkr,
    /// Perez et al. (1990).
    Perez,
}

/// Which cell-temperature model to use (see `heliospro_thermal`).
#[derive(Debug, Clone, Copy)]
pub enum ThermalModel {
    /// NOCT model; requires the module's datasheet NOCT value, °C.
    Noct {
        /// Datasheet Nominal Operating Cell Temperature, °C.
        noct_celsius: f64,
    },
    /// Sandia (SAPM) model; requires mounting-configuration coefficients.
    Sandia {
        /// Empirical mounting coefficients (see
        /// [`SandiaMountingCoefficients`]).
        coefficients: SandiaMountingCoefficients,
    },
}

/// All inputs required to compute one instant's complete energy balance.
#[derive(Debug, Clone, Copy)]
pub struct EnergyBalanceInput {
    /// UTC instant.
    pub datetime_utc: CalendarDateTime,
    /// Observer (site) location.
    pub observer: Observer,
    /// Local atmospheric conditions (refraction correction).
    pub atmosphere: AtmosphericConditions,
    /// Orientation of the receiving surface (module plane).
    pub surface: SurfaceOrientation,
    /// Ground albedo in front of the surface.
    pub ground_albedo: Ratio,
    /// Weather inputs.
    pub weather: WeatherInput,
    /// Sky-diffuse transposition model to use.
    pub transposition_model: TranspositionModel,
    /// Cell-temperature model to use.
    pub thermal_model: ThermalModel,
}

/// The complete result of one instant's energy balance computation.
#[derive(Debug, Clone, Copy)]
pub struct EnergyBalanceResult {
    /// Full topocentric solar position at this instant.
    pub solar_position: SolarPosition,
    /// Extraterrestrial irradiance normal to the Sun's rays.
    pub extraterrestrial_normal: Irradiance,
    /// Extraterrestrial irradiance on a horizontal plane.
    pub extraterrestrial_horizontal: Irradiance,
    /// Global Horizontal Irradiance used (as supplied).
    pub ghi: Irradiance,
    /// Direct Normal Irradiance used (measured or decomposed).
    pub dni: Irradiance,
    /// Diffuse Horizontal Irradiance used (measured or decomposed).
    pub dhi: Irradiance,
    /// Whether `dni`/`dhi` were estimated via Erbs decomposition (`true`)
    /// or taken directly from the supplied measurement (`false`).
    pub decomposition_used: bool,
    /// Angle of incidence between the Sun's rays and the surface normal.
    pub angle_of_incidence: Angle,
    /// Plane-of-array irradiance (beam, sky-diffuse, ground-reflected,
    /// global).
    pub poa: PlaneOfArrayIrradiance,
    /// Estimated PV cell temperature.
    pub cell_temperature: Temperature,
}
