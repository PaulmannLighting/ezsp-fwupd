use ashv2::{Actor, TryCloneNative};
use ezsp::Bootloader;
use ezsp::uart::Uart;
use log::{debug, error};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

use crate::discard_callbacks;

const MODE: u8 = 0x00;

/// Launch a standalone bootloader on the Zigbee NIC's UART.
pub trait LaunchBootloader {
    /// Launch a standalone bootloader on the Zigbee NIC's UART.
    fn launch_bootloader(self) -> impl Future<Output = std::io::Result<()>>;
}

impl<T> LaunchBootloader for T
where
    T: SerialPort + TryCloneNative + Send + Sync + 'static,
{
    async fn launch_bootloader(self) -> std::io::Result<()> {
        let (response_tx, response_rx) = channel(8);
        let (actor, proxy) = Actor::new(self, response_tx, 8)?;
        let (tx_handle, rx_handle) = actor.spawn();
        let (callbacks_tx, callbacks_rx) = channel(8);
        discard_callbacks(callbacks_rx);
        let mut uart = Uart::new(proxy, response_rx, callbacks_tx, 8, 8);
        debug!("Launching standalone bootloader...");
        uart.launch_standalone_bootloader(MODE)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });
        Ok(())
    }
}
