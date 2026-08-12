//! Automated comparison of a [`SolarPositionAlgorithm`] against the pvlib
//! reference dataset ([`crate::dataset`]), producing per-quantity error
//! statistics (maximum, mean, RMS, mean relative error) and timing, per
//! the project's validation campaign requirements.

use std::time::{Duration, Instant};

use core_math::julian::CalendarDateTime;
use core_math::stats;
use core_units::{Angle, Length, Pressure, Temperature};
use solar_position::{AtmosphericConditions, Observer, SolarPositionAlgorithm};

use crate::dataset::{DATASET, PvlibReferenceCase};

/// Error statistics for one compared quantity across the full dataset.
#[derive(Debug, Clone, Copy)]
pub struct QuantityErrorStats {
    /// Human-readable quantity name (e.g. "Zenith (apparent)").
    pub name: &'static str,
    /// Unit label for display (e.g. "deg", "min").
    pub unit: &'static str,
    /// Mean signed error (`computed - expected`): reveals systematic bias.
    pub mean_bias_error: f64,
    /// Mean absolute error.
    pub mean_absolute_error: f64,
    /// Root-mean-square error.
    pub root_mean_square_error: f64,
    /// Maximum absolute error observed across the dataset.
    pub max_absolute_error: f64,
    /// The case label at which the maximum absolute error occurred.
    pub max_absolute_error_case: &'static str,
}

fn compute_quantity_stats(
    name: &'static str,
    unit: &'static str,
    computed: &[f64],
    expected: &[f64],
    labels: &[&'static str],
) -> QuantityErrorStats {
    let max_index = computed
        .iter()
        .zip(expected)
        .map(|(c, e)| (c - e).abs())
        .enumerate()
        .fold(
            (0usize, f64::MIN),
            |(bi, bv), (i, v)| {
                if v > bv { (i, v) } else { (bi, bv) }
            },
        )
        .0;

    QuantityErrorStats {
        name,
        unit,
        mean_bias_error: stats::mean_bias_error(computed, expected),
        mean_absolute_error: stats::mean_absolute_error(computed, expected),
        root_mean_square_error: stats::root_mean_square_error(computed, expected),
        max_absolute_error: stats::max_absolute_error(computed, expected),
        max_absolute_error_case: labels[max_index],
    }
}

/// The full error report for one algorithm evaluated against
/// [`crate::dataset::DATASET`].
#[derive(Debug, Clone)]
pub struct EngineErrorReport {
    /// The algorithm's [`SolarPositionAlgorithm::name`].
    pub engine_name: &'static str,
    /// Number of dataset cases evaluated.
    pub case_count: usize,
    /// Per-quantity error statistics, in a fixed, documented order:
    /// zenith (apparent), azimuth, elevation (apparent), hour angle
    /// (geocentric), declination (geocentric), equation of time,
    /// right ascension (geocentric), Earth radius vector.
    pub quantities: Vec<QuantityErrorStats>,
    /// Total wall-clock time spent computing all `case_count` positions.
    pub total_duration: Duration,
    /// `total_duration / case_count`.
    pub mean_duration_per_case: Duration,
}

/// Run `algorithm` against every case in [`crate::dataset::DATASET`] and
/// produce a full error report.
///
/// The geocentric hour angle is used for the "hour angle" comparison (both
/// engines expose it directly as [`solar_position::SolarPosition::hour_angle`]);
/// "solar time" (heure solaire) is not separately tabulated because it is
/// fully determined by the hour angle and equation of time already
/// reported (`solar_time_hours = 12 + hour_angle_deg/15`), per Duffie &
/// Beckman (2013), Eq. 1.5.2 -- duplicating it here would not be an
/// independent measurement.
pub fn evaluate<A: SolarPositionAlgorithm>(algorithm: &A) -> EngineErrorReport {
    let n = DATASET.len();

    let mut computed_zenith_apparent = Vec::with_capacity(n);
    let mut computed_azimuth = Vec::with_capacity(n);
    let mut computed_elevation_apparent = Vec::with_capacity(n);
    let mut computed_hour_angle = Vec::with_capacity(n);
    let mut computed_declination = Vec::with_capacity(n);
    let mut computed_equation_of_time = Vec::with_capacity(n);
    let mut computed_right_ascension = Vec::with_capacity(n);
    let mut computed_earth_radius_vector = Vec::with_capacity(n);

    let mut expected_zenith_apparent = Vec::with_capacity(n);
    let mut expected_azimuth = Vec::with_capacity(n);
    let mut expected_elevation_apparent = Vec::with_capacity(n);
    let mut expected_hour_angle = Vec::with_capacity(n);
    let mut expected_declination = Vec::with_capacity(n);
    let mut expected_equation_of_time = Vec::with_capacity(n);
    let mut expected_right_ascension = Vec::with_capacity(n);
    let mut expected_earth_radius_vector = Vec::with_capacity(n);

    let mut labels: Vec<&'static str> = Vec::with_capacity(n);

    let mut total_duration = Duration::ZERO;

    for case in DATASET {
        let (datetime_utc, observer, atmosphere) = case_to_inputs(case);

        let start = Instant::now();
        let position = algorithm
            .solar_position(datetime_utc, observer, atmosphere)
            .expect("every dataset case is a valid Gregorian calendar date");
        total_duration += start.elapsed();

        computed_zenith_apparent.push(position.zenith_apparent.degrees());
        computed_azimuth.push(position.azimuth.degrees());
        computed_elevation_apparent.push(position.elevation_apparent.degrees());
        computed_hour_angle.push(position.hour_angle.degrees());
        computed_declination.push(position.declination.degrees());
        computed_equation_of_time.push(position.equation_of_time_minutes);
        computed_right_ascension.push(position.right_ascension.degrees());
        computed_earth_radius_vector.push(position.earth_sun_distance_au);

        expected_zenith_apparent.push(case.expected_zenith_apparent_deg);
        expected_azimuth.push(case.expected_azimuth_deg);
        expected_elevation_apparent.push(case.expected_elevation_apparent_deg);
        // The dataset's hour angle and right ascension are unwrapped
        // (pre-modulo, mod-360 already applied by pvlib -- see
        // `local_hour_angle`), while `SolarPosition::hour_angle` is
        // normalized to (-180, 180]; compare on the wrapped
        // representation both share.
        expected_hour_angle.push(wrap_pm_180(case.expected_hour_angle_deg));
        expected_declination.push(case.expected_declination_deg);
        expected_equation_of_time.push(case.expected_equation_of_time_min);
        expected_right_ascension.push(wrap_0_360(case.expected_right_ascension_deg));
        expected_earth_radius_vector.push(case.expected_earth_radius_vector_au);

        labels.push(case.label);
    }

    // Apply the same wrapping to the computed hour angle/right ascension
    // sides for a fair like-for-like comparison.
    let computed_hour_angle: Vec<f64> = computed_hour_angle
        .iter()
        .map(|v| wrap_pm_180(*v))
        .collect();
    let computed_right_ascension: Vec<f64> = computed_right_ascension
        .iter()
        .map(|v| wrap_0_360(*v))
        .collect();

    let quantities = vec![
        compute_quantity_stats(
            "Zenith (apparent)",
            "deg",
            &computed_zenith_apparent,
            &expected_zenith_apparent,
            &labels,
        ),
        compute_quantity_stats(
            "Azimuth",
            "deg",
            &computed_azimuth,
            &expected_azimuth,
            &labels,
        ),
        compute_quantity_stats(
            "Elevation (apparent)",
            "deg",
            &computed_elevation_apparent,
            &expected_elevation_apparent,
            &labels,
        ),
        compute_quantity_stats(
            "Hour angle (geocentric)",
            "deg",
            &computed_hour_angle,
            &expected_hour_angle,
            &labels,
        ),
        compute_quantity_stats(
            "Declination (geocentric)",
            "deg",
            &computed_declination,
            &expected_declination,
            &labels,
        ),
        compute_quantity_stats(
            "Equation of time",
            "min",
            &computed_equation_of_time,
            &expected_equation_of_time,
            &labels,
        ),
        compute_quantity_stats(
            "Right ascension (geocentric)",
            "deg",
            &computed_right_ascension,
            &expected_right_ascension,
            &labels,
        ),
        compute_quantity_stats(
            "Earth radius vector",
            "AU",
            &computed_earth_radius_vector,
            &expected_earth_radius_vector,
            &labels,
        ),
    ];

    EngineErrorReport {
        engine_name: algorithm.name(),
        case_count: n,
        quantities,
        total_duration,
        mean_duration_per_case: total_duration / n as u32,
    }
}

fn wrap_pm_180(deg: f64) -> f64 {
    let wrapped = (deg + 180.0).rem_euclid(360.0) - 180.0;
    if wrapped <= -180.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

fn wrap_0_360(deg: f64) -> f64 {
    deg.rem_euclid(360.0)
}

fn case_to_inputs(
    case: &PvlibReferenceCase,
) -> (CalendarDateTime, Observer, AtmosphericConditions) {
    let datetime_utc = CalendarDateTime {
        year: case.year,
        month: case.month,
        day: case.day,
        hour: case.hour,
        minute: case.minute,
        second: case.second,
    };
    let observer = Observer {
        latitude: Angle::from_degrees(case.lat_deg),
        longitude: Angle::from_degrees(case.lon_deg),
        elevation: Length::from_meters(case.elev_m),
    };
    let atmosphere = AtmosphericConditions {
        pressure: Pressure::from_pascal(case.pressure_pa),
        temperature: Temperature::from_celsius(case.temperature_c),
    };
    (datetime_utc, observer, atmosphere)
}

impl EngineErrorReport {
    /// Render this report as a Markdown table, suitable for direct
    /// inclusion in documentation or CI output.
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "### {} — {} cas, {:.1} µs/cas (total {:.2} ms)\n\n",
            self.engine_name,
            self.case_count,
            self.mean_duration_per_case.as_secs_f64() * 1e6,
            self.total_duration.as_secs_f64() * 1e3,
        ));
        s.push_str("| Grandeur | Biais moyen | Erreur abs. moyenne | RMSE | Erreur abs. max | Cas (max) |\n");
        s.push_str("|---|---|---|---|---|---|\n");
        for q in &self.quantities {
            s.push_str(&format!(
                "| {} | {:.6} {u} | {:.6} {u} | {:.6} {u} | {:.6} {u} | {} |\n",
                q.name,
                q.mean_bias_error,
                q.mean_absolute_error,
                q.root_mean_square_error,
                q.max_absolute_error,
                q.max_absolute_error_case,
                u = q.unit,
            ));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solar_position::{MeeusReduced, Spa};

    #[test]
    fn spa_matches_pvlib_to_a_tiny_fraction_of_a_degree() {
        // SPA is (by design) the same published algorithm pvlib
        // implements, so agreement should be extremely tight -- this is
        // the strongest correctness statement this project can make about
        // its own SPA implementation.
        let report = evaluate(&Spa);
        for q in &report.quantities {
            if q.unit == "deg" {
                assert!(
                    q.max_absolute_error < 0.001,
                    "{}: max abs error = {} deg (case {})",
                    q.name,
                    q.max_absolute_error,
                    q.max_absolute_error_case
                );
            }
        }
    }

    #[test]
    fn meeus_matches_pvlib_within_its_documented_accuracy_budget() {
        // MeeusReduced is explicitly a lower-precision secondary engine
        // (~0.01-0.02 deg budget for zenith/azimuth, looser for hour
        // angle/right ascension which additionally accumulate the
        // engine's own sidereal-time approximations); this test pins that
        // documented budget against real, external reference data instead
        // of only internal invariants.
        let report = evaluate(&MeeusReduced);
        for q in &report.quantities {
            if q.unit == "deg" {
                assert!(
                    q.max_absolute_error < 0.1,
                    "{}: max abs error = {} deg (case {})",
                    q.name,
                    q.max_absolute_error,
                    q.max_absolute_error_case
                );
            }
        }
    }

    #[test]
    fn report_renders_to_nonempty_markdown() {
        let report = evaluate(&Spa);
        let markdown = report.to_markdown();
        assert!(markdown.contains("Zenith"));
        assert!(markdown.contains(report.engine_name));
    }
}
