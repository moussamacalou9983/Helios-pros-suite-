//! Property-based tests of the single-diode kernel — **ÉTAPE 8**.
//!
//! ## What a property test is for here
//!
//! Seventeen of the twenty-two properties of
//! `docs/design/36-statut-epistemique-electrical-model.md` §7.2 translate
//! a **proved** property. A generative test therefore cannot establish
//! them — they are already established. Its role is the opposite one: to
//! **refute**. If one of these suites fails, the property is not false;
//! either the proof is wrong or the code does not implement it, and both
//! are worth finding.
//!
//! The five remaining properties are absolute invariants — no `NaN`, no
//! refusal-free path outside `𝓗`, bit-for-bit determinism — which a proof
//! does not cover and only exercise can probe.
//!
//! ## Reproducibility
//!
//! The generator is SplitMix64, seeded deterministically. **Every suite
//! prints its seed**, and each property derives its own stream from that
//! seed so two properties never examine the same draws. A failure is
//! replayed exactly with
//!
//! ```text
//! HELIOSPRO_PB_SEED=<seed> cargo test -p electrical_model --test properties
//! ```
//!
//! `HELIOSPRO_PB_CASES` raises the case count for a longer campaign; it
//! cannot lower it below [`MINIMUM_CASES_PER_PROPERTY`], so a shortened
//! run cannot pass off as a full one.
//!
//! ## What is deliberately absent
//!
//! `PB-06`–`PB-09`, `PB-13` and `PB-20`–`PB-22` are **not implemented in
//! this sub-phase**, and their absence is recorded rather than hidden:
//! they concern the power `P = V·I`, the maximum-power point and the
//! irradiance translation, none of which exists yet. They are listed in
//! the census as deferred, with the reason, so the count of implemented
//! properties can never be mistaken for the count of specified ones.

mod common;

use common::{Generator, describe, open_circuit_voltage};
use core_units::{Current, Resistance, Voltage};
use electrical_model::error::ElectricalModelError;
use electrical_model::single_diode::{
    FALLBACK_ITERATION_BOUND, ModifiedIdealityFactor, NEWTON_ITERATION_BUDGET,
    SingleDiodeParameters, SolveMethod, current_at_voltage, current_at_voltage_by_bracketing,
    current_voltage_slope, diode_term, exponent, fallback_iteration_bound,
    junction_current_curvature, junction_current_slope, junction_voltage, residual,
    residual_derivative, residual_second_derivative, residual_tolerance,
    residual_voltage_derivative, short_circuit_current,
};

// =====================================================================
// The campaign's parameters
// =====================================================================

/// The default seed.
///
/// Any value would do; this one is fixed so that the default run is
/// identical on every machine and in every CI job. A campaign that varied
/// its seed silently would turn an intermittent failure into folklore.
const DEFAULT_SEED: u64 = 0x4845_4C49_4F53_5052; // "HELIOSPR" in ASCII

/// The floor on generated cases per property, protected by the suite
/// itself.
///
/// The scenario-count discipline established on `pv_module` applies here
/// in its generative form (§7.4): the count is **measured**, not claimed,
/// and cannot silently fall.
const MINIMUM_CASES_PER_PROPERTY: u32 = 200;

/// The default number of generated cases per property.
const DEFAULT_CASES_PER_PROPERTY: u32 = 256;

fn seed() -> u64 {
    std::env::var("HELIOSPRO_PB_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_SEED)
}

fn cases_per_property() -> u32 {
    let requested = std::env::var("HELIOSPRO_PB_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CASES_PER_PROPERTY);
    // A campaign may be lengthened, never shortened.
    requested.max(MINIMUM_CASES_PER_PROPERTY)
}

/// Derive a per-property stream from the campaign seed.
///
/// FNV-1a over the identifier, mixed into the seed. Without this every
/// property would draw the same sequence, and twenty-two properties would
/// explore the domain of one.
fn stream_seed(id: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    seed() ^ hash
}

// =====================================================================
// The registry
// =====================================================================

/// One property of §7.2, as **data**.
///
/// Holding the properties as data rather than as a list of test functions
/// is what makes the census a measurement: the driver counts what it ran,
/// it does not restate what someone believed was run.
struct Property {
    /// `PB-nn`.
    id: &'static str,
    /// The statement being probed, in one line.
    statement: &'static str,
    /// The proved property it translates, or the reason it has none.
    translates: &'static str,
    /// One generated case. `Err` carries a replayable description.
    run: fn(&mut Generator) -> Result<(), String>,
}

/// A property of §7.2 that this sub-phase does not implement, with the
/// reason. Recorded so the census cannot overstate coverage.
struct Deferred {
    id: &'static str,
    statement: &'static str,
    reason: &'static str,
}

/// The properties deferred with the maximum-power point and the
/// irradiance translation.
const DEFERRED: [Deferred; 8] = [
    Deferred {
        id: "PB-06",
        statement: "P = V·I to 2e-12 relative",
        reason: "power is not computed in this sub-phase",
    },
    Deferred {
        id: "PB-07",
        statement: "the MPP exists",
        reason: "no MPP search — forbidden until this kernel is validated",
    },
    Deferred {
        id: "PB-08",
        statement: "the MPP is unique: two starts converge to one point",
        reason: "no MPP search — forbidden until this kernel is validated",
    },
    Deferred {
        id: "PB-09",
        statement: "P(V_mp) >= P(V) for every drawn V",
        reason: "no MPP search — forbidden until this kernel is validated",
    },
    Deferred {
        id: "PB-13",
        statement: "G = 0 exactly: null output, solver not called",
        reason: "irradiance belongs to the translation model, not the kernel",
    },
    Deferred {
        id: "PB-20",
        statement: "psi strictly increasing on the domain",
        reason: "psi is defined from dP/dV_j, which does not exist yet",
    },
    Deferred {
        id: "PB-21",
        statement: "dP/dV_j > 0 wherever psi < 0",
        reason: "power derivatives belong to the MPP sub-phase",
    },
    Deferred {
        id: "PB-22",
        statement: "d2P/dV_j2 < 0 wherever psi >= 0",
        reason: "power derivatives belong to the MPP sub-phase",
    },
];

/// Declare a property once, and get both a named `#[test]` and a registry
/// entry from it.
///
/// The two must not be allowed to drift: a property tested but missing
/// from the census would inflate nothing, but a property in the census
/// and not tested would inflate everything.
macro_rules! properties {
    ($( $test:ident : $id:literal, $statement:literal, $translates:literal, $run:path );* $(;)?) => {
        fn registry() -> Vec<Property> {
            vec![$(
                Property { id: $id, statement: $statement, translates: $translates, run: $run },
            )*]
        }

        $(
            #[test]
            fn $test() {
                let property = Property {
                    id: $id,
                    statement: $statement,
                    translates: $translates,
                    run: $run,
                };
                let report = exercise(&property);
                report.assert_clean();
            }
        )*
    };
}

properties! {
    pb_01_the_root_exists_and_is_unique:
        "PB-01", "two independent methods agree within their error bounds",
        "P-08, P-09", pb_01;
    pb_02_the_characteristic_is_strictly_decreasing:
        "PB-02", "I(V) decreases, strictly wherever the drop exceeds the error",
        "P-11", pb_02;
    pb_03_the_current_stays_between_zero_and_the_short_circuit:
        "PB-03", "I(V) in [0, I_sc], and I(V_oc) = 0 within the residual bound",
        "P-10", pb_03;
    pb_04_the_characteristic_is_lipschitz:
        "PB-04", "|I(V+d) - I(V)| bounded by the analytic slope times d",
        "P-11", pb_04;
    pb_05_no_non_finite_value_is_ever_produced:
        "PB-05", "no NaN and no infinity crosses the API boundary",
        "absolute invariant", pb_05;
    pb_10_the_zero_series_branch_is_exact_and_direct:
        "PB-10", "R_s = 0 matches equation (18) and takes no iteration",
        "P-23, P-24", pb_10;
    pb_11_the_leak_free_limit_is_approached_at_the_proved_rate:
        "PB-11", "|I - I_leakfree| <= V/R_sh, the exact residual perturbation",
        "P-11", pb_11;
    pb_12_a_vanishing_photocurrent_degrades_continuously:
        "PB-12", "I_ph -> 0 gives V_oc -> 0 and I_sc -> 0 without NaN",
        "continuity", pb_12;
    pb_14_the_residual_bounds_the_returned_error:
        "PB-14", "|f(I_returned)| <= tau_f, hence |I - I*| <= tau_f",
        "P-15", pb_14;
    pb_15_the_iteration_count_stays_within_its_bound:
        "PB-15", "iterations <= 82 in every circumstance",
        "P-19", pb_15;
    pb_16_the_derivative_is_at_most_minus_one_everywhere:
        "PB-16", "df/dI <= -1 at every evaluated point, not only at the root",
        "P-04", pb_16;
    pb_17_the_newton_iterates_stay_in_their_proved_interval:
        "PB-17", "I_k in [I*, I_ph] for every k >= 1",
        "P-13", pb_17;
    pb_18_a_tuple_outside_the_hypotheses_is_refused:
        "PB-18", "outside H: a refusal, never a result",
        "H-1..H-7", pb_18;
    pb_19_two_identical_calls_agree_bit_for_bit:
        "PB-19", "determinism, to the bit",
        "absolute invariant", pb_19;
}

// =====================================================================
// The driver
// =====================================================================

/// What a campaign over one property actually did.
struct Report {
    id: &'static str,
    statement: &'static str,
    translates: &'static str,
    cases: u32,
    failures: Vec<String>,
}

impl Report {
    #[track_caller]
    fn assert_clean(&self) {
        assert!(
            self.cases >= MINIMUM_CASES_PER_PROPERTY,
            "{} ran {} case(s), below the floor of {MINIMUM_CASES_PER_PROPERTY}",
            self.id,
            self.cases
        );
        assert!(
            self.failures.is_empty(),
            "{} — {} — failed {} of {} case(s) with seed {:#x}:\n{}",
            self.id,
            self.statement,
            self.failures.len(),
            self.cases,
            seed(),
            // Five is enough to see a pattern; the rest would bury it.
            self.failures
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

fn exercise(property: &Property) -> Report {
    let cases = cases_per_property();
    let mut generator = Generator::new(stream_seed(property.id));
    let mut failures = Vec::new();
    for case in 0..cases {
        if let Err(message) = (property.run)(&mut generator) {
            failures.push(format!("  case {case}: {message}"));
        }
    }
    Report {
        id: property.id,
        statement: property.statement,
        translates: property.translates,
        cases,
        failures,
    }
}

// =====================================================================
// Drawing helpers
// =====================================================================

/// A parameter set together with its open-circuit voltage.
///
/// `V_oc` comes from [`open_circuit_voltage`], a bisection on the residual
/// alone. It never calls the solver, so the sweep bounds are independent
/// of what is being tested — the requirement of ÉTAPE 9.
fn draw_case(generator: &mut Generator) -> (SingleDiodeParameters, f64) {
    let params = generator.admissible_parameters();
    let v_oc = open_circuit_voltage(&params).volts();
    (params, v_oc)
}

/// A voltage in the closed quadrant `[0, V_oc]`.
///
/// Both endpoints are drawn explicitly — roughly one case in fifty each —
/// because they are exactly where a domain test written with the wrong
/// comparison would show, and a purely interior draw would never reach
/// them.
fn draw_voltage(generator: &mut Generator, v_oc: f64) -> Voltage {
    let t = generator.unit();
    Voltage::from_volts(if t < 0.02 {
        0.0
    } else if t > 0.98 {
        v_oc
    } else {
        v_oc * t
    })
}

// =====================================================================
// PB-01 — existence and uniqueness
// =====================================================================

/// Newton and the bracketed fallback are independent: one uses the
/// derivative and no bracket, the other a bracket and no derivative.
/// Agreement within the sum of their own error bounds is the strongest
/// statement available without an external oracle — and by equation (14)
/// each bound is rigorous, so the comparison is not a tolerance chosen to
/// make the test pass.
fn pb_01(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let v = draw_voltage(generator, v_oc);

    let by_newton = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;
    let by_bracket = current_at_voltage_by_bracketing(&params, v).map_err(|e| {
        format!(
            "{} at V = {} V: bracketing {e}",
            describe(&params),
            v.volts()
        )
    })?;

    let gap = (by_newton.current().amperes() - by_bracket.current().amperes()).abs();
    let allowance = by_newton.error_bound(&params) + by_bracket.error_bound(&params);
    if gap > allowance {
        return Err(format!(
            "{} at V = {} V: gap {gap:e} A exceeds the combined bound {allowance:e} A",
            describe(&params),
            v.volts()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-02 and PB-04 — monotonicity and continuity, two sides of one bound
// =====================================================================

/// The magnitude of `dI/dV` increases with `V` on `[0, V_oc]`.
///
/// This is not an assumption; it follows from two properties already
/// proved and already tested as invariants:
///
/// 1. `V_j` is strictly increasing in `V` (`INV-EM-MPP-02`), because by
///    equation (10) `dV_j/dV = 1/(1 + R_s/R_sh + R_s g) > 0` where
///    `g = I_0 e^{V_j/a} / a > 0`;
/// 2. writing equation (10) as `dI/dV = −(1/R_sh + g)/(1 + R_s/R_sh + R_s g)`
///    and differentiating with respect to `g` gives exactly
///    `−1/(1 + R_s/R_sh + R_s g)² < 0`.
///
/// So `g` increases with `V`, and `dI/dV` decreases — its magnitude grows.
/// Consequently, on `[V, V + δ]`, `|dI/dV|` is smallest at `V` and largest
/// at `V + δ`, which brackets the true drop from both sides:
///
/// ```text
/// |dI/dV(V)|·δ  ≤  I(V) − I(V+δ)  ≤  |dI/dV(V+δ)|·δ
/// ```
///
/// by the mean value theorem. Each computed current carries its own
/// rigorous error bound, so both sides are widened by the sum of the two.
fn slope_bracket(
    params: &SingleDiodeParameters,
    v_low: Voltage,
    v_high: Voltage,
) -> Result<(f64, f64, f64, f64), String> {
    let low = current_at_voltage(params, v_low)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(params), v_low.volts()))?;
    let high = current_at_voltage(params, v_high)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(params), v_high.volts()))?;

    let slope_low = current_voltage_slope(low.current(), v_low, params)
        .ok_or_else(|| format!("{}: dI/dV undefined at V_low", describe(params)))?;
    let slope_high = current_voltage_slope(high.current(), v_high, params)
        .ok_or_else(|| format!("{}: dI/dV undefined at V_high", describe(params)))?;

    let allowance = low.error_bound(params) + high.error_bound(params);
    Ok((
        low.current().amperes() - high.current().amperes(),
        slope_low.abs(),
        slope_high.abs(),
        allowance,
    ))
}

/// `PB-02` — the characteristic decreases, strictly wherever the true drop
/// is resolvable.
///
/// Strictness is asserted **conditionally**, and the condition is not a
/// convenience. Over a step `δ` the true drop is at least `|dI/dV(V)|·δ`;
/// where that is smaller than the solvers' combined error bound, two
/// computed currents may legitimately come out equal — a shunt resistance
/// of `10¹² Ω` makes the characteristic flatter, near `V = 0`, than the
/// tolerance can resolve. Demanding strict inequality there would be
/// demanding an accuracy the model's own error bound denies, so the test
/// asserts what is true: non-increase always, strict decrease exactly when
/// the drop is above the noise floor.
fn pb_02(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let steps = 12;
    for k in 0..steps {
        let v_low = Voltage::from_volts(v_oc * (f64::from(k) / f64::from(steps)));
        let v_high = Voltage::from_volts(v_oc * (f64::from(k + 1) / f64::from(steps)));
        let delta = v_high.volts() - v_low.volts();
        let (drop, slope_low, _, allowance) = slope_bracket(&params, v_low, v_high)?;

        if drop < -allowance {
            return Err(format!(
                "{}: I rose by {:e} A between {} V and {} V (allowance {allowance:e})",
                describe(&params),
                -drop,
                v_low.volts(),
                v_high.volts()
            ));
        }
        let resolvable = slope_low * delta;
        if resolvable > allowance && drop <= 0.0 {
            return Err(format!(
                "{}: drop {drop:e} A is not strict between {} V and {} V \
                 although the slope requires at least {resolvable:e} A",
                describe(&params),
                v_low.volts(),
                v_high.volts()
            ));
        }
    }
    Ok(())
}

/// `PB-04` — continuity, as the upper half of the same bracket.
fn pb_04(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let steps = 12;
    for k in 0..steps {
        let v_low = Voltage::from_volts(v_oc * (f64::from(k) / f64::from(steps)));
        let v_high = Voltage::from_volts(v_oc * (f64::from(k + 1) / f64::from(steps)));
        let delta = v_high.volts() - v_low.volts();
        let (drop, _, slope_high, allowance) = slope_bracket(&params, v_low, v_high)?;

        let ceiling = slope_high * delta + allowance;
        if drop > ceiling {
            return Err(format!(
                "{}: drop {drop:e} A between {} V and {} V exceeds the \
                 Lipschitz ceiling {ceiling:e} A",
                describe(&params),
                v_low.volts(),
                v_high.volts()
            ));
        }
    }
    Ok(())
}

// =====================================================================
// PB-03 — bounds
// =====================================================================

/// `PB-03` — `I(V) ∈ [0, I_sc]`, and `I(V_oc) = 0` to the residual bound.
///
/// The upper bound is `I_sc` rather than `I_ph + I_0`: the latter is the
/// bracket the solver is proved to stay inside, the former is the tighter
/// physical statement that the characteristic is highest at short circuit.
/// Both hold; testing the tighter one is worth more.
///
/// At `V_oc` the exact root is zero. The computed one is bounded by
/// `|f(0; V_oc)|` through equation (14) — and `f(0; V_oc)` is not exactly
/// zero, because `V_oc` itself is only located to a floating-point
/// neighbourhood. Using that residual as the allowance is therefore not a
/// slackening: it is the bound equation (14) actually provides.
fn pb_03(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let short_circuit = short_circuit_current(&params)
        .map_err(|e| format!("{}: short circuit {e}", describe(&params)))?;
    let i_sc = short_circuit.current().amperes();

    let v = draw_voltage(generator, v_oc);
    let out = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;
    let i = out.current().amperes();
    let allowance = out.error_bound(&params) + short_circuit.error_bound(&params);

    if i < -allowance {
        return Err(format!(
            "{} at V = {} V: I = {i:e} A is negative beyond {allowance:e}",
            describe(&params),
            v.volts()
        ));
    }
    if i > i_sc + allowance {
        return Err(format!(
            "{} at V = {} V: I = {i:e} A exceeds I_sc = {i_sc:e} A",
            describe(&params),
            v.volts()
        ));
    }

    let at_open_circuit = current_at_voltage(&params, Voltage::from_volts(v_oc))
        .map_err(|e| format!("{} at V_oc: {e}", describe(&params)))?;
    let residual_at_zero = residual(
        Current::from_amperes(0.0),
        Voltage::from_volts(v_oc),
        &params,
    );
    let ceiling = residual_at_zero.abs() + at_open_circuit.error_bound(&params);
    if at_open_circuit.current().amperes().abs() > ceiling {
        return Err(format!(
            "{}: I(V_oc) = {:e} A exceeds the equation (14) bound {ceiling:e} A",
            describe(&params),
            at_open_circuit.current().amperes()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-05 — no non-finite value
// =====================================================================

/// `PB-05` — nothing non-finite crosses the API boundary.
///
/// Checked on **every** public quantity of the kernel, not only the
/// current: a derivative that returns an infinity would be just as much a
/// defect, and would surface later and further from its cause.
fn pb_05(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    for k in 0..=8 {
        let v = Voltage::from_volts(v_oc * (f64::from(k) / 8.0));
        let out = current_at_voltage(&params, v)
            .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;
        let i = out.current();

        let v_j = junction_voltage(v, i, params.series_resistance());
        let quantities: [(&str, f64); 11] = [
            ("I", i.amperes()),
            ("residual", out.residual()),
            ("u", exponent(v, i, &params)),
            ("diode term", diode_term(exponent(v, i, &params), &params)),
            ("df/dI", residual_derivative(i, v, &params)),
            ("d2f/dI2", residual_second_derivative(i, v, &params)),
            ("df/dV", residual_voltage_derivative(i, v, &params)),
            (
                "dI/dV",
                current_voltage_slope(i, v, &params)
                    .ok_or_else(|| format!("{}: dI/dV undefined", describe(&params)))?,
            ),
            ("V_j", v_j.volts()),
            ("phi'", junction_current_slope(v_j, &params)),
            ("phi''", junction_current_curvature(v_j, &params)),
        ];
        for (name, value) in quantities {
            if !value.is_finite() {
                return Err(format!(
                    "{} at V = {} V: {name} = {value}",
                    describe(&params),
                    v.volts()
                ));
            }
        }
    }
    Ok(())
}

// =====================================================================
// PB-10 — the closed form
// =====================================================================

/// `PB-10` — with `R_s = 0` the answer is equation (18), evaluated once.
///
/// The specification asks for agreement "to 1 ulp". That target is
/// achievable on the *terms*, not on the result: equation (18) sums three
/// quantities of magnitude up to `I_ph` which cancel to near zero as `V`
/// approaches `V_oc`, so the reachable accuracy is a few ULP of `I_ph`
/// however the sum is arranged. The tolerance below states that honestly
/// rather than restating a target the arithmetic cannot meet.
fn pb_10(generator: &mut Generator) -> Result<(), String> {
    let params = generator.admissible_parameters_with_series(0.0);
    let v_oc = open_circuit_voltage(&params).volts();
    let v = draw_voltage(generator, v_oc);

    let out = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;
    if out.method() != SolveMethod::ClosedFormZeroSeries {
        return Err(format!(
            "{}: R_s = 0 took the {:?} branch",
            describe(&params),
            out.method()
        ));
    }
    if out.iterations() != 0 {
        return Err(format!(
            "{}: the closed form iterated {} time(s)",
            describe(&params),
            out.iterations()
        ));
    }

    // Equation (18) written out, using none of the solver's machinery.
    let i_ph = params.photocurrent().amperes();
    let i_0 = params.saturation_current().amperes();
    let a = params.modified_ideality_factor().volts();
    let r_sh = params.shunt_resistance().ohms();
    let independent = i_ph - i_0 * ((v.volts() / a).exp() - 1.0) - v.volts() / r_sh;

    let ceiling = 8.0 * f64::EPSILON * i_ph;
    let gap = (out.current().amperes() - independent).abs();
    if gap > ceiling {
        return Err(format!(
            "{} at V = {} V: closed form gives {:e} A, equation (18) gives \
             {independent:e} A, gap {gap:e} exceeds {ceiling:e}",
            describe(&params),
            v.volts(),
            out.current().amperes()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-11 — the leak-free limit
// =====================================================================

/// `PB-11` — as `R_sh → ∞` the solution approaches the leak-free one, and
/// the rate is not measured but **derived**.
///
/// Let `I_∞` be the root of the leak-free residual
/// `f_∞(I) = I_ph − I_0(e^{V/a} − 1) − I`. Evaluating the *leaky*
/// residual there gives exactly
///
/// ```text
/// f(I_∞) = f_∞(I_∞) − V/R_sh = −V/R_sh
/// ```
///
/// (with `R_s = 0`, so the shunt term is `V/R_sh` alone), and equation
/// (14) turns that into `|I_∞ − I*| ≤ V/R_sh`. The check therefore
/// compares against a closed-form reference with a closed-form allowance —
/// no limit is taken numerically and no rate is fitted.
fn pb_11(generator: &mut Generator) -> Result<(), String> {
    // `R_s = 0` is what makes the reference closed-form and hence
    // independent of the solver.
    let mut params = generator.admissible_parameters_with_series(0.0);
    // Redraw until the shunt resistance is genuinely large; the property
    // is about that regime.
    let mut guard = 0;
    while params.shunt_resistance().ohms() < 1e9 {
        params = generator.admissible_parameters_with_series(0.0);
        guard += 1;
        if guard > 10_000 {
            return Err("no set with R_sh >= 1e9 ohm was drawn".to_owned());
        }
    }

    let v_oc = open_circuit_voltage(&params).volts();
    let v = draw_voltage(generator, v_oc);
    let out = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;

    let i_ph = params.photocurrent().amperes();
    let i_0 = params.saturation_current().amperes();
    let a = params.modified_ideality_factor().volts();
    let leak_free = i_ph - i_0 * ((v.volts() / a).exp() - 1.0);

    let ceiling = v.volts() / params.shunt_resistance().ohms()
        + out.error_bound(&params)
        + 8.0 * f64::EPSILON * i_ph;
    let gap = (out.current().amperes() - leak_free).abs();
    if gap > ceiling {
        return Err(format!(
            "{} at V = {} V: gap to the leak-free root is {gap:e} A, above \
             the derived bound {ceiling:e} A",
            describe(&params),
            v.volts()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-12 — a vanishing photocurrent
// =====================================================================

/// `PB-12` — the kernel degrades continuously as `I_ph → 0`.
///
/// **This is the kernel-level form of the property.** §7.2 states it in
/// terms of irradiance `G`, which the kernel does not know: irradiance
/// reaches the single-diode equation only through the translation model,
/// which this sub-phase does not implement. What the kernel *can* be held
/// to is the behaviour of its own input, `I_ph`, and that is what is
/// checked here. The irradiance form remains owed, and `PB-13` — the
/// exact `G = 0` predicate — is recorded as deferred for the same reason.
///
/// The photocurrent is drawn far below the generator's usual floor, down
/// to `10⁻¹²` A, which is nine orders of magnitude under a module at
/// standard conditions.
fn pb_12(generator: &mut Generator) -> Result<(), String> {
    let i_ph = generator.log_uniform(1e-12, 1e-3);
    let i_0 = generator.log_uniform(1e-15, 1e-6);
    let a = generator.uniform(0.5, 10.0);
    let r_sh = generator.log_uniform(1.0, 1e12);
    let r_s = generator.uniform(0.0, 5.0);

    let params = SingleDiodeParameters::new(
        Current::from_amperes(i_ph),
        Current::from_amperes(i_0),
        Resistance::from_ohms(r_s),
        Resistance::from_ohms(r_sh),
        ModifiedIdealityFactor::from_volts(a),
    )
    .map_err(|e| format!("I_ph = {i_ph:e} A refused: {e}"))?;

    // The closed-form bound `V_oc <= a·ln(1 + I_ph/I_0)` vanishes with
    // `I_ph`; check that it does, rather than assuming it.
    let bound = params.open_circuit_voltage_bound().volts();
    let expected = a * (1.0 + i_ph / i_0).ln();
    if !bound.is_finite() || (bound - expected).abs() > 8.0 * f64::EPSILON * expected.max(1.0) {
        return Err(format!(
            "{}: the V_oc bound is {bound} where a·ln(1 + I_ph/I_0) = {expected}",
            describe(&params)
        ));
    }

    let out = short_circuit_current(&params)
        .map_err(|e| format!("{}: short circuit {e}", describe(&params)))?;
    let i_sc = out.current().amperes();
    if !i_sc.is_finite() {
        return Err(format!("{}: I_sc = {i_sc}", describe(&params)));
    }
    // At `V = 0` the residual reduces to `I_ph − I_0(e^{I·R_s/a} − 1)
    // − I·R_s/R_sh − I`, so `0 <= I_sc <= I_ph`: both subtracted terms are
    // non-negative for `I >= 0`.
    if i_sc < -out.error_bound(&params) || i_sc > i_ph + out.error_bound(&params) {
        return Err(format!(
            "{}: I_sc = {i_sc:e} A is outside [0, I_ph]",
            describe(&params)
        ));
    }
    Ok(())
}

// =====================================================================
// PB-14 — the residual bounds the error
// =====================================================================

/// `PB-14` — the returned residual meets the tolerance, and it is the
/// residual of the returned current.
///
/// The second half matters as much as the first: a solver that reported a
/// stale residual would satisfy the tolerance check while returning
/// something else entirely. The residual is therefore recomputed here from
/// the returned current, independently of the solve.
fn pb_14(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let v = draw_voltage(generator, v_oc);
    let out = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;

    let tolerance = residual_tolerance(&params);
    if out.residual().abs() > tolerance {
        return Err(format!(
            "{} at V = {} V: |f| = {:e} exceeds tau_f = {tolerance:e}",
            describe(&params),
            v.volts(),
            out.residual().abs()
        ));
    }

    let recomputed = residual(out.current(), v, &params);
    if recomputed != out.residual() {
        return Err(format!(
            "{} at V = {} V: the reported residual {:e} is not the residual \
             of the reported current, {recomputed:e}",
            describe(&params),
            v.volts(),
            out.residual()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-15 — the iteration bound
// =====================================================================

/// `PB-15` — every method stays inside its own bound.
///
/// Newton's 30 is a **budget** and the fallback's 82 is a **bound**; the
/// distinction is preserved here rather than flattened into one number,
/// because only one of the two is proved.
fn pb_15(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let v = draw_voltage(generator, v_oc);

    let out = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;
    let ceiling = match out.method() {
        SolveMethod::ClosedFormZeroSeries => 0,
        SolveMethod::Newton => NEWTON_ITERATION_BUDGET,
        SolveMethod::BracketedFallback => fallback_iteration_bound(&params),
        // `SolveMethod` is `#[non_exhaustive]`. A method added later has
        // no proved bound until one is derived for it, so the property
        // must fail rather than wave it through under someone else's.
        other => {
            return Err(format!(
                "{}: method {other:?} has no declared iteration bound",
                describe(&params)
            ));
        }
    };
    if out.iterations() > ceiling {
        return Err(format!(
            "{} at V = {} V: {:?} took {} iteration(s), above {ceiling}",
            describe(&params),
            v.volts(),
            out.method(),
            out.iterations()
        ));
    }

    let bracketed = current_at_voltage_by_bracketing(&params, v).map_err(|e| {
        format!(
            "{} at V = {} V: bracketing {e}",
            describe(&params),
            v.volts()
        )
    })?;
    let derived = fallback_iteration_bound(&params);
    if bracketed.iterations() > derived || derived > FALLBACK_ITERATION_BOUND {
        return Err(format!(
            "{} at V = {} V: {} iteration(s) against a derived bound of \
             {derived} and a global bound of {FALLBACK_ITERATION_BOUND}",
            describe(&params),
            v.volts(),
            bracketed.iterations()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-16 — the derivative
// =====================================================================

/// `PB-16` — `∂f/∂I ≤ −1` at points drawn across the whole bracket, not
/// only at the root.
///
/// Equation (5) holds for every `I`, so testing it only where the solver
/// happens to land would test a corollary rather than the property. The
/// current is drawn uniformly across `[0, I_ph + I_0]`.
fn pb_16(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let v = draw_voltage(generator, v_oc);
    let upper = params.photocurrent().amperes() + params.saturation_current().amperes();

    for _ in 0..4 {
        let i = Current::from_amperes(generator.uniform(0.0, upper));
        let derivative = residual_derivative(i, v, &params);
        // The `is_nan` arm is not decoration: `derivative > -1.0` alone is
        // false for a `NaN`, so a derivative that failed to be computable
        // would be recorded as satisfying equation (5).
        if derivative.is_nan() || derivative > -1.0 {
            return Err(format!(
                "{} at V = {} V, I = {} A: df/dI = {derivative}",
                describe(&params),
                v.volts(),
                i.amperes()
            ));
        }
    }
    Ok(())
}

// =====================================================================
// PB-17 — the interval invariant
// =====================================================================

/// Why the Newton recurrence stopped, in the vocabulary of §4.5.
#[derive(Debug, PartialEq, Eq)]
enum NewtonExit {
    /// `|f| ≤ τ_f` was reached inside the budget.
    Converged,
    /// The budget of thirty was spent. A **legitimate** exit, and the
    /// reason the budget is documented as a budget and not as a bound.
    BudgetSpent,
    /// An iterate left `[0, I_ph + I_0]` or stopped being finite.
    LeftTheBracket,
}

/// `PB-17` — the Newton iterates stay in `[I*, I_ph]` for every `k ≥ 1`,
/// and the method the solver reports matches the exit that actually
/// occurred.
///
/// **What this checks, exactly.** The recurrence of equation (11) is
/// replayed here from `I₀ = 0` using the crate's public `f` and `∂f/∂I`,
/// and the interval invariant is asserted at every step. That is a check
/// of the *recurrence*, which is what proposition N-2 is about.
///
/// It then checks the solver's own loop the only way the public API
/// allows: §4.5 lists three objective exits, and the reported method is a
/// function of which one occurred. Convergence inside the budget must
/// yield `Newton`; anything else must yield `BracketedFallback`. Tying
/// the two together is what makes this more than a restatement — a solver
/// that fell back for an undeclared reason would fail here.
///
/// `k = 0` is excluded by the proposition itself, not for convenience:
/// the start point is `0 ≤ I*`, and the first step is what carries the
/// iterate past the root — the concavity of `f` puts `I₁` above `I*`, and
/// every later iterate descends towards it.
///
/// ## The budget is spent for a reason that can be computed in advance
///
/// Exhausting the budget is not rare and not pathological. Deep in the
/// exponential regime the residual is `f ≈ −I_0 e^u` and its derivative
/// `f' ≈ −(R_s I_0/a) e^u`, so the Newton step is
///
/// ```text
/// −f/f' ≈ −a/R_s
/// ```
///
/// — a **constant**, independent of `I_0`, of `V` and of how far the
/// iterate still is from the root. The descent from `I₁ ≈ I_ph` to `I*`
/// therefore costs about `(I_ph − I*)·R_s/a` iterations, bounded above by
/// `I_ph·R_s/a`: the same group that dominates the overflow guard.
///
/// For a real module — `I_ph ≈ 10 A`, `R_s ≈ 0.3 Ω`, `a ≈ 1.85 V` — that
/// is under two iterations. The generator's declared domain reaches
/// `R_s = 5 Ω`, which no module has, and there the group passes 200. The
/// `newton_budget_census` example measures it over 200 000 admissible
/// draws: **0.642 % above the budget, worst count 218 against 217.1
/// predicted**. The fallback is therefore not a theoretical safety net —
/// it is reached, and `PB-01`, `PB-14` and `PB-15` are what establish
/// that the answer it returns is the right one.
///
/// This is an **[O]**, not a **[P]**: the estimate is asymptotic in the
/// deep-exponential regime and says nothing rigorous about the transition.
/// It is recorded because it explains an exit, not because it bounds one.
fn pb_17(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    if params.has_zero_series_resistance() {
        // No Newton iteration exists in this branch; the closed form is
        // exercised by PB-10.
        return Ok(());
    }
    let v = draw_voltage(generator, v_oc);
    let out = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;

    let i_star = out.current().amperes();
    let i_ph = params.photocurrent().amperes();
    let bracket_top = i_ph + params.saturation_current().amperes();
    let allowance = out.error_bound(&params);
    let tolerance = residual_tolerance(&params);

    let mut iterate = 0.0_f64;
    let mut exit = NewtonExit::BudgetSpent;
    for k in 0..=NEWTON_ITERATION_BUDGET {
        let current = Current::from_amperes(iterate);
        let value = residual(current, v, &params);
        if !value.is_finite() {
            exit = NewtonExit::LeftTheBracket;
            break;
        }
        if value.abs() <= tolerance {
            exit = NewtonExit::Converged;
            break;
        }
        if k == NEWTON_ITERATION_BUDGET {
            break;
        }

        let derivative = residual_derivative(current, v, &params);
        if !derivative.is_finite() || derivative >= 0.0 {
            exit = NewtonExit::LeftTheBracket;
            break;
        }
        let next = iterate - value / derivative;

        // Equation (12), asserted for k + 1 >= 1. This is the property.
        if !(next >= i_star - allowance && next <= i_ph + allowance) {
            return Err(format!(
                "{} at V = {} V: iterate {} = {next:e} A left [I*, I_ph] = \
                 [{i_star:e}, {i_ph:e}]",
                describe(&params),
                v.volts(),
                k + 1
            ));
        }
        // The solver's own, looser guard.
        if !next.is_finite() || next < 0.0 || next > bracket_top {
            exit = NewtonExit::LeftTheBracket;
            break;
        }
        iterate = next;
    }

    let expected = if exit == NewtonExit::Converged {
        SolveMethod::Newton
    } else {
        SolveMethod::BracketedFallback
    };
    if out.method() != expected {
        return Err(format!(
            "{} at V = {} V: the recurrence exited by {exit:?}, so the method \
             should be {expected:?}, but {:?} was reported",
            describe(&params),
            v.volts(),
            out.method()
        ));
    }
    Ok(())
}

// =====================================================================
// PB-18 — refusal outside the hypotheses
// =====================================================================

/// `PB-18` — outside `𝓗` the construction refuses, and names what it
/// refused.
///
/// A refusal that did not identify the offending parameter would be
/// almost as bad as no refusal: the caller would know the set is
/// rejected without knowing which of five values to look at. The error is
/// therefore required to carry a fault and a hypothesis identifier, not
/// only a variant.
fn pb_18(generator: &mut Generator) -> Result<(), String> {
    let (i_ph, i_0, r_s, r_sh, a) = generator.inadmissible_tuple();
    let outcome = SingleDiodeParameters::new(
        Current::from_amperes(i_ph),
        Current::from_amperes(i_0),
        Resistance::from_ohms(r_s),
        Resistance::from_ohms(r_sh),
        ModifiedIdealityFactor::from_volts(a),
    );
    match outcome {
        Ok(_) => Err(format!(
            "accepted an inadmissible tuple: I_ph = {i_ph:e}, I_0 = {i_0:e}, \
             R_s = {r_s:e}, R_sh = {r_sh:e}, a = {a:e}"
        )),
        Err(ElectricalModelError::InvalidParameters { fault, .. }) => {
            if fault.hypothesis().is_empty() || fault.parameter().is_empty() {
                Err(format!(
                    "the refusal named neither parameter nor hypothesis: {fault:?}"
                ))
            } else {
                Ok(())
            }
        }
        Err(other) => Err(format!(
            "refused for the wrong reason: {other} (I_ph = {i_ph:e}, I_0 = {i_0:e}, \
             R_s = {r_s:e}, R_sh = {r_sh:e}, a = {a:e})"
        )),
    }
}

// =====================================================================
// PB-19 — determinism
// =====================================================================

/// `PB-19` — two identical calls agree to the bit.
///
/// Compared with `to_bits`, not with `==`: the latter would call two
/// different values equal if both were zero of opposite sign, and would
/// call two `NaN`s unequal. Neither should ever occur, and a bitwise
/// comparison is the only one that would say so.
fn pb_19(generator: &mut Generator) -> Result<(), String> {
    let (params, v_oc) = draw_case(generator);
    let v = draw_voltage(generator, v_oc);

    let first = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;
    let second = current_at_voltage(&params, v)
        .map_err(|e| format!("{} at V = {} V: {e}", describe(&params), v.volts()))?;

    if first.current().amperes().to_bits() != second.current().amperes().to_bits()
        || first.residual().to_bits() != second.residual().to_bits()
        || first.iterations() != second.iterations()
        || first.method() != second.method()
    {
        return Err(format!(
            "{} at V = {} V: {:?} and {:?} differ",
            describe(&params),
            v.volts(),
            (
                first.current().amperes(),
                first.residual(),
                first.iterations(),
                first.method()
            ),
            (
                second.current().amperes(),
                second.residual(),
                second.iterations(),
                second.method()
            )
        ));
    }
    Ok(())
}

// =====================================================================
// The census
// =====================================================================

/// Escape a statement for a Markdown table cell.
///
/// Several statements are inequalities on absolute values — `|f| <= tau_f`
/// — and a bare pipe ends the cell. The generated matrix is read as
/// evidence, so it has to render; a table silently broken into the wrong
/// number of columns is evidence of nothing.
fn escape_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// Renders the campaign's table.
///
/// Named `zz_` so it runs last under the default alphabetical ordering,
/// and so the table it writes describes a suite that has been exercised.
#[test]
fn zz_the_property_census_is_written_to_disk() {
    let properties = registry();
    let reports: Vec<Report> = properties.iter().map(exercise).collect();

    let mut table = String::new();
    table.push_str("# Matrice des tests de propriétés — `electrical_model`\n\n");
    table.push_str(
        "Générée par `cargo test -p electrical_model --test properties`.\n\
         **Ne pas éditer à la main** : ce fichier est une mesure, pas une\n\
         déclaration.\n\n",
    );
    table.push_str(&format!(
        "- Graine de campagne : `{:#018x}`\n- Cas par propriété : **{}** \
         (plancher `{}`)\n- Générateur : SplitMix64 (Steele, Lea & Flood, \
         OOPSLA 2014)\n\n",
        seed(),
        cases_per_property(),
        MINIMUM_CASES_PER_PROPERTY
    ));

    table.push_str("## Propriétés vérifiées\n\n");
    table.push_str("| Propriété | Énoncé | Traduit | Cas | Échecs |\n");
    table.push_str("|---|---|---|---:|---:|\n");
    for report in &reports {
        table.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            report.id,
            escape_cell(report.statement),
            report.translates,
            report.cases,
            report.failures.len()
        ));
    }

    let total: u32 = reports.iter().map(|r| r.cases).sum();
    let failed: usize = reports.iter().map(|r| r.failures.len()).sum();
    table.push_str(&format!(
        "\n**{} propriétés, {total} cas générés, {failed} échec(s).**\n",
        reports.len()
    ));

    table.push_str("\n## Propriétés différées\n\n");
    table.push_str(
        "Listées pour que le nombre de propriétés vérifiées ne puisse pas être\n\
         confondu avec le nombre de propriétés spécifiées.\n\n",
    );
    table.push_str("| Propriété | Énoncé | Motif du report |\n|---|---|---|\n");
    for deferred in &DEFERRED {
        table.push_str(&format!(
            "| `{}` | {} | {} |\n",
            deferred.id,
            escape_cell(deferred.statement),
            deferred.reason
        ));
    }
    table.push_str(&format!(
        "\n**{} différées sur {} spécifiées au §7.2.**\n",
        DEFERRED.len(),
        reports.len() + DEFERRED.len()
    ));

    println!("\n{table}");

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../validation-data/electrical-model-property-matrix.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the validation-data directory must be creatable");
    }
    std::fs::write(&path, &table).expect("the property matrix must be writable");

    // The written table is the evidence, so it must be self-consistent.
    assert_eq!(failed, 0, "the matrix records {failed} failing case(s)");
    assert_eq!(
        reports.len() + DEFERRED.len(),
        22,
        "the specification lists twenty-two properties at §7.2"
    );
    for report in &reports {
        assert!(
            report.cases >= MINIMUM_CASES_PER_PROPERTY,
            "{} ran {} case(s)",
            report.id,
            report.cases
        );
    }
}

/// The registry must not contain a duplicate or a malformed identifier,
/// and must not overlap the deferred list.
#[test]
fn the_property_inventory_is_coherent() {
    let properties = registry();
    let mut ids: Vec<&str> = properties
        .iter()
        .map(|p| p.id)
        .chain(DEFERRED.iter().map(|d| d.id))
        .collect();
    assert_eq!(ids.len(), 22);

    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), before, "an identifier appears twice");

    for id in &ids {
        assert!(id.starts_with("PB-"), "{id} is malformed");
        assert_eq!(id.len(), 5, "{id} is malformed");
    }

    // Every implemented property must state what it translates; a blank
    // would let a proved property be tested as if it were folklore.
    for property in &properties {
        assert!(
            !property.translates.is_empty(),
            "{} states nothing",
            property.id
        );
        assert!(
            !property.statement.is_empty(),
            "{} states nothing",
            property.id
        );
    }
}
