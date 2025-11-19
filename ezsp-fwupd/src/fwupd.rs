use std::time::Duration;

use indicatif::ProgressBar;
use log::{debug, info};
use serialport::SerialPort;

use self::prepare_bootloader::PrepareBootloader;
pub use self::reset::Reset;
use self::transmit::Transmit;
pub use crate::xmodem::FrameCount;
use crate::{ClearBuffer, FlashProgress};

mod prepare_bootloader;
mod reset;
mod transmit;

/// Trait for firmware update operations using a serial port.
pub trait Fwupd: Sized {
    /// Performs a firmware update operation.
    fn fwupd(
        self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        no_prepare: bool,
        progress_bar: Option<&ProgressBar>,
    ) -> impl Future<Output = std::io::Result<Self>>;
}

impl<T> Fwupd for T
where
    T: SerialPort + 'static,
{
    async fn fwupd(
        mut self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        no_prepare: bool,
        progress_bar: Option<&ProgressBar>,
    ) -> std::io::Result<Self> {
        if !no_prepare {
            info!("Preparing bootloader...");
            self = self.prepare_bootloader().await?;
        }

        let original_timeout = self.timeout();

        if let Some(timeout) = timeout {
            self.set_timeout(timeout)?;
        }

        self.clear_buffer()?;

        debug!("Initializing stage 1...");
        self.init_stage1()?;

        debug!("Initializing stage 2...");
        self.init_stage2()?;

        if let Err(error) = self.transmit(firmware, Some(original_timeout), progress_bar) {
            self.reset(timeout)?;
            return Err(error);
        }

        progress_bar.set_message("Firmware update complete, resetting device...");
        self.reset(timeout)?;
        Ok(self)
    }
}
