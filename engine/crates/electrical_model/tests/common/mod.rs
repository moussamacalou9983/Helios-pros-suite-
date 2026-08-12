//! Fixtures and a deterministic generator, shared by the invariant and
//! property suites.
//!
//! **The parameter sets are synthetic and documented as such.** None
//! claims to describe a real product. The nine reference sets are those
//! used for the numerical corroboration of
//! `docs/design/34-audit-mathematique-electrical-model.md` §16.

#![allow(dead_code)]

use core_units::{Current, Resistance, Voltage};
use electrical_model::single_diode::{
    ModifiedIdealityFactor, SingleDiodeParameters, current_at_voltage, residual,
};

/// Build a parameter set from bare SI magnitudes.
///
/// # Panics
///
/// Panics if the set violates `𝓗` — the fixtures are all inside it by
/// construction, so a panic here is a defect in the fixture.
pub fn set(i_ph: f64, i_0: f64, r_s: f64, r_sh: f64, a: f64) -> SingleDiodeParameters {
    SingleDiodeParameters::new(
        Current::from_amperes(i_ph),
        Current::from_amperes(i_0),
        Resistance::from_ohms(r_s),
        Resistance::from_ohms(r_sh),
        ModifiedIdealityFactor::from_volts(a),
    )
    .expect("the fixtures satisfy H by construction")
}

/// T-1 — the nominal 72-cell reference.
pub fn nominal() -> SingleDiodeParameters {
    set(10.4, 2.5e-10, 0.30, 800.0, 1.85)
}

/// The nine reference sets of the audit, with their names.
pub fn reference_sets() -> Vec<(&'static str, SingleDiodeParameters)> {
    vec![
        (
            "T-1 nominal 72 cells",
            set(10.4, 2.5e-10, 0.30, 800.0, 1.85),
        ),
        ("T-2 R_s = 0", set(10.4, 2.5e-10, 0.0, 800.0, 1.85)),
        ("T-3 R_s -> 0", set(10.4, 2.5e-10, 1e-9, 800.0, 1.85)),
        ("T-4 R_s high", set(10.4, 2.5e-10, 2.0, 800.0, 1.85)),
        ("T-5 R_sh -> inf", set(10.4, 2.5e-10, 0.30, 1e12, 1.85)),
        ("T-6 R_sh low", set(10.4, 2.5e-10, 0.30, 5.0, 1.85)),
        ("T-7 R_sh very low", set(10.4, 2.5e-10, 0.30, 1.5, 1.85)),
        (
            "T-8 very low irradiance",
            set(0.0104, 2.5e-10, 0.30, 8e5, 1.85),
        ),
        ("T-9 thin film", set(1.9, 1e-11, 3.0, 2000.0, 4.0)),
    ]
}

/// A voltage safely inside the operating quadrant, as a fraction of the
/// closed-form open-circuit bound.
///
/// The bound majorises `V_oc`, so a fraction of it is not guaranteed to be
/// admissible for every set — the caller must still handle a refusal. This
/// helper exists to sweep the quadrant, not to assume it.
pub fn voltage_fraction(params: &SingleDiodeParameters, fraction: f64) -> Voltage {
    Voltage::from_volts(params.open_circuit_voltage_bound().volts() * fraction)
}

/// The largest admissible voltage, located by bisection on the domain
/// predicate `f(0; V) >= 0`.
///
/// **An independent reference**: it uses only the residual, never the
/// solver, so it can be used to check the solver's own domain test.
pub fn open_circuit_voltage(params: &SingleDiodeParameters) -> Voltage {
    let mut low = 0.0;
    let mut high = params.open_circuit_voltage_bound().volts();
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if residual(Current::from_amperes(0.0), Voltage::from_volts(mid), params) >= 0.0 {
            low = mid;
        } else {
            high = mid;
        }
    }
    Voltage::from_volts(low)
}

/// Sweep the operating quadrant of a set, calling `f` at each admissible
/// voltage.
///
/// The fraction is formed **before** multiplying by `V_oc`, and that
/// ordering is load-bearing. Written the other way round,
/// `(v_oc * k) / points` rounds twice and at `k = points` can land one ULP
/// *above* `V_oc`, outside the closed domain `[0, V_oc]` — the sweep would
/// then be testing a refusal rather than the invariant. With the fraction
/// first, `k / points` is exactly `1.0` at the endpoint (IEEE 754
/// division of a value by itself), so the last point is `V_oc` itself, and
/// for `k < points` the fraction is strictly below one, which makes the
/// rounded product no greater than `V_oc`.
pub fn sweep(params: &SingleDiodeParameters, points: u32, mut f: impl FnMut(Voltage)) {
    let v_oc = open_circuit_voltage(params).volts();
    for k in 0..=points {
        f(Voltage::from_volts(
            v_oc * (f64::from(k) / f64::from(points)),
        ));
    }
}

/// A deterministic generator for property-based tests.
///
/// SplitMix64 (Steele, Lea & Flood, *Fast splittable pseudorandom number
/// generators*, OOPSLA 2014; public-domain reference implementation by
/// Vigna). Chosen because it is a dozen lines, has no state beyond a
/// `u64`, and needs no external crate — the engine's zero-dependency
/// policy applies to test code as well.
///
/// **The seed is printed by every suite that uses it**, so any failure is
/// reproducible exactly.
pub struct Generator {
    state: u64,
}

impl Generator {
    /// Start from a seed.
    pub const fn new(seed: u64) -> Self {
        Generator { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value uniform on `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        // 53 bits of mantissa, the most a f64 can carry exactly.
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A value uniform on `[low, high]`.
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }

    /// A value log-uniform on `[low, high]`, both strictly positive.
    ///
    /// Used for quantities spanning many orders of magnitude — `I_0` runs
    /// from 1e-15 to 1e-6 — where a uniform draw would sample only the
    /// top decade.
    pub fn log_uniform(&mut self, low: f64, high: f64) -> f64 {
        (self.uniform(low.ln(), high.ln())).exp()
    }

    /// Draw a parameter set from the admissible domain `𝓗`.
    ///
    /// Ranges per `docs/design/36-statut-epistemique-electrical-model.md`
    /// §7.1. `R_s = 0` is drawn explicitly one time in sixteen, so the
    /// exact-predicate branch is exercised by the generator rather than
    /// only by hand-written cases.
    ///
    /// Sets violating the overflow guard are **rejected at generation**:
    /// the properties are proved on the domain where they hold, and
    /// testing them elsewhere would test nothing.
    pub fn admissible_parameters(&mut self) -> SingleDiodeParameters {
        self.draw_parameters(None)
    }

    /// Draw from `𝓗` with the series resistance **imposed**.
    ///
    /// Used by the properties that are about a specific branch — `R_s = 0`
    /// for the closed form, a fixed small `R_s` for the general one.
    /// Rejection sampling on the full generator would spend fifteen draws
    /// in sixteen to reach the zero case, and would not reach a chosen
    /// non-zero value at all.
    pub fn admissible_parameters_with_series(&mut self, r_s: f64) -> SingleDiodeParameters {
        self.draw_parameters(Some(r_s))
    }

    fn draw_parameters(&mut self, imposed_series: Option<f64>) -> SingleDiodeParameters {
        loop {
            let i_ph = self.log_uniform(1e-3, 30.0);
            let i_0 = self.log_uniform(1e-15, 1e-6);
            let r_s = match imposed_series {
                Some(value) => value,
                None if self.next_u64().is_multiple_of(16) => 0.0,
                None => self.uniform(0.0, 5.0),
            };
            let r_sh = self.log_uniform(1.0, 1e12);
            let a = self.uniform(0.5, 10.0);
            if let Ok(p) = SingleDiodeParameters::new(
                Current::from_amperes(i_ph),
                Current::from_amperes(i_0),
                Resistance::from_ohms(r_s),
                Resistance::from_ohms(r_sh),
                ModifiedIdealityFactor::from_volts(a),
            ) {
                return p;
            }
        }
    }

    /// Draw a tuple that is **outside** `𝓗`, for the refusal properties.
    pub fn inadmissible_tuple(&mut self) -> (f64, f64, f64, f64, f64) {
        let mut i_ph = self.log_uniform(1e-3, 30.0);
        let mut i_0 = self.log_uniform(1e-15, 1e-6);
        let mut r_s = self.uniform(0.0, 5.0);
        let mut r_sh = self.log_uniform(1.0, 1e12);
        let mut a = self.uniform(0.5, 10.0);
        let bad =
            [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY][(self.next_u64() % 5) as usize];
        match self.next_u64() % 5 {
            0 => i_ph = bad,
            1 => i_0 = bad,
            2 => r_s = if bad == 0.0 { -1.0 } else { bad },
            3 => r_sh = bad,
            _ => a = bad,
        }
        (i_ph, i_0, r_s, r_sh, a)
    }
}

/// A one-line description of a set, for failure messages.
///
/// A property failure is only actionable if the exact draw can be
/// replayed: the seed gives the stream, this gives the case.
pub fn describe(params: &SingleDiodeParameters) -> String {
    format!(
        "I_ph = {:e} A, I_0 = {:e} A, R_s = {:e} ohm, R_sh = {:e} ohm, a = {:e} V",
        params.photocurrent().amperes(),
        params.saturation_current().amperes(),
        params.series_resistance().ohms(),
        params.shunt_resistance().ohms(),
        params.modified_ideality_factor().volts()
    )
}

/// Solve and return the current in amperes, or panic with the reason.
///
/// # Panics
///
/// Panics if the solve fails; used only where the domain has already been
/// established, so a failure is a defect rather than an expected outcome.
#[track_caller]
pub fn solve(params: &SingleDiodeParameters, v: Voltage) -> f64 {
    current_at_voltage(params, v)
        .unwrap_or_else(|e| panic!("solve failed at V = {} V: {e}", v.volts()))
        .current()
        .amperes()
}
