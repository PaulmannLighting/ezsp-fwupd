use std::time::Duration;

use indicatif::ProgressBar;
use log::{debug, info};
use serialport::SerialPort;

pub use crate::xmodem::FrameCount;
pub use reset::Reset;
pub use tty::Tty;

use crate::ClearBuffer;
use crate::FlashProgress;
use prepare_bootloader::PrepareBootloader;
use transmit::Transmit;

mod prepare_bootloader;
mod reset;
mod transmit;
mod tty;

/// Trait for firmware update operations using the `Tty` serial port.
pub trait Fwupd {
    /// Performs a firmware update operation.
    fn fwupd(
        &self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        no_prepare: bool,
        progress_bar: Option<&ProgressBar>,
    ) -> impl Future<Output = std::io::Result<()>>;
}

impl Fwupd for Tty {
    async fn fwupd(
        &self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        no_prepare: bool,
        progress_bar: Option<&ProgressBar>,
    ) -> std::io::Result<()> {
        if !no_prepare {
            info!("Preparing bootloader...");
            self.open()?.prepare_bootloader(progress_bar).await?;
        }

        let mut serial_port = self.open()?;
        let original_timeout = serial_port.timeout();

        if let Some(timeout) = timeout {
            serial_port.set_timeout(timeout)?;
        }

        serial_port.clear_buffer()?;

        debug!("Initializing stage 1...");
        serial_port.init_stage1()?;

        debug!("Initializing stage 2...");
        serial_port.init_stage2()?;

        if let Err(error) = serial_port.transmit(firmware, Some(original_timeout), progress_bar) {
            serial_port.reset(timeout)?;
            return Err(error);
        }

        progress_bar.set_message("Firmware update complete, resetting device...");
        serial_port.reset(timeout)?;
        Ok(())
    }
}
