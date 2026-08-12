use crate::{Current, Voltage};

/// Electrical resistance, stored internally in ohms (Ω), the SI derived
/// unit.
///
/// Required by the single-diode model, whose series and shunt resistances
/// are both in ohms and both appear in expressions where confusing them
/// with any other quantity would be silent
/// (`docs/design/35-formalisation-mathematique-electrical-model.md` §1.3).
///
/// **Sign convention**: a resistance is non-negative. This type does not
/// enforce it, because the single-diode model must be able to *hold* a
/// negative value long enough to **reject** it with a diagnostic — the
/// same reasoning that keeps `fill_factor` a bare `f64` in `pv_module`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Resistance(f64);

impl Resistance {
    /// Construct a resistance from a value in ohms.
    #[inline]
    pub const fn from_ohms(ohms: f64) -> Self {
        Resistance(ohms)
    }

    /// Construct a resistance from a value in milliohms.
    #[inline]
    pub fn from_milliohms(milliohms: f64) -> Self {
        Resistance(milliohms / 1000.0)
    }

    /// The resistance's value in ohms.
    #[inline]
    pub const fn ohms(self) -> f64 {
        self.0
    }

    /// The resistance's value in milliohms.
    #[inline]
    pub fn milliohms(self) -> f64 {
        self.0 * 1000.0
    }
}

/// Ohm's law: `R × I = V`.
///
/// Typed so that the dimensional identity `Ω · A = V`, on which the
/// homogeneity of the single-diode equation rests, is enforced by the
/// compiler rather than checked by eye.
impl core::ops::Mul<Current> for Resistance {
    type Output = Voltage;

    #[inline]
    fn mul(self, current: Current) -> Voltage {
        Voltage::from_volts(self.0 * current.amperes())
    }
}

/// Ohm's law, written the other way round.
impl core::ops::Mul<Resistance> for Current {
    type Output = Voltage;

    #[inline]
    fn mul(self, resistance: Resistance) -> Voltage {
        Voltage::from_volts(self.amperes() * resistance.ohms())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milliohm_roundtrip() {
        let r = Resistance::from_milliohms(300.0);
        assert!((r.ohms() - 0.3).abs() < 1e-12);
        assert!((r.milliohms() - 300.0).abs() < 1e-9);
    }

    #[test]
    fn ohms_law_yields_volts_in_both_orders() {
        let r = Resistance::from_ohms(0.3);
        let i = Current::from_amperes(10.4);
        let expected = 3.12;
        assert!(((r * i).volts() - expected).abs() < 1e-12);
        assert!(((i * r).volts() - expected).abs() < 1e-12);
    }

    #[test]
    fn a_negative_resistance_is_representable_so_it_can_be_rejected() {
        // The single-diode model must hold a non-physical value long
        // enough to refuse it with a diagnostic naming the parameter.
        let r = Resistance::from_ohms(-800.0);
        assert!(r.ohms() < 0.0);
    }

    #[test]
    fn zero_resistance_is_representable() {
        // R_s = 0 is admissible and has a closed-form solution; it must
        // not be a construction error.
        assert_eq!(Resistance::from_ohms(0.0).ohms(), 0.0);
    }
}
