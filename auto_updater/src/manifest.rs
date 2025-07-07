use serde::Deserialize;

use metadata::Metadata;

mod metadata;

/// Represents a manifest containing information about the active firmware update.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Manifest {
    active: Option<Metadata>,
}

impl Manifest {
    /// Returns the active metadata of the manifest.
    #[must_use]
    pub fn active(self) -> Option<Metadata> {
        self.active
    }
}
