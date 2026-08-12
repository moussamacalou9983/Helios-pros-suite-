//! The single-diode model: parameters, residual, derivatives, solver.
//!
//! This module is a **translation of the mathematics**, not an
//! interpretation of it. Every function names the equation of
//! `docs/design/35-formalisation-mathematique-electrical-model.md` it
//! implements, and no equation has been rearranged for the convenience of
//! the code.
//!
//! ## What is here
//!
//! | Concern | Where |
//! |---|---|
//! | The five parameters, validated against `𝓗` | [`parameters`] |
//! | `f`, `∂f/∂I`, `∂²f/∂I²`, `∂f/∂V`, `dI/dV`, `φ`, `φ'`, `φ''` | [`mod@residual`] |
//! | Newton, the closed form for `R_s = 0`, the bracketed fallback | [`solver`] |
//!
//! ## What is deliberately absent
//!
//! No maximum-power-point search, no `dP/dV`, no De Soto translation, no
//! CEC, no Sandia, no IEC 61853. Those follow the validation of this
//! kernel, in separate sub-phases.
//!
//! ## Dimensional homogeneity is a compile-time property
//!
//! `INV-EM-DIM-01` translates P-02: equation (1) is homogeneous in
//! amperes. No runtime test can check that — by the time a quantity is an
//! `f64` its dimension is gone. What is checkable is that the code offers
//! no way to write an inhomogeneous expression, and that is a property of
//! the type system. The three blocks below are **compiled and required to
//! fail** by `cargo test --doc`, which is the invariant itself:
//!
//! ```compile_fail
//! use core_units::{Current, Voltage};
//! // A voltage is not a current, whatever their magnitudes.
//! let _ = Current::from_amperes(1.0) + Voltage::from_volts(1.0);
//! ```
//!
//! ```compile_fail
//! use core_units::Voltage;
//! use electrical_model::single_diode::ModifiedIdealityFactor;
//! // `a` is in volts but is not a module voltage: `a + V_oc` is
//! // physically meaningless and must not typecheck (ADR-0009 D-2).
//! let _ = ModifiedIdealityFactor::from_volts(1.85) + Voltage::from_volts(45.0);
//! ```
//!
//! ```compile_fail
//! use core_units::{Current, Voltage};
//! use electrical_model::single_diode::{SingleDiodeParameters, residual};
//! // `residual` takes a current then a voltage. Swapping them is a type
//! // error, not a plausible wrong number.
//! fn f(p: &SingleDiodeParameters) -> f64 {
//!     residual(Voltage::from_volts(30.0), Current::from_amperes(5.0), p)
//! }
//! ```
//!
//! And the positive direction, which must compile: `Ω · A` is a voltage,
//! in either order.
//!
//! ```
//! use core_units::{Current, Resistance, Voltage};
//! let drop: Voltage = Resistance::from_ohms(0.30) * Current::from_amperes(10.0);
//! let commuted: Voltage = Current::from_amperes(10.0) * Resistance::from_ohms(0.30);
//! assert_eq!(drop.volts(), commuted.volts());
//! ```
//!
//! ## The guarantees, and their conditions
//!
//! Under the hypothesis set `𝓗` — enforced at construction of
//! [`parameters::SingleDiodeParameters`], never assumed — and on the
//! operating quadrant `V ∈ [0, V_oc]`:
//!
//! - the residual is strictly decreasing with `∂f/∂I ≤ −1` (eq. 5) and,
//!   for `R_s > 0`, strictly concave (eq. 7);
//! - the root exists and is unique (§3.2, §3.3), and lies in
//!   `[0, I_ph + I_0]` (eq. 9);
//! - from `I₀ = 0`, Newton's iterates remain in `[I*, I_ph]` (eq. 12);
//! - the residual **bounds** the error, `|I − I*| ≤ |f(I)|` (eq. 14);
//! - the fallback terminates within a derived bound (eq. 16).
//!
//! None of these holds outside `𝓗`. That is why construction refuses
//! rather than warns: outside `𝓗` the algorithm would not fail visibly,
//! it would return a plausible and wrong number.

/// The five parameters, validated against the hypothesis set `𝓗`.
pub mod parameters;
/// The residual and its derivatives — equations (3), (4), (6), (8).
pub mod residual;
/// Newton, the closed form for `R_s = 0`, and the bracketed fallback.
pub mod solver;

pub use parameters::{LN_F64_MAX, ModifiedIdealityFactor, SingleDiodeParameters};
pub use residual::{
    current_at_junction_voltage, current_voltage_slope, diode_exponential, diode_term, exponent,
    junction_current_curvature, junction_current_slope, junction_voltage, residual,
    residual_derivative, residual_second_derivative, residual_voltage_derivative,
};
pub use solver::{
    FALLBACK_ITERATION_BOUND, NEWTON_ITERATION_BUDGET, RELATIVE_RESIDUAL_TOLERANCE, SolveMethod,
    SolveOutcome, current_at_voltage, current_at_voltage_by_bracketing, fallback_iteration_bound,
    residual_tolerance, short_circuit_current,
};
