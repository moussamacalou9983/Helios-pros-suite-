use super::ValidationStatus;

/// One invariant's finding, with everything needed to act on it.
///
/// `explanation` is mandatory by construction. A status without an
/// explanation would leave the user facing an opaque refusal, which
/// amounts to arbitrating without saying so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    invariant_id: &'static str,
    status: ValidationStatus,
    fields: Vec<&'static str>,
    observed: Option<String>,
    expected: Option<String>,
    explanation: String,
}

impl ValidationIssue {
    /// Record a finding. `explanation` is required.
    #[inline]
    pub fn new(
        invariant_id: &'static str,
        status: ValidationStatus,
        fields: Vec<&'static str>,
        explanation: impl Into<String>,
    ) -> Self {
        ValidationIssue {
            invariant_id,
            status,
            fields,
            observed: None,
            expected: None,
            explanation: explanation.into(),
        }
    }

    /// Record what was observed.
    #[inline]
    #[must_use]
    pub fn with_observed(mut self, observed: impl Into<String>) -> Self {
        self.observed = Some(observed.into());
        self
    }

    /// Record what was expected, or the tolerance that was exceeded.
    #[inline]
    #[must_use]
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// The invariant's stable identifier, e.g. `"INV-GEO-002"`.
    #[inline]
    pub const fn invariant_id(&self) -> &'static str {
        self.invariant_id
    }

    /// The finding's severity.
    #[inline]
    pub const fn status(&self) -> ValidationStatus {
        self.status
    }

    /// The model fields the finding concerns.
    #[inline]
    pub fn fields(&self) -> &[&'static str] {
        &self.fields
    }

    /// What was observed, if recorded.
    #[inline]
    pub fn observed(&self) -> Option<&str> {
        self.observed.as_deref()
    }

    /// What was expected, if recorded.
    #[inline]
    pub fn expected(&self) -> Option<&str> {
        self.expected.as_deref()
    }

    /// The plain-language explanation. Never empty.
    #[inline]
    pub fn explanation(&self) -> &str {
        &self.explanation
    }
}

/// Every finding produced while validating one module.
///
/// The report is **attached to the constructed module**, not discarded: a
/// module carrying warnings must remain inspectable by downstream
/// engines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// An empty report.
    #[inline]
    pub const fn new() -> Self {
        ValidationReport { issues: Vec::new() }
    }

    /// Add a finding.
    #[inline]
    pub fn push(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Every finding, in the order the invariants were evaluated.
    #[inline]
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// The most severe status found, or [`ValidationStatus::Valid`] if
    /// the report is empty.
    #[inline]
    pub fn worst_status(&self) -> ValidationStatus {
        self.issues
            .iter()
            .map(ValidationIssue::status)
            .max()
            .unwrap_or(ValidationStatus::Valid)
    }

    /// Findings at exactly the given status.
    pub fn issues_at(&self, status: ValidationStatus) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(move |i| i.status() == status)
    }

    /// Whether a finding with the given invariant identifier is present.
    pub fn contains(&self, invariant_id: &str) -> bool {
        self.issues.iter().any(|i| i.invariant_id() == invariant_id)
    }

    /// The finding for a given invariant, if any.
    pub fn find(&self, invariant_id: &str) -> Option<&ValidationIssue> {
        self.issues
            .iter()
            .find(|i| i.invariant_id() == invariant_id)
    }

    /// Every conflict, which are the only findings an explicit
    /// resolution can lift.
    pub fn conflicts(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues_at(ValidationStatus::Conflict)
    }

    /// Whether any finding is invalid — the unconditional refusal.
    #[inline]
    pub fn has_invalid(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.status() == ValidationStatus::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(id: &'static str, status: ValidationStatus) -> ValidationIssue {
        ValidationIssue::new(id, status, vec!["field"], "explanation")
    }

    #[test]
    fn an_empty_report_is_valid() {
        let r = ValidationReport::new();
        assert_eq!(r.worst_status(), ValidationStatus::Valid);
        assert!(!r.has_invalid());
    }

    #[test]
    fn worst_status_is_the_most_severe_finding() {
        let mut r = ValidationReport::new();
        r.push(issue("A", ValidationStatus::Warning));
        r.push(issue("B", ValidationStatus::Unknown));
        assert_eq!(r.worst_status(), ValidationStatus::Warning);

        r.push(issue("C", ValidationStatus::Conflict));
        assert_eq!(r.worst_status(), ValidationStatus::Conflict);

        r.push(issue("D", ValidationStatus::Invalid));
        assert_eq!(r.worst_status(), ValidationStatus::Invalid);
        assert!(r.has_invalid());
    }

    #[test]
    fn findings_are_retrievable_by_identifier_and_status() {
        let mut r = ValidationReport::new();
        r.push(issue("INV-GEO-002", ValidationStatus::Conflict));
        r.push(issue("INV-ELE-003", ValidationStatus::Warning));

        assert!(r.contains("INV-GEO-002"));
        assert!(!r.contains("INV-POS-001"));
        assert_eq!(
            r.find("INV-ELE-003").unwrap().status(),
            ValidationStatus::Warning
        );
        assert_eq!(r.conflicts().count(), 1);
        assert_eq!(r.issues_at(ValidationStatus::Warning).count(), 1);
    }

    #[test]
    fn an_issue_records_observed_and_expected_values() {
        let i = ValidationIssue::new(
            "INV-GEO-002",
            ValidationStatus::Conflict,
            vec!["gross_area", "rectangular_area"],
            "stated area contradicts the bounding rectangle",
        )
        .with_observed("gross_area = 0.577 m2, rectangular_area = 1.153 m2")
        .with_expected("relative difference below 1 %");

        assert_eq!(i.invariant_id(), "INV-GEO-002");
        assert_eq!(i.fields().len(), 2);
        assert!(i.observed().unwrap().contains("0.577"));
        assert!(i.expected().unwrap().contains("1 %"));
        assert!(!i.explanation().is_empty());
    }
}
