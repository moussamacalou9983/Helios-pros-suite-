//! # validation
//!
//! External validation of HeliosPro's solar-position engines against an
//! independent reference: [`dataset::DATASET`], 91 test cases computed by
//! pvlib-python 0.15.2 (BSD-3-Clause), an independent, widely used,
//! peer-reviewed open-source implementation of the NREL Solar Position
//! Algorithm (Reda & Andreas, 2004).
//!
//! [`report::evaluate`] runs a [`solar_position::SolarPositionAlgorithm`]
//! against every case and produces per-quantity error statistics (mean
//! bias, mean absolute error, RMSE, maximum absolute error) plus timing,
//! per the project's validation campaign requirements — see
//! [`report::EngineErrorReport`].
//!
//! ## What this crate validates, and what it does not
//!
//! This crate validates HeliosPro's solar-position engines
//! ([`solar_position::Spa`], [`solar_position::MeeusReduced`]) against
//! pvlib. It does **not** yet validate the irradiance/transposition/
//! thermal layers against NREL SAM or PVsyst output (no access to those
//! tools was available when this crate was built — see
//! `docs/design/16-phase1-validation-report.md`). That remains explicit,
//! tracked follow-up work, not a silently narrowed scope: the crate name
//! and this module documentation describe exactly what is covered today.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod dataset;
pub mod report;

pub use report::{EngineErrorReport, QuantityErrorStats, evaluate};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use solar_position::{MeeusReduced, Spa};
    use std::fs;
    use std::path::PathBuf;

    /// Generates the full validation report (both engines) and writes it
    /// to `engine/validation-data/latest_validation_report.md`, per the
    /// project requirement to "produire automatiquement un rapport
    /// d'erreur". Re-running the test suite regenerates this file
    /// deterministically from the committed dataset and the current
    /// engine code, so it always reflects the current state of the
    /// implementation rather than a stale, manually written summary.
    #[test]
    fn generate_and_write_full_validation_report() {
        let spa_report = evaluate(&Spa);
        let meeus_report = evaluate(&MeeusReduced);

        let mut markdown = String::new();
        markdown.push_str("# HeliosPro — Rapport de validation automatique\n\n");
        markdown.push_str(&format!(
            "Genere par `validation::integration_tests::generate_and_write_full_validation_report` \
             a partir de {} cas de reference (voir `engine/validation-data/README.md`).\n\n",
            spa_report.case_count
        ));
        markdown.push_str(&spa_report.to_markdown());
        markdown.push('\n');
        markdown.push_str(&meeus_report.to_markdown());

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let output_path = manifest_dir
            .join("..")
            .join("..")
            .join("validation-data")
            .join("latest_validation_report.md");
        fs::write(&output_path, &markdown).unwrap_or_else(|e| {
            panic!("failed to write validation report to {output_path:?}: {e}")
        });
    }
}
