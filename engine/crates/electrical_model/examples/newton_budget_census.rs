//! Measures how many iterations the Newton recurrence of equation (11)
//! actually needs, and how often the budget of thirty is not enough.
//!
//! This example exists because a number quoted in the documentation must
//! be reproducible. The observation it supports —
//!
//! > over the declared generation domain, 0.642 % of admissible sets need
//! > more than thirty Newton iterations, the worst needing 218
//!
//! — is an **[O]**, and an [O] that cannot be re-measured is folklore.
//! Run it with:
//!
//! ```text
//! cargo run --release -p electrical_model --example newton_budget_census
//! ```
//!
//! ## What it also checks
//!
//! Deep in the exponential regime the residual is `f ≈ −I_0 e^u` and its
//! derivative `f' ≈ −(R_s I_0/a) e^u`, so the Newton step `−f/f'` is
//! `−a/R_s`: a constant, independent of `I_0`, of `V`, and of the
//! distance still to travel. The descent from `I₁ ≈ I_ph` to `I*` should
//! then cost about `(I_ph − I*)·R_s/a` iterations. The example prints the
//! measured count beside that prediction so the explanation is checked
//! rather than asserted.
//!
//! The domain swept is the one declared in
//! `docs/design/36-statut-epistemique-electrical-model.md` §7.1, whose
//! `R_s` range reaches 5 Ω — far beyond any real module, which is
//! precisely why the phenomenon shows here and not on the nine reference
//! sets.

use core_units::{Current, Resistance, Voltage};
use electrical_model::single_diode::{
    ModifiedIdealityFactor, NEWTON_ITERATION_BUDGET, SingleDiodeParameters, residual,
    residual_derivative, residual_tolerance,
};

/// SplitMix64, as in the test suites — same generator, same reason.
struct Generator(u64);

impl Generator {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }

    fn log_uniform(&mut self, low: f64, high: f64) -> f64 {
        self.uniform(low.ln(), high.ln()).exp()
    }
}

/// A ceiling far above the budget, so that the measurement is not itself
/// truncated by the quantity it is measuring.
const MEASUREMENT_CEILING: u32 = 4_000;

/// The number of admissible sets to draw.
const DRAWS: u32 = 200_000;

fn main() {
    let mut generator = Generator(0x4845_4C49_4F53_5052);
    let mut examined = 0_u32;
    let mut over_budget = 0_u32;
    let mut worst = 0_u32;
    let mut worst_case = String::new();
    let mut histogram = [0_u32; 8];

    while examined < DRAWS {
        let i_ph = generator.log_uniform(1e-3, 30.0);
        let i_0 = generator.log_uniform(1e-15, 1e-6);
        let r_s = generator.uniform(0.0, 5.0);
        let r_sh = generator.log_uniform(1.0, 1e12);
        let a = generator.uniform(0.5, 10.0);

        let Ok(params) = SingleDiodeParameters::new(
            Current::from_amperes(i_ph),
            Current::from_amperes(i_0),
            Resistance::from_ohms(r_s),
            Resistance::from_ohms(r_sh),
            ModifiedIdealityFactor::from_volts(a),
        ) else {
            // Refused by the overflow guard: outside the domain where the
            // properties are proved, so outside what is being measured.
            continue;
        };
        examined += 1;

        // Inside the quadrant by construction: the closed-form bound
        // majorises `V_oc`, and the residual test below confirms it.
        let v = Voltage::from_volts(params.open_circuit_voltage_bound().volts() * generator.unit());
        if residual(Current::from_amperes(0.0), v, &params) < 0.0 {
            continue;
        }

        let tolerance = residual_tolerance(&params);
        let mut iterate = 0.0_f64;
        let mut iterations = 0_u32;
        loop {
            let f = residual(Current::from_amperes(iterate), v, &params);
            if !f.is_finite() || f.abs() <= tolerance || iterations >= MEASUREMENT_CEILING {
                break;
            }
            let slope = residual_derivative(Current::from_amperes(iterate), v, &params);
            if !slope.is_finite() || slope >= 0.0 {
                break;
            }
            iterate -= f / slope;
            iterations += 1;
        }

        histogram[(iterations as usize / 10).min(7)] += 1;
        if iterations > NEWTON_ITERATION_BUDGET {
            over_budget += 1;
        }
        if iterations > worst {
            worst = iterations;
            // `iterate` is the converged root, so the prediction can use
            // the true distance travelled rather than its upper bound.
            worst_case = format!(
                "I_ph = {i_ph:.4e} A, I_0 = {i_0:.4e} A, R_s = {r_s:.4e} ohm, \
                 R_sh = {r_sh:.4e} ohm, a = {a:.4e} V, V = {:.4} V\n    \
                 predicted (I_ph - I*)*R_s/a = {:.1}, a priori cap I_ph*R_s/a = {:.1}",
                v.volts(),
                (i_ph - iterate) * r_s / a,
                i_ph * r_s / a
            );
        }
    }

    println!("Newton iteration census — recurrence of equation (11), from I_0 = 0");
    println!("  domain          : section 7.1, R_s in (0, 5] ohm");
    println!("  admissible sets : {examined}");
    println!(
        "  above the budget: {over_budget} ({:.3} %) of {NEWTON_ITERATION_BUDGET} iterations",
        100.0 * f64::from(over_budget) / f64::from(examined)
    );
    println!("  worst count     : {worst}");
    println!("  worst case      : {worst_case}");
    println!(
        "  histogram       : 0-9 {}, 10-19 {}, 20-29 {}, 30-39 {}, 40-49 {}, 50-59 {}, 60-69 {}, 70+ {}",
        histogram[0],
        histogram[1],
        histogram[2],
        histogram[3],
        histogram[4],
        histogram[5],
        histogram[6],
        histogram[7]
    );
}
