# HeliosPro Scientific Engine

Rust workspace (edition 2024) implementing HeliosPro's PV scientific core:
no GUI, no database, no I/O beyond pure function calls. Architecture,
module order, and engineering rules follow the project's development
brief (see `../docs/design/`).

## Workspace layout (14 crates)

```
engine/
├── Cargo.toml                # workspace manifest
└── crates/
    ├── core_constants/        # referenced physical/astronomical constants               [done]
    ├── core_units/            # SI unit newtypes (Angle, Temperature, Irradiance, ...)    [done]
    ├── core_math/             # Julian Day, interpolation, error statistics               [done]
    ├── solar_geometry/        # solar time, geocentric ephemeris (spa + meeus), incidence [done]
    ├── solar_position/        # topocentric solar position, selectable Spa/MeeusReduced   [done]
    ├── irradiance/            # extraterrestrial irradiance, GHI/DNI/DHI decomposition    [done]
    ├── transposition/         # isotropic, Hay-Davies, HDKR, Perez POA models             [done]
    ├── pv_module/             # PV module datasheet domain model                          [scaffolding]
    ├── thermal_model/         # NOCT, Sandia cell-temperature models                      [done]
    ├── electrical_model/      # single-diode electrical model                             [scaffolding]
    ├── losses/                # systematic loss models (soiling, mismatch, wiring, ...)   [scaffolding]
    ├── simulation/            # top-level orchestration ("bilan energetique")             [done, partial*]
    ├── validation/            # pvlib reference dataset + automated error reporting       [done]
    └── tests/                 # cross-crate regression ("golden dataset") tests           [done]
```

`[done]` = implemented, unit-tested, documented, and confirmed compiling as
of the latest commit. `[scaffolding]` = crate exists with a documented
scope and citations for what it *will* contain, but no logic yet — created
now so the workspace's member list matches the target architecture, per
the project's "never write approximate code" rule: an honest, empty,
clearly-labelled placeholder is not approximate code; a rushed or
under-validated implementation would be. `[done, partial*]` = `simulation`
implements the full chain from solar position through plane-of-array
irradiance to cell temperature (the "bilan énergétique complet" called for
in the original Phase 1 brief), but does not yet call into
`electrical_model`/`losses` to produce DC/AC electrical output — that
extension is the natural next step once those crates land.

### Two solar-position engines, one default

`solar_geometry` and `solar_position` each contain **two independent
implementations**:

- **`spa`** — the full NREL Solar Position Algorithm (Reda & Andreas,
  2004), complete with its published coefficient tables. **This is the
  default, primary engine** (`solar_position::solar_position(...)` and
  `solar_position::Spa` both use it).
- **`meeus`** — a reduced-order engine (Meeus, 1998), kept only as an
  **independent secondary engine for cross-validation**
  (`solar_position::MeeusReduced`).

See `crates/solar_geometry/src/lib.rs` for the full rationale, and
`crates/validation/` for the automated comparison of both against an
external reference (pvlib-python).

### Dependency graph

```
core_constants, core_units, core_math      (no internal dependencies)
        |
solar_geometry (needs core_*; contains both the spa and meeus engines)
        |
        +--> solar_position (needs solar_geometry; selects spa/meeus)
        +--> irradiance      (needs solar_geometry)
        +--> transposition   (needs solar_geometry)
        |
pv_module (needs core_units)
        |
        +--> electrical_model (needs pv_module, core_*)
        |
thermal_model (needs core_units)
losses (needs core_units)
        |
simulation (needs solar_position, irradiance, transposition, thermal_model, core_*)
        |
tests (dev-depends on everything above)
validation (needs core_math, core_units, solar_position)
```

Each crate is independently buildable and testable
(`cargo test -p <crate_name>`).

## Building and testing

```bash
cd engine
cargo build --workspace                  # compile every crate
cargo test --workspace                   # run every test
cargo clippy --workspace --all-targets   # lint (zero warnings)
cargo fmt --all -- --check               # formatting (clean)
cargo doc --workspace --no-deps --open   # browse the referenced API documentation
```

No network access, no external services, and no dependency outside the
Rust standard library are required to build or test this engine (verify
with `cargo tree --workspace`, which lists path dependencies only). The
`validation` crate's reference dataset was generated offline once (see
`validation-data/README.md`) and is committed; building/testing this
engine never requires network access or a Python interpreter.

## Scientific validation

`cargo test -p validation` runs both engines against two independent
pvlib-python 0.15.2 reference datasets and asserts documented accuracy
budgets:

- **91 hand-picked cases** (`dataset`) spanning representative sites/dates
  — regenerates `validation-data/latest_validation_report.md`. SPA agrees
  with pvlib to within 0.0002° across every angular quantity checked
  (zenith, azimuth, elevation, hour angle, declination, right ascension)
  and to machine precision on the Earth radius vector.
- **~2900-case global campaign** (`global_campaign`) stratified across the
  full latitude (-90..90°) / longitude (-180..180°) / altitude (0-6000 m) /
  year (1900-2100) envelope, plus explicit polar day/night, sunrise/
  sunset-sweep, subsolar, and century/leap-year categories — regenerates
  `validation-data/latest_global_campaign_report.md`. Every computed
  quantity is asserted finite across the full sweep for both engines (the
  numerical-stability guarantee); two genuine near-singularity effects the
  full envelope surfaces (a near-horizon refraction-cutoff divergence
  between the two engines, and an azimuth coordinate singularity at zenith
  transit) are documented and bounded rather than hidden — see that
  module's doc comment
  (`engine/crates/validation/src/global_campaign/mod.rs`).

`cargo test -p tests` additionally exercises
`tests/numerical_stability.rs` (deliberately degenerate synthetic edge
cases: the geographic poles, years 1582-9999, angles at/near 0/90/180°,
UTC midnight/day-boundary crossings — no accuracy claim, only a
finite-output/no-panic claim) and `tests/benchmarks.rs` (wall-clock timing
mean/standard deviation and heap-allocation count/bytes for both engines,
via a small counting `#[global_allocator]`; regenerates
`validation-data/latest_benchmark_report.md` — both engines currently
measure **zero heap allocations** in steady state).

See `docs/design/16-phase1-validation-report.md` for the full methodology
(including what remains unvalidated: irradiance/transposition/thermal
layers have no external reference comparison yet) and
`docs/design/17-analyse-incertitude-numerique.md` for the numerical
uncertainty analysis (rounding, `f32` vs `f64`, sensitivity to
astronomical constants) underlying these accuracy budgets.

## Extending the engine: architecture for future modules

Per the project's roadmap (PV done or in progress; storage/batteries,
microgrids, solar pumping, wind, hydrogen, HVAC, economic analysis, AI
optimization planned), this workspace's layering is deliberately
domain-agnostic where it needs to be, so those future modules can be
added as new crates without modifying the scientific core built so far:

- **`core_constants`, `core_units`, `core_math`** contain no PV-specific
  concepts whatsoever (physical constants, SI unit newtypes, calendar/
  numerical utilities) — any future energy-domain crate (battery
  electrochemistry, hydraulic pumping, HVAC thermodynamics, economic
  cashflow models) depends on these same three crates the same way the PV
  chain does, and never needs to depend on `solar_geometry`,
  `transposition`, or any other PV-specific crate.
- **`solar_geometry`/`solar_position`** are themselves reusable beyond PV:
  solar pumping, HVAC solar-gain calculations, and building energy models
  all need the same solar position and irradiance geometry computed here.
- Each crate exposes a narrow, typed public API (see each crate's `lib.rs`
  module documentation) rather than a shared mutable "god object" — a
  future `battery_model` or `microgrid_dispatch` crate would consume
  `simulation`'s `EnergyBalanceResult` (or a similar typed output) as an
  *input*, the same way `simulation` today consumes `transposition`'s and
  `thermal_model`'s outputs, without those upstream crates needing to know
  the downstream crate exists.
- The `SolarPositionAlgorithm` trait in `solar_position` (implemented by
  both `Spa` and `MeeusReduced`) is the template for how future
  multi-engine domains (e.g. multiple battery degradation models, multiple
  MPPT/dispatch strategies) should expose a selectable-implementation
  abstraction: a small trait, one struct per engine, a documented default.
- No module implemented so far reaches into another module's private
  state — every cross-crate interaction goes through `pub` types and
  functions with documented units and citations, which is what makes
  independent addition of new domains (rather than invasive refactoring)
  possible.

## Documentation

- Design-phase documents: `../docs/design/`
- Phase 1 validation report: `../docs/design/16-phase1-validation-report.md`
- Numerical uncertainty analysis: `../docs/design/17-analyse-incertitude-numerique.md`
- Technical annex (algorithms, equations, proofs, diagrams, references): `../docs/design/18-annexe-technique-position-solaire.md`
- Public API audit and SemVer policy: `../docs/design/19-politique-semver-api.md`
- Validation dataset provenance: `validation-data/README.md`
- Latest automated validation reports: `validation-data/latest_validation_report.md`,
  `validation-data/latest_global_campaign_report.md`,
  `validation-data/latest_benchmark_report.md`,
  `validation-data/latest_coverage_report.txt`
- API documentation: generate with `cargo doc` (every public item is
  documented with a citation to its source, per project rules)
