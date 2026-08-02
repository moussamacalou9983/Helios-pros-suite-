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
    ├── solar_geometry/        # solar time, geocentric ephemeris, angle of incidence      [done]
    ├── solar_position/        # topocentric solar position (SPA-class algorithm)          [done]
    ├── irradiance/            # extraterrestrial irradiance, GHI/DNI/DHI decomposition    [done]
    ├── transposition/         # isotropic, Hay-Davies, HDKR, Perez POA models             [done]
    ├── pv_module/             # PV module datasheet domain model                          [scaffolding]
    ├── thermal_model/         # NOCT, Sandia cell-temperature models                      [done]
    ├── electrical_model/      # single-diode electrical model                             [scaffolding]
    ├── losses/                # systematic loss models (soiling, mismatch, wiring, ...)   [scaffolding]
    ├── simulation/            # top-level orchestration ("bilan energetique")             [done, partial*]
    ├── validation/            # external validation vs. NREL SPA/pvlib/SAM/PVsyst         [scaffolding]
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

### Dependency graph

```
core_constants, core_units, core_math      (no internal dependencies)
        |
solar_geometry (needs core_*)
        |
        +--> solar_position (needs solar_geometry)
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
validation (needs core_math; will need everything, once implemented)
```

Each crate is independently buildable and testable
(`cargo test -p <crate_name>`).

## Building and testing

```bash
cd engine
cargo build --workspace                  # compile every crate
cargo test --workspace                   # run every test (114 tests as of this commit)
cargo clippy --workspace --all-targets   # lint (zero warnings)
cargo fmt --all -- --check               # formatting (clean)
cargo doc --workspace --no-deps --open   # browse the referenced API documentation
```

No network access, no external services, and no dependency outside the
Rust standard library are required to build or test this engine (verify
with `cargo tree --workspace`, which lists path dependencies only).

## Precision scope: read before using in production

The Sun-position algorithm implemented here is **not** a byte-for-byte
reproduction of the full NREL SPA (Reda & Andreas 2004) coefficient tables —
see the Architecture Decision Record at the top of
`crates/solar_geometry/src/lib.rs` for exactly why, and
`docs/design/16-phase1-validation-report.md` for the full validation
methodology and its current limits. In short: this development session had
no working network access to the primary NREL source, so a reduced-order,
still fully-cited (Meeus 1998) solar-position series is used instead
(~0.01-0.02° accuracy vs. SPA's ±0.0003°), with the full multi-step
topocentric procedure otherwise implemented exactly. This is judged
sufficient for the <1% energy-yield deviation target, but has not yet been
cross-validated against actual PVsyst/SAM/pvlib/Archelios Calc output (also
unavailable in this session — see `crates/validation/src/lib.rs`) — only
against internal physical/geometric invariants, algebraic invariants, and
independent formula cross-checks. See the validation report for the
complete, honest accounting of what has and has not been verified.

## Documentation

- Design-phase documents: `../docs/design/`
- Phase 1 validation report: `../docs/design/16-phase1-validation-report.md`
- API documentation: generate with `cargo doc` (every public item is
  documented with a citation to its source, per project rules)
