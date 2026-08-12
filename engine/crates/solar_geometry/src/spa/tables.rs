//! Periodic term coefficient tables for the full-precision NREL Solar
//! Position Algorithm.
//!
//! **Source**: Reda, I. & Andreas, A. (2004), "Solar Position Algorithm for
//! Solar Radiation Applications", *Solar Energy*, 76(5), 577-589
//! (NREL/TP-560-34302), Appendix A, Tables A4.2 (heliocentric longitude),
//! A4.3 (heliocentric latitude), A4.4 (heliocentric radius vector), A3.2
//! (nutation in longitude and obliquity). These are the VSOP87-derived
//! terms Reda & Andreas selected from the full VSOP87 series to reach
//! their claimed +/-0.0003 degree accuracy, and the IAU 1980 nutation
//! series terms.
//!
//! **Provenance of this file**: extracted programmatically (no manual
//! transcription of individual numbers) from `pvlib.spa`
//! (pvlib-python, BSD-3-Clause license), an independent, widely used,
//! peer-reviewed open-source implementation of the same NREL publication.
//! Row counts were verified to match the published table sizes exactly
//! (L0:64, L1:34, L2:20, L3:7, L4:3, L5:1, B0:5, B1:2, R0:40, R1:10, R2:6,
//! R3:2, R4:1, nutation:63) before extraction. See
//! `engine/validation-data/README.md` for the exact extraction script and
//! independent numerical cross-validation against this engine's canonical
//! test case (`solar_geometry::spa::tests::reda_andreas_2004_worked_example`).
//!
//! The coefficient values themselves are scientific data derived from a
//! U.S. government technical report (public domain under 17 U.S.C. § 105),
//! not copyrightable creative expression; reproducing them here is
//! standard practice, done identically by essentially every independent
//! SPA implementation. No pvlib source code, algorithm structure, or
//! control flow is copied — this crate's algorithm implementation
//! (`super::compute`) is original.
//!
//! A handful of published phase-angle coefficients in these tables (e.g.
//! `3.14`, `3.142`) are legitimate empirical VSOP87 term values that
//! happen to be close to `pi` by coincidence, not intended approximations
//! of the mathematical constant; `clippy::approx_constant` is disabled for
//! this file for that reason (never for arithmetic elsewhere in the
//! engine, where using the imprecise literal instead of `f64::consts::PI`
//! would be a genuine bug).
#![allow(clippy::approx_constant)]

/// One periodic term `A * cos(B + C*x)` used in the heliocentric
/// longitude/latitude/radius series.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Term {
    pub(crate) a: f64,
    pub(crate) b: f64,
    pub(crate) c: f64,
}

/// One row of the nutation amplitude table: longitude coefficients (A, B)
/// and obliquity coefficients (C, D), in units of 0.0001 arcsecond.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NutationCoeff {
    pub(crate) a: f64,
    pub(crate) b: f64,
    pub(crate) c: f64,
    pub(crate) d: f64,
}

/// One row of the nutation argument-multiplier table: integer multiples of
/// the five fundamental arguments X0 (mean elongation of the Moon from the
/// Sun), X1 (mean anomaly of the Sun), X2 (mean anomaly of the Moon), X3
/// (Moon's argument of latitude), X4 (longitude of the ascending node of
/// the Moon's mean orbit).
#[derive(Debug, Clone, Copy)]
pub(crate) struct YTerm {
    pub(crate) y0: i32,
    pub(crate) y1: i32,
    pub(crate) y2: i32,
    pub(crate) y3: i32,
    pub(crate) y4: i32,
}

/// Heliocentric longitude periodic terms, order 0 (64 terms).
pub(crate) const L0_TERMS: [Term; 64] = [
    Term {
        a: 175347046.0,
        b: 0.0,
        c: 0.0,
    },
    Term {
        a: 3341656.0,
        b: 4.6692568,
        c: 6283.07585,
    },
    Term {
        a: 34894.0,
        b: 4.6261,
        c: 12566.1517,
    },
    Term {
        a: 3497.0,
        b: 2.7441,
        c: 5753.3849,
    },
    Term {
        a: 3418.0,
        b: 2.8289,
        c: 3.5231,
    },
    Term {
        a: 3136.0,
        b: 3.6277,
        c: 77713.7715,
    },
    Term {
        a: 2676.0,
        b: 4.4181,
        c: 7860.4194,
    },
    Term {
        a: 2343.0,
        b: 6.1352,
        c: 3930.2097,
    },
    Term {
        a: 1324.0,
        b: 0.7425,
        c: 11506.7698,
    },
    Term {
        a: 1273.0,
        b: 2.0371,
        c: 529.691,
    },
    Term {
        a: 1199.0,
        b: 1.1096,
        c: 1577.3435,
    },
    Term {
        a: 990.0,
        b: 5.233,
        c: 5884.927,
    },
    Term {
        a: 902.0,
        b: 2.045,
        c: 26.298,
    },
    Term {
        a: 857.0,
        b: 3.508,
        c: 398.149,
    },
    Term {
        a: 780.0,
        b: 1.179,
        c: 5223.694,
    },
    Term {
        a: 753.0,
        b: 2.533,
        c: 5507.553,
    },
    Term {
        a: 505.0,
        b: 4.583,
        c: 18849.228,
    },
    Term {
        a: 492.0,
        b: 4.205,
        c: 775.523,
    },
    Term {
        a: 357.0,
        b: 2.92,
        c: 0.067,
    },
    Term {
        a: 317.0,
        b: 5.849,
        c: 11790.629,
    },
    Term {
        a: 284.0,
        b: 1.899,
        c: 796.298,
    },
    Term {
        a: 271.0,
        b: 0.315,
        c: 10977.079,
    },
    Term {
        a: 243.0,
        b: 0.345,
        c: 5486.778,
    },
    Term {
        a: 206.0,
        b: 4.806,
        c: 2544.314,
    },
    Term {
        a: 205.0,
        b: 1.869,
        c: 5573.143,
    },
    Term {
        a: 202.0,
        b: 2.458,
        c: 6069.777,
    },
    Term {
        a: 156.0,
        b: 0.833,
        c: 213.299,
    },
    Term {
        a: 132.0,
        b: 3.411,
        c: 2942.463,
    },
    Term {
        a: 126.0,
        b: 1.083,
        c: 20.775,
    },
    Term {
        a: 115.0,
        b: 0.645,
        c: 0.98,
    },
    Term {
        a: 103.0,
        b: 0.636,
        c: 4694.003,
    },
    Term {
        a: 102.0,
        b: 0.976,
        c: 15720.839,
    },
    Term {
        a: 102.0,
        b: 4.267,
        c: 7.114,
    },
    Term {
        a: 99.0,
        b: 6.21,
        c: 2146.17,
    },
    Term {
        a: 98.0,
        b: 0.68,
        c: 155.42,
    },
    Term {
        a: 86.0,
        b: 5.98,
        c: 161000.69,
    },
    Term {
        a: 85.0,
        b: 1.3,
        c: 6275.96,
    },
    Term {
        a: 85.0,
        b: 3.67,
        c: 71430.7,
    },
    Term {
        a: 80.0,
        b: 1.81,
        c: 17260.15,
    },
    Term {
        a: 79.0,
        b: 3.04,
        c: 12036.46,
    },
    Term {
        a: 75.0,
        b: 1.76,
        c: 5088.63,
    },
    Term {
        a: 74.0,
        b: 3.5,
        c: 3154.69,
    },
    Term {
        a: 74.0,
        b: 4.68,
        c: 801.82,
    },
    Term {
        a: 70.0,
        b: 0.83,
        c: 9437.76,
    },
    Term {
        a: 62.0,
        b: 3.98,
        c: 8827.39,
    },
    Term {
        a: 61.0,
        b: 1.82,
        c: 7084.9,
    },
    Term {
        a: 57.0,
        b: 2.78,
        c: 6286.6,
    },
    Term {
        a: 56.0,
        b: 4.39,
        c: 14143.5,
    },
    Term {
        a: 56.0,
        b: 3.47,
        c: 6279.55,
    },
    Term {
        a: 52.0,
        b: 0.19,
        c: 12139.55,
    },
    Term {
        a: 52.0,
        b: 1.33,
        c: 1748.02,
    },
    Term {
        a: 51.0,
        b: 0.28,
        c: 5856.48,
    },
    Term {
        a: 49.0,
        b: 0.49,
        c: 1194.45,
    },
    Term {
        a: 41.0,
        b: 5.37,
        c: 8429.24,
    },
    Term {
        a: 41.0,
        b: 2.4,
        c: 19651.05,
    },
    Term {
        a: 39.0,
        b: 6.17,
        c: 10447.39,
    },
    Term {
        a: 37.0,
        b: 6.04,
        c: 10213.29,
    },
    Term {
        a: 37.0,
        b: 2.57,
        c: 1059.38,
    },
    Term {
        a: 36.0,
        b: 1.71,
        c: 2352.87,
    },
    Term {
        a: 36.0,
        b: 1.78,
        c: 6812.77,
    },
    Term {
        a: 33.0,
        b: 0.59,
        c: 17789.85,
    },
    Term {
        a: 30.0,
        b: 0.44,
        c: 83996.85,
    },
    Term {
        a: 30.0,
        b: 2.74,
        c: 1349.87,
    },
    Term {
        a: 25.0,
        b: 3.16,
        c: 4690.48,
    },
];

/// Heliocentric longitude periodic terms, order 1 (34 terms).
pub(crate) const L1_TERMS: [Term; 34] = [
    Term {
        a: 628331966747.0,
        b: 0.0,
        c: 0.0,
    },
    Term {
        a: 206059.0,
        b: 2.678235,
        c: 6283.07585,
    },
    Term {
        a: 4303.0,
        b: 2.6351,
        c: 12566.1517,
    },
    Term {
        a: 425.0,
        b: 1.59,
        c: 3.523,
    },
    Term {
        a: 119.0,
        b: 5.796,
        c: 26.298,
    },
    Term {
        a: 109.0,
        b: 2.966,
        c: 1577.344,
    },
    Term {
        a: 93.0,
        b: 2.59,
        c: 18849.23,
    },
    Term {
        a: 72.0,
        b: 1.14,
        c: 529.69,
    },
    Term {
        a: 68.0,
        b: 1.87,
        c: 398.15,
    },
    Term {
        a: 67.0,
        b: 4.41,
        c: 5507.55,
    },
    Term {
        a: 59.0,
        b: 2.89,
        c: 5223.69,
    },
    Term {
        a: 56.0,
        b: 2.17,
        c: 155.42,
    },
    Term {
        a: 45.0,
        b: 0.4,
        c: 796.3,
    },
    Term {
        a: 36.0,
        b: 0.47,
        c: 775.52,
    },
    Term {
        a: 29.0,
        b: 2.65,
        c: 7.11,
    },
    Term {
        a: 21.0,
        b: 5.34,
        c: 0.98,
    },
    Term {
        a: 19.0,
        b: 1.85,
        c: 5486.78,
    },
    Term {
        a: 19.0,
        b: 4.97,
        c: 213.3,
    },
    Term {
        a: 17.0,
        b: 2.99,
        c: 6275.96,
    },
    Term {
        a: 16.0,
        b: 0.03,
        c: 2544.31,
    },
    Term {
        a: 16.0,
        b: 1.43,
        c: 2146.17,
    },
    Term {
        a: 15.0,
        b: 1.21,
        c: 10977.08,
    },
    Term {
        a: 12.0,
        b: 2.83,
        c: 1748.02,
    },
    Term {
        a: 12.0,
        b: 3.26,
        c: 5088.63,
    },
    Term {
        a: 12.0,
        b: 5.27,
        c: 1194.45,
    },
    Term {
        a: 12.0,
        b: 2.08,
        c: 4694.0,
    },
    Term {
        a: 11.0,
        b: 0.77,
        c: 553.57,
    },
    Term {
        a: 10.0,
        b: 1.3,
        c: 6286.6,
    },
    Term {
        a: 10.0,
        b: 4.24,
        c: 1349.87,
    },
    Term {
        a: 9.0,
        b: 2.7,
        c: 242.73,
    },
    Term {
        a: 9.0,
        b: 5.64,
        c: 951.72,
    },
    Term {
        a: 8.0,
        b: 5.3,
        c: 2352.87,
    },
    Term {
        a: 6.0,
        b: 2.65,
        c: 9437.76,
    },
    Term {
        a: 6.0,
        b: 4.67,
        c: 4690.48,
    },
];

/// Heliocentric longitude periodic terms, order 2 (20 terms).
pub(crate) const L2_TERMS: [Term; 20] = [
    Term {
        a: 52919.0,
        b: 0.0,
        c: 0.0,
    },
    Term {
        a: 8720.0,
        b: 1.0721,
        c: 6283.0758,
    },
    Term {
        a: 309.0,
        b: 0.867,
        c: 12566.152,
    },
    Term {
        a: 27.0,
        b: 0.05,
        c: 3.52,
    },
    Term {
        a: 16.0,
        b: 5.19,
        c: 26.3,
    },
    Term {
        a: 16.0,
        b: 3.68,
        c: 155.42,
    },
    Term {
        a: 10.0,
        b: 0.76,
        c: 18849.23,
    },
    Term {
        a: 9.0,
        b: 2.06,
        c: 77713.77,
    },
    Term {
        a: 7.0,
        b: 0.83,
        c: 775.52,
    },
    Term {
        a: 5.0,
        b: 4.66,
        c: 1577.34,
    },
    Term {
        a: 4.0,
        b: 1.03,
        c: 7.11,
    },
    Term {
        a: 4.0,
        b: 3.44,
        c: 5573.14,
    },
    Term {
        a: 3.0,
        b: 5.14,
        c: 796.3,
    },
    Term {
        a: 3.0,
        b: 6.05,
        c: 5507.55,
    },
    Term {
        a: 3.0,
        b: 1.19,
        c: 242.73,
    },
    Term {
        a: 3.0,
        b: 6.12,
        c: 529.69,
    },
    Term {
        a: 3.0,
        b: 0.31,
        c: 398.15,
    },
    Term {
        a: 3.0,
        b: 2.28,
        c: 553.57,
    },
    Term {
        a: 2.0,
        b: 4.38,
        c: 5223.69,
    },
    Term {
        a: 2.0,
        b: 3.75,
        c: 0.98,
    },
];

/// Heliocentric longitude periodic terms, order 3 (7 terms).
pub(crate) const L3_TERMS: [Term; 7] = [
    Term {
        a: 289.0,
        b: 5.844,
        c: 6283.076,
    },
    Term {
        a: 35.0,
        b: 0.0,
        c: 0.0,
    },
    Term {
        a: 17.0,
        b: 5.49,
        c: 12566.15,
    },
    Term {
        a: 3.0,
        b: 5.2,
        c: 155.42,
    },
    Term {
        a: 1.0,
        b: 4.72,
        c: 3.52,
    },
    Term {
        a: 1.0,
        b: 5.3,
        c: 18849.23,
    },
    Term {
        a: 1.0,
        b: 5.97,
        c: 242.73,
    },
];

/// Heliocentric longitude periodic terms, order 4 (3 terms).
pub(crate) const L4_TERMS: [Term; 3] = [
    Term {
        a: 114.0,
        b: 3.142,
        c: 0.0,
    },
    Term {
        a: 8.0,
        b: 4.13,
        c: 6283.08,
    },
    Term {
        a: 1.0,
        b: 3.84,
        c: 12566.15,
    },
];

/// Heliocentric longitude periodic terms, order 5 (1 term).
pub(crate) const L5_TERMS: [Term; 1] = [Term {
    a: 1.0,
    b: 3.14,
    c: 0.0,
}];

/// Heliocentric latitude periodic terms, order 0 (5 terms).
pub(crate) const B0_TERMS: [Term; 5] = [
    Term {
        a: 280.0,
        b: 3.199,
        c: 84334.662,
    },
    Term {
        a: 102.0,
        b: 5.422,
        c: 5507.553,
    },
    Term {
        a: 80.0,
        b: 3.88,
        c: 5223.69,
    },
    Term {
        a: 44.0,
        b: 3.7,
        c: 2352.87,
    },
    Term {
        a: 32.0,
        b: 4.0,
        c: 1577.34,
    },
];

/// Heliocentric latitude periodic terms, order 1 (2 terms).
pub(crate) const B1_TERMS: [Term; 2] = [
    Term {
        a: 9.0,
        b: 3.9,
        c: 5507.55,
    },
    Term {
        a: 6.0,
        b: 1.73,
        c: 5223.69,
    },
];

/// Heliocentric radius vector periodic terms, order 0 (40 terms).
pub(crate) const R0_TERMS: [Term; 40] = [
    Term {
        a: 100013989.0,
        b: 0.0,
        c: 0.0,
    },
    Term {
        a: 1670700.0,
        b: 3.0984635,
        c: 6283.07585,
    },
    Term {
        a: 13956.0,
        b: 3.05525,
        c: 12566.1517,
    },
    Term {
        a: 3084.0,
        b: 5.1985,
        c: 77713.7715,
    },
    Term {
        a: 1628.0,
        b: 1.1739,
        c: 5753.3849,
    },
    Term {
        a: 1576.0,
        b: 2.8469,
        c: 7860.4194,
    },
    Term {
        a: 925.0,
        b: 5.453,
        c: 11506.77,
    },
    Term {
        a: 542.0,
        b: 4.564,
        c: 3930.21,
    },
    Term {
        a: 472.0,
        b: 3.661,
        c: 5884.927,
    },
    Term {
        a: 346.0,
        b: 0.964,
        c: 5507.553,
    },
    Term {
        a: 329.0,
        b: 5.9,
        c: 5223.694,
    },
    Term {
        a: 307.0,
        b: 0.299,
        c: 5573.143,
    },
    Term {
        a: 243.0,
        b: 4.273,
        c: 11790.629,
    },
    Term {
        a: 212.0,
        b: 5.847,
        c: 1577.344,
    },
    Term {
        a: 186.0,
        b: 5.022,
        c: 10977.079,
    },
    Term {
        a: 175.0,
        b: 3.012,
        c: 18849.228,
    },
    Term {
        a: 110.0,
        b: 5.055,
        c: 5486.778,
    },
    Term {
        a: 98.0,
        b: 0.89,
        c: 6069.78,
    },
    Term {
        a: 86.0,
        b: 5.69,
        c: 15720.84,
    },
    Term {
        a: 86.0,
        b: 1.27,
        c: 161000.69,
    },
    Term {
        a: 65.0,
        b: 0.27,
        c: 17260.15,
    },
    Term {
        a: 63.0,
        b: 0.92,
        c: 529.69,
    },
    Term {
        a: 57.0,
        b: 2.01,
        c: 83996.85,
    },
    Term {
        a: 56.0,
        b: 5.24,
        c: 71430.7,
    },
    Term {
        a: 49.0,
        b: 3.25,
        c: 2544.31,
    },
    Term {
        a: 47.0,
        b: 2.58,
        c: 775.52,
    },
    Term {
        a: 45.0,
        b: 5.54,
        c: 9437.76,
    },
    Term {
        a: 43.0,
        b: 6.01,
        c: 6275.96,
    },
    Term {
        a: 39.0,
        b: 5.36,
        c: 4694.0,
    },
    Term {
        a: 38.0,
        b: 2.39,
        c: 8827.39,
    },
    Term {
        a: 37.0,
        b: 0.83,
        c: 19651.05,
    },
    Term {
        a: 37.0,
        b: 4.9,
        c: 12139.55,
    },
    Term {
        a: 36.0,
        b: 1.67,
        c: 12036.46,
    },
    Term {
        a: 35.0,
        b: 1.84,
        c: 2942.46,
    },
    Term {
        a: 33.0,
        b: 0.24,
        c: 7084.9,
    },
    Term {
        a: 32.0,
        b: 0.18,
        c: 5088.63,
    },
    Term {
        a: 32.0,
        b: 1.78,
        c: 398.15,
    },
    Term {
        a: 28.0,
        b: 1.21,
        c: 6286.6,
    },
    Term {
        a: 28.0,
        b: 1.9,
        c: 6279.55,
    },
    Term {
        a: 26.0,
        b: 4.59,
        c: 10447.39,
    },
];

/// Heliocentric radius vector periodic terms, order 1 (10 terms).
pub(crate) const R1_TERMS: [Term; 10] = [
    Term {
        a: 103019.0,
        b: 1.10749,
        c: 6283.07585,
    },
    Term {
        a: 1721.0,
        b: 1.0644,
        c: 12566.1517,
    },
    Term {
        a: 702.0,
        b: 3.142,
        c: 0.0,
    },
    Term {
        a: 32.0,
        b: 1.02,
        c: 18849.23,
    },
    Term {
        a: 31.0,
        b: 2.84,
        c: 5507.55,
    },
    Term {
        a: 25.0,
        b: 1.32,
        c: 5223.69,
    },
    Term {
        a: 18.0,
        b: 1.42,
        c: 1577.34,
    },
    Term {
        a: 10.0,
        b: 5.91,
        c: 10977.08,
    },
    Term {
        a: 9.0,
        b: 1.42,
        c: 6275.96,
    },
    Term {
        a: 9.0,
        b: 0.27,
        c: 5486.78,
    },
];

/// Heliocentric radius vector periodic terms, order 2 (6 terms).
pub(crate) const R2_TERMS: [Term; 6] = [
    Term {
        a: 4359.0,
        b: 5.7846,
        c: 6283.0758,
    },
    Term {
        a: 124.0,
        b: 5.579,
        c: 12566.152,
    },
    Term {
        a: 12.0,
        b: 3.14,
        c: 0.0,
    },
    Term {
        a: 9.0,
        b: 3.63,
        c: 77713.77,
    },
    Term {
        a: 6.0,
        b: 1.87,
        c: 5573.14,
    },
    Term {
        a: 3.0,
        b: 5.47,
        c: 18849.23,
    },
];

/// Heliocentric radius vector periodic terms, order 3 (2 terms).
pub(crate) const R3_TERMS: [Term; 2] = [
    Term {
        a: 145.0,
        b: 4.273,
        c: 6283.076,
    },
    Term {
        a: 7.0,
        b: 3.92,
        c: 12566.15,
    },
];

/// Heliocentric radius vector periodic terms, order 4 (1 term).
pub(crate) const R4_TERMS: [Term; 1] = [Term {
    a: 4.0,
    b: 2.56,
    c: 6283.08,
}];

/// Nutation periodic term coefficients A, B, C, D (units of 0.0001 arcsecond).
///
/// Source: Reda & Andreas (2004), NREL/TP-560-34302, Table A3.2.
pub(crate) const NUTATION_ABCD: [NutationCoeff; 63] = [
    NutationCoeff {
        a: -171996.0,
        b: -174.2,
        c: 92025.0,
        d: 8.9,
    },
    NutationCoeff {
        a: -13187.0,
        b: -1.6,
        c: 5736.0,
        d: -3.1,
    },
    NutationCoeff {
        a: -2274.0,
        b: -0.2,
        c: 977.0,
        d: -0.5,
    },
    NutationCoeff {
        a: 2062.0,
        b: 0.2,
        c: -895.0,
        d: 0.5,
    },
    NutationCoeff {
        a: 1426.0,
        b: -3.4,
        c: 54.0,
        d: -0.1,
    },
    NutationCoeff {
        a: 712.0,
        b: 0.1,
        c: -7.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -517.0,
        b: 1.2,
        c: 224.0,
        d: -0.6,
    },
    NutationCoeff {
        a: -386.0,
        b: -0.4,
        c: 200.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -301.0,
        b: 0.0,
        c: 129.0,
        d: -0.1,
    },
    NutationCoeff {
        a: 217.0,
        b: -0.5,
        c: -95.0,
        d: 0.3,
    },
    NutationCoeff {
        a: -158.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 129.0,
        b: 0.1,
        c: -70.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 123.0,
        b: 0.0,
        c: -53.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 63.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 63.0,
        b: 0.1,
        c: -33.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -59.0,
        b: 0.0,
        c: 26.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -58.0,
        b: -0.1,
        c: 32.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -51.0,
        b: 0.0,
        c: 27.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 48.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 46.0,
        b: 0.0,
        c: -24.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -38.0,
        b: 0.0,
        c: 16.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -31.0,
        b: 0.0,
        c: 13.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 29.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 29.0,
        b: 0.0,
        c: -12.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 26.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -22.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 21.0,
        b: 0.0,
        c: -10.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 17.0,
        b: -0.1,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 16.0,
        b: 0.0,
        c: -8.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -16.0,
        b: 0.1,
        c: 7.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -15.0,
        b: 0.0,
        c: 9.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -13.0,
        b: 0.0,
        c: 7.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -12.0,
        b: 0.0,
        c: 6.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 11.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -10.0,
        b: 0.0,
        c: 5.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -8.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 7.0,
        b: 0.0,
        c: -3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -7.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -7.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -7.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 6.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 6.0,
        b: 0.0,
        c: -3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 6.0,
        b: 0.0,
        c: -3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -6.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -6.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 5.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -5.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -5.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -5.0,
        b: 0.0,
        c: 3.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 4.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 4.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 4.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -4.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -4.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -4.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: 3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
    NutationCoeff {
        a: -3.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
    },
];

/// Nutation periodic term multipliers Y0..Y4 (integer multiples of the five
/// fundamental arguments X0..X4).
///
/// Source: Reda & Andreas (2004), NREL/TP-560-34302, Table A3.2.
pub(crate) const NUTATION_YTERM: [YTerm; 63] = [
    YTerm {
        y0: 0,
        y1: 0,
        y2: 0,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 0,
        y3: 0,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 1,
        y2: 0,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: 1,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: -2,
        y1: -1,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: -1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: 0,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 1,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: -1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: -1,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 1,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 2,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: -2,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 2,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 2,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: -1,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 2,
        y2: 0,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: -1,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 2,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 1,
        y2: 0,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 1,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: -1,
        y2: 0,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 2,
        y3: -2,
        y4: 0,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: -1,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: 1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 1,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: -2,
        y1: 1,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: -1,
        y2: 0,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: 0,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 2,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 1,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: -2,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 2,
        y1: 0,
        y2: 0,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: -1,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: -1,
        y2: 0,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 0,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 2,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 0,
        y2: 2,
        y3: 0,
        y4: 1,
    },
    YTerm {
        y0: -2,
        y1: 1,
        y2: 0,
        y3: 2,
        y4: 1,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 1,
        y3: -2,
        y4: 0,
    },
    YTerm {
        y0: -1,
        y1: 0,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: -2,
        y1: 1,
        y2: 0,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 1,
        y1: 0,
        y2: 0,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 1,
        y3: 2,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: -2,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: -1,
        y1: -1,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: 1,
        y2: 1,
        y3: 0,
        y4: 0,
    },
    YTerm {
        y0: 0,
        y1: -1,
        y2: 1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 2,
        y1: -1,
        y2: -1,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 0,
        y1: 0,
        y2: 3,
        y3: 2,
        y4: 2,
    },
    YTerm {
        y0: 2,
        y1: -1,
        y2: 0,
        y3: 2,
        y4: 2,
    },
];
