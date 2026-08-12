use crate::provenance::CalendarDate;

/// A conflict between two candidate values, presented so a decision can
/// be made about it.
///
/// The engine produces this and stops. It does not choose. Both values,
/// their sources and the discrepancy are named so that whoever decides
/// has what they need.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// The invariant that detected the conflict.
    pub invariant_id: &'static str,
    /// A label for the first candidate, e.g. `"gross_area"`.
    pub first_label: String,
    /// The first candidate value, rendered for display.
    pub first_value: String,
    /// Where the first candidate came from, if known.
    pub first_source: Option<String>,
    /// A label for the second candidate.
    pub second_label: String,
    /// The second candidate value, rendered for display.
    pub second_value: String,
    /// Where the second candidate came from, if known.
    pub second_source: Option<String>,
    /// The relative discrepancy between the two, where meaningful.
    pub relative_difference: Option<f64>,
    /// What a resolution must justify.
    pub expected_justification: String,
}

/// Which way a conflict was resolved.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ResolutionDecision {
    /// Keep the first candidate.
    UseFirst,
    /// Keep the second candidate.
    UseSecond,
    /// Substitute a third value entirely.
    UseOverride {
        /// The substituted value, rendered for the record.
        value: String,
    },
    /// Proceed without choosing, accepting the inconsistency knowingly.
    Ignore,
}

/// An explicit, recorded decision that lifts one conflict.
///
/// **This is how "the engine never arbitrates" becomes operational
/// rather than merely restrictive.** A conflict blocks construction; it
/// is passable only by a decision that names who made it, why, and when.
/// The resolution is kept in the constructed module and becomes
/// provenance in its own right.
///
/// `decided_by` and `rationale` are mandatory: a resolution without them
/// would be an anonymous arbitration, which is what the policy forbids.
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictResolution {
    invariant_id: &'static str,
    decision: ResolutionDecision,
    decided_by: String,
    rationale: String,
    decided_on: Option<CalendarDate>,
    references: Vec<String>,
}

impl ConflictResolution {
    /// Record a decision. The decider and the rationale are required.
    ///
    /// # Panics
    ///
    /// Panics if `decided_by` or `rationale` is empty after trimming. An
    /// empty rationale is an anonymous arbitration, which this type
    /// exists to prevent; accepting one silently would defeat its whole
    /// purpose. Callers building resolutions from untrusted input should
    /// use [`ConflictResolution::try_new`].
    pub fn new(
        invariant_id: &'static str,
        decision: ResolutionDecision,
        decided_by: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self::try_new(invariant_id, decision, decided_by, rationale)
            .expect("a conflict resolution requires a non-empty decider and rationale")
    }

    /// Record a decision, returning `None` if the decider or rationale is
    /// empty.
    pub fn try_new(
        invariant_id: &'static str,
        decision: ResolutionDecision,
        decided_by: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Option<Self> {
        let decided_by = decided_by.into();
        let rationale = rationale.into();
        if decided_by.trim().is_empty() || rationale.trim().is_empty() {
            return None;
        }
        Some(ConflictResolution {
            invariant_id,
            decision,
            decided_by,
            rationale,
            decided_on: None,
            references: Vec::new(),
        })
    }

    /// Record when the decision was made.
    #[inline]
    #[must_use]
    pub fn with_date(mut self, date: CalendarDate) -> Self {
        self.decided_on = Some(date);
        self
    }

    /// Record a supporting reference.
    #[inline]
    #[must_use]
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.references.push(reference.into());
        self
    }

    /// The invariant this resolution lifts.
    #[inline]
    pub const fn invariant_id(&self) -> &'static str {
        self.invariant_id
    }

    /// Which way it was resolved.
    #[inline]
    pub const fn decision(&self) -> &ResolutionDecision {
        &self.decision
    }

    /// Who decided. Never empty.
    #[inline]
    pub fn decided_by(&self) -> &str {
        &self.decided_by
    }

    /// Why. Never empty.
    #[inline]
    pub fn rationale(&self) -> &str {
        &self.rationale
    }

    /// When, if recorded.
    #[inline]
    pub const fn decided_on(&self) -> Option<CalendarDate> {
        self.decided_on
    }

    /// Supporting references.
    #[inline]
    pub fn references(&self) -> &[String] {
        &self.references
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resolution_requires_a_decider_and_a_rationale() {
        assert!(
            ConflictResolution::try_new("INV-GEO-002", ResolutionDecision::UseFirst, "", "because")
                .is_none()
        );
        assert!(
            ConflictResolution::try_new(
                "INV-GEO-002",
                ResolutionDecision::UseFirst,
                "operator",
                "   "
            )
            .is_none()
        );
        assert!(
            ConflictResolution::try_new(
                "INV-GEO-002",
                ResolutionDecision::UseFirst,
                "operator",
                "stated area matches the cell count"
            )
            .is_some()
        );
    }

    #[test]
    #[should_panic(expected = "non-empty decider and rationale")]
    fn the_panicking_constructor_rejects_an_empty_rationale() {
        let _ = ConflictResolution::new("INV-GEO-002", ResolutionDecision::UseFirst, "op", "");
    }

    #[test]
    fn a_resolution_records_the_full_decision_trail() {
        let r = ConflictResolution::new(
            "INV-GEO-002",
            ResolutionDecision::UseFirst,
            "metrology team",
            "stated area is consistent with 21 cells of 166 mm",
        )
        .with_date(CalendarDate::new(2026, 8, 6).unwrap())
        .with_reference("doc 26 section 1.2");

        assert_eq!(r.invariant_id(), "INV-GEO-002");
        assert_eq!(*r.decision(), ResolutionDecision::UseFirst);
        assert_eq!(r.decided_by(), "metrology team");
        assert!(r.rationale().contains("166 mm"));
        assert_eq!(r.decided_on().unwrap().year, 2026);
        assert_eq!(r.references().len(), 1);
    }

    #[test]
    fn an_override_carries_the_substituted_value() {
        let r = ConflictResolution::new(
            "INV-GEO-002",
            ResolutionDecision::UseOverride {
                value: "0.5772 m2".to_string(),
            },
            "operator",
            "remeasured",
        );
        match r.decision() {
            ResolutionDecision::UseOverride { value } => assert_eq!(value, "0.5772 m2"),
            other => panic!("expected an override, got {other:?}"),
        }
    }

    #[test]
    fn a_conflict_names_both_candidates_and_their_gap() {
        let c = Conflict {
            invariant_id: "INV-GEO-002",
            first_label: "gross_area".to_string(),
            first_value: "0.577 m2".to_string(),
            first_source: Some("CEC".to_string()),
            second_label: "rectangular_area".to_string(),
            second_value: "1.153 m2".to_string(),
            second_source: Some("derived from dimensions".to_string()),
            relative_difference: Some(0.5),
            expected_justification: "state which dimension source is trusted".to_string(),
        };
        assert_eq!(c.invariant_id, "INV-GEO-002");
        assert!((c.relative_difference.unwrap() - 0.5).abs() < 1e-12);
        assert!(c.first_source.is_some());
        assert!(c.second_source.is_some());
    }
}
