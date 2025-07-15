use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};
use log::{debug, error};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

const MODE: u8 = 0x00;

/// Trait for preparing the bootloader for firmware updates.
pub trait PrepareBootloader: Sized {
    /// Prepare the bootloader for firmware updates.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if the operation fails.
    fn prepare_bootloader(self) -> impl Future<Output = std::io::Result<Self>>;
}

impl<T> PrepareBootloader for T
where
    T: SerialPort + 'static,
{
    async fn prepare_bootloader(self) -> std::io::Result<Self> {
        let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
        let mut uart = Uart::new(self, callbacks_tx, 8, 8);

        debug!("Launching standalone bootloader...");
        uart.launch_standalone_bootloader(MODE)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });
        Ok(uart.terminate())
    }
}
