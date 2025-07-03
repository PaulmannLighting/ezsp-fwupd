use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;

/// A structure representing metadata for a firmware update, including its version and filename.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct Metadata {
    version: Version,
    filename: PathBuf,
}

impl Metadata {
    /// Returns the version of the metadata.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the filename associated with the metadata.
    #[must_use]
    pub fn filename(&self) -> &Path {
        &self.filename
    }
}
