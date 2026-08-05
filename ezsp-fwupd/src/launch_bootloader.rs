use std::io;

use ezsp::Bootloader;
use log::{debug, error};
use serialport::SerialPort;

use crate::make_uart;

const CALLBACK_CHANNEL_SIZE: usize = 8;
const MODE: u8 = 0x00;
const PROTOCOL_VERSION: u8 = 8;
const RESPONSE_CHANNEL_SIZE: usize = 8;

/// Launch a standalone bootloader on the Zigbee NIC's UART.
pub trait LaunchBootloader: Sized {
    /// Launch a standalone bootloader on the Zigbee NIC's UART.
    fn launch_bootloader(self) -> impl Future<Output = io::Result<Self>>;
}

impl<T> LaunchBootloader for T
where
    T: SerialPort + Send + 'static,
{
    async fn launch_bootloader(self) -> io::Result<T> {
        let (tasks, mut connection) = make_uart(
            self,
            CALLBACK_CHANNEL_SIZE,
            RESPONSE_CHANNEL_SIZE,
            PROTOCOL_VERSION,
        )
        .await
        .map_err(io::Error::other)?;
        debug!("Launching standalone bootloader...");
        connection
            .launch_standalone_bootloader(MODE)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });
        drop(connection);
        let serial_port = tasks.terminate().await.map_err(|error| {
            io::Error::other(format!("Failed to terminate actor tasks: {error}"))
        })?;
        Ok(serial_port)
    }
}
