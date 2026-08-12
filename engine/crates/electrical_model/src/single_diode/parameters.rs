use core_constants::BOLTZMANN_EV_PER_K;
use core_units::{Current, Resistance, Temperature, Voltage};

use crate::error::{ElectricalModelError, ParameterFault};

/// The modified ideality factor `a = n·N_s·k·T_c/q`, in volts.
///
/// **It is expressed in volts without being a voltage of the module.**
/// `a + V_oc` is a syntactically valid, physically meaningless expression,
/// which is why this is a distinct type rather than a
/// [`core_units::Voltage`] (ADR-0009 D-2, restated in
/// `docs/design/35-formalisation-mathematique-electrical-model.md` §1.4).
///
/// `n` is not measurable by any instrument: `a` is the quantity a model
/// identifies, and `n` is derived from it. Identifying `n` separately
/// would create a second route to the same quantity, hence an opportunity
/// to diverge.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ModifiedIdealityFactor(f64);

impl ModifiedIdealityFactor {
    /// Construct from a value in volts, already the product
    /// `n·N_s·k·T_c/q`.
    ///
    /// No validity check here: a non-physical value must survive long
    /// enough for [`SingleDiodeParameters::new`] to **reject** it with a
    /// diagnostic naming the parameter.
    #[inline]
    pub const fn from_volts(volts: f64) -> Self {
        ModifiedIdealityFactor(volts)
    }

    /// Build from the diode ideality factor, the cell count and the cell
    /// temperature — equation (2) of the formalisation.
    ///
    /// `a = n · N_s · k · T_c / q`, with `k/q` taken from
    /// [`core_constants::BOLTZMANN_EV_PER_K`], which is **derived** from
    /// the two exact SI constants rather than transcribed.
    ///
    /// **`temperature` is a cell temperature.** The distinction between
    /// cell and ambient is carried upstream by
    /// `pv_module::TemperatureSpecification`; this function cannot
    /// re-check it, so it is stated here.
    ///
    /// # Errors
    ///
    /// Refuses rather than returns, on any of three hypotheses:
    ///
    /// - `n > 0` and finite — a non-positive ideality factor has no
    ///   physical reading, and a zero one would produce `a = 0`, which
    ///   [`SingleDiodeParameters::new`] would then have to refuse further
    ///   downstream and less informatively;
    /// - **H-6**, `N_s ≥ 1` — a module has at least one cell, and `N_s = 0`
    ///   would silently yield `a = 0`;
    /// - **H-7**, `T_c > 0` in kelvins — the thermal voltage is
    ///   proportional to it, and a non-positive absolute temperature is not
    ///   a cold module but a unit error, most often celsius passed as
    ///   kelvin.
    ///
    /// Each is a declared hypothesis of `𝓗` with its own invariant
    /// (`INV-EM-DOM-06`, `INV-EM-DOM-07`). Returning a value here and
    /// leaving the check to the constructor would report the wrong
    /// parameter: the caller would be told `a ≤ 0` when what they got
    /// wrong was the cell count.
    pub fn from_diode(
        n: f64,
        cells_in_series: u32,
        cell_temperature: Temperature,
    ) -> Result<Self, ElectricalModelError> {
        if !n.is_finite() || n <= 0.0 {
            return Err(ElectricalModelError::InvalidParameters {
                fault: ParameterFault::DiodeIdealityFactor,
                value: n,
            });
        }
        if cells_in_series < 1 {
            return Err(ElectricalModelError::InvalidParameters {
                fault: ParameterFault::CellsInSeries,
                value: f64::from(cells_in_series),
            });
        }
        let kelvin = cell_temperature.kelvin();
        if !kelvin.is_finite() || kelvin <= 0.0 {
            return Err(ElectricalModelError::InvalidParameters {
                fault: ParameterFault::CellTemperature,
                value: kelvin,
            });
        }
        Ok(ModifiedIdealityFactor(
            n * f64::from(cells_in_series) * Self::thermal_voltage(cell_temperature),
        ))
    }

    /// The value in volts.
    #[inline]
    pub const fn volts(self) -> f64 {
        self.0
    }

    /// The thermal voltage per cell, `k·T_c/q`, in volts.
    ///
    /// The single place this product is written. [`Self::from_diode`] and
    /// [`Self::diode_ideality`] both go through it, so the two directions
    /// of equation (2) cannot drift apart — the same reason
    /// [`core_constants::BOLTZMANN_EV_PER_K`] is derived rather than
    /// transcribed.
    #[inline]
    pub fn thermal_voltage(cell_temperature: Temperature) -> f64 {
        BOLTZMANN_EV_PER_K * cell_temperature.kelvin()
    }

    /// Recover the diode ideality factor `n` from `a` and the cell count.
    ///
    /// Returns `None` when the denominator vanishes, rather than yielding
    /// an infinity.
    #[inline]
    pub fn diode_ideality(
        self,
        cells_in_series: u32,
        cell_temperature: Temperature,
    ) -> Option<f64> {
        let denominator = f64::from(cells_in_series) * Self::thermal_voltage(cell_temperature);
        if denominator == 0.0 || !denominator.is_finite() {
            None
        } else {
            Some(self.0 / denominator)
        }
    }
}

/// The natural logarithm of `f64::MAX`, the point beyond which
/// `exp` overflows in IEEE 754 binary64.
///
/// **Not a chosen threshold**: `ln(2^1024) ≈ 709.783`, imposed by the
/// format. Used by [`SingleDiodeParameters::exponent_headroom`].
pub const LN_F64_MAX: f64 = 709.782_712_893_384;

/// The five parameters of the single-diode model, validated against the
/// hypothesis set `𝓗`.
///
/// **Construction is the validation gate.** A value of this type is known
/// to satisfy `𝓗`, which is what every proof in
/// `docs/design/35-formalisation-mathematique-electrical-model.md`
/// is conditioned on. Outside `𝓗` nothing is guaranteed, so nothing is
/// attempted: the algorithm would not raise a detectable error, it would
/// return a plausible and wrong number.
///
/// All five are **fitted** quantities (class F in `pv_module`'s
/// classification): none is measurable on a real module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SingleDiodeParameters {
    photocurrent: Current,
    saturation_current: Current,
    series_resistance: Resistance,
    shunt_resistance: Resistance,
    modified_ideality_factor: ModifiedIdealityFactor,
}

impl SingleDiodeParameters {
    /// Validate against `𝓗` and construct.
    ///
    /// # Errors
    ///
    /// [`ElectricalModelError::InvalidParameters`] naming the first
    /// parameter that violates its hypothesis, then
    /// [`ElectricalModelError::NumericalOverflow`] if the set would
    /// overflow the exponential.
    ///
    /// The order matters: the overflow guard divides by `a` and takes the
    /// logarithm of `I_0 + I_ph`, so it is only meaningful once `𝓗` holds.
    pub fn new(
        photocurrent: Current,
        saturation_current: Current,
        series_resistance: Resistance,
        shunt_resistance: Resistance,
        modified_ideality_factor: ModifiedIdealityFactor,
    ) -> Result<Self, ElectricalModelError> {
        // H-1 to H-5, in the order of the formalisation.
        let checks: [(ParameterFault, f64, bool); 5] = [
            (
                ParameterFault::Photocurrent,
                photocurrent.amperes(),
                photocurrent.amperes() > 0.0,
            ),
            (
                ParameterFault::SaturationCurrent,
                saturation_current.amperes(),
                saturation_current.amperes() > 0.0,
            ),
            (
                ParameterFault::SeriesResistance,
                series_resistance.ohms(),
                series_resistance.ohms() >= 0.0,
            ),
            (
                ParameterFault::ShuntResistance,
                shunt_resistance.ohms(),
                shunt_resistance.ohms() > 0.0,
            ),
            (
                ParameterFault::ModifiedIdealityFactor,
                modified_ideality_factor.volts(),
                modified_ideality_factor.volts() > 0.0,
            ),
        ];
        for (fault, value, positive) in checks {
            if !value.is_finite() || !positive {
                return Err(ElectricalModelError::InvalidParameters { fault, value });
            }
        }

        let candidate = SingleDiodeParameters {
            photocurrent,
            saturation_current,
            series_resistance,
            shunt_resistance,
            modified_ideality_factor,
        };

        let guard = candidate.exponent_headroom();
        // Written as "not admissible" rather than "admissible", and the
        // `is_nan` arm is what makes the two different. `guard >= LIMIT`
        // alone is false for a `NaN`, which would let an uncomputable
        // headroom through as if it had been checked. The five parameters
        // have already been established finite above, so a `NaN` here is
        // unreachable — the refusal exists so that it stays unreachable
        // rather than becoming a silent acceptance if it ever is not.
        if guard.is_nan() || guard >= LN_F64_MAX {
            return Err(ElectricalModelError::NumericalOverflow {
                guard,
                limit: LN_F64_MAX,
            });
        }

        Ok(candidate)
    }

    /// Photocurrent `I_ph`.
    #[inline]
    pub const fn photocurrent(&self) -> Current {
        self.photocurrent
    }

    /// Diode reverse saturation current `I_0`.
    #[inline]
    pub const fn saturation_current(&self) -> Current {
        self.saturation_current
    }

    /// Series resistance `R_s`.
    #[inline]
    pub const fn series_resistance(&self) -> Resistance {
        self.series_resistance
    }

    /// Shunt resistance `R_sh`.
    #[inline]
    pub const fn shunt_resistance(&self) -> Resistance {
        self.shunt_resistance
    }

    /// Modified ideality factor `a`.
    #[inline]
    pub const fn modified_ideality_factor(&self) -> ModifiedIdealityFactor {
        self.modified_ideality_factor
    }

    /// Whether the series resistance is **exactly** zero.
    ///
    /// An exact predicate, not a threshold. There is no singularity at
    /// `R_s = 0` — the general branch never divides by `R_s` — so no
    /// epsilon is required, and none is used
    /// (`docs/design/35-formalisation-mathematique-electrical-model.md` §8.2).
    #[inline]
    pub fn has_zero_series_resistance(&self) -> bool {
        self.series_resistance.ohms() == 0.0
    }

    /// A closed-form upper bound on the open-circuit voltage.
    ///
    /// At `I = 0` the residual gives
    /// `I_0[e^{V_oc/a} − 1] + V_oc/R_sh = I_ph`. Dropping the
    /// non-negative shunt term can only increase the root, so
    ///
    /// ```text
    /// V_oc  ≤  a · ln(1 + I_ph / I_0)
    /// ```
    ///
    /// **This bound is what makes the overflow guard computable without
    /// solving anything.** The formalisation states the guard in terms of
    /// `V_oc` (§7.2); this bound removes the circularity of needing a
    /// solve before being allowed to solve. It is a derivation from the
    /// specified inequality, not a change to it.
    ///
    /// Verified to be a genuine majorant on all nine reference parameter
    /// sets, tight to 0.02 % for coherent ones and conservative (up to
    /// 2.9×) when `R_sh` is small — conservative in the safe direction.
    #[inline]
    pub fn open_circuit_voltage_bound(&self) -> Voltage {
        Voltage::from_volts(
            self.modified_ideality_factor.volts()
                * (self.photocurrent.amperes() / self.saturation_current.amperes()).ln_1p(),
        )
    }

    /// The overflow guard of §7.2, in log space.
    ///
    /// The largest quantity ever evaluated is `I_0·e^u` in the residual,
    /// or `(I_0 R_s/a)·e^u` in its derivative. On the domain proved by the
    /// interval invariant, `u ≤ u_max`, and substituting the bound above
    /// collapses the expression to
    ///
    /// ```text
    /// ln(I_0 + I_ph)  +  max(0, ln(R_s/a))  +  I_ph·R_s/a
    /// ```
    ///
    /// entirely closed-form. It must stay below [`LN_F64_MAX`], a limit
    /// **imposed by IEEE 754 binary64**, not chosen.
    ///
    /// For a self-consistent parameter set `V_oc/a ≈ ln(I_ph/I_0) ≈ 25`,
    /// so a violation indicates incoherent parameters rather than an
    /// intrinsic numerical difficulty.
    pub fn exponent_headroom(&self) -> f64 {
        let a = self.modified_ideality_factor.volts();
        let r_s = self.series_resistance.ohms();
        let total_current = self.saturation_current.amperes() + self.photocurrent.amperes();
        let derivative_factor = (r_s / a).max(1.0).ln();
        total_current.ln() + derivative_factor + self.photocurrent.amperes() * r_s / a
    }

    /// The greatest value the exponential's argument can take over the
    /// proved iteration domain — equation (13), with the closed-form
    /// bound substituted for `V_oc`.
    #[inline]
    pub fn max_exponent_argument(&self) -> f64 {
        let a = self.modified_ideality_factor.volts();
        (self.open_circuit_voltage_bound().volts()
            + self.photocurrent.amperes() * self.series_resistance.ohms())
            / a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal() -> SingleDiodeParameters {
        SingleDiodeParameters::new(
            Current::from_amperes(10.4),
            Current::from_amperes(2.5e-10),
            Resistance::from_ohms(0.30),
            Resistance::from_ohms(800.0),
            ModifiedIdealityFactor::from_volts(1.85),
        )
        .expect("the nominal set satisfies H")
    }

    #[test]
    fn the_modified_ideality_factor_is_built_from_the_derived_constant() {
        // 72 cells, n = 1, 25 degC: a = 1 * 72 * (k/q) * 298.15.
        let a = ModifiedIdealityFactor::from_diode(1.0, 72, Temperature::from_celsius(25.0))
            .expect("72 cells at 25 degC satisfy H-6 and H-7");
        let expected = 72.0 * BOLTZMANN_EV_PER_K * 298.15;
        assert!((a.volts() - expected).abs() < 1e-15);
        // Around 1.85 V, the value used throughout the reference sets.
        assert!((a.volts() - 1.85).abs() < 0.01, "a = {}", a.volts());
    }

    #[test]
    fn the_diode_ideality_round_trips_through_the_modified_factor() {
        let t = Temperature::from_celsius(25.0);
        let a = ModifiedIdealityFactor::from_diode(1.3, 60, t)
            .expect("60 cells at 25 degC satisfy H-6 and H-7");
        let n = a
            .diode_ideality(60, t)
            .expect("the denominator is non-zero");
        assert!((n - 1.3).abs() < 1e-12);
    }

    #[test]
    fn the_thermal_voltage_is_the_single_source_of_both_directions() {
        let t = Temperature::from_celsius(25.0);
        let v_th = ModifiedIdealityFactor::thermal_voltage(t);

        // The textbook value at standard conditions, 25.693 mV.
        assert!((v_th - 25.693e-3).abs() < 1e-6, "V_th = {v_th}");

        // Both directions of equation (2) must be exactly this product
        // times the cell count — not merely close to it. An `assert_eq`
        // is what would catch the expression being written out a second
        // time somewhere and drifting.
        let a = ModifiedIdealityFactor::from_diode(1.0, 72, t)
            .expect("72 cells at 25 degC satisfy H-6 and H-7");
        assert_eq!(a.volts(), 72.0 * v_th);
        let n = ModifiedIdealityFactor::from_volts(72.0 * v_th)
            .diode_ideality(72, t)
            .expect("the denominator is non-zero");
        assert_eq!(n, 1.0);
    }

    #[test]
    fn a_zero_cell_count_yields_no_ideality_rather_than_an_infinity() {
        let a = ModifiedIdealityFactor::from_volts(1.85);
        assert_eq!(a.diode_ideality(0, Temperature::from_celsius(25.0)), None);
    }

    #[test]
    fn each_hypothesis_is_refused_by_name() {
        let cases: [(ParameterFault, f64, f64, f64, f64, f64); 5] = [
            (ParameterFault::Photocurrent, 0.0, 2.5e-10, 0.3, 800.0, 1.85),
            (
                ParameterFault::SaturationCurrent,
                10.4,
                0.0,
                0.3,
                800.0,
                1.85,
            ),
            (
                ParameterFault::SeriesResistance,
                10.4,
                2.5e-10,
                -0.3,
                800.0,
                1.85,
            ),
            (
                ParameterFault::ShuntResistance,
                10.4,
                2.5e-10,
                0.3,
                -800.0,
                1.85,
            ),
            (
                ParameterFault::ModifiedIdealityFactor,
                10.4,
                2.5e-10,
                0.3,
                800.0,
                0.0,
            ),
        ];
        for (expected, i_ph, i_0, r_s, r_sh, a) in cases {
            let outcome = SingleDiodeParameters::new(
                Current::from_amperes(i_ph),
                Current::from_amperes(i_0),
                Resistance::from_ohms(r_s),
                Resistance::from_ohms(r_sh),
                ModifiedIdealityFactor::from_volts(a),
            );
            match outcome {
                Err(ElectricalModelError::InvalidParameters { fault, .. }) => {
                    assert_eq!(fault, expected);
                }
                other => panic!("expected {expected:?} to be refused, got {other:?}"),
            }
        }
    }

    #[test]
    fn non_finite_parameters_are_refused() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let outcome = SingleDiodeParameters::new(
                Current::from_amperes(bad),
                Current::from_amperes(2.5e-10),
                Resistance::from_ohms(0.3),
                Resistance::from_ohms(800.0),
                ModifiedIdealityFactor::from_volts(1.85),
            );
            assert!(matches!(
                outcome,
                Err(ElectricalModelError::InvalidParameters { .. })
            ));
        }
    }

    #[test]
    fn zero_series_resistance_is_admissible_and_exactly_detected() {
        let p = SingleDiodeParameters::new(
            Current::from_amperes(10.4),
            Current::from_amperes(2.5e-10),
            Resistance::from_ohms(0.0),
            Resistance::from_ohms(800.0),
            ModifiedIdealityFactor::from_volts(1.85),
        )
        .expect("R_s = 0 satisfies H-5");
        assert!(p.has_zero_series_resistance());
        // An exact predicate: the smallest positive resistance is not zero.
        let almost = SingleDiodeParameters::new(
            Current::from_amperes(10.4),
            Current::from_amperes(2.5e-10),
            Resistance::from_ohms(1e-300),
            Resistance::from_ohms(800.0),
            ModifiedIdealityFactor::from_volts(1.85),
        )
        .unwrap();
        assert!(!almost.has_zero_series_resistance());
    }

    #[test]
    fn the_open_circuit_bound_is_a_genuine_majorant() {
        let p = nominal();
        // The exact V_oc of this set is 45.2249 V (independent bisection).
        let bound = p.open_circuit_voltage_bound().volts();
        assert!(bound >= 45.2249, "bound = {bound}");
        assert!(bound < 45.24, "the bound must stay tight: {bound}");
    }

    #[test]
    fn the_overflow_guard_admits_coherent_sets_and_refuses_incoherent_ones() {
        assert!(nominal().exponent_headroom() < LN_F64_MAX);

        // The guard's dominant term is I_ph*R_s/a, not V_oc/a: the
        // closed-form bound caps ln(1 + I_ph/I_0) at around 24 for any
        // physically shaped set. Overflow is therefore driven by the
        // ohmic drop at full photocurrent measured in units of `a`.
        for (r_s, a) in [(200.0, 1.85), (0.30, 0.001), (50.0, 0.5)] {
            let outcome = SingleDiodeParameters::new(
                Current::from_amperes(10.4),
                Current::from_amperes(2.5e-10),
                Resistance::from_ohms(r_s),
                Resistance::from_ohms(800.0),
                ModifiedIdealityFactor::from_volts(a),
            );
            match outcome {
                Err(ElectricalModelError::NumericalOverflow { guard, limit }) => {
                    assert!(guard > limit, "R_s = {r_s}, a = {a}: guard = {guard}");
                    assert_eq!(limit, LN_F64_MAX);
                }
                other => panic!("R_s = {r_s}, a = {a} must be refused, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_small_ideality_factor_alone_does_not_overflow() {
        // Corrects an earlier reading of the guard. With a = 0.05 V the
        // open-circuit voltage simply shrinks to about 1.22 V, so u_max
        // stays around 66 — the set is unusual but numerically sound, and
        // refusing it would be wrong.
        let p = SingleDiodeParameters::new(
            Current::from_amperes(10.4),
            Current::from_amperes(2.5e-10),
            Resistance::from_ohms(0.30),
            Resistance::from_ohms(800.0),
            ModifiedIdealityFactor::from_volts(0.05),
        )
        .expect("a small `a` is not by itself an overflow risk");
        assert!(p.open_circuit_voltage_bound().volts() < 2.0);
        assert!(p.exponent_headroom() < LN_F64_MAX);
    }

    #[test]
    fn the_ieee_limit_is_the_real_one() {
        // Not a chosen constant: the point at which exp overflows.
        assert!(LN_F64_MAX.exp().is_finite());
        assert!(
            (LN_F64_MAX + 1e-9).exp().is_finite() || (LN_F64_MAX * 1.000_001).exp().is_infinite()
        );
        assert!((LN_F64_MAX + 1.0).exp().is_infinite());
    }

    #[test]
    fn the_maximum_exponent_argument_is_computable_without_solving() {
        let p = nominal();
        let u_max = p.max_exponent_argument();
        // Around 26 for the nominal set, far below the 709.78 limit.
        assert!(u_max > 20.0 && u_max < 30.0, "u_max = {u_max}");
        assert!(u_max.is_finite());
    }
}
