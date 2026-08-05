use std::time::Duration;

use indicatif::ProgressBar;
use log::{debug, error, info};
use serialport::{FlowControl, SerialPort};

pub use self::gecko_bootloader::GeckoBootloader;
use self::transmit::Transmit;
use crate::launch_bootloader::LaunchBootloader;
pub use crate::xmodem::FrameCount;
use crate::{ClearBuffer, FlashProgress};

mod gecko_bootloader;
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
    T: SerialPort + Send + 'static,
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
        let original_flow_control = self.flow_control()?;
        self = self.launch_bootloader().await?;
        self.set_flow_control(FlowControl::None)?;
        let original_timeout = self.timeout();

        let update_result = (|| {
            if let Some(timeout) = timeout {
                self.set_timeout(timeout)?;
            }

            self.clear_buffer()?;

            debug!("Waking the bootloader menu...");
            self.wake_bootloader_menu()?;

            debug!("Starting the XMODEM upload...");
            self.start_xmodem_upload()?;

            debug!("Transmitting firmware...");
            self.transmit(firmware, Some(original_timeout), progress_bar)?;

            progress_bar.set_message("Firmware update complete, resetting device...");
            self.run_application(timeout)
        })();
        let restore_result = self
            .set_flow_control(original_flow_control)
            .map_err(std::io::Error::from);

        match (update_result, restore_result) {
            (Ok(()), Ok(())) => Ok(self),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(update_error), Err(restore_error)) => {
                error!("Failed to restore serial flow control: {restore_error}");
                Err(update_error)
            }
        }
    }
}
