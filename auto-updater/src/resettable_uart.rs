use std::array::TryFromSliceError;
use std::time::Duration;

use ezsp::ezsp::value::EmberVersion;
use ezsp::uart::Uart;
use ezsp::{Callback, GetValueExt};
use ezsp_fwupd::Reset;
use log::{debug, error, info};
use semver::Version;
use serialport::SerialPort;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;

use crate::make_uart::make_uart;

/// A UART connection that can be reset and recreated.
#[derive(Debug)]
pub struct ResettableUart<T> {
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
    uart: Option<Uart<T>>,
    callbacks: Receiver<Callback>,
}

impl<T> ResettableUart<T>
where
    T: SerialPort + 'static,
{
    /// Creates a new `ResettableUart` instance.
    pub fn new(
        serial_port: T,
        callback_channel_size: usize,
        response_channel_size: usize,
        protocol_version: u8,
    ) -> Self {
        let (uart, callbacks) = make_uart(
            serial_port,
            callback_channel_size,
            response_channel_size,
            protocol_version,
        );
        Self {
            callback_channel_size,
            response_channel_size,
            protocol_version,
            uart: Some(uart),
            callbacks,
        }
    }

    /// Awaits the current version from the device, retrying on failure.
    pub async fn await_current_version(
        &mut self,
        retry_interval: Duration,
        mut max_retries: usize,
    ) -> Option<(Version, Uart<T>)> {
        loop {
            let mut uart = self
                .uart
                .take()
                .expect("There should always be a UART here.");

            match uart.get_ember_version().await {
                Ok(result) => return parse_version(result).map(|version| (version, uart)),
                Err(error) => {
                    debug!("Failed to get version info: {error}");

                    if let Some(retries) = max_retries.checked_sub(1) {
                        max_retries = retries;
                    } else {
                        error!("Max retries reached: {error}");
                        return None;
                    }

                    let mut serial_port = uart.terminate();

                    match serial_port.reset(Some(retry_interval)) {
                        Ok(()) => info!("Successfully reset the device."),
                        Err(error) => {
                            error!("Failed to reset the device: {error}");
                            sleep(retry_interval).await;
                        }
                    }

                    self.recreate_uart(serial_port);
                }
            }
        }
    }

    fn recreate_uart(&mut self, serial_port: T) {
        let (uart, callbacks) = make_uart(
            serial_port,
            self.callback_channel_size,
            self.response_channel_size,
            self.protocol_version,
        );
        self.uart.replace(uart);
        self.callbacks = callbacks;
    }
}

/// Parse the version information from the device.
fn parse_version(result: Result<EmberVersion, TryFromSliceError>) -> Option<Version> {
    match result {
        Ok(version_info) => match version_info.try_into() {
            Ok(version) => Some(version),
            Err(error) => {
                error!("Failed to parse version info: {error}");
                None
            }
        },
        Err(error) => {
            error!("Failed to parse version info: {error}");
            None
        }
    }
}
