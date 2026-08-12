//! The residual and its derivatives — equations (3), (4), (6) and (8) of
//! `docs/design/35-formalisation-mathematique-electrical-model.md`.
//!
//! These are the mathematical primitives of the single-diode model, and
//! nothing else. They perform no iteration, take no decision, and know
//! nothing of any solver.
//!
//! ## Numerical policy
//!
//! Two transformations, both specified in §7.3 of the formalisation, are
//! applied here and nowhere else:
//!
//! - the product `I_0·e^u` is evaluated as `exp(ln I_0 + u)`, which moves
//!   the overflow threshold by `|ln I_0| ≈ 23` units;
//! - the term `e^u − 1` is evaluated with `exp_m1` for small `u`, against
//!   the catastrophic cancellation that would otherwise occur near
//!   `V = 0`.
//!
//! Neither changes the mathematics. Both are refusals to lose precision
//! that the exact expression does not lose.

use core_units::{Current, Resistance, Voltage};

use super::parameters::SingleDiodeParameters;

/// The junction voltage `V_j = V + I·R_s` — equation (20).
///
/// The voltage across the junction proper, after the series ohmic drop.
/// Typed as a [`Voltage`] because it **is** a voltage, unlike `a`.
#[inline]
pub fn junction_voltage(voltage: Voltage, current: Current, series: Resistance) -> Voltage {
    Voltage::from_volts(voltage.volts() + current.amperes() * series.ohms())
}

/// The dimensionless exponent `u = (V + I·R_s)/a` — equation (3).
///
/// Dimensionless because `[a] = V`: this is the property on which the
/// homogeneity of the whole equation rests
/// (formalisation §1.4).
#[inline]
pub fn exponent(voltage: Voltage, current: Current, params: &SingleDiodeParameters) -> f64 {
    junction_voltage(voltage, current, params.series_resistance()).volts()
        / params.modified_ideality_factor().volts()
}

/// `I_0·(e^u − 1)`, the diode term, evaluated without avoidable loss.
///
/// Splits at `u = 1`, a value chosen only to separate two regimes and
/// affecting no result:
///
/// - below it, `exp_m1(u)` gives `e^u − 1` without the cancellation that
///   `exp(u) − 1.0` suffers as `u → 0` (near `V = 0`, where the diode term
///   is the small difference of two nearly equal numbers);
/// - above it, `exp(ln I_0 + u)` keeps the product representable far
///   beyond where `I_0 · exp(u)` would overflow, and the `−1` is
///   subtracted afterwards where it is harmless.
#[inline]
pub fn diode_term(u: f64, params: &SingleDiodeParameters) -> f64 {
    let i_0 = params.saturation_current().amperes();
    if u < 1.0 {
        i_0 * u.exp_m1()
    } else {
        (i_0.ln() + u).exp() - i_0
    }
}

/// `I_0·e^u`, evaluated in log space — used by the derivatives.
#[inline]
pub fn diode_exponential(u: f64, params: &SingleDiodeParameters) -> f64 {
    (params.saturation_current().amperes().ln() + u).exp()
}

/// The residual `f(I; V, θ)` — **equation (3)**.
///
/// ```text
/// f = I_ph − I_0[e^u − 1] − (V + I·R_s)/R_sh − I,   u = (V + I·R_s)/a
/// ```
///
/// Homogeneous in amperes (formalisation §1.4). Solving the single-diode
/// equation means annulling this.
///
/// Defined for every real `I`: the function is smooth on all of `ℝ`, with
/// no singularity (§2.1).
#[inline]
pub fn residual(current: Current, voltage: Voltage, params: &SingleDiodeParameters) -> f64 {
    let u = exponent(voltage, current, params);
    let v_j = junction_voltage(voltage, current, params.series_resistance()).volts();
    params.photocurrent().amperes()
        - diode_term(u, params)
        - v_j / params.shunt_resistance().ohms()
        - current.amperes()
}

/// `∂f/∂I` — **equation (4)**.
///
/// ```text
/// ∂f/∂I = −(I_0·R_s/a)·e^u − R_s/R_sh − 1
/// ```
///
/// **Dimensionless** — the derivative of a current with respect to a
/// current. This is what makes the error bound `|I − I*| ≤ |f(I)|` of
/// equation (14) dimensionally sound, and what stops it transposing to
/// the voltage form, where the derivative is a conductance (§5.4).
///
/// Under `𝓗`, this is always `≤ −1` (equation (5)): the three terms are
/// non-positive and the last is exactly `−1`.
#[inline]
pub fn residual_derivative(
    current: Current,
    voltage: Voltage,
    params: &SingleDiodeParameters,
) -> f64 {
    let u = exponent(voltage, current, params);
    let r_s = params.series_resistance().ohms();
    let a = params.modified_ideality_factor().volts();
    -(r_s / a) * diode_exponential(u, params) - r_s / params.shunt_resistance().ohms() - 1.0
}

/// `∂²f/∂I²` — **equation (6)**.
///
/// ```text
/// ∂²f/∂I² = −(I_0·R_s²/a²)·e^u
/// ```
///
/// Strictly negative for `R_s > 0`, and **exactly zero** for `R_s = 0`
/// (equation (7)). The concavity it expresses is what makes Newton's
/// iteration monotone from the first step; its vanishing at `R_s = 0` is
/// what makes the residual affine there, hence exactly solvable (§8.1).
#[inline]
pub fn residual_second_derivative(
    current: Current,
    voltage: Voltage,
    params: &SingleDiodeParameters,
) -> f64 {
    let u = exponent(voltage, current, params);
    let r_s = params.series_resistance().ohms();
    let a = params.modified_ideality_factor().volts();
    -(r_s * r_s / (a * a)) * diode_exponential(u, params)
}

/// `∂f/∂V` — **equation (8)**.
///
/// ```text
/// ∂f/∂V = −(I_0/a)·e^u − 1/R_sh
/// ```
///
/// A **conductance**, in Ω⁻¹ — not a dimensionless number. Strictly
/// negative under `𝓗`, which is what makes the residual at `I = 0`
/// decreasing in `V`, hence `f(0; V) ≥ 0` for every `V ≤ V_oc` — the
/// lower bound of the bracket (§3.4).
#[inline]
pub fn residual_voltage_derivative(
    current: Current,
    voltage: Voltage,
    params: &SingleDiodeParameters,
) -> f64 {
    let u = exponent(voltage, current, params);
    let a = params.modified_ideality_factor().volts();
    -diode_exponential(u, params) / a - 1.0 / params.shunt_resistance().ohms()
}

/// `dI/dV` by the implicit function theorem — **equation (10)**.
///
/// ```text
/// dI/dV = −(∂f/∂V) / (∂f/∂I)
/// ```
///
/// Legitimate because `∂f/∂I ≤ −1 ≠ 0` everywhere, which is exactly the
/// hypothesis the theorem needs; it also establishes that `I(V)` is
/// smooth. Strictly negative, and bounded because the denominator of the
/// expanded form is `≥ 1` (§3.5).
///
/// Returns `None` if either derivative fails to be finite, rather than
/// yielding a `NaN`.
#[inline]
pub fn current_voltage_slope(
    current: Current,
    voltage: Voltage,
    params: &SingleDiodeParameters,
) -> Option<f64> {
    let numerator = residual_voltage_derivative(current, voltage, params);
    let denominator = residual_derivative(current, voltage, params);
    let slope = -numerator / denominator;
    slope.is_finite().then_some(slope)
}

/// The explicit current at a given junction voltage — **equation (21)**.
///
/// ```text
/// φ(V_j) = I_ph + I_0 − I_0·e^{V_j/a} − V_j/R_sh
/// ```
///
/// **Explicit**: no iteration. Changing variable to the junction voltage
/// removes the implicitness entirely. Provided here because it is a
/// primitive of the model; the maximum-power-point search that exploits
/// it belongs to a later sub-phase and is **not** implemented.
#[inline]
pub fn current_at_junction_voltage(v_j: Voltage, params: &SingleDiodeParameters) -> f64 {
    let u = v_j.volts() / params.modified_ideality_factor().volts();
    params.photocurrent().amperes()
        - diode_term(u, params)
        - v_j.volts() / params.shunt_resistance().ohms()
}

/// `φ'(V_j)` — **equation (22)**. Strictly negative under `𝓗`.
#[inline]
pub fn junction_current_slope(v_j: Voltage, params: &SingleDiodeParameters) -> f64 {
    let u = v_j.volts() / params.modified_ideality_factor().volts();
    -diode_exponential(u, params) / params.modified_ideality_factor().volts()
        - 1.0 / params.shunt_resistance().ohms()
}

/// `φ''(V_j)` — **equation (23)**. Strictly negative under `𝓗`.
#[inline]
pub fn junction_current_curvature(v_j: Voltage, params: &SingleDiodeParameters) -> f64 {
    let a = params.modified_ideality_factor().volts();
    let u = v_j.volts() / a;
    -diode_exponential(u, params) / (a * a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::single_diode::parameters::ModifiedIdealityFactor;

    fn nominal() -> SingleDiodeParameters {
        SingleDiodeParameters::new(
            Current::from_amperes(10.4),
            Current::from_amperes(2.5e-10),
            Resistance::from_ohms(0.30),
            Resistance::from_ohms(800.0),
            ModifiedIdealityFactor::from_volts(1.85),
        )
        .unwrap()
    }

    fn with_series(r_s: f64) -> SingleDiodeParameters {
        SingleDiodeParameters::new(
            Current::from_amperes(10.4),
            Current::from_amperes(2.5e-10),
            Resistance::from_ohms(r_s),
            Resistance::from_ohms(800.0),
            ModifiedIdealityFactor::from_volts(1.85),
        )
        .unwrap()
    }

    #[test]
    fn the_exponent_is_dimensionless_and_matches_its_definition() {
        let p = nominal();
        let v = Voltage::from_volts(30.0);
        let i = Current::from_amperes(5.0);
        let expected = (30.0 + 5.0 * 0.30) / 1.85;
        assert!((exponent(v, i, &p) - expected).abs() < 1e-12);
    }

    #[test]
    fn the_junction_voltage_adds_the_ohmic_drop() {
        let v_j = junction_voltage(
            Voltage::from_volts(30.0),
            Current::from_amperes(5.0),
            Resistance::from_ohms(0.30),
        );
        assert!((v_j.volts() - 31.5).abs() < 1e-12);
    }

    #[test]
    fn the_diode_term_agrees_with_the_naive_form_where_both_are_safe() {
        // Independent reference: the textbook expression, valid in the
        // range where it neither cancels nor overflows.
        let p = nominal();
        let i_0 = 2.5e-10;
        for u in [0.5_f64, 1.0, 5.0, 15.0, 25.0] {
            let naive = i_0 * (u.exp() - 1.0);
            let ours = diode_term(u, &p);
            let rel = (ours - naive).abs() / naive.abs().max(1e-300);
            assert!(rel < 1e-12, "u = {u}, ours = {ours}, naive = {naive}");
        }
    }

    #[test]
    fn the_diode_term_keeps_precision_where_the_naive_form_cancels() {
        // As u -> 0, exp(u) - 1 loses all significance; exp_m1 does not.
        let p = nominal();
        let u = 1e-12_f64;
        let ours = diode_term(u, &p);
        // The exact value is I_0 * u to first order.
        let exact = 2.5e-10 * u;
        let rel = (ours - exact).abs() / exact;
        assert!(rel < 1e-9, "ours = {ours:e}, exact = {exact:e}");
    }

    #[test]
    fn the_residual_at_zero_current_is_positive_below_open_circuit() {
        // The lower bound of the bracket, equation (9).
        let p = nominal();
        for v in [0.0, 10.0, 30.0, 45.0] {
            let f = residual(Current::from_amperes(0.0), Voltage::from_volts(v), &p);
            assert!(f >= 0.0, "V = {v}, f = {f}");
        }
    }

    #[test]
    fn the_residual_at_the_upper_bracket_is_negative() {
        // The upper bound of the bracket, equation (9).
        let p = nominal();
        let i_high = Current::from_amperes(10.4 + 2.5e-10);
        for v in [0.0, 10.0, 30.0, 45.0] {
            let f = residual(i_high, Voltage::from_volts(v), &p);
            assert!(f < 0.0, "V = {v}, f = {f}");
        }
    }

    #[test]
    fn the_first_derivative_is_at_most_minus_one() {
        // Equation (5), the property everything else rests on.
        let p = nominal();
        for v in [0.0, 15.0, 30.0, 45.0] {
            for i in [0.0, 2.6, 5.2, 7.8, 10.4] {
                let d = residual_derivative(Current::from_amperes(i), Voltage::from_volts(v), &p);
                assert!(d <= -1.0, "V = {v}, I = {i}, f' = {d}");
            }
        }
    }

    #[test]
    fn the_second_derivative_is_negative_and_vanishes_at_zero_series_resistance() {
        // Equation (7), both branches.
        let p = nominal();
        let d2 =
            residual_second_derivative(Current::from_amperes(5.0), Voltage::from_volts(30.0), &p);
        assert!(d2 < 0.0, "f'' = {d2}");

        let flat = with_series(0.0);
        let d2 = residual_second_derivative(
            Current::from_amperes(5.0),
            Voltage::from_volts(30.0),
            &flat,
        );
        assert_eq!(d2, 0.0, "R_s = 0 must give exactly zero curvature");
    }

    #[test]
    fn the_voltage_derivative_is_a_negative_conductance() {
        // Equation (8): negative, and in inverse ohms.
        let p = nominal();
        let d =
            residual_voltage_derivative(Current::from_amperes(5.0), Voltage::from_volts(30.0), &p);
        assert!(d < 0.0, "df/dV = {d}");
        // At least as large in magnitude as the shunt conductance alone.
        assert!(d <= -1.0 / 800.0);
    }

    #[test]
    fn the_derivatives_agree_with_independent_finite_differences() {
        // Independent reference: central differences on the residual
        // itself, which never uses the analytic derivative expressions.
        let p = nominal();
        let v = Voltage::from_volts(30.0);
        let i0 = 5.0;
        let h = 1e-6;

        let f_plus = residual(Current::from_amperes(i0 + h), v, &p);
        let f_minus = residual(Current::from_amperes(i0 - h), v, &p);
        let numeric = (f_plus - f_minus) / (2.0 * h);
        let analytic = residual_derivative(Current::from_amperes(i0), v, &p);
        // The tolerance is that of the *finite differences*, not of the
        // analytic expression: a central difference carries an error of
        // order h^2 + eps/h, about 1e-6 relative at this step size. A
        // tighter threshold would be testing round-off, not agreement.
        assert!(
            (analytic - numeric).abs() / numeric.abs() < 1e-5,
            "analytic = {analytic}, numeric = {numeric}"
        );

        // A second difference needs a much larger step: its numerator is
        // of order f''*h^2, which at h = 1e-6 falls below the round-off of
        // f itself and evaluates to exactly zero. The optimal step for a
        // second derivative is around eps^(1/4); 1e-3 leaves five orders
        // of magnitude of signal above the noise.
        let h2 = 1e-3;
        let f_c = residual(Current::from_amperes(i0), v, &p);
        let numeric2 = (residual(Current::from_amperes(i0 + h2), v, &p) - 2.0 * f_c
            + residual(Current::from_amperes(i0 - h2), v, &p))
            / (h2 * h2);
        let analytic2 = residual_second_derivative(Current::from_amperes(i0), v, &p);
        assert!(
            (analytic2 - numeric2).abs() / analytic2.abs() < 1e-3,
            "analytic'' = {analytic2}, numeric'' = {numeric2}"
        );
    }

    #[test]
    fn the_voltage_derivative_agrees_with_finite_differences() {
        let p = nominal();
        let i = Current::from_amperes(5.0);
        let v0 = 30.0;
        let h = 1e-6;
        let numeric = (residual(i, Voltage::from_volts(v0 + h), &p)
            - residual(i, Voltage::from_volts(v0 - h), &p))
            / (2.0 * h);
        let analytic = residual_voltage_derivative(i, Voltage::from_volts(v0), &p);
        // Again the tolerance is the finite differences' own error.
        assert!(
            (analytic - numeric).abs() / numeric.abs() < 1e-5,
            "analytic = {analytic}, numeric = {numeric}"
        );
    }

    #[test]
    fn the_current_voltage_slope_is_negative_and_bounded() {
        // Equation (10): strictly negative, and never explosive.
        let p = nominal();
        let slope =
            current_voltage_slope(Current::from_amperes(5.0), Voltage::from_volts(30.0), &p)
                .expect("both derivatives are finite here");
        assert!(slope < 0.0, "dI/dV = {slope}");
        assert!(slope.abs() < 1e6);
    }

    #[test]
    fn the_junction_form_reproduces_the_residual_exactly() {
        // phi(V_j) is the explicit form of the same equation: at the
        // solution, phi(V + I R_s) must equal I.
        let p = nominal();
        let v = Voltage::from_volts(30.0);
        let i = 9.9_f64;
        let v_j = junction_voltage(v, Current::from_amperes(i), p.series_resistance());
        let phi = current_at_junction_voltage(v_j, &p);
        let f = residual(Current::from_amperes(i), v, &p);
        // f = phi(V_j) - I by construction.
        assert!(
            (f - (phi - i)).abs() < 1e-12,
            "f = {f}, phi - I = {}",
            phi - i
        );
    }

    #[test]
    fn the_junction_derivatives_are_negative() {
        // Equations (22) and (23).
        let p = nominal();
        let v_j = Voltage::from_volts(30.0);
        assert!(junction_current_slope(v_j, &p) < 0.0);
        assert!(junction_current_curvature(v_j, &p) < 0.0);
    }

    #[test]
    fn the_junction_derivatives_agree_with_finite_differences() {
        let p = nominal();
        let v0 = 30.0;
        let h = 1e-6;
        let numeric = (current_at_junction_voltage(Voltage::from_volts(v0 + h), &p)
            - current_at_junction_voltage(Voltage::from_volts(v0 - h), &p))
            / (2.0 * h);
        let analytic = junction_current_slope(Voltage::from_volts(v0), &p);
        assert!((analytic - numeric).abs() / numeric.abs() < 1e-5);
    }

    #[test]
    fn every_primitive_stays_finite_across_the_operating_quadrant() {
        let p = nominal();
        for k in 0..=50 {
            let v = Voltage::from_volts(45.2 * f64::from(k) / 50.0);
            for j in 0..=20 {
                let i = Current::from_amperes(10.4 * f64::from(j) / 20.0);
                assert!(residual(i, v, &p).is_finite());
                assert!(residual_derivative(i, v, &p).is_finite());
                assert!(residual_second_derivative(i, v, &p).is_finite());
                assert!(residual_voltage_derivative(i, v, &p).is_finite());
            }
        }
    }
}
