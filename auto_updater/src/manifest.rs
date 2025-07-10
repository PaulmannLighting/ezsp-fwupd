use serde::Deserialize;
use std::fs::read_to_string;
use std::io::ErrorKind;
use std::path::Path;

use metadata::Metadata;

mod metadata;

/// Represents a manifest containing information about the active firmware update.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
struct Manifest {
    active: Option<Metadata>,
}

impl Manifest {
    /// Returns the active metadata of the manifest.
    #[must_use]
    pub fn active(self) -> Option<Metadata> {
        self.active
    }
}

pub fn get_metadata(path: &Path) -> Result<Option<Metadata>, String> {
    match serde_json::from_str::<Manifest>(&match read_to_string(path) {
        Ok(json) => json,
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                return Ok(None);
            }

            return Err(format!(
                "Failed to read manifest file '{}': {error}",
                path.display()
            ));
        }
    }) {
        Ok(manifest) => Ok(manifest.active()),
        Err(error) => Err(format!("Failed to parse manifest file: {error}")),
    }
}
