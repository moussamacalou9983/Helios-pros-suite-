//! Typed failures.
//!
//! Every way this crate can decline to produce a number has its own
//! variant, carrying what a caller needs to act on it. **No path in the
//! engine uses `unwrap` or `expect`**, and no failure is signalled by a
//! sentinel value: a `NaN` crossing the crate boundary is a defect, never
//! a result.

use core_units::{Current, Voltage};

/// Which parameter of the single-diode model was rejected, and why.
///
/// Traces to the hypothesis set `𝓗` of
/// `docs/design/35-formalisation-mathematique-electrical-model.md` §3.1.
/// Every proof in that document is conditioned on `𝓗`; outside it nothing
/// is guaranteed, so nothing is attempted.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ParameterFault {
    /// `I_ph ≤ 0`, or not finite. Hypothesis H-1.
    Photocurrent,
    /// `I_0 ≤ 0`, or not finite. Hypothesis H-2.
    SaturationCurrent,
    /// `a ≤ 0`, or not finite. Hypothesis H-3.
    ModifiedIdealityFactor,
    /// `R_sh ≤ 0`, or not finite. Hypothesis H-4.
    ShuntResistance,
    /// `R_s < 0`, or not finite. Hypothesis H-5.
    SeriesResistance,
    /// `N_s < 1`. Hypothesis H-6.
    CellsInSeries,
    /// `T_c ≤ 0` in kelvins. Hypothesis H-7.
    CellTemperature,
    /// `n ≤ 0`, or not finite.
    DiodeIdealityFactor,
}

impl ParameterFault {
    /// The parameter's name, as it appears in the formalisation.
    pub const fn parameter(self) -> &'static str {
        match self {
            ParameterFault::Photocurrent => "I_ph",
            ParameterFault::SaturationCurrent => "I_0",
            ParameterFault::ModifiedIdealityFactor => "a",
            ParameterFault::ShuntResistance => "R_sh",
            ParameterFault::SeriesResistance => "R_s",
            ParameterFault::CellsInSeries => "N_s",
            ParameterFault::CellTemperature => "T_c",
            ParameterFault::DiodeIdealityFactor => "n",
        }
    }

    /// The hypothesis of `𝓗` this fault violates.
    pub const fn hypothesis(self) -> &'static str {
        match self {
            ParameterFault::Photocurrent => "H-1: I_ph > 0 and finite",
            ParameterFault::SaturationCurrent => "H-2: I_0 > 0 and finite",
            ParameterFault::ModifiedIdealityFactor => "H-3: a > 0 and finite",
            ParameterFault::ShuntResistance => "H-4: R_sh > 0 and finite",
            ParameterFault::SeriesResistance => "H-5: R_s >= 0 and finite",
            ParameterFault::CellsInSeries => "H-6: N_s >= 1",
            ParameterFault::CellTemperature => "H-7: T_c > 0 kelvin",
            ParameterFault::DiodeIdealityFactor => "n > 0 and finite",
        }
    }
}

/// Why this crate declined to produce a number.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ElectricalModelError {
    /// A parameter lies outside the hypothesis set `𝓗`.
    ///
    /// **Refused before any computation.** The proofs hold only under
    /// `𝓗`; applying the algorithm outside it would not raise a
    /// detectable error — it would return a plausible, wrong number.
    InvalidParameters {
        /// Which parameter, and which hypothesis it violates.
        fault: ParameterFault,
        /// The offending value, for the diagnostic.
        value: f64,
    },

    /// The parameter set would make the exponential overflow `f64`.
    ///
    /// Detected **a priori** by the closed-form guard of
    /// [`crate::single_diode::SingleDiodeParameters::exponent_headroom`],
    /// never by catching an overflow after the fact.
    ///
    /// For a self-consistent parameter set `V_oc/a ≈ ln(I_ph/I_0) ≈ 25`,
    /// so this is a symptom of incoherent parameters rather than an
    /// intrinsic numerical difficulty.
    NumericalOverflow {
        /// The guard's value, `ln(I_0 + I_ph) + max(0, ln(R_s/a)) + I_ph·R_s/a`.
        guard: f64,
        /// The IEEE 754 binary64 limit it must stay below, `ln(f64::MAX)`.
        limit: f64,
    },

    /// The requested voltage lies outside the specified operating
    /// quadrant `[0, V_oc]`.
    ///
    /// Reverse bias belongs to mismatch and shading, which are out of
    /// scope; the bracket of §3.4 is proved only on this quadrant.
    OutOfDomain {
        /// The voltage that was requested.
        requested: Voltage,
        /// The upper bound of the specified domain.
        upper_bound: Voltage,
    },

    /// Neither the primary iteration nor the fallback reached the
    /// tolerance within its bound.
    ///
    /// Carries the diagnostic needed to understand why, rather than a
    /// best-effort value that would be indistinguishable from a converged
    /// one.
    NonConvergence {
        /// Iterations spent before giving up.
        iterations: u32,
        /// The residual still outstanding, in amperes.
        residual: f64,
        /// The tolerance that was being sought, in amperes.
        tolerance: f64,
        /// The last iterate, for diagnosis only — **not** a result.
        last_iterate: Current,
    },

    /// A quantity that must be finite was not.
    ///
    /// Should be unreachable given the guards; it exists so that the
    /// invariant "no `NaN` or infinity ever crosses the boundary" is
    /// enforced rather than assumed.
    NonFiniteValue {
        /// What was being computed.
        quantity: &'static str,
        /// The offending value.
        value: f64,
    },
}

impl core::fmt::Display for ElectricalModelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ElectricalModelError::InvalidParameters { fault, value } => write!(
                f,
                "{} = {value} violates {}",
                fault.parameter(),
                fault.hypothesis()
            ),
            ElectricalModelError::NumericalOverflow { guard, limit } => write!(
                f,
                "the parameter set would overflow the exponential: guard = {guard:.2} exceeds the \
                 IEEE 754 binary64 limit ln(f64::MAX) = {limit:.2}; for a self-consistent set \
                 V_oc/a is around 25, so this indicates incoherent parameters"
            ),
            ElectricalModelError::OutOfDomain {
                requested,
                upper_bound,
            } => write!(
                f,
                "voltage {} V is outside the specified operating quadrant [0, {} V]",
                requested.volts(),
                upper_bound.volts()
            ),
            ElectricalModelError::NonConvergence {
                iterations,
                residual,
                tolerance,
                ..
            } => write!(
                f,
                "no convergence after {iterations} iteration(s): residual {residual:.3e} A still \
                 above tolerance {tolerance:.3e} A"
            ),
            ElectricalModelError::NonFiniteValue { quantity, value } => {
                write!(f, "{quantity} evaluated to {value}, which is not finite")
            }
        }
    }
}

impl core::error::Error for ElectricalModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_parameter_fault_names_its_parameter_and_hypothesis() {
        let faults = [
            ParameterFault::Photocurrent,
            ParameterFault::SaturationCurrent,
            ParameterFault::ModifiedIdealityFactor,
            ParameterFault::ShuntResistance,
            ParameterFault::SeriesResistance,
            ParameterFault::CellsInSeries,
            ParameterFault::CellTemperature,
            ParameterFault::DiodeIdealityFactor,
        ];
        for fault in faults {
            assert!(!fault.parameter().is_empty());
            assert!(fault.hypothesis().contains(fault.parameter()));
        }
    }

    #[test]
    fn errors_describe_themselves_usefully() {
        let e = ElectricalModelError::InvalidParameters {
            fault: ParameterFault::ShuntResistance,
            value: -800.0,
        };
        let text = e.to_string();
        assert!(text.contains("R_sh"));
        assert!(text.contains("-800"));

        let e = ElectricalModelError::NumericalOverflow {
            guard: 942.08,
            limit: 709.78,
        };
        assert!(e.to_string().contains("incoherent parameters"));

        let e = ElectricalModelError::OutOfDomain {
            requested: Voltage::from_volts(60.0),
            upper_bound: Voltage::from_volts(45.2),
        };
        assert!(e.to_string().contains("operating quadrant"));

        // `NonFiniteValue` is raised only by guards the mathematics says
        // are unreachable under `𝓗`. That is exactly why its message is
        // tested here: the one occasion it would ever be read is an
        // occasion when something already thought impossible has happened,
        // and a message that had never been exercised would fail then.
        let e = ElectricalModelError::NonFiniteValue {
            quantity: "residual at I = 0",
            value: f64::NAN,
        };
        let text = e.to_string();
        assert!(text.contains("residual at I = 0"), "{text}");
        assert!(text.contains("NaN"), "{text}");
    }

    #[test]
    fn non_convergence_carries_its_diagnostic_not_a_result() {
        let e = ElectricalModelError::NonConvergence {
            iterations: 82,
            residual: 1e-6,
            tolerance: 1e-11,
            last_iterate: Current::from_amperes(9.9),
        };
        let text = e.to_string();
        assert!(text.contains("82"));
        // The last iterate is available for diagnosis but is not presented
        // as an answer.
        assert!(!text.contains("9.9"));
    }
}
