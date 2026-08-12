//! The scenario harness: what a scenario is, how it is judged, and how the
//! census over all of them is produced.
//!
//! ## Why the scenarios are data rather than statements
//!
//! Counting `#[test]` functions says nothing about how thoroughly an
//! invariant is exercised: one function may assert three cases or thirty,
//! and no tool can tell from the outside. Making each scenario a value
//! makes the count a measurement instead of a claim — the census in
//! `tests/invariants.rs` enumerates the very same values the assertions
//! run on, so the two cannot drift.
//!
//! Each scenario names the invariant it belongs to, states what it expects
//! *before* running anything, and is judged by comparing the expectation
//! against the report the check actually produced. A scenario that expects
//! silence is as much a scenario as one that expects a finding: an
//! invariant that fires when it should not is exactly as defective as one
//! that stays quiet when it should not.

#![allow(dead_code)]

use pv_module::{ValidationIssue, ValidationReport, ValidationStatus};

/// What a scenario asserts about the invariant it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// The invariant must say nothing: either it holds, or the data it
    /// needs is absent, or the case belongs to a different invariant.
    Silent,
    /// The invariant must report exactly this status.
    Reported(ValidationStatus),
}

impl Expect {
    /// A short label for the census table.
    pub fn label(self) -> &'static str {
        match self {
            Expect::Silent => "silent",
            Expect::Reported(ValidationStatus::Valid) => "INFO",
            Expect::Reported(ValidationStatus::Unknown) => "UNKNOWN",
            Expect::Reported(ValidationStatus::Warning) => "WARNING",
            Expect::Reported(ValidationStatus::Conflict) => "CONFLICT",
            Expect::Reported(ValidationStatus::Invalid) => "INVALID",
        }
    }
}

/// An extra assertion on the finding a scenario produced, for the cases
/// where the status alone is not the whole claim — that a conflict names
/// both candidate values, for instance.
///
/// Returns `Err` rather than panicking so that one failing scenario does
/// not hide the outcome of the others in the census.
pub type Inspect = fn(&ValidationIssue) -> Result<(), String>;

/// One executed case: an invariant, a situation, and the outcome expected
/// of it.
pub struct Scenario {
    /// The invariant this case belongs to, e.g. `"INV-GEO-002"`.
    pub invariant: &'static str,
    /// What the case is, in words, for the census table.
    pub case: &'static str,
    /// What the invariant must do about it.
    pub expect: Expect,
    /// Builds the situation and runs the check on it.
    pub run: fn() -> ValidationReport,
    /// An optional further assertion on the finding.
    pub inspect: Option<Inspect>,
}

impl Scenario {
    /// Run the case and judge it. `Ok(())` means the invariant did exactly
    /// what the scenario said it would.
    pub fn evaluate(&self) -> Result<(), String> {
        let report = (self.run)();
        let found = report.find(self.invariant);

        match (self.expect, found) {
            (Expect::Silent, None) => Ok(()),
            (Expect::Silent, Some(issue)) => Err(format!(
                "expected silence, but the invariant reported {:?} ({})",
                issue.status(),
                issue.explanation()
            )),
            (Expect::Reported(expected), None) => {
                Err(format!("expected {expected:?}, but nothing was reported"))
            }
            (Expect::Reported(expected), Some(issue)) => {
                if issue.status() != expected {
                    return Err(format!(
                        "expected {expected:?}, but the invariant reported {:?}",
                        issue.status()
                    ));
                }
                // Every finding owes the reader an explanation and the
                // fields it concerns; a status alone is an opaque refusal.
                if issue.explanation().trim().is_empty() {
                    return Err("the finding carries no explanation".to_string());
                }
                if issue.fields().is_empty() {
                    return Err("the finding names no field".to_string());
                }
                match self.inspect {
                    Some(inspect) => inspect(issue),
                    None => Ok(()),
                }
            }
        }
    }
}

/// Declare a scenario expecting the invariant to stay silent.
pub const fn silent(
    invariant: &'static str,
    case: &'static str,
    run: fn() -> ValidationReport,
) -> Scenario {
    Scenario {
        invariant,
        case,
        expect: Expect::Silent,
        run,
        inspect: None,
    }
}

/// Declare a scenario expecting the invariant to report a given status.
pub const fn reported(
    invariant: &'static str,
    case: &'static str,
    status: ValidationStatus,
    run: fn() -> ValidationReport,
) -> Scenario {
    Scenario {
        invariant,
        case,
        expect: Expect::Reported(status),
        run,
        inspect: None,
    }
}

/// Declare a scenario expecting a status *and* something about the finding.
pub const fn reported_with(
    invariant: &'static str,
    case: &'static str,
    status: ValidationStatus,
    run: fn() -> ValidationReport,
    inspect: Inspect,
) -> Scenario {
    Scenario {
        invariant,
        case,
        expect: Expect::Reported(status),
        run,
        inspect: Some(inspect),
    }
}

/// What running every scenario of one invariant established.
#[derive(Debug, Clone)]
pub struct InvariantCensus {
    /// The invariant's identifier.
    pub invariant: &'static str,
    /// How many scenarios were actually executed for it.
    pub scenarios: usize,
    /// How many produced the outcome they declared.
    pub passed: usize,
    /// The ones that did not, with the reason.
    pub failures: Vec<(&'static str, String)>,
}

impl InvariantCensus {
    /// How many scenarios did not do what they said.
    pub fn failed(&self) -> usize {
        self.failures.len()
    }
}

/// The minimum number of executed scenarios each invariant must carry.
///
/// Three is the floor stated at validation of the sub-phase: in practice
/// it forces at least the coherent case, the violated case, and one
/// boundary or absence case, which together are what distinguish a real
/// check from one that merely fires.
pub const MINIMUM_SCENARIOS_PER_INVARIANT: usize = 3;

/// Run every scenario, grouped by invariant, preserving declaration order.
pub fn census(scenarios: &[Scenario]) -> Vec<InvariantCensus> {
    let mut result: Vec<InvariantCensus> = Vec::new();
    for scenario in scenarios {
        let index = match result
            .iter()
            .position(|c| c.invariant == scenario.invariant)
        {
            Some(existing) => existing,
            None => {
                result.push(InvariantCensus {
                    invariant: scenario.invariant,
                    scenarios: 0,
                    passed: 0,
                    failures: Vec::new(),
                });
                result.len() - 1
            }
        };
        let entry = &mut result[index];
        entry.scenarios += 1;
        match scenario.evaluate() {
            Ok(()) => entry.passed += 1,
            Err(reason) => entry.failures.push((scenario.case, reason)),
        }
    }
    result.sort_by_key(|c| c.invariant);
    result
}

/// Render the census as the table required by the traceability check.
pub fn render_table(census: &[InvariantCensus], scenarios: &[Scenario]) -> String {
    let total: usize = census.iter().map(|c| c.scenarios).sum();
    let passed: usize = census.iter().map(|c| c.passed).sum();
    let failed: usize = census.iter().map(InvariantCensus::failed).sum();
    let deficient = census
        .iter()
        .filter(|c| c.scenarios < MINIMUM_SCENARIOS_PER_INVARIANT)
        .count();

    let mut out = String::new();
    out.push_str("# Matrice de scénarios par invariant — `pv_module`\n\n");
    out.push_str(
        "Généré par `pv_module::tests::invariants::the_scenario_census_is_written_to_disk`.\n\
         **Ne pas modifier à la main.**\n\n\
         Chaque ligne compte les scénarios *réellement exécutés*, pas les fonctions de test.\n\
         Un scénario déclare son résultat attendu avant exécution ; il est compté PASS\n\
         lorsque l'invariant a produit exactement ce résultat, explication et champs\n\
         nommés compris.\n\n",
    );
    out.push_str(&format!(
        "- Invariants recensés : **{}**\n\
         - Scénarios exécutés : **{total}**\n\
         - PASS : **{passed}** · FAIL : **{failed}**\n\
         - Seuil requis : **{MINIMUM_SCENARIOS_PER_INVARIANT}** scénarios par invariant \
         — invariants en dessous : **{deficient}**\n\n",
        census.len()
    ));

    out.push_str("| INVARIANT | NOMBRE DE SCÉNARIOS | PASS | FAIL |\n");
    out.push_str("|---|---:|---:|---:|\n");
    for entry in census {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            entry.invariant,
            entry.scenarios,
            entry.passed,
            entry.failed()
        ));
    }
    out.push_str(&format!(
        "| **Total** | **{total}** | **{passed}** | **{failed}** |\n"
    ));

    out.push_str("\n## Détail des scénarios\n\n");
    out.push_str("| INVARIANT | SCÉNARIO | ATTENDU |\n|---|---|---|\n");
    let mut sorted: Vec<&Scenario> = scenarios.iter().collect();
    sorted.sort_by_key(|s| s.invariant);
    for scenario in sorted {
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            scenario.invariant,
            scenario.case,
            scenario.expect.label()
        ));
    }
    out
}

// =====================================================================
// Self-tests of the harness
//
// The census gates the build, so it must itself be trustworthy: a counter
// that never reports a shortfall, or a judge that passes everything,
// would turn the whole check into decoration. These exercise the harness
// against synthetic scenarios whose outcome is known by construction.
// =====================================================================

fn empty_report() -> ValidationReport {
    ValidationReport::new()
}

fn warning_report() -> ValidationReport {
    let mut report = ValidationReport::new();
    report.push(ValidationIssue::new(
        "INV-TEST-001",
        ValidationStatus::Warning,
        vec!["a_field"],
        "an explanation",
    ));
    report
}

fn undocumented_report() -> ValidationReport {
    let mut report = ValidationReport::new();
    report.push(ValidationIssue::new(
        "INV-TEST-001",
        ValidationStatus::Warning,
        vec!["a_field"],
        "",
    ));
    report
}

fn fieldless_report() -> ValidationReport {
    let mut report = ValidationReport::new();
    report.push(ValidationIssue::new(
        "INV-TEST-001",
        ValidationStatus::Warning,
        Vec::new(),
        "an explanation",
    ));
    report
}

#[test]
fn the_judge_accepts_only_the_declared_outcome() {
    // Silence expected and obtained.
    assert!(
        silent("INV-TEST-001", "quiet", empty_report)
            .evaluate()
            .is_ok()
    );
    // Silence expected, a finding produced.
    assert!(
        silent("INV-TEST-001", "noisy", warning_report)
            .evaluate()
            .is_err()
    );
    // A finding expected and obtained.
    assert!(
        reported(
            "INV-TEST-001",
            "warns",
            ValidationStatus::Warning,
            warning_report
        )
        .evaluate()
        .is_ok()
    );
    // A finding expected, nothing produced.
    assert!(
        reported(
            "INV-TEST-001",
            "silent",
            ValidationStatus::Warning,
            empty_report
        )
        .evaluate()
        .is_err()
    );
    // The right invariant, the wrong severity.
    assert!(
        reported(
            "INV-TEST-001",
            "wrong status",
            ValidationStatus::Invalid,
            warning_report
        )
        .evaluate()
        .is_err()
    );
}

#[test]
fn a_finding_without_an_explanation_or_a_field_is_rejected() {
    // A status with no explanation is an opaque refusal, which the
    // specification forbids; the harness refuses to pass one.
    let bare = reported(
        "INV-TEST-001",
        "no explanation",
        ValidationStatus::Warning,
        undocumented_report,
    );
    assert!(bare.evaluate().unwrap_err().contains("no explanation"));

    let anonymous = reported(
        "INV-TEST-001",
        "no field named",
        ValidationStatus::Warning,
        fieldless_report,
    );
    assert!(anonymous.evaluate().unwrap_err().contains("names no field"));
}

#[test]
fn an_extra_assertion_can_fail_an_otherwise_correct_status() {
    let scenario = reported_with(
        "INV-TEST-001",
        "right status, wrong detail",
        ValidationStatus::Warning,
        warning_report,
        |issue| {
            if issue.observed().is_some() {
                Ok(())
            } else {
                Err("no observed value".to_string())
            }
        },
    );
    assert!(scenario.evaluate().is_err());
}

#[test]
fn the_census_counts_scenarios_and_separates_pass_from_fail() {
    let scenarios = vec![
        silent("INV-TEST-001", "a", empty_report),
        silent("INV-TEST-001", "b", empty_report),
        // Declares silence but the check reports: a genuine failure.
        silent("INV-TEST-001", "c", warning_report),
        silent("INV-TEST-002", "d", empty_report),
    ];
    let counted = census(&scenarios);

    assert_eq!(counted.len(), 2);
    let first = &counted[0];
    assert_eq!(first.invariant, "INV-TEST-001");
    assert_eq!(first.scenarios, 3);
    assert_eq!(first.passed, 2);
    assert_eq!(first.failed(), 1);
    assert_eq!(first.failures[0].0, "c");

    let second = &counted[1];
    assert_eq!(second.scenarios, 1);
    assert_eq!(second.failed(), 0);
}

#[test]
fn the_census_detects_an_invariant_below_the_floor() {
    // Two scenarios is one short: the guard must see it. Without this
    // test, a census that never reported a shortfall would look identical
    // to one that had nothing to report.
    let scenarios = vec![
        silent("INV-TEST-001", "a", empty_report),
        silent("INV-TEST-001", "b", empty_report),
    ];
    let counted = census(&scenarios);
    assert!(counted[0].scenarios < MINIMUM_SCENARIOS_PER_INVARIANT);

    let sufficient = vec![
        silent("INV-TEST-001", "a", empty_report),
        silent("INV-TEST-001", "b", empty_report),
        silent("INV-TEST-001", "c", empty_report),
    ];
    assert!(census(&sufficient)[0].scenarios >= MINIMUM_SCENARIOS_PER_INVARIANT);
}

#[test]
fn the_rendered_table_reports_what_the_census_found() {
    let scenarios = vec![
        silent("INV-TEST-001", "a passing case", empty_report),
        silent("INV-TEST-001", "a failing case", warning_report),
    ];
    let table = render_table(&census(&scenarios), &scenarios);

    assert!(table.contains("| INVARIANT | NOMBRE DE SCÉNARIOS | PASS | FAIL |"));
    assert!(table.contains("| `INV-TEST-001` | 2 | 1 | 1 |"));
    // A shortfall is stated rather than left to be inferred.
    assert!(table.contains("invariants en dessous : **1**"));
    // Each scenario appears with its declared expectation.
    assert!(table.contains("a passing case"));
    assert!(table.contains("silent"));
}
