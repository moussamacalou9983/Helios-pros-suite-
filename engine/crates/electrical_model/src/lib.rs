//! # electrical_model
//!
//! The electrical behaviour of a photovoltaic module.
//!
//! Where [`pv_module`] describes **what a module is**, this crate
//! describes **how it behaves electrically**. The boundary is not
//! organisational: a module has an open-circuit voltage, but it does not
//! have a series resistance. `R_s` exists only inside a model that
//! postulates an equivalent circuit, and two models fitted to the same
//! module yield two different values of it, each correct in its own frame
//! and meaningless outside it.
//!
//! ## Scope of this release
//!
//! **The single-diode kernel, and nothing else.**
//!
//! | Delivered | Deliberately absent |
//! |---|---|
//! | The residual and its derivatives | The maximum-power-point search |
//! | Newton, closed form for `R_s = 0`, bracketed fallback | `dP/dV`, `V_mp`, `I_mp`, `P_mp` |
//! | The hypothesis set, enforced at construction | De Soto translation, CEC, Sandia, IEC 61853 |
//!
//! Those follow in separate sub-phases, after this kernel is validated.
//!
//! ## The code is a translation of the mathematics
//!
//! `docs/design/35-formalisation-mathematique-electrical-model.md` is the
//! normative reference. It numbers 29 equations and marks the epistemic
//! status of every statement: **`[P]`** proved, **`[O]`** numerically
//! observed, **`[H]`** hypothesis, **`[E]`** empirical validation required.
//! `docs/design/36-statut-epistemique-electrical-model.md` tabulates them.
//!
//! Three rules follow, and are visible throughout this crate:
//!
//! 1. **No equation is rearranged for the convenience of the code.** Each
//!    function names the equation it implements.
//! 2. **No observation is promoted to a guarantee.** Newton's thirty is a
//!    *budget*, not a bound; the fallback's bound is *derived* and is the
//!    one that terminates the algorithm.
//!
//!    The distinction is not academic. The formalisation records 2 to 13
//!    iterations measured on nine reference sets — true of those nine,
//!    and not a general statement. Over the declared generation domain
//!    the budget is exceeded in **0.642 %** of admissible sets, the worst
//!    needing **218** iterations, because the count grows like
//!    `(I_ph − I*)·R_s/a` and that domain reaches `R_s = 5 Ω`. A real
//!    module gives 1.6. Had the thirty been recorded as a bound rather
//!    than a budget, the fallback would look like dead code instead of a
//!    path taken once in 156. See the `newton_budget_census` example,
//!    which re-measures it.
//! 3. **Every proved property becomes a named invariant or a test.**
//!
//! ## The hypothesis set is a refusal, not a warning
//!
//! Every proof is conditioned on
//!
//! ```text
//! 𝓗 :  I_ph > 0,  I_0 > 0,  a > 0,  R_sh > 0,  R_s ≥ 0,  all finite
//! ```
//!
//! together with the operating quadrant `V ∈ [0, V_oc]`.
//!
//! [`single_diode::SingleDiodeParameters`] cannot be constructed outside
//! `𝓗`. This matters because outside `𝓗` the algorithm would **not** fail
//! visibly: it would return a plausible and wrong number. Refusing is the
//! only detectable behaviour.
//!
//! ## What this crate will be able to claim, and what it will not
//!
//! It can claim that, under `𝓗`, it returns the unique solution of the
//! single-diode equation with an error bounded by the tolerance, in a
//! bounded number of iterations.
//!
//! It cannot claim that the single-diode model describes a real
//! photovoltaic module. That would require measured I-V curves, which are
//! not available; it is recorded as an outstanding empirical validation
//! rather than quietly assumed.
//!
//! ## Units
//!
//! Every physical quantity crosses the public boundary as a strong type
//! from `core_units`. The one exception is deliberate: the modified
//! ideality factor `a` is in volts but is **not** a voltage of the module,
//! so it has its own type — `a + V_oc` must be impossible to write.
//!
//! ## References
//!
//! De Soto, W., Klein, S.A., Beckman, W.A. (2006), "Improvement and
//! validation of a model for photovoltaic array performance", *Solar
//! Energy* 80(1), 78-88 — the five-parameter model whose kernel this is.
//! IEC 60904-1 for the measurement conditions of an I-V characteristic.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod single_diode;

pub use error::{ElectricalModelError, ParameterFault};
pub use single_diode::{
    ModifiedIdealityFactor, SingleDiodeParameters, SolveMethod, SolveOutcome, current_at_voltage,
    current_at_voltage_by_bracketing, short_circuit_current,
};
