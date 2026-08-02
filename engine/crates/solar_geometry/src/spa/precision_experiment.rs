//! Empirical `f32` vs `f64` precision experiment, supporting the
//! uncertainty-analysis document
//! (`docs/design/17-analyse-incertitude-numerique.md`, §3 "Simple vs
//! double précision").
//!
//! This is deliberately **not** a claim derived from theory alone: it
//! recomputes the single largest, most cancellation-prone step in the SPA
//! pipeline -- the 64-term `L0` heliocentric-longitude periodic series
//! (Reda & Andreas (2004), Table A4.2, terms up to `A ≈ 1.75e8`) -- once in
//! `f64` (as the production engine does) and once with every operand
//! narrowed to `f32`, for the exact Julian Ephemeris Millennium of the
//! paper's own canonical worked example, and measures the actual resulting
//! divergence. Kept as a permanent, reproducible test (not a throwaway
//! script) so the number cited in the uncertainty document is re-verified,
//! not just asserted, every time the test suite runs.

#[cfg(test)]
mod tests {
    use super::super::tables;
    use core_math::julian::CalendarDateTime;

    fn sum_l0_f64(jme: f64) -> f64 {
        tables::L0_TERMS
            .iter()
            .map(|t| t.a * (t.b + t.c * jme).cos())
            .sum()
    }

    fn sum_l0_f32(jme: f32) -> f32 {
        tables::L0_TERMS
            .iter()
            .map(|t| (t.a as f32) * ((t.b as f32) + (t.c as f32) * jme).cos())
            .sum()
    }

    /// Reproduces the Reda & Andreas (2004) canonical worked example's
    /// Julian Ephemeris Millennium (see
    /// `super::super::tests::reda_andreas_2004_worked_example_geocentric`
    /// for the same input reused here) and sums the 64-term `L0` series in
    /// both precisions.
    #[test]
    fn f32_vs_f64_summation_of_the_largest_periodic_series() {
        let dt = CalendarDateTime {
            year: 2003,
            month: 10,
            day: 17,
            hour: 19,
            minute: 30,
            second: 30.0,
        };
        let jd_ut = dt.to_julian_day().unwrap();
        let delta_t = 67.0;
        let jde = jd_ut + delta_t / core_constants::SECONDS_PER_DAY;
        let jce = (jde - core_constants::J2000_EPOCH_JD) / core_constants::DAYS_PER_JULIAN_CENTURY;
        let jme = jce / 10.0;

        let l0_f64 = sum_l0_f64(jme);
        let l0_f32 = sum_l0_f32(jme as f32) as f64;

        // The production formula is `L0/1e8` radians -> degrees -> arcsec;
        // apply the same scaling here so the reported number is directly
        // an angular (arcsecond) error, not a raw series-sum difference.
        let diff_arcsec = ((l0_f64 - l0_f32) / 1.0e8).to_degrees() * 3600.0;

        println!(
            "L0 series sum: f64 = {l0_f64:.6}, f32 = {l0_f32:.6}, \
             difference = {diff_arcsec:.6} arcsec"
        );

        // f32 has ~7 significant decimal digits; the largest L0 term is
        // ~1.75e8, so naive f32 accumulation is expected to lose several
        // orders of magnitude of precision relative to f64's ~1e-16
        // relative epsilon -- but should still land in the arcsecond
        // range, not degrees, because the series terms are summed (not
        // subtracted) and none individually approaches f32's dynamic
        // range limit. This assertion pins that expectation as a
        // regression check: a result far outside this bracket would
        // indicate this experiment itself is broken, not that f32 is
        // suddenly fine for astronomical series summation.
        assert!(
            diff_arcsec.abs() > 1e-4,
            "f32 summation unexpectedly matched f64 to less than 1e-4 arcsec \
             ({diff_arcsec} arcsec) -- re-check the experiment, this series is \
             known to be precision-sensitive"
        );
        assert!(
            diff_arcsec.abs() < 10.0,
            "f32 summation diverged from f64 by {diff_arcsec} arcsec, more than \
             an order of magnitude beyond the expected single-precision floor \
             -- re-check the experiment"
        );
    }
}
