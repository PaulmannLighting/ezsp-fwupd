use std::io::ErrorKind;
use std::path::PathBuf;

use regex::{Captures, Regex};
use semver::Version;

use crate::FirmwareUpdater;

use z3gateway_host::Z3GatewayHost;

mod z3gateway_host;

const BAUD_RATE: u32 = 115200;
const VERSION_REGEX: &str = r"\[(\d+).(\d+).(\d+) (?:.+) build (\d+)\]";

/// Represents the Silicon Labs MGM210P22A device.
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
    pub fn tty(&self) -> &PathBuf {
        &self.tty
    }

    pub fn read_version(&self) -> std::io::Result<Version> {
        let output = Z3GatewayHost::default()
            .arg("-n")
            .arg(1.to_string())
            .arg("-b")
            .arg(BAUD_RATE.to_string())
            .arg("-f")
            .arg("x")
            .arg("-p")
            .arg(self.tty())
            .run()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let regex = Regex::new(VERSION_REGEX).expect("Failed to compile version regex");
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

    fn install_version(&self, version: Self::Version) -> std::io::Result<()> {
        todo!()
    }
}

fn capture_version(captures: Captures) -> Option<Version> {
    Version::parse(&format!(
        "{}.{}.{}-{}",
        captures.get(1)?.as_str(),
        captures.get(2)?.as_str(),
        captures.get(3)?.as_str(),
        captures.get(4)?.as_str()
    ))
    .ok()
}

#[cfg(test)]
mod tests {
    use semver::Version;

    const VERSION: &str = "6.10.3-297";

    #[test]
    fn test_version() {
        Version::parse(VERSION).expect("Failed to parse version");
    }
}
