use std::error::Error;
use std::fs::read_to_string;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

pub use self::metadata::Metadata;

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

/// Loads the active firmware metadata, returning `None` when the manifest is missing or inactive.
///
/// # Errors
///
/// Returns an error if the manifest cannot be read (except for a missing file) or parsed.
pub fn get_metadata(path: &Path) -> Result<Option<Metadata>, Box<dyn Error>> {
    let json = match read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let manifest: Manifest = serde_json::from_str(&json)?;
    Ok(manifest.active())
}
