use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;

/// Represents the manifest for versioned firmware files.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Manifest {
    version: Version,
    filename: PathBuf,
}

impl Manifest {
    /// Returns the version of the firmware.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the filename of the firmware file.
    pub fn filename(&self) -> &Path {
        &self.filename
    }
}
