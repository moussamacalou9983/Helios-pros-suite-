/// The outcome of evaluating one invariant.
///
/// Ordered by severity so that a report can report its worst finding:
/// `Valid < Unknown < Warning < Conflict < Invalid`.
///
/// The distinction that matters most is [`ValidationStatus::Conflict`]
/// versus [`ValidationStatus::Invalid`]:
///
/// - **Invalid** — nothing can be arbitrated. `v_oc < v_mpp` cannot be
///   repaired by choosing between values, because no value is right.
/// - **Conflict** — both candidate values are individually plausible,
///   one of them is wrong, and nothing in the model says which. This is
///   the case direct inspection of the CEC library found on 29 records
///   (`docs/design/26-validation-empirique-cec-sandia.md` §1).
///
/// A conflict is therefore blocking but *resolvable* by an explicit,
/// recorded human decision; an invalid record is blocking outright.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidationStatus {
    /// The invariant holds.
    Valid,
    /// The invariant could not be evaluated because a field it needs is
    /// absent. Not a defect: an absence of information.
    Unknown,
    /// The invariant holds but the value is atypical — physically
    /// possible, statistically suspect.
    Warning,
    /// Two sources or two fields contradict each other and the engine
    /// cannot choose between them.
    Conflict,
    /// Physically impossible or structurally incoherent.
    Invalid,
}

impl ValidationStatus {
    /// Whether this status blocks construction of a module.
    ///
    /// Conflicts block, but may be lifted by a matching
    /// [`crate::ConflictResolution`]. Invalid records block outright.
    #[inline]
    pub const fn is_blocking(self) -> bool {
        matches!(self, ValidationStatus::Conflict | ValidationStatus::Invalid)
    }

    /// Whether an explicit resolution can lift this status.
    #[inline]
    pub const fn is_resolvable(self) -> bool {
        matches!(self, ValidationStatus::Conflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_from_valid_to_invalid() {
        assert!(ValidationStatus::Valid < ValidationStatus::Unknown);
        assert!(ValidationStatus::Unknown < ValidationStatus::Warning);
        assert!(ValidationStatus::Warning < ValidationStatus::Conflict);
        assert!(ValidationStatus::Conflict < ValidationStatus::Invalid);
    }

    #[test]
    fn only_conflicts_and_invalids_block() {
        assert!(ValidationStatus::Conflict.is_blocking());
        assert!(ValidationStatus::Invalid.is_blocking());
        assert!(!ValidationStatus::Warning.is_blocking());
        assert!(!ValidationStatus::Unknown.is_blocking());
        assert!(!ValidationStatus::Valid.is_blocking());
    }

    #[test]
    fn only_conflicts_are_resolvable() {
        assert!(ValidationStatus::Conflict.is_resolvable());
        assert!(!ValidationStatus::Invalid.is_resolvable());
        assert!(!ValidationStatus::Warning.is_resolvable());
    }
}
