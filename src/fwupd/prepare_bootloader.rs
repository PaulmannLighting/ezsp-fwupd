use ezsp::{Bootloader, Callback, uart::Uart};
use indicatif::ProgressBar;
use log::{debug, error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

const MODE: u8 = 0x00;

/// Trait for preparing the bootloader for firmware updates.
pub trait PrepareBootloader {
    /// Prepare the bootloader for firmware updates.
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
        match uart
            .get_standalone_bootloader_version_plat_micro_phy()
            .await
        {
            Ok(info) => {
                if let Some(progress_bar) = progress_bar {
                    progress_bar.println(format!("{info:?}"));
                } else {
                    info!("Bootloader info: {info:#?}");
                }
            }
            Err(error) => {
                error!("Failed to get bootloader info: {error}");
                return Err(std::io::Error::other(error));
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
