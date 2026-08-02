# HeliosPro Scientific Engine — Phase 1: Core Engine

This directory contains the Phase 1 deliverable of HeliosPro's development:
the **scientific core engine**, a dependency-light Rust workspace with no
GUI, no database, and no I/O beyond pure function calls, per the Phase 1
brief.

## Scope (per the Phase 1 brief)

- Physical constants library (referenced)
- SI units (strongly-typed newtypes)
- Math library (calendar/Julian Day, interpolation, error statistics)
- Solar time
- Solar position (SPA-class algorithm — see the precision ADR below)
- Angle of incidence
- Extraterrestrial irradiance
- GHI / DNI / DHI relationships and decomposition
- Sky-diffuse transposition models: isotropic, Hay-Davies, Perez, HDKR
- Plane-of-array irradiance
- Cell temperature: NOCT, Sandia (SAPM)
- Complete energy balance (orchestration of all of the above)

Explicitly out of scope for this phase (per the brief): PV electrical
modelling (single-diode), inverter modelling, storage, dispatch,
optimization, reporting, any UI, any persistence layer.

## Workspace layout

```
engine/
├── Cargo.toml                    # workspace manifest
└── crates/
    ├── heliospro-constants/       # referenced physical/astronomical constants
    ├── heliospro-units/           # SI unit newtypes (Angle, Temperature, Irradiance, ...)
    ├── heliospro-math/            # Julian Day, interpolation, error statistics
    ├── heliospro-solar/           # solar time, SPA-class position, incidence, extraterrestrial, decomposition
    ├── heliospro-transposition/   # isotropic, Hay-Davies, HDKR, Perez POA models
    ├── heliospro-thermal/         # NOCT, Sandia cell-temperature models
    └── heliospro-energy-balance/  # top-level orchestration ("bilan énergétique complet")
```

Each crate is independently buildable and testable (`cargo test -p <crate>`)
and depends only on the crates strictly below it in the physical model
(constants/units/math have no internal dependencies; solar depends on
constants/units/math; transposition and thermal depend on solar/units;
energy-balance depends on all of the above).

## Building and testing

```bash
cd engine
cargo build --workspace            # compile every crate
cargo test --workspace             # run every unit test (111 tests, Phase 1)
cargo clippy --workspace --all-targets   # lint (zero warnings as of this phase)
cargo doc --workspace --no-deps --open   # browse the referenced API documentation
```

No network access, no external services, and no non-Rust-standard-library
runtime dependency are required to build or test this engine.

## Precision scope: read before using in production

The Sun-position algorithm implemented here is **not** a byte-for-byte
reproduction of the full NREL SPA (Reda & Andreas 2004) coefficient tables —
see the Architecture Decision Record at the top of
`crates/heliospro-solar/src/lib.rs` for exactly why, and
`docs/design/16-phase1-validation-report.md` for the full validation
methodology and its current limits. In short: this development session had
no working network access to the primary NREL source, so a reduced-order,
still fully-cited (Meeus 1998) solar-position series is used instead
(~0.01-0.02° accuracy vs. SPA's ±0.0003°), with the full multi-step
topocentric procedure otherwise implemented exactly. This is judged
sufficient for the <1% energy-yield deviation target, but has not yet been
cross-validated against actual PVsyst/SAM/Archelios Calc output (also
unavailable in this session) — only against internal physical/geometric
invariants and independent formula cross-checks. See the validation report
for the complete, honest accounting of what has and has not been verified.

## Documentation

- Design-phase documents: `../docs/design/`
- Phase 1 validation report: `../docs/design/16-phase1-validation-report.md`
- API documentation: generate with `cargo doc` (every public item is
  documented with a citation to its source, per project rules)
