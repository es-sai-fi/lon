pub mod niv;
pub mod npins;

use anyhow::Result;

use crate::sources::Sources;

/// A trait for lock files that can be converted to a Lon `Source`.
///
/// Thus, they can eventually be converted to a Lon lock file.
pub trait Convertible {
    fn convert(&self) -> Result<Sources>;
}
