//! Manual performance benchmarks (`std::time::Instant`-based, no external
//! benchmarking crate — see `engine/README.md` for why this workspace has
//! zero non-standard-library dependencies, a property these benchmarks
//! deliberately preserve). These are regression guards, not micro-
//! benchmarks: they assert a generous wall-clock budget for a realistic
//! workload (one year of hourly solar-position evaluations, the unit of
//! work `simulation`'s hourly energy-balance loop performs 8760 times per
//! simulated year) so an accidental algorithmic regression (e.g. an O(n)
//! table scan turning into O(n^2)) would be caught, without being so tight
//! that ordinary machine-to-machine performance variance causes flaky
//! failures.
//!
//! For exact per-call timing figures (not just a pass/fail budget), see
//! `validation-data/latest_validation_report.md`, which `cargo test -p
//! validation` regenerates with real timing from the 91-case reference
//! dataset on every run.

use std::time::Instant;

use core_math::julian::CalendarDateTime;
use core_units::{Angle, Length};
use solar_position::{AtmosphericConditions, MeeusReduced, Observer, SolarPositionAlgorithm, Spa};

const HOURS_PER_YEAR: u32 = 8760;

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

fn benchmark_engine<A: SolarPositionAlgorithm>(algorithm: &A) -> std::time::Duration {
    let observer = Observer {
        latitude: Angle::from_degrees(45.0),
        longitude: Angle::from_degrees(5.0),
        elevation: Length::from_meters(200.0),
    };
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
