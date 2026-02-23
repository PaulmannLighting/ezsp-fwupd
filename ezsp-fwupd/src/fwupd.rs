use std::time::Duration;

use ashv2::TryCloneNative;
use indicatif::ProgressBar;
use log::{debug, info};
use serialport::SerialPort;

pub use self::reset::Reset;
use self::transmit::Transmit;
use crate::launch_bootloader::LaunchBootloader;
pub use crate::xmodem::FrameCount;
use crate::{ClearBuffer, FlashProgress};

mod reset;
mod transmit;

/// Trait for firmware update operations using a serial port.
pub trait Fwupd: Sized {
    /// Performs a firmware update operation.
    fn fwupd<F>(
        self,
        firmware: F,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> impl Future<Output = std::io::Result<Self>>
    where
        F: IntoIterator<Item = u8>;
}

impl<T> Fwupd for T
where
    T: SerialPort + TryCloneNative + Send + Sync + 'static,
{
    async fn fwupd<F>(
        mut self,
        firmware: F,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> std::io::Result<Self>
    where
        F: IntoIterator<Item = u8>,
    {
        info!("Preparing bootloader...");
        self = self.launch_bootloader().await?;
        let original_timeout = self.timeout();

        if let Some(timeout) = timeout {
            self.set_timeout(timeout)?;
        }

        self.clear_buffer()?;

        debug!("Initializing stage 1...");
        self.init_stage1()?;

        debug!("Initializing stage 2...");
        self.init_stage2()?;

        debug!("Transmitting firmware...");
        self.transmit(firmware, Some(original_timeout), progress_bar)?;

        progress_bar.set_message("Firmware update complete, resetting device...");
        self.reset(timeout)?;
        Ok(self)
    }
}
