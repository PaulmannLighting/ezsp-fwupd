use std::path::PathBuf;

use semver::Version;

use crate::FirmwareUpdater;

/// Represents the Silicon Labs MGM210P22A device.
pub struct MGM210P22A {
    tty: PathBuf,
}

impl FirmwareUpdater for MGM210P22A {
    const BASE_DIR: &'static str = "MGM210P22A";

    type Version = Version;

    fn current_version(&self) -> std::io::Result<Self::Version> {
        todo!()
    }

    fn latest_version(&self) -> Option<Self::Version> {
        todo!()
    }

    fn available_versions(&self) -> Vec<Self::Version> {
        todo!()
    }

    fn install_version(&self, version: Self::Version) -> std::io::Result<()> {
        todo!()
    }
}
