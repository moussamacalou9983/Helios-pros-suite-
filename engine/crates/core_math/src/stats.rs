//! Error statistics used to validate engine outputs against reference
//! (measured or published) data series, per the engine's validation
//! strategy (`docs/design/14-strategie-tests.md`, §14.4).

/// Arithmetic mean of a slice. Returns `f64::NAN` for an empty slice.
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Sample standard deviation of a slice (Bessel's correction, divisor
/// `n - 1`): the usual choice when `values` is itself a sample used to
/// estimate the variability of a larger population of possible
/// measurements (e.g. per-call timing samples from a benchmark run), as
/// opposed to the population standard deviation (divisor `n`) used when
/// `values` *is* the entire population of interest. Returns `f64::NAN` for
/// a slice of fewer than 2 values (sample standard deviation is undefined
/// for `n < 2`: dividing by `n - 1 = 0`).
pub fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return f64::NAN;
    }
    let m = mean(values);
    let sum_sq_dev: f64 = values.iter().map(|v| (v - m).powi(2)).sum();
    (sum_sq_dev / (values.len() - 1) as f64).sqrt()
}

/// Mean bias error: the mean of `(computed - reference)`, preserving sign.
/// A systematic model bias shows up as a non-zero value here even when the
/// scatter (RMSE) is small.
///
/// # Panics
/// Panics if `computed` and `reference` differ in length.
pub fn mean_bias_error(computed: &[f64], reference: &[f64]) -> f64 {
    assert_eq!(computed.len(), reference.len());
    let diffs: Vec<f64> = computed.iter().zip(reference).map(|(c, r)| c - r).collect();
    mean(&diffs)
}

/// Mean absolute error between `computed` and `reference`.
///
/// # Panics
/// Panics if `computed` and `reference` differ in length.
pub fn mean_absolute_error(computed: &[f64], reference: &[f64]) -> f64 {
    assert_eq!(computed.len(), reference.len());
    let diffs: Vec<f64> = computed
        .iter()
        .zip(reference)
        .map(|(c, r)| (c - r).abs())
        .collect();
    mean(&diffs)
}

/// Root-mean-square error between `computed` and `reference`.
///
/// # Panics
/// Panics if `computed` and `reference` differ in length.
pub fn root_mean_square_error(computed: &[f64], reference: &[f64]) -> f64 {
    assert_eq!(computed.len(), reference.len());
    let sq_diffs: Vec<f64> = computed
        .iter()
        .zip(reference)
        .map(|(c, r)| (c - r).powi(2))
        .collect();
    mean(&sq_diffs).sqrt()
}

/// Maximum absolute error between `computed` and `reference`.
///
/// # Panics
/// Panics if `computed` and `reference` differ in length, or either is
/// empty.
pub fn max_absolute_error(computed: &[f64], reference: &[f64]) -> f64 {
    assert_eq!(computed.len(), reference.len());
    computed
        .iter()
        .zip(reference)
        .map(|(c, r)| (c - r).abs())
        .fold(f64::MIN, f64::max)
}

/// Signed relative error `(value - reference) / reference`, expressed as a
/// dimensionless fraction (multiply by 100 for a percentage). Returns
/// `f64::NAN` if `reference` is zero.
pub fn relative_error(value: f64, reference: f64) -> f64 {
    if reference == 0.0 {
        return f64::NAN;
    }
    (value - reference) / reference
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_simple_series() {
        assert!((mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn standard_deviation_of_a_known_series() {
        // [2, 4, 4, 4, 5, 5, 7, 9]: textbook example, sample std dev = 2.13809...
        // (Wikipedia "Standard deviation", worked example).
        let values = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = standard_deviation(&values);
        assert!((sd - 2.138_089_935_299_395).abs() < 1e-12, "sd = {sd}");
    }

    #[test]
    fn standard_deviation_of_identical_values_is_zero() {
        assert!((standard_deviation(&[5.0, 5.0, 5.0]) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn standard_deviation_is_nan_below_two_samples() {
        assert!(standard_deviation(&[]).is_nan());
        assert!(standard_deviation(&[1.0]).is_nan());
    }

    #[test]
    fn mean_bias_error_detects_systematic_offset() {
        let computed = [11.0, 21.0, 31.0];
        let reference = [10.0, 20.0, 30.0];
        assert!((mean_bias_error(&computed, &reference) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn mean_absolute_error_ignores_sign() {
        let computed = [9.0, 22.0];
        let reference = [10.0, 20.0];
        // |9-10| = 1, |22-20| = 2 -> mean = 1.5
        assert!((mean_absolute_error(&computed, &reference) - 1.5).abs() < 1e-12);
    }

    #[test]
    fn rmse_penalizes_large_errors_more_than_mae() {
        let computed = [0.0, 10.0];
        let reference = [0.0, 0.0];
        let rmse = root_mean_square_error(&computed, &reference);
        let mae = mean_absolute_error(&computed, &reference);
        assert!(rmse >= mae);
    }

    #[test]
    fn max_absolute_error_finds_worst_case() {
        let computed = [1.0, 5.0, 3.0];
        let reference = [1.0, 1.0, 3.0];
        assert!((max_absolute_error(&computed, &reference) - 4.0).abs() < 1e-12);
    }

    #[test]
    fn relative_error_percentage() {
        let err = relative_error(103.0, 100.0);
        assert!((err - 0.03).abs() < 1e-12);
    }
}
