use std::path::Path;

use log::error;
use semver::Version;

/// A trait for extracting version information from filenames.
pub trait VersionFromFilename {
    /// Extract the version from the filename.
    ///
    /// # Returns
    ///
    /// If the filename contains a valid version, returns `Some(version)`.
    /// Otherwise, returns `None`.
    fn version_from_filename(&self) -> Option<Version>;
}

impl<T> VersionFromFilename for T
where
    T: AsRef<Path>,
{
    fn version_from_filename(&self) -> Option<Version> {
        Version::parse(self.as_ref().file_stem()?.to_str()?)
            .inspect_err(|error| error!("{error}"))
            .ok()
    }
}
