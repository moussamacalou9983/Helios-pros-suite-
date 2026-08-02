# Validation data — provenance and regeneration

This directory holds the external reference dataset used by the
`validation` crate to check HeliosPro's solar-position engines
(`solar_position::Spa`, `solar_position::MeeusReduced`) against an
independent implementation of the NREL Solar Position Algorithm.

## Contents

- `generate_pvlib_reference_dataset.py` — the exact script used to produce
  the dataset. Re-running it regenerates `pvlib_reference_dataset.json`
  byte-for-byte (deterministic: no randomness, fixed inputs).
- `pvlib_reference_dataset.json` — the 91 generated test cases (raw
  output of the script above), committed so the dataset can be inspected
  or reprocessed without needing Python/pvlib installed.
- `latest_validation_report.md` — **generated automatically** by
  `cargo test -p validation` (specifically the
  `integration_tests::generate_and_write_full_validation_report` test in
  `engine/crates/validation/src/lib.rs`) every time the test suite runs.
  This file is *not* hand-maintained — do not edit it directly; edit the
  engine code or the dataset generator instead and re-run the tests.

## How the dataset was produced

1. Installed `pvlib-python` 0.15.2 (BSD-3-Clause license) via `pip`.
2. Ran `generate_pvlib_reference_dataset.py`, which calls the individual
   step-by-step functions of `pvlib.spa` (itself a direct, well-known
   open-source implementation of Reda, I. & Andreas, A. (2004), "Solar
   Position Algorithm for Solar Radiation Applications", *Solar Energy*,
   76(5), 577-589, NREL/TP-560-34302) for:
   - 1 canonical case: the paper's own published worked example (Golden,
     Colorado, 2003-10-17 12:30:30 MST), using the exact inputs
     (pressure=820 hPa, temperature=11°C, delta_t=67 s,
     atmos_refract=0.5667°) reproduced in pvlib's own test suite
     (`tests/conftest.py` fixtures `golden_mst`/`expected_solpos`,
     `tests/test_solarposition.py::test_spa_python_numpy_physical`) —
     independently cross-checked in this project against the same
     fixtures before use.
   - 90 campaign cases: 6 dates (both equinoxes, both solstices, and two
     additional dates) x 3 UTC times x 5 locations spanning both
     hemispheres and a range of latitudes (Paris, Reykjavik, Singapore,
     Sydney, Golden CO), generic atmosphere (101325 Pa, 15°C,
     delta_t=69 s).
3. For every case, every intermediate SPA quantity was extracted (Julian
   Day/Century/Millennium, heliocentric L/B/R, geocentric longitude/
   latitude, the five fundamental arguments X0-X4, nutation, obliquity,
   aberration, apparent longitude, sidereal time, right ascension,
   declination, equation of time, hour angle, parallax terms, topocentric
   right ascension/declination/hour angle, elevation/zenith with and
   without refraction, azimuth) — not just the final zenith/azimuth — so
   that HeliosPro's own SPA implementation could be checked stage by
   stage, not only at the final output.

## Why pvlib and not PVsyst/SAM/NREL's own reference implementation directly

This development environment had no working network access to
`nrel.gov` (DNS failures) nor to PVsyst or SAM (neither tool is installed
or licensed in this environment). `raw.githubusercontent.com` was
reachable, and `pip install pvlib` succeeded, making pvlib the only
practical source of an independently-computed, byte-verifiable reference
in this session. pvlib is not a competitor product in the sense the
project's "never copy competitor code" rule is concerned with (PVsyst,
PV\*SOL, Archelios Calc) — it is an open-source (BSD-3-Clause), peer-
reviewed scientific library, and is used here purely as a *comparison
oracle* (running its functions to get numbers to compare against), never
as a source of code, algorithm structure, or interface design copied into
HeliosPro. HeliosPro's own SPA implementation
(`engine/crates/solar_geometry/src/spa/`) is original Rust code following
the published Reda & Andreas (2004) paper directly; only the numeric
periodic-term coefficient tables (public-domain U.S. government data, see
that module's doc comments) were cross-referenced against pvlib's
reproduction of the same public-domain source, for exactly the reasons
documented there.

Extending this dataset with genuine PVsyst/SAM/NREL-reference-C
comparisons remains explicit tracked follow-up work for whenever those
tools become reachable from a future development session — see
`docs/design/16-phase1-validation-report.md`.

## Regenerating the dataset and generated Rust sources

Three scripts, run in order, fully regenerate every derived artifact from
scratch:

```bash
cd engine/validation-data

# 1. Regenerate the SPA periodic-term coefficient tables (fetches
#    pvlib-python's spa.py directly from GitHub; no local pvlib install
#    needed for this step).
python3 generate_spa_tables_rust.py
# writes tables_generated_body.rs -- diff against
# ../crates/solar_geometry/src/spa/tables.rs's data section before
# replacing (that file also has hand-written struct defs/module docs
# prepended, which this script intentionally does not regenerate).

# 2. Regenerate the pvlib reference dataset (requires pvlib installed).
pip install pvlib==0.15.2
python3 generate_pvlib_reference_dataset.py
# writes pvlib_reference_dataset.json in this directory.

# 3. Convert the JSON dataset into the Rust const array consumed by the
#    validation crate.
python3 generate_rust_dataset.py
# writes ../crates/validation/src/dataset/generated_dataset.rs directly.

# 4. Re-run the full test suite, including the validation campaign, which
#    also regenerates latest_validation_report.md.
cd ../.. && cargo test --workspace
```
