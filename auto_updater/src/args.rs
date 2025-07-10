use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;

const DEFAULT_MANIFEST: &str = "/etc/ezsp-firmware-update.json";
const DEFAULT_TIMEOUT: u64 = 1000; // Milliseconds
const DEFAULT_REBOOT_GRACE_TIME: u64 = 4000; // Milliseconds

/// Command line arguments for the firmware update tool.
#[derive(Debug, Parser)]
pub struct Args {
    #[clap(index = 1, help = "the serial port to use for firmware update")]
    tty: String,
    #[clap(long, short, help = "the firmware manifest file", default_value = DEFAULT_MANIFEST)]
    manifest: PathBuf,
    #[clap(long, short, help = "serial port timeout in milliseconds", default_value_t = DEFAULT_TIMEOUT)]
    timeout: u64,
    #[clap(long, short, help = "grace time to wait for the device to reboot", default_value_t = DEFAULT_REBOOT_GRACE_TIME)]
    reboot_grace_time: u64,
}

impl Args {
    /// Return the serial port to use for firmware update.
    pub fn tty(&self) -> &str {
        &self.tty
    }

    /// Return the firmware manifest file.
    pub fn manifest(&self) -> &Path {
        &self.manifest
    }

    /// Return the serial port timeout.
    pub const fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout)
    }

    /// Return the grace time to wait for the device to reboot.
    pub const fn reboot_grace_time(&self) -> Duration {
        Duration::from_millis(self.reboot_grace_time)
    }
}
