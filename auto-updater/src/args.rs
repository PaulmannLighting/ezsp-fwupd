use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;

const DEFAULT_MANIFEST: &str = "/etc/ezsp-firmware-update.json";
const DEFAULT_TIMEOUT: u64 = 1000; // Milliseconds
const DEFAULT_REBOOT_GRACE_TIME: u64 = 4000; // Milliseconds
const DEFAULT_CHANNEL_SIZE: usize = 8;
const PROTOCOL_VERSION: u8 = 8;
const MAX_RETRIES: u8 = 5;

/// Command line arguments for the firmware update tool.
#[derive(Debug, Parser)]
pub struct Args {
    #[clap(index = 1, help = "the serial port to use for firmware update")]
    tty: String,
    #[clap(long, short, help = "the firmware manifest file", default_value = DEFAULT_MANIFEST)]
    manifest: PathBuf,
    #[clap(long, short = 't', help = "serial port timeout in milliseconds", default_value_t = DEFAULT_TIMEOUT)]
    timeout: u64,
    #[clap(long, short = 'r', help = "grace time to wait for the device to reboot", default_value_t = DEFAULT_REBOOT_GRACE_TIME)]
    reboot_grace_time: u64,
    #[clap(long, short = 'C', help = "callback channel size", default_value_t = DEFAULT_CHANNEL_SIZE)]
    callback_channel_size: usize,
    #[clap(long, short = 'R', help = "response channel size", default_value_t = DEFAULT_CHANNEL_SIZE)]
    response_channel_size: usize,
    #[clap(long, short = 'p', help = "EZSP protocol version to use", default_value_t = PROTOCOL_VERSION)]
    protocol_version: u8,
    #[clap(long, short = 'm', help = "maximum amount of retries on repeatable fallible operations", default_value_t = MAX_RETRIES)]
    max_retries: u8,
}

impl Args {
    /// Return the serial port to use for firmware update.
    #[must_use]
    pub fn tty(&self) -> &str {
        &self.tty
    }

    /// Return the firmware manifest file.
    #[must_use]
    pub fn manifest(&self) -> &Path {
        &self.manifest
    }

    /// Return the serial port timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout)
    }

    /// Return the grace time to wait for the device to reboot.
    #[must_use]
    pub const fn reboot_grace_time(&self) -> Duration {
        Duration::from_millis(self.reboot_grace_time)
    }

    /// Return the callback channel size.
    #[must_use]
    pub const fn callback_channel_size(&self) -> usize {
        self.callback_channel_size
    }

    /// Return the response channel size.
    #[must_use]
    pub const fn response_channel_size(&self) -> usize {
        self.response_channel_size
    }

    /// Return the EZSP protocol version to use.
    #[must_use]
    pub const fn protocol_version(&self) -> u8 {
        self.protocol_version
    }

    /// Return the maximum amount of retries on repeatable fallible operations.
    #[must_use]
    pub const fn max_retries(&self) -> u8 {
        self.max_retries
    }
}
