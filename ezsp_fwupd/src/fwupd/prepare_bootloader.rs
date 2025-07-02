use ezsp::{Bootloader, Callback, GetValueExt, uart::Uart};
use indicatif::ProgressBar;
use log::{debug, error};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

use crate::FlashProgress;

const MODE: u8 = 0x00;

/// Trait for preparing the bootloader for firmware updates.
pub trait PrepareBootloader {
    /// Prepare the bootloader for firmware updates.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if the operation fails.
    fn prepare_bootloader(
        self,
        progress_bar: Option<&ProgressBar>,
    ) -> impl Future<Output = std::io::Result<()>>;
}

impl<T> PrepareBootloader for T
where
    T: SerialPort + 'static,
{
    async fn prepare_bootloader(self, progress_bar: Option<&ProgressBar>) -> std::io::Result<()> {
        let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
        let mut uart = Uart::new(self, callbacks_tx, 8, 8);

        debug!("Getting bootloader version...");
        match uart.get_ember_version().await {
            Ok(response) => match response {
                Ok(ember_version) => {
                    progress_bar.println(format!("Current version: {ember_version}"));
                }
                Err(error) => {
                    error!("Failed to parse version info: {error}");
                }
            },
            Err(error) => {
                error!("Failed to get version info: {error}");
            }
        }

        debug!("Launching standalone bootloader...");
        uart.launch_standalone_bootloader(MODE)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });
        Ok(())
    }
}
