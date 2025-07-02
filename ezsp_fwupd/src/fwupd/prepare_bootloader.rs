use ezsp::{Bootloader, Callback, Ezsp, uart::Uart};
use indicatif::ProgressBar;
use log::{debug, error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

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
        match uart.init().await {
            Ok(response) => {
                if let Some(progress_bar) = progress_bar {
                    progress_bar.println(format!(
                        "EZSP version:  {:#04X}",
                        response.protocol_version()
                    ));
                    progress_bar.println(format!("Stack type:    {:#04X}", response.stack_type()));
                    progress_bar.println(format!("Stack version: {}", response.stack_version()));
                } else {
                    info!("EZSP version:  {:#04X}", response.protocol_version());
                    info!("Stack type:    {:#04X}", response.stack_type());
                    info!("Stack version: {}", response.stack_version());
                }
            }
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
