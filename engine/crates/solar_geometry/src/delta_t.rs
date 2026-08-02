//! Estimation of ΔT = TT − UT1 (the difference between Terrestrial Time and
//! Universal Time), needed to convert a UTC calendar timestamp into the
//! Julian Ephemeris Day used by `crate::meeus::compute` or `crate::spa::compute`.
//!
//! ΔT arises from the slow, irregular deceleration of Earth's rotation and
//! cannot be predicted exactly; it is measured after the fact by
//! international timing services (IERS) and only approximated for future
//! dates.
//!
//! **Practical significance for this engine**: Earth's mean rotation rate
//! is ~15 arcseconds of hour angle per second of time (15°/hour), so a ΔT
//! error of `n` seconds shifts the Sun's apparent position by
//! approximately `n * 0.0041667°` -- e.g. a 1 s error is ~0.004°, but a
//! 60 s error is already ~0.25°, not negligible. **Within** the
//! documented 2005-2050 validity window, this estimator's own deviation
//! from measured ΔT is at most a few seconds (a few times 0.004° ≈ well
//! under 0.01°, consistent with the ~0.01-0.02° accuracy floor of the
//! reduced Meeus series -- see the ADR in `lib.rs`), which is why a simple
//! polynomial approximation is sufficient for HeliosPro's purposes
//! *inside that window*. **Outside** 2005-2050 (e.g. year 1900 or 2100),
//! evaluating this same polynomial is an uncontrolled extrapolation whose
//! deviation from the true historical/future ΔT can reach several dozen
//! seconds -- a position error approaching a degree, not negligible; see
//! `docs/design/17-analyse-incertitude-numerique.md`, §17.4a, for the
//! exact quadratic-term analysis quantifying this. Callers needing
//! accuracy outside 2005-2050 should supply a known/reference `delta_t`
//! directly to `crate::spa::compute`/`crate::meeus::compute` (or, in the
//! `solar_position` crate, via its `spa::solar_position_precise` /
//! `meeus::solar_position_precise` entry points) rather than rely on this
//! estimator's extrapolation, exactly as `validation::global_campaign`
//! does for its 1900-2100 sweep.

/// Estimate ΔT (seconds) for a given UTC calendar year and month, using the
/// polynomial expression valid for the years 2005-2050.
///
/// **Source**: Espenak, F. & Meeus, J. (2006), *Five Millennium Canon of
/// Solar Eclipses: -1999 to +3000*, NASA/TP-2006-214141, "Polynomial
/// Expressions for Delta T", the 2005-2050 branch:
/// `ΔT = 62.92 + 0.32217*y + 0.005589*y²`, where `y = year + (month - 0.5)/12
/// - 2000`.
///
/// For years outside 2005-2050, the same polynomial is still evaluated
/// (rather than returning an error) because, per the module-level docs
/// above, the sensitivity of solar position to ΔT is negligible at this
/// engine's accuracy target; callers needing eclipse-grade precision for
/// historical dates should supply a measured ΔT directly instead of using
/// this estimator.
pub fn estimate_delta_t_seconds(year: i32, month: u32) -> f64 {
    let y = year as f64 + (month as f64 - 0.5) / 12.0 - 2000.0;
    62.92 + 0.322_17 * y + 0.005_589 * y * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_t_is_a_few_tens_of_seconds_in_the_2020s() {
        // Observed ΔT in the early 2020s is approximately 69-73 seconds
        // (IERS Bulletin A historical record, public knowledge). The
        // polynomial should land in a physically reasonable neighborhood.
        let dt = estimate_delta_t_seconds(2024, 6);
        assert!((60.0..90.0).contains(&dt), "delta_t = {dt}");
    }

    #[test]
    fn delta_t_increases_slowly_over_multi_decade_span() {
        let dt_2010 = estimate_delta_t_seconds(2010, 1);
        let dt_2040 = estimate_delta_t_seconds(2040, 1);
        assert!(dt_2040 > dt_2010);
    }
}
