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
- `generate_global_campaign.py` — generator for the much larger **global
  validation campaign** dataset (below). Uses the same pvlib step-by-step
  pipeline as `generate_pvlib_reference_dataset.py`, with a fixed random
  seed (`20260802`) for reproducibility.
- `global_campaign_dataset.csv` — ~2900 generated test cases (raw output
  of the script above), committed so the dataset can be inspected or
  reprocessed without needing Python/pvlib installed. Embedded directly
  into the `validation` crate via `include_str!` and parsed at first use
  by a small hand-rolled parser (`validation::global_campaign::dataset`) —
  CSV rather than a generated Rust array at this scale, to avoid an
  unwieldy multi-megabyte source file; see that module's doc comment for
  the full rationale.
- `latest_global_campaign_report.md` — **generated automatically** by
  `cargo test -p validation` (the
  `integration_tests::generate_and_write_global_campaign_report` test),
  same non-hand-maintained rule as `latest_validation_report.md` above.

## The global validation campaign: validating the scientific model, not only the code

`pvlib_reference_dataset.json` (91 cases, above) checks that HeliosPro's
SPA implementation reproduces an independent oracle on a small,
hand-picked sample of representative sites and dates. `global_campaign_dataset.csv`
goes further: it is a **~2900-case stratified sweep of the entire input
envelope** the engine is specified to support —

- latitude -90° to +90°, longitude -180° to +180°, altitude 0-6000 m
  (ISO 2533:1975 standard-atmosphere pressure at each altitude)
- years 1900-2100, including the Gregorian centennial leap-year exception
  (1900 and 2100 are *not* leap years; 2000 *is*, divisible by 400)
- explicit edge categories: extreme latitude/longitude/altitude, century
  years, leap years, near-midnight UTC (Julian Day rollover), polar day,
  polar night, sunrise/sunset sweeps (5 sites x 2 dates x 48 half-hour
  samples), and subsolar/near-zenith cases

so that the campaign exercises the *scientific model itself* — including
the conditions where a solar-position algorithm is hardest to get right
(grazing-horizon refraction, near-zenith azimuth, polar geometry) — rather
than only the code paths a small sample happens to touch.

`delta_t` (TT - UT1) is specified exactly per case (matching what pvlib
was given) rather than estimated from the calendar date, because
`core_math`'s own delta_t estimator (Espenak & Meeus, 2006) is documented
as valid only for 2005-2050 and this campaign deliberately spans
1900-2100; using the estimator here would conflate delta_t-estimation
error with solar-position-algorithm error. Both `solar_position::spa` and
`solar_position::meeus` expose explicit-delta_t `solar_position_precise`
entry points for exactly this reason.

**"Changements d'heure" (DST) and "fuseaux horaires" (time zones)**:
HeliosPro's solar-position engines are, by explicit architectural
decision, functions of UTC calendar fields only — civil time zone
handling and DST transitions are an application-layer concern (convert to
UTC before calling the engine), not part of the scientific core. A civil
time zone is mathematically equivalent, for a fixed UTC instant and
geographic location, to a longitude offset plus a constant hour shift;
there is no separate "time zone" input this engine could take that would
change its output. This campaign represents that requested coverage
through its full longitude sweep (every UTC offset a civil zone could
correspond to) and its `near_midnight_utc` category (which exercises the
UTC day-boundary arithmetic a naive civil-time implementation would get
wrong), rather than by simulating civil calendars with DST rules —
simulating those would test application-layer logic this engine
intentionally does not contain. See
`engine/crates/validation/src/global_campaign/mod.rs`'s module doc for
the same note in context, and its "genuine, documented near-horizon
divergence" and "zenith singularity" sections for two real,
evidence-bounded edge effects this campaign surfaced (differing
refraction-cutoff conventions between the two engines near the horizon;
the azimuth formula's coordinate singularity at zenith transit) — neither
is a coding defect, and both are explained and bounded rather than
hidden behind a loosened threshold.

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

# 4. Regenerate the global validation campaign dataset (requires pvlib
#    installed, same as step 2; fixed random seed, so re-running is
#    reproducible).
python3 generate_global_campaign.py
# writes global_campaign_dataset.csv in this directory -- no separate
# Rust-generation step needed, it is embedded via include_str! and parsed
# at first use by validation::global_campaign::dataset.

# 5. Re-run the full test suite, including both validation campaigns,
#    which also regenerates latest_validation_report.md and
#    latest_global_campaign_report.md.
cd ../.. && cargo test --workspace
```
