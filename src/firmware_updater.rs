/// A trait for firmware updating functionality.
pub trait FirmwareUpdater {
    /// Represents the version type used by the firmware updater.
    type Version: Eq + PartialOrd;

    /// Returns the current firmware version.
    fn current_version(&self) -> Self::Version;

    /// Returns the latest available firmware version.
    fn latest_version(&self) -> Option<Self::Version>;

    /// Returns a list of all available firmware versions.
    fn available_versions(&self) -> Vec<Self::Version>;

    /// Checks if an update is available.
    fn is_update_available(&self) -> bool {
        let Some(latest_version) = self.latest_version() else {
            return false;
        };

        latest_version > self.current_version()
    }

    /// Installs the specified firmware version.
    fn install_version(&self, version: Self::Version) -> std::io::Result<()>;

    /// Updates to the latest firmware version if available.
    fn update_to_latest(&self) -> std::io::Result<()> {
        if let Some(latest_version) = self.latest_version() {
            self.install_version(latest_version)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No latest version available",
            ))
        }
    }
}
