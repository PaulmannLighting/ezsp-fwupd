use serde::Deserialize;

use metadata::Metadata;

mod metadata;

/// Represents a manifest containing information about the active firmware update.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Manifest {
    active: Metadata,
}

impl Manifest {
    /// Returns the active metadata of the manifest.
    #[must_use]
    pub const fn active(&self) -> &Metadata {
        &self.active
    }
}
