//! Piecewise-linear interpolation over tabulated data.

/// Linearly interpolate `y` at `x` given sorted (ascending) breakpoints
/// `xs` and corresponding values `ys`.
///
/// Behaviour at the boundaries is **clamped extrapolation**: if `x` lies
/// before `xs[0]` or after `xs[xs.len()-1]`, the nearest endpoint value is
/// returned rather than extrapolating the line, which avoids unbounded
/// output for lookup tables such as inverter efficiency curves or pump
/// curves where extrapolation is physically meaningless.
///
/// # Panics
///
/// Panics (via `debug_assert`) in debug builds if `xs` and `ys` differ in
/// length, if either has fewer than 2 points, if `xs` is not sorted in
/// non-decreasing order, or if `x` is NaN (a NaN `x` compares false against
/// both boundary checks below and would otherwise reach the binary search
/// with an undefined ordering, which is the one case release builds cannot
/// simply "skip" safely — see the `unwrap()` justification at the search
/// call site). Release builds skip the `debug_assert!` checks for
/// performance; callers of a public API embedding this function are
/// expected to validate their static tables once at construction time and
/// to never pass a NaN query point (which has no meaningful interpolated
/// value in any case).
pub fn linear_interpolate(x: f64, xs: &[f64], ys: &[f64]) -> f64 {
    debug_assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");
    debug_assert!(xs.len() >= 2, "need at least two points to interpolate");
    debug_assert!(
        xs.windows(2).all(|w| w[0] <= w[1]),
        "xs must be sorted in non-decreasing order"
    );
    debug_assert!(!x.is_nan(), "x must not be NaN");

    if x <= xs[0] {
        return ys[0];
    }
    let last = xs.len() - 1;
    if x >= xs[last] {
        return ys[last];
    }

    // Binary search for the bracketing interval [xs[i], xs[i+1]]. The
    // `unwrap()` here is justified, not merely convenient: `x` is finite
    // and strictly between two finite, sorted `xs` bounds at this point
    // (both NaN and out-of-range `x` were already handled above, and `xs`
    // is documented as containing no NaN), so `partial_cmp` cannot return
    // `None` for any `probe` in `xs` -- this is a total order in practice,
    // `PartialOrd` only because `f64` as a whole is not.
    let i = match xs.binary_search_by(|probe| probe.partial_cmp(&x).unwrap()) {
        Ok(exact) => return ys[exact],
        Err(insert_at) => insert_at - 1,
    };

    let x0 = xs[i];
    let x1 = xs[i + 1];
    let y0 = ys[i];
    let y1 = ys[i + 1];
    let t = (x - x0) / (x1 - x0);
    y0 + t * (y1 - y0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_midpoint() {
        let xs = [0.0, 10.0];
        let ys = [0.0, 100.0];
        assert!((linear_interpolate(5.0, &xs, &ys) - 50.0).abs() < 1e-12);
    }

    #[test]
    fn matches_exact_breakpoint() {
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys = [0.0, 10.0, 20.0, 30.0];
        assert!((linear_interpolate(2.0, &xs, &ys) - 20.0).abs() < 1e-12);
    }

    #[test]
    fn clamps_below_range() {
        let xs = [1.0, 2.0, 3.0];
        let ys = [10.0, 20.0, 30.0];
        assert!((linear_interpolate(-5.0, &xs, &ys) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn clamps_above_range() {
        let xs = [1.0, 2.0, 3.0];
        let ys = [10.0, 20.0, 30.0];
        assert!((linear_interpolate(100.0, &xs, &ys) - 30.0).abs() < 1e-12);
    }

    #[test]
    fn interpolates_within_multi_segment_table() {
        let xs = [0.0, 5.0, 10.0, 20.0];
        let ys = [0.0, 50.0, 60.0, 100.0];
        // Between x=10 and x=20: linear from 60 to 100.
        assert!((linear_interpolate(15.0, &xs, &ys) - 80.0).abs() < 1e-12);
    }
}
