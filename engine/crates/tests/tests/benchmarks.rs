//! Manual performance benchmarks (`std::time::Instant`-based, no external
//! benchmarking crate — see `engine/README.md` for why this workspace has
//! zero non-standard-library dependencies, a property these benchmarks
//! deliberately preserve). The first three tests below are regression
//! guards, not micro-benchmarks: they assert a generous wall-clock budget
//! for a realistic workload (one year of hourly solar-position
//! evaluations, the unit of work `simulation`'s hourly energy-balance loop
//! performs 8760 times per simulated year) so an accidental algorithmic
//! regression (e.g. an O(n) table scan turning into O(n^2)) would be
//! caught, without being so tight that ordinary machine-to-machine
//! performance variance causes flaky failures.
//!
//! The remaining tests extend this to the full set of measurements
//! requested for the solar-position engine's exit criteria: mean *and*
//! standard deviation of per-call timing (not just a pooled average), and
//! memory consumed / number of allocations, measured via a small counting
//! `#[global_allocator]` wrapper defined in this file.
//!
//! **Why this one file uses `unsafe`, and why that is still consistent
//! with the project's "zero unsafe in the engine" rule**: implementing
//! `std::alloc::GlobalAlloc` requires `unsafe fn` bodies (the trait's own
//! definition, not a choice made here) — there is no safe way to hook
//! global allocation/deallocation in std-only Rust, and a global allocator
//! is the only way to count allocations without adding a profiling
//! dependency (which would violate the same zero-non-std-dependency
//! policy). `engine/crates/tests/Cargo.toml` relaxes `unsafe_code` from
//! the workspace default (`forbid`) to `allow` for exactly this reason,
//! documented there; every other crate in the workspace — the actual
//! scientific engine — keeps `unsafe_code = "forbid"` unchanged.
//!
//! For exact per-call timing figures from the external validation
//! datasets (not this file's synthetic one-year sweep), see
//! `validation-data/latest_validation_report.md` and
//! `latest_global_campaign_report.md`. This file's own results are written
//! to `validation-data/latest_benchmark_report.md`, regenerated on every
//! `cargo test -p tests` run.

use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use core_math::julian::CalendarDateTime;
use core_math::stats;
use core_units::{Angle, Length};
use solar_position::{AtmosphericConditions, MeeusReduced, Observer, SolarPositionAlgorithm, Spa};

const HOURS_PER_YEAR: u32 = 8760;

// --- Counting global allocator -------------------------------------------

static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

/// Wraps [`System`], counting every `alloc`/`realloc` call and the bytes
/// each requests, so a benchmark can measure allocations attributable to a
/// specific code section by reading the counters before and after (see
/// [`measure_allocations`]). `dealloc` is passed through without counting
/// (deallocation count always equals allocation count for code with no
/// intentional leaks, so it carries no additional information here; bytes
/// freed are not subtracted from `ALLOC_BYTES`, which therefore measures
/// *total bytes requested*, not peak or net resident memory — the
/// simplest, least assumption-laden definition of "memory consumed" for a
/// short, single-threaded benchmark loop).
struct CountingAllocator;

// SAFETY: every method delegates directly to `System` (the platform
// default allocator, itself a valid `GlobalAlloc` implementation) for the
// actual memory operation; the only added logic is `Ordering::Relaxed`
// atomic counter increments before/after the delegated call, which have no
// memory-safety preconditions of their own. This is the standard shape of
// an allocation-counting wrapper (as used by, e.g., the `stats_alloc`
// crate this project deliberately does not depend on).
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        if new_size > layout.size() {
            ALLOC_BYTES.fetch_add(new_size - layout.size(), Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// `cargo test` runs the `#[test]` functions within one binary
/// concurrently, on multiple threads, by default. `ALLOC_CALLS`/
/// `ALLOC_BYTES` are process-global counters, and this whole benchmark
/// suite also shares CPU cores when measuring wall-clock timing, so every
/// `#[test]` in this file acquires this mutex as its very first action --
/// serializing the entire binary -- to avoid two failure modes that would
/// otherwise be silently possible: (a) a sibling test's heap allocations
/// landing inside another test's allocation-count measurement window
/// (corrupting the "zero allocations" claim with unrelated activity), and
/// (b) sibling tests contending for CPU time during a timing measurement
/// (inflating the reported mean/stddev with scheduler noise beyond what
/// §17.3 of `docs/design/17-analyse-incertitude-numerique.md` already
/// attributes to `Instant::now()` overhead). This has no effect on any
/// *other* test binary (`regression.rs`, `numerical_stability.rs`): each
/// file under `tests/` compiles to its own separate process, so this
/// mutex only ever serializes tests against each other within this one
/// file.
static TEST_SERIAL: Mutex<()> = Mutex::new(());

fn serialize_tests() -> MutexGuard<'static, ()> {
    TEST_SERIAL.lock().unwrap_or_else(|poisoned| {
        // A prior test in this file panicked while holding the lock; that
        // failure is already reported by the test harness independently,
        // and poisoning must not cascade into every subsequent test in
        // this file failing for an unrelated reason.
        poisoned.into_inner()
    })
}

/// Allocation activity attributable to a measured section, per
/// [`measure_allocations`].
#[derive(Debug, Clone, Copy)]
struct AllocationProfile {
    /// Number of `alloc`/`realloc` calls made while `f` ran.
    calls: usize,
    /// Total bytes requested across those calls (see
    /// [`CountingAllocator`]'s doc comment for exactly what this does and
    /// does not measure).
    bytes: usize,
}

/// Runs `f`, returning the allocator activity attributable to it (the
/// difference in the global counters before/after). Relies on the caller
/// already holding [`TEST_SERIAL`] (every `#[test]` in this file does, via
/// [`serialize_tests`]) so no other test's allocations land inside `f`'s
/// measurement window.
fn measure_allocations<F: FnOnce()>(f: F) -> AllocationProfile {
    let calls_before = ALLOC_CALLS.load(Ordering::Relaxed);
    let bytes_before = ALLOC_BYTES.load(Ordering::Relaxed);
    f();
    let calls_after = ALLOC_CALLS.load(Ordering::Relaxed);
    let bytes_after = ALLOC_BYTES.load(Ordering::Relaxed);
    AllocationProfile {
        calls: calls_after - calls_before,
        bytes: bytes_after - bytes_before,
    }
}

// --- Shared benchmark inputs ----------------------------------------------

fn hourly_datetimes(year: i32) -> impl Iterator<Item = CalendarDateTime> {
    (0..HOURS_PER_YEAR).map(move |h| {
        let day_of_year = h / 24;
        let hour = h % 24;
        // Deliberately not calendar-accurate month/day bookkeeping (this
        // is a performance benchmark, not a calendar test -- month/day
        // wrap-around past 28 is irrelevant to timing) -- reuses
        // `CalendarDateTime`'s own validation range for `day` up to 31,
        // cycling within a plausible month so every call is still a valid
        // Gregorian date.
        let month = 1 + (day_of_year / 28) % 12;
        let day = 1 + (day_of_year % 28);
        CalendarDateTime {
            year,
            month,
            day,
            hour,
            minute: 0,
            second: 0.0,
        }
    })
}

fn benchmark_observer() -> Observer {
    Observer {
        latitude: Angle::from_degrees(45.0),
        longitude: Angle::from_degrees(5.0),
        elevation: Length::from_meters(200.0),
    }
}

fn benchmark_engine<A: SolarPositionAlgorithm>(algorithm: &A) -> Duration {
    let observer = benchmark_observer();
    let atmosphere = AtmosphericConditions::default();

    let start = Instant::now();
    for dt in hourly_datetimes(2024) {
        let position = algorithm.solar_position(dt, observer, atmosphere).unwrap();
        // Prevent the optimizer from eliding the computation entirely.
        std::hint::black_box(position);
    }
    start.elapsed()
}

#[test]
fn spa_computes_one_year_of_hourly_positions_within_budget() {
    let _guard = serialize_tests();
    let elapsed = benchmark_engine(&Spa);
    eprintln!(
        "SPA: {HOURS_PER_YEAR} hourly positions in {:.2} ms ({:.2} us/call)",
        elapsed.as_secs_f64() * 1e3,
        elapsed.as_secs_f64() * 1e6 / HOURS_PER_YEAR as f64
    );
    // Generous budget: the validation campaign measured ~5.6 us/call, so
    // 8760 calls should take on the order of 50 ms; 2 s leaves more than
    // an order of magnitude of headroom for slower CI hardware while still
    // catching an accidental quadratic-time regression.
    assert!(
        elapsed.as_secs_f64() < 2.0,
        "SPA took {:.3} s for one year of hourly positions (budget: 2 s)",
        elapsed.as_secs_f64()
    );
}

#[test]
fn meeus_computes_one_year_of_hourly_positions_within_budget() {
    let _guard = serialize_tests();
    let elapsed = benchmark_engine(&MeeusReduced);
    eprintln!(
        "Meeus: {HOURS_PER_YEAR} hourly positions in {:.2} ms ({:.2} us/call)",
        elapsed.as_secs_f64() * 1e3,
        elapsed.as_secs_f64() * 1e6 / HOURS_PER_YEAR as f64
    );
    assert!(
        elapsed.as_secs_f64() < 2.0,
        "Meeus took {:.3} s for one year of hourly positions (budget: 2 s)",
        elapsed.as_secs_f64()
    );
}

#[test]
fn spa_is_not_dramatically_slower_than_meeus() {
    let _guard = serialize_tests();
    // SPA sums much larger periodic-term tables than Meeus per call, so
    // some slowdown is expected and acceptable; this guards against a
    // pathological regression (e.g. an accidental O(n^2) table lookup)
    // rather than asserting a specific ratio.
    let spa_elapsed = benchmark_engine(&Spa);
    let meeus_elapsed = benchmark_engine(&MeeusReduced);
    let ratio = spa_elapsed.as_secs_f64() / meeus_elapsed.as_secs_f64();
    eprintln!("SPA/Meeus wall-clock ratio: {ratio:.1}x");
    assert!(
        ratio < 100.0,
        "SPA is {ratio:.1}x slower than Meeus (budget: 100x)"
    );
}

// --- Per-call timing: mean and standard deviation --------------------------

/// Per-call timing statistics over one simulated year of hourly calls.
#[derive(Debug, Clone, Copy)]
struct TimingProfile {
    sample_count: usize,
    mean_us: f64,
    /// Sample standard deviation ([`stats::standard_deviation`]) of the
    /// per-call time, in microseconds. Note this measures timer *and*
    /// scheduler noise as well as genuine algorithmic variance:
    /// `Instant::now()` itself has nonzero, platform-dependent overhead
    /// (commonly tens of nanoseconds) that is not negligible relative to a
    /// single ~1-10 us call, so this standard deviation is an upper bound
    /// on the engine's own timing variability, not a purified measurement
    /// of it -- reported transparently rather than filtered, per the
    /// project's "no unsourced/adjusted numbers" rule.
    stddev_us: f64,
    min_us: f64,
    max_us: f64,
}

fn benchmark_per_call_timing<A: SolarPositionAlgorithm>(algorithm: &A) -> TimingProfile {
    let observer = benchmark_observer();
    let atmosphere = AtmosphericConditions::default();

    let mut samples_us: Vec<f64> = Vec::with_capacity(HOURS_PER_YEAR as usize);
    for dt in hourly_datetimes(2024) {
        let start = Instant::now();
        let position = algorithm.solar_position(dt, observer, atmosphere).unwrap();
        let elapsed = start.elapsed();
        std::hint::black_box(position);
        samples_us.push(elapsed.as_secs_f64() * 1e6);
    }

    let mean_us = stats::mean(&samples_us);
    let stddev_us = stats::standard_deviation(&samples_us);
    let min_us = samples_us.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_us = samples_us.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    TimingProfile {
        sample_count: samples_us.len(),
        mean_us,
        stddev_us,
        min_us,
        max_us,
    }
}

#[test]
fn spa_timing_mean_and_standard_deviation() {
    let _guard = serialize_tests();
    let profile = benchmark_per_call_timing(&Spa);
    eprintln!(
        "SPA per-call timing over {} samples: mean={:.3} us, stddev={:.3} us, min={:.3} us, max={:.3} us",
        profile.sample_count, profile.mean_us, profile.stddev_us, profile.min_us, profile.max_us
    );
    assert!(profile.mean_us.is_finite() && profile.mean_us > 0.0);
    assert!(profile.stddev_us.is_finite() && profile.stddev_us >= 0.0);
}

#[test]
fn meeus_timing_mean_and_standard_deviation() {
    let _guard = serialize_tests();
    let profile = benchmark_per_call_timing(&MeeusReduced);
    eprintln!(
        "Meeus per-call timing over {} samples: mean={:.3} us, stddev={:.3} us, min={:.3} us, max={:.3} us",
        profile.sample_count, profile.mean_us, profile.stddev_us, profile.min_us, profile.max_us
    );
    assert!(profile.mean_us.is_finite() && profile.mean_us > 0.0);
    assert!(profile.stddev_us.is_finite() && profile.stddev_us >= 0.0);
}

// --- Memory / allocation profile -------------------------------------------

#[test]
fn spa_allocates_nothing_on_the_heap_per_call() {
    let _guard = serialize_tests();
    // Warm up with a FULL pass first (not just one call): the very first
    // few calls into a fresh process can trigger one-time, amortized
    // lazy-initialization allocations in the Rust runtime/OS timing
    // machinery (e.g. `Instant::now()`'s first VDSO/clock-source lookup),
    // which are a startup cost, not a per-call cost of the position
    // algorithm -- excluding a full warm-up pass isolates the steady-state
    // claim this test actually makes.
    std::hint::black_box(benchmark_engine(&Spa));

    let profile = measure_allocations(|| {
        let elapsed = benchmark_engine(&Spa);
        std::hint::black_box(elapsed);
    });
    eprintln!(
        "SPA: {} allocator calls, {} bytes requested, over {HOURS_PER_YEAR} position computations \
         ({:.4} calls/position, {:.2} bytes/position)",
        profile.calls,
        profile.bytes,
        profile.calls as f64 / HOURS_PER_YEAR as f64,
        profile.bytes as f64 / HOURS_PER_YEAR as f64,
    );
    // The SPA pipeline (periodic-term summation via `Iterator::sum`,
    // trigonometric functions, struct returns) is entirely stack-based --
    // no `Vec`/`String`/`Box` is constructed per call -- so zero heap
    // allocation is the specific, falsifiable claim being tested here, not
    // just "few".
    assert_eq!(
        profile.calls, 0,
        "SPA made {} heap allocator calls for {HOURS_PER_YEAR} position computations, expected 0",
        profile.calls
    );
}

#[test]
fn meeus_allocates_nothing_on_the_heap_per_call() {
    let _guard = serialize_tests();
    // See `spa_allocates_nothing_on_the_heap_per_call` for why this is a
    // full warm-up pass, not a single call.
    std::hint::black_box(benchmark_engine(&MeeusReduced));

    let profile = measure_allocations(|| {
        let elapsed = benchmark_engine(&MeeusReduced);
        std::hint::black_box(elapsed);
    });
    eprintln!(
        "Meeus: {} allocator calls, {} bytes requested, over {HOURS_PER_YEAR} position computations",
        profile.calls, profile.bytes,
    );
    assert_eq!(
        profile.calls, 0,
        "Meeus made {} heap allocator calls for {HOURS_PER_YEAR} position computations, expected 0",
        profile.calls
    );
}

// --- Report generation -------------------------------------------------

/// Regenerates `validation-data/latest_benchmark_report.md` from a fresh
/// run of every measurement above, so the numbers cited in
/// `docs/design/` documents can always be traced back to a reproducible
/// `cargo test` run rather than a stale, hand-copied figure.
#[test]
fn generate_and_write_benchmark_report() {
    let _guard = serialize_tests();
    let spa_wall_clock = benchmark_engine(&Spa);
    let meeus_wall_clock = benchmark_engine(&MeeusReduced);
    let spa_timing = benchmark_per_call_timing(&Spa);
    let meeus_timing = benchmark_per_call_timing(&MeeusReduced);
    let spa_alloc = measure_allocations(|| {
        std::hint::black_box(benchmark_engine(&Spa));
    });
    let meeus_alloc = measure_allocations(|| {
        std::hint::black_box(benchmark_engine(&MeeusReduced));
    });

    let mut md = String::new();
    md.push_str("# HeliosPro — Rapport de performance (moteur de position solaire)\n\n");
    md.push_str(
        "Genere par `tests::benchmarks::generate_and_write_benchmark_report` \
         (`cargo test -p tests --release`), sur un an d'appels horaires \
         (8760 positions), a chaque execution de la suite de tests. Mesures \
         manuelles via `std::time::Instant` et un allocateur global \
         compteur (voir le commentaire de module de ce fichier) -- aucune \
         dependance de benchmarking externe.\n\n",
    );
    md.push_str("| Mesure | SPA | Meeus (reduit) |\n");
    md.push_str("|---|---|---|\n");
    md.push_str(&format!(
        "| Temps total (8760 appels) | {:.2} ms | {:.2} ms |\n",
        spa_wall_clock.as_secs_f64() * 1e3,
        meeus_wall_clock.as_secs_f64() * 1e3
    ));
    md.push_str(&format!(
        "| Temps moyen par appel | {:.3} us | {:.3} us |\n",
        spa_timing.mean_us, meeus_timing.mean_us
    ));
    md.push_str(&format!(
        "| Ecart-type par appel | {:.3} us | {:.3} us |\n",
        spa_timing.stddev_us, meeus_timing.stddev_us
    ));
    md.push_str(&format!(
        "| Minimum / maximum par appel | {:.3} / {:.3} us | {:.3} / {:.3} us |\n",
        spa_timing.min_us, spa_timing.max_us, meeus_timing.min_us, meeus_timing.max_us
    ));
    md.push_str(&format!(
        "| Appels allocateur (8760 positions) | {} | {} |\n",
        spa_alloc.calls, meeus_alloc.calls
    ));
    md.push_str(&format!(
        "| Octets alloues (8760 positions) | {} | {} |\n",
        spa_alloc.bytes, meeus_alloc.bytes
    ));
    md.push_str(&format!(
        "\nRatio de temps moyen SPA/Meeus : {:.2}x.\n",
        spa_timing.mean_us / meeus_timing.mean_us
    ));

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_path = manifest_dir
        .join("..")
        .join("..")
        .join("validation-data")
        .join("latest_benchmark_report.md");
    fs::write(&output_path, &md)
        .unwrap_or_else(|e| panic!("failed to write benchmark report to {output_path:?}: {e}"));
}
