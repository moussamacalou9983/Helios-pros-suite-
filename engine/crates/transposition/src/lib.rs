//! # transposition
//!
//! Sky-diffuse transposition models: computing the irradiance received on
//! an arbitrarily tilted plane (plane-of-array, POA) from the horizontal
//! irradiance components (GHI, DNI, DHI) and the Sun's position.
//!
//! Four models are implemented, in increasing order of physical realism
//! (and, generally, accuracy):
//!
//! - [`isotropic`]: Liu, B.Y. & Jordan, R.C. (1963), "The interrelationship
//!   and characteristic distribution of direct, diffuse, and total solar
//!   radiation", *Solar Energy*, 4(3), 1-19.
//! - [`hay_davies`]: Hay, J.E. & Davies, J.A. (1980), "Calculation of the
//!   solar radiation incident on an inclined surface", *Proc. First
//!   Canadian Solar Radiation Data Workshop*, Toronto.
//! - [`hdkr`]: Reindl, D.T., Beckman, W.A., Duffie, J.A. (1990),
//!   "Evaluation of hourly tilted surface radiation models", *Solar
//!   Energy*, 45(1), 9-17 (combining the Hay-Davies anisotropic-sky model
//!   with Klucher's (1979) horizon-brightening term — hence "HDKR" =
//!   Hay-Davies-Klucher-Reindl).
//! - [`perez`]: Perez, R., Ineichen, P., Seals, R., Michalsky, J., Stewart,
//!   R. (1990), "Modeling daylight availability and irradiance components
//!   from direct and global irradiance", *Solar Energy*, 44(5), 271-289.
//!
//! All four models share the input struct [`TranspositionInputs`] and
//! return the same output struct [`PlaneOfArrayIrradiance`], so they are
//! directly interchangeable (see the `simulation` crate for the
//! orchestration layer that selects a model).
//!
//! ## Validated invariant
//!
//! For a horizontal surface (`tilt = 0`), every one of these models reduces
//! algebraically to `POA_global = GHI` (given a self-consistent DNI/DHI/GHI
//! triple satisfying the exact closure relation from
//! `irradiance::decomposition`). This is proven by direct substitution
//! for each model and enforced by the `tilt_zero_recovers_ghi_*` test in
//! every model's test module — the primary correctness check for this
//! crate, chosen because it does not depend on any external reference
//! dataset that could not be independently verified in this development
//! session (see the crate-level precision ADR in `solar_geometry::lib`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod inputs;

pub mod hay_davies;
pub mod hdkr;
pub mod isotropic;
pub mod perez;

pub use inputs::{PlaneOfArrayIrradiance, TranspositionInputs};
