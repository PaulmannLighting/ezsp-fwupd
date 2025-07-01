use ezsp::{Bootloader, Callback, uart::Uart};
use log::{error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

/// Trait for preparing the bootloader for firmware updates.
pub trait PrepareBootloader {
    /// Prepare the bootloader for firmware updates.
    fn prepare_bootloader(self) -> impl Future<Output = std::io::Result<()>>;
}

impl<T> PrepareBootloader for T
where
    T: SerialPort + 'static,
{
    async fn prepare_bootloader(self) -> std::io::Result<()> {
        let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
        let mut uart = Uart::new(self, callbacks_tx, 8, 8);

        info!("Getting bootloader version...");
        match uart
            .get_standalone_bootloader_version_plat_micro_phy()
            .await
        {
            Ok(info) => {
                info!("Bootloader info: {info:#?}");
            }
            Err(error) => {
                error!("Failed to get bootloader info: {error}");
                return Err(std::io::Error::other(error));
            }
        }

        info!("Launching standalone bootloader...");
        uart.launch_standalone_bootloader(0x00)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });
        Ok(())
    }
}
