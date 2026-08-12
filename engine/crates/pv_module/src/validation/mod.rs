//! Physical validation: the invariants, the findings they produce, and
//! the explicit decisions that lift a conflict.
//!
//! ## The five statuses
//!
//! [`ValidationStatus`] separates five outcomes that a boolean "valid /
//! invalid" would collapse:
//!
//! | Status | Meaning | Blocks? |
//! |---|---|---|
//! | `Valid` | the invariant holds | no |
//! | `Unknown` | a field it needs is absent | no |
//! | `Warning` | possible but atypical | no |
//! | `Conflict` | two plausible values contradict each other | yes, resolvable |
//! | `Invalid` | physically impossible | yes, outright |
//!
//! The distinction between `Conflict` and `Invalid` is the crate's central
//! epistemic commitment. `v_oc < v_mpp` is `Invalid`: no choice between
//! values repairs it. A stated area contradicting the module's own
//! dimensions is a `Conflict`: both figures are plausible, one is wrong,
//! and nothing in the data says which. Direct inspection of the CEC
//! library found 29 such records
//! (`docs/design/26-validation-empirique-cec-sandia.md` §1).
//!
//! ## The rule
//!
//! > **A conflicting value is never arbitrated silently.**
//!
//! A conflict stops construction and presents both candidates through
//! [`Conflict`]. It is passable only by a [`ConflictResolution`] naming
//! who decided, why, and — optionally — when and on what evidence. That
//! resolution is kept in the built module and becomes provenance in its
//! own right.
//!
//! ## The inventory
//!
//! [`invariants::ALL_INVARIANT_IDS`] is the authoritative list of the 52
//! invariants this crate can emit. Each check is a free function taking
//! only the parts of a module it needs, so every invariant is exercisable
//! in isolation as well as through [`invariants::validate_module`].

pub mod invariants;

mod conflict;
mod issue;
mod status;

pub use conflict::{Conflict, ConflictResolution, ResolutionDecision};
pub use issue::{ValidationIssue, ValidationReport};
pub use status::ValidationStatus;
