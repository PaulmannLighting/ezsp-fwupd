use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};
use log::{debug, error};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

use crate::discard_callbacks;

const MODE: u8 = 0x00;

/// Trait for preparing the bootloader for firmware updates.
pub trait PrepareBootloader: Sized {
    /// Prepare the bootloader for firmware updates.
    ///
    /// A failure to prepare the bootloader will be logged, but not returned,
    /// since the device may already be in bootloader mode e.g. due to a
    /// previously failed or interrupted firmware update.
    fn prepare_bootloader(self) -> impl Future<Output = Self>;
}

impl<T> PrepareBootloader for T
where
    T: SerialPort + 'static,
{
    async fn prepare_bootloader(self) -> Self {
        let (callbacks_tx, callbacks_rx) = channel::<Callback>(8);
        discard_callbacks(callbacks_rx);
        let mut uart = Uart::new(self, callbacks_tx, 8, 8);

        debug!("Launching standalone bootloader...");
        uart.launch_standalone_bootloader(MODE)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });
        uart.terminate()
    }
}
