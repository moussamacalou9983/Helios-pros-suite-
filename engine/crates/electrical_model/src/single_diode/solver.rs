//! Solving the implicit single-diode equation for the current.
//!
//! Three branches, in the order the formalisation establishes them:
//!
//! | Branch | When | Guarantee |
//! |---|---|---|
//! | Closed form | `R_s == 0` **exactly** | Exact, no iteration (§8.1) |
//! | Newton | otherwise | Converges from `I₀ = 0`, iterates provably in `[I*, I_ph]` (§4.4) |
//! | Bracketed fallback | Newton leaves its invariant | Terminates within a **derived** bound (§6.3) |
//!
//! **The fallback is not Brent.** It is the scheme specified in §6.2 of
//! `docs/design/35-formalisation-mathematique-electrical-model.md`: a
//! bisection is forced on every second iteration, which is what makes its
//! bound provable. Brent's own guarantee is a quadratic-in-`log` bound of
//! a different order, and is not claimed here.

use core_units::{Current, Voltage};

use super::parameters::SingleDiodeParameters;
use super::residual::{residual, residual_derivative};
use crate::error::ElectricalModelError;

/// Relative tolerance on the residual — §13 of the formalisation.
///
/// Scaled by the photocurrent, which majorises the short-circuit current,
/// so the two ratios that matter stay invariant across module sizes:
/// **450×** the round-off floor `10·ε·I` (termination is safe) and
/// **10⁻¹⁰** of the data uncertainty (the numerical error is physically
/// undetectable).
///
/// The formalisation states the scaling against `I_sc`; using `I_ph`
/// avoids a nested solve and is slightly more permissive, since
/// `I_ph ≥ I_sc`. Both scale identically, so the margin analysis is
/// unchanged.
pub const RELATIVE_RESIDUAL_TOLERANCE: f64 = 1e-12;

/// Iterations Newton may spend before the fallback takes over.
///
/// A **budget, not a bound**: no bound on Newton's iteration count is
/// proved. Measured on the nine reference parameter sets: 2 to 13.
///
/// Exceeding it is **not a pathology**, and calling it one would be
/// wrong. Deep in the exponential regime `f ≈ −I_0 e^u` and
/// `f' ≈ −(R_s I_0/a) e^u`, so the Newton step is `−a/R_s` — a constant,
/// independent of `I_0`, of `V`, and of how far the iterate still is from
/// the root. The descent from `I₁ ≈ I_ph` to `I*` therefore costs about
/// `(I_ph − I*)·R_s/a` iterations, the same group that dominates the
/// overflow guard.
///
/// A real module gives 1.6. Over the declared generation domain, which
/// reaches `R_s = 5 Ω`, the budget is exceeded in 0.642 % of admissible
/// sets and the worst case needs 218 — see the `newton_budget_census`
/// example, and O-16 in
/// `docs/design/36-statut-epistemique-electrical-model.md`. When that
/// happens the bracketed fallback — which *is* bounded — takes over and
/// returns the same root, which is what `PB-01` checks.
///
/// This is an **`[O]`**, not a **`[P]`**: the estimate is asymptotic and says
/// nothing rigorous about the transition regime.
pub const NEWTON_ITERATION_BUDGET: u32 = 30;

/// Iterations the bracketed fallback may spend — **derived**, §6.3.
///
/// With a bisection forced every second iteration, `m` bisections halve
/// the bracket `m` times, so `2·⌈log₂(W₀/τ)⌉` iterations suffice. For
/// `W₀ = I_ph + I_0` and `τ = 10⁻¹²·I_ph` the ratio is `≈ 10¹²`
/// independently of module size, giving `2 × 40 = 80`. The constant is set
/// to 82 — the value derived in the formalisation for its slightly wider
/// bracket — so that it is a conservative envelope of the bound actually
/// required here.
///
/// [`fallback_iteration_bound`] recomputes the requirement from the
/// parameters; a test asserts it never exceeds this constant.
pub const FALLBACK_ITERATION_BOUND: u32 = 82;

/// The bound the fallback actually needs for a given parameter set.
///
/// `2·⌈log₂(W₀/τ)⌉`, with `W₀ = I_ph + I_0` the proved bracket width and
/// `τ` the residual tolerance. Independent of the machine, and — because
/// numerator and denominator are both proportional to the photocurrent —
/// independent of the module.
pub fn fallback_iteration_bound(params: &SingleDiodeParameters) -> u32 {
    let width = params.photocurrent().amperes() + params.saturation_current().amperes();
    let tolerance = residual_tolerance(params);
    if tolerance <= 0.0 || !width.is_finite() {
        return FALLBACK_ITERATION_BOUND;
    }
    let halvings = (width / tolerance).log2().ceil();
    if halvings.is_finite() && halvings > 0.0 {
        // `as` is saturating for f64 -> u32 in Rust, so a pathological
        // value cannot wrap; it is clamped to the constant regardless.
        ((2.0 * halvings) as u32).min(FALLBACK_ITERATION_BOUND)
    } else {
        FALLBACK_ITERATION_BOUND
    }
}

/// The absolute residual tolerance for a parameter set, in amperes.
#[inline]
pub fn residual_tolerance(params: &SingleDiodeParameters) -> f64 {
    RELATIVE_RESIDUAL_TOLERANCE * params.photocurrent().amperes()
}

/// Which branch produced a result.
///
/// Exposed so that the guarantees can be **tested** rather than assumed:
/// a test can assert that `R_s = 0` took the closed form, or that a given
/// set never needed the fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SolveMethod {
    /// `R_s == 0`: the residual is affine and the root is exact (§8.1).
    ClosedFormZeroSeries,
    /// Newton from `I₀ = 0`, all iterates within the proved invariant.
    Newton,
    /// The bracketed fallback of §6.2 — **not** Brent.
    BracketedFallback,
}

/// A solved point, with the evidence needed to check the guarantees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolveOutcome {
    current: Current,
    method: SolveMethod,
    iterations: u32,
    residual: f64,
}

impl SolveOutcome {
    /// The current.
    #[inline]
    pub const fn current(&self) -> Current {
        self.current
    }

    /// Which branch produced it.
    #[inline]
    pub const fn method(&self) -> SolveMethod {
        self.method
    }

    /// Iterations spent. Zero for the closed form.
    #[inline]
    pub const fn iterations(&self) -> u32 {
        self.iterations
    }

    /// The residual at the returned point, in amperes.
    #[inline]
    pub const fn residual(&self) -> f64 {
        self.residual
    }

    /// A rigorous upper bound on the error of [`SolveOutcome::current`].
    ///
    /// By equation (14), `|I − I*| ≤ |f(I)|`, which holds because
    /// `∂f/∂I ≤ −1` everywhere under `𝓗`. This is a **bound**, not an
    /// estimate.
    ///
    /// In floating point the guaranteed bound is `|f̂| + η` with
    /// `η ≈ 10·ε·I_ph` the evaluation error; the correction is around
    /// `2·10⁻³` of the tolerance and is included here.
    #[inline]
    pub fn error_bound(&self, params: &SingleDiodeParameters) -> f64 {
        self.residual.abs() + 10.0 * f64::EPSILON * params.photocurrent().amperes()
    }
}

/// The current at a given terminal voltage.
///
/// # Errors
///
/// - [`ElectricalModelError::OutOfDomain`] if `V < 0` or `V > V_oc`. The
///   test is exact and costs no solve: `V ≤ V_oc` **is** `f(0; V) ≥ 0`,
///   because `f(0; ·)` is strictly decreasing in `V` by equation (8).
/// - [`ElectricalModelError::NonConvergence`] if neither branch reaches
///   the tolerance within its bound.
/// - [`ElectricalModelError::NonFiniteValue`] if a result fails to be
///   finite — unreachable given the guards, and present so that the
///   invariant is enforced rather than assumed.
///
/// The parameters are already known to satisfy `𝓗` and the overflow
/// guard: [`SingleDiodeParameters`] cannot be constructed otherwise.
pub fn current_at_voltage(
    params: &SingleDiodeParameters,
    voltage: Voltage,
) -> Result<SolveOutcome, ElectricalModelError> {
    let upper_bound = params.open_circuit_voltage_bound();

    if !voltage.volts().is_finite() || voltage.volts() < 0.0 {
        return Err(ElectricalModelError::OutOfDomain {
            requested: voltage,
            upper_bound,
        });
    }

    // `V <= V_oc` if and only if `f(0; V) >= 0`. Exact, and no solve.
    let residual_at_zero = residual(Current::from_amperes(0.0), voltage, params);
    if !residual_at_zero.is_finite() {
        return Err(ElectricalModelError::NonFiniteValue {
            quantity: "residual at I = 0",
            value: residual_at_zero,
        });
    }
    if residual_at_zero < 0.0 {
        return Err(ElectricalModelError::OutOfDomain {
            requested: voltage,
            upper_bound,
        });
    }

    let outcome = if params.has_zero_series_resistance() {
        closed_form_zero_series(params, voltage, residual_at_zero)
    } else {
        match newton(params, voltage) {
            Some(outcome) => outcome,
            None => bracketed_fallback(params, voltage, residual_at_zero)?,
        }
    };

    if !outcome.current.amperes().is_finite() {
        return Err(ElectricalModelError::NonFiniteValue {
            quantity: "solved current",
            value: outcome.current.amperes(),
        });
    }
    Ok(outcome)
}

/// The short-circuit current, `I(V = 0)`.
///
/// A named point of the model, not a separate computation.
///
/// # Errors
///
/// Propagates [`current_at_voltage`].
#[inline]
pub fn short_circuit_current(
    params: &SingleDiodeParameters,
) -> Result<SolveOutcome, ElectricalModelError> {
    current_at_voltage(params, Voltage::from_volts(0.0))
}

/// The exact root when `R_s = 0` — **equation (18)**.
///
/// With `R_s = 0` the residual is **affine** in `I`, since `∂²f/∂I² = 0`
/// by equation (7):
///
/// ```text
/// f(I) = [ I_ph − I_0(e^{V/a} − 1) − V/R_sh ] − I
/// ```
///
/// so the bracketed quantity **is** the root. It is exactly `f(0)`, which
/// the caller has already evaluated — so the closed form costs nothing
/// beyond what the domain test already computed.
///
/// No division by `R_s` occurs anywhere: the general branch does not
/// contain one either (§8.2), which is why no threshold on `R_s → 0` is
/// needed and none is used.
fn closed_form_zero_series(
    params: &SingleDiodeParameters,
    voltage: Voltage,
    residual_at_zero: f64,
) -> SolveOutcome {
    let current = Current::from_amperes(residual_at_zero);
    SolveOutcome {
        current,
        method: SolveMethod::ClosedFormZeroSeries,
        iterations: 0,
        residual: residual(current, voltage, params),
    }
}

/// Newton from `I₀ = 0`, refusing to leave the proved invariant.
///
/// Returns `None` — handing over to the fallback — when an iterate stops
/// being finite, leaves `[0, I_ph + I_0]`, or the budget runs out. Those
/// are the three objective conditions of §4.5; none is a heuristic.
///
/// The convergence test is `|f(I_k)| ≤ τ`, which by equation (14) *is* the
/// error bound `|I_k − I*| ≤ τ`. A separate test on the step would be
/// redundant.
fn newton(params: &SingleDiodeParameters, voltage: Voltage) -> Option<SolveOutcome> {
    let tolerance = residual_tolerance(params);
    let upper = params.photocurrent().amperes() + params.saturation_current().amperes();

    let mut current = 0.0_f64;
    for iteration in 0..=NEWTON_ITERATION_BUDGET {
        let f = residual(Current::from_amperes(current), voltage, params);
        if !f.is_finite() {
            return None;
        }
        if f.abs() <= tolerance {
            return Some(SolveOutcome {
                current: Current::from_amperes(current),
                method: SolveMethod::Newton,
                iterations: iteration,
                residual: f,
            });
        }
        if iteration == NEWTON_ITERATION_BUDGET {
            // `break`, not `return`: leaving by the loop's own exit keeps
            // the trailing `None` reachable. Written as a `return` it was
            // dead code, and dead code in a solver is a line no reviewer
            // can reason about and no test can reach.
            break;
        }

        let slope = residual_derivative(Current::from_amperes(current), voltage, params);
        // Under H the slope is at most -1, so it can be neither zero nor
        // positive; the test guards against a floating-point surprise, not
        // against the mathematics.
        if !slope.is_finite() || slope >= 0.0 {
            return None;
        }

        let next = current - f / slope;
        if !next.is_finite() || next < 0.0 || next > upper {
            // The interval invariant of equation (12) is violated: hand
            // over rather than continue outside the proved domain.
            return None;
        }
        current = next;
    }
    None
}

/// The bracketed fallback of §6.2 — **bisection forced every second
/// iteration**.
///
/// Not Brent, and not described as such. The bracket `[0, I_ph + I_0]` is
/// proved to contain the root by equation (9), and its sign change is
/// established by the same equation. Interleaving false position with a
/// forced bisection keeps the practical speed of the former while making
/// the bound of the latter apply.
///
/// # Errors
///
/// [`ElectricalModelError::NonConvergence`] if the derived bound is
/// exhausted, carrying the residual and the last iterate for diagnosis —
/// never as a result.
fn bracketed_fallback(
    params: &SingleDiodeParameters,
    voltage: Voltage,
    residual_at_zero: f64,
) -> Result<SolveOutcome, ElectricalModelError> {
    let tolerance = residual_tolerance(params);
    let bound = fallback_iteration_bound(params);

    let mut low = 0.0_f64;
    let mut f_low = residual_at_zero;
    let mut high = params.photocurrent().amperes() + params.saturation_current().amperes();
    let mut f_high = residual(Current::from_amperes(high), voltage, params);

    if f_low.abs() <= tolerance {
        return Ok(SolveOutcome {
            current: Current::from_amperes(low),
            method: SolveMethod::BracketedFallback,
            iterations: 0,
            residual: f_low,
        });
    }

    for iteration in 1..=bound {
        // Odd iterations attempt false position; even ones are forced
        // bisections. The forcing is what makes the bound provable.
        let candidate = if iteration % 2 == 1 {
            let denominator = f_high - f_low;
            if denominator.abs() > 0.0 && denominator.is_finite() {
                let interpolated = low - f_low * (high - low) / denominator;
                if interpolated.is_finite() && interpolated > low && interpolated < high {
                    interpolated
                } else {
                    0.5 * (low + high)
                }
            } else {
                0.5 * (low + high)
            }
        } else {
            0.5 * (low + high)
        };

        let f_candidate = residual(Current::from_amperes(candidate), voltage, params);
        if !f_candidate.is_finite() {
            return Err(ElectricalModelError::NonFiniteValue {
                quantity: "residual during the bracketed fallback",
                value: f_candidate,
            });
        }
        if f_candidate.abs() <= tolerance {
            return Ok(SolveOutcome {
                current: Current::from_amperes(candidate),
                method: SolveMethod::BracketedFallback,
                iterations: iteration,
                residual: f_candidate,
            });
        }

        // f is strictly decreasing, so f_low > 0 > f_high throughout.
        if f_candidate > 0.0 {
            low = candidate;
            f_low = f_candidate;
        } else {
            high = candidate;
            f_high = f_candidate;
        }

        // The bracket itself bounds the error, independently of the
        // residual: any point of it is within its width of the root.
        if high - low <= tolerance {
            let midpoint = 0.5 * (low + high);
            return Ok(SolveOutcome {
                current: Current::from_amperes(midpoint),
                method: SolveMethod::BracketedFallback,
                iterations: iteration,
                residual: residual(Current::from_amperes(midpoint), voltage, params),
            });
        }
    }

    let midpoint = 0.5 * (low + high);
    Err(ElectricalModelError::NonConvergence {
        iterations: bound,
        residual: residual(Current::from_amperes(midpoint), voltage, params).abs(),
        tolerance,
        last_iterate: Current::from_amperes(midpoint),
    })
}

/// Solve for the current using the bracketed fallback only.
///
/// Exposed so that the three branches can be **cross-checked against each
/// other** — an independent reference that owes nothing to Newton
/// (formalisation §6.4, test family E1.6). Not the production path.
///
/// # Errors
///
/// As [`current_at_voltage`].
pub fn current_at_voltage_by_bracketing(
    params: &SingleDiodeParameters,
    voltage: Voltage,
) -> Result<SolveOutcome, ElectricalModelError> {
    let upper_bound = params.open_circuit_voltage_bound();
    if !voltage.volts().is_finite() || voltage.volts() < 0.0 {
        return Err(ElectricalModelError::OutOfDomain {
            requested: voltage,
            upper_bound,
        });
    }
    let residual_at_zero = residual(Current::from_amperes(0.0), voltage, params);
    if residual_at_zero < 0.0 {
        return Err(ElectricalModelError::OutOfDomain {
            requested: voltage,
            upper_bound,
        });
    }
    bracketed_fallback(params, voltage, residual_at_zero)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::single_diode::parameters::ModifiedIdealityFactor;
    use core_units::Resistance;

    fn set(i_ph: f64, i_0: f64, r_s: f64, r_sh: f64, a: f64) -> SingleDiodeParameters {
        SingleDiodeParameters::new(
            Current::from_amperes(i_ph),
            Current::from_amperes(i_0),
            Resistance::from_ohms(r_s),
            Resistance::from_ohms(r_sh),
            ModifiedIdealityFactor::from_volts(a),
        )
        .expect("the test sets satisfy H")
    }

    fn nominal() -> SingleDiodeParameters {
        set(10.4, 2.5e-10, 0.30, 800.0, 1.85)
    }

    #[test]
    fn the_nominal_set_solves_by_newton_within_its_budget() {
        let p = nominal();
        let out = current_at_voltage(&p, Voltage::from_volts(36.0)).unwrap();
        assert_eq!(out.method(), SolveMethod::Newton);
        assert!(out.iterations() <= NEWTON_ITERATION_BUDGET);
        assert!(out.residual().abs() <= residual_tolerance(&p));
    }

    #[test]
    fn zero_series_resistance_takes_the_closed_form_with_no_iteration() {
        let p = set(10.4, 2.5e-10, 0.0, 800.0, 1.85);
        let out = current_at_voltage(&p, Voltage::from_volts(36.0)).unwrap();
        assert_eq!(out.method(), SolveMethod::ClosedFormZeroSeries);
        assert_eq!(out.iterations(), 0);
    }

    #[test]
    fn the_closed_form_matches_an_independent_evaluation_of_equation_18() {
        // Independent reference: equation (18) written out directly, using
        // none of the solver's machinery.
        let p = set(10.4, 2.5e-10, 0.0, 800.0, 1.85);
        for v in [0.0, 10.0, 25.0, 40.0, 45.0] {
            let expected = 10.4 - 2.5e-10 * ((v / 1.85_f64).exp() - 1.0) - v / 800.0;
            let got = current_at_voltage(&p, Voltage::from_volts(v))
                .unwrap()
                .current()
                .amperes();
            assert!(
                (got - expected).abs() <= 1e-12 * expected.abs().max(1.0),
                "V = {v}: got {got}, expected {expected}"
            );
        }
    }

    #[test]
    fn newton_and_the_bracketed_fallback_agree() {
        // Cross-check between two independent methods — the strongest
        // verification available without an external oracle.
        let p = nominal();
        for k in 0..=20 {
            let v = Voltage::from_volts(45.0 * f64::from(k) / 20.0);
            let by_newton = current_at_voltage(&p, v).unwrap();
            let by_bracket = current_at_voltage_by_bracketing(&p, v).unwrap();
            let gap = (by_newton.current().amperes() - by_bracket.current().amperes()).abs();
            assert!(
                gap <= 2.0 * residual_tolerance(&p),
                "V = {}: gap = {gap:e}",
                v.volts()
            );
        }
    }

    #[test]
    fn the_residual_bounds_the_error() {
        // Equation (14) made operational.
        let p = nominal();
        let v = Voltage::from_volts(36.0);
        let out = current_at_voltage(&p, v).unwrap();
        let reference = current_at_voltage_by_bracketing(&p, v).unwrap();
        let actual_error = (out.current().amperes() - reference.current().amperes()).abs();
        assert!(
            actual_error <= out.error_bound(&p) + reference.error_bound(&p),
            "error {actual_error:e} exceeds the bound"
        );
    }

    #[test]
    fn a_voltage_beyond_open_circuit_is_refused() {
        let p = nominal();
        // V_oc is about 45.22 V for this set.
        let err = current_at_voltage(&p, Voltage::from_volts(60.0)).unwrap_err();
        assert!(matches!(err, ElectricalModelError::OutOfDomain { .. }));
    }

    #[test]
    fn a_negative_or_non_finite_voltage_is_refused() {
        let p = nominal();
        for v in [-1.0, f64::NAN, f64::INFINITY] {
            let err = current_at_voltage(&p, Voltage::from_volts(v)).unwrap_err();
            assert!(matches!(err, ElectricalModelError::OutOfDomain { .. }));
        }
    }

    #[test]
    fn the_short_circuit_current_is_close_to_the_photocurrent() {
        let p = nominal();
        let i_sc = short_circuit_current(&p).unwrap().current().amperes();
        assert!(i_sc > 0.0 && i_sc <= 10.4, "I_sc = {i_sc}");
        // The shunt takes a small share; the difference is real but small.
        assert!(10.4 - i_sc < 0.05, "I_sc = {i_sc}");
    }

    #[test]
    fn the_fallback_bound_is_derived_and_never_exceeds_the_constant() {
        for p in [
            nominal(),
            set(0.5, 1e-11, 0.3, 16000.0, 1.85),
            set(20.0, 1e-9, 0.1, 400.0, 1.85),
            set(0.0104, 2.5e-10, 0.30, 8e5, 1.85),
        ] {
            let bound = fallback_iteration_bound(&p);
            assert!(bound <= FALLBACK_ITERATION_BOUND, "bound = {bound}");
            // The derivation gives about 80 for these sets, invariant of
            // module size because both terms scale with I_ph.
            assert!((78..=82).contains(&bound), "bound = {bound}");
        }
    }

    #[test]
    fn the_fallback_stays_within_its_bound() {
        let p = nominal();
        let bound = fallback_iteration_bound(&p);
        for k in 0..=20 {
            let v = Voltage::from_volts(45.0 * f64::from(k) / 20.0);
            let out = current_at_voltage_by_bracketing(&p, v).unwrap();
            assert!(
                out.iterations() <= bound,
                "iterations = {}",
                out.iterations()
            );
        }
    }

    #[test]
    fn pathological_shunt_resistance_still_solves() {
        // R_sh = 1.5 ohm collapses the fill factor but makes |f'| larger,
        // so the solver converges faster rather than slower.
        let p = set(10.4, 2.5e-10, 0.30, 1.5, 1.85);
        let out = current_at_voltage(&p, Voltage::from_volts(7.0)).unwrap();
        assert_eq!(out.method(), SolveMethod::Newton);
        assert!(out.iterations() <= 10);
    }

    #[test]
    fn a_thin_film_set_with_a_high_open_circuit_voltage_solves() {
        let p = set(1.9, 1e-11, 3.0, 2000.0, 4.0);
        let out = current_at_voltage(&p, Voltage::from_volts(80.0)).unwrap();
        assert!(out.residual().abs() <= residual_tolerance(&p));
    }

    #[test]
    fn results_are_deterministic() {
        let p = nominal();
        let v = Voltage::from_volts(31.7);
        let a = current_at_voltage(&p, v).unwrap();
        let b = current_at_voltage(&p, v).unwrap();
        assert_eq!(
            a.current().amperes().to_bits(),
            b.current().amperes().to_bits()
        );
        assert_eq!(a.iterations(), b.iterations());
    }
}
