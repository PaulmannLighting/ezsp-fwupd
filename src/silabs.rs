use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str::FromStr;

use regex::{Captures, Regex};
use semver::{BuildMetadata, Version};

use ctrl_c_and_wait_with_output::CtrlCAndWaitWithOutput;
use z3gateway_host::Z3GatewayHost;

use crate::FirmwareUpdater;

pub mod ctrl_c_and_wait_with_output;
mod manifest;
mod z3gateway_host;

const BAUD_RATE: u32 = 115200;
const VERSION_REGEX: &str = r"\[(\d+\.\d+\.\d+) (?:.+) build (\d+)\]";

/// Represents the Silicon Labs MGM210P22A device.
#[derive(Debug)]
pub struct MGM210P22A {
    tty: PathBuf,
}

impl MGM210P22A {
    /// Creates a new instance of `MGM210P22A`.
    ///
    /// # Arguments
    ///
    /// * `tty` - The path to the TTY device.
    pub fn new(tty: PathBuf) -> Self {
        Self { tty }
    }

    /// Returns the TTY path for the device.
    pub fn tty(&self) -> &Path {
        &self.tty
    }

    /// Read out the status of the device connected to the specified TTY.
    fn status(&self) -> std::io::Result<Output> {
        Command::z3gateway_host()
            .arg("-n")
            .arg(1.to_string())
            .arg("-b")
            .arg(BAUD_RATE.to_string())
            .arg("-f")
            .arg("x")
            .arg("-p")
            .arg(self.tty())
            .ctrl_c_and_wait_with_output()
    }

    fn read_version(&self) -> std::io::Result<Version> {
        let output = self.status()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let regex = Regex::new(VERSION_REGEX)
            .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?;
        stdout
            .lines()
            .find_map(|line| regex.captures(line).and_then(capture_version))
            .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "Version not found"))
    }
}

impl FirmwareUpdater for MGM210P22A {
    const BASE_DIR: &'static str = "MGM210P22A";

    type Version = Version;

    fn current_version(&self) -> std::io::Result<Self::Version> {
        self.read_version()
    }

    fn latest_version(&self) -> Option<Self::Version> {
        todo!()
    }

    fn available_versions(&self) -> Vec<Self::Version> {
        todo!()
    }

    fn install(&self, _version: &Self::Version) -> std::io::Result<()> {
        todo!()
    }
}

fn capture_version(captures: Captures) -> Option<Version> {
    let mut version = Version::parse(captures.get(1)?.as_str()).ok()?;
    version.build = BuildMetadata::from_str(captures.get(2)?.as_str()).ok()?;
    Some(version)
}

#[cfg(test)]
mod tests {
    use crate::silabs::{VERSION_REGEX, capture_version};
    use regex::Regex;
    use semver::{BuildMetadata, Version};

    const VERSION_LINE: &str = "ezsp ver 0x08 stack type 0x02 stack ver. [6.10.3 GA build 297]";

    #[test]
    fn test_version() {
        let mut version = Version::new(6, 10, 3);
        Version::new(6, 10, 3);
        version.build = BuildMetadata::new(297.to_string().as_str()).unwrap();
        assert_eq!(
            capture_version(
                Regex::new(VERSION_REGEX)
                    .unwrap()
                    .captures(VERSION_LINE)
                    .unwrap()
            ),
            Some(version)
        );
    }
}
