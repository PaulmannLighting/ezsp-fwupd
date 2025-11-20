use std::array::TryFromSliceError;

use ezsp::GetValueExt;
use ezsp::ezsp::value::EmberVersion;
use ezsp::uart::Uart;
use log::{debug, error};
use semver::Version;
use serialport::SerialPort;

use crate::constants::{CALLBACK_CHANNEL_SIZE, PROTOCOL_VERSION, RESPONSE_CHANNEL_SIZE};
use crate::make_uart::make_uart;

/// Extension trait for getting the current firmware version from a Zigbee device.
pub trait CurrentVersion {
    /// Await the current firmware version from the Zigbee device.
    fn get_current_version(&mut self) -> impl Future<Output = Option<Version>>;

    /// Parse the version information from the device.
    fn parse_version(&self, result: Result<EmberVersion, TryFromSliceError>) -> Option<Version> {
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
}

impl<T> CurrentVersion for Uart<T>
where
    T: SerialPort + 'static,
{
    async fn get_current_version(&mut self) -> Option<Version> {
        match self.get_ember_version().await {
            Ok(result) => self.parse_version(result),
            Err(error) => {
                debug!("Failed to get version info: {error}");
                None
            }
        }
    }
}

/// Get the current firmware version from the Zigbee device.
pub async fn get_current_version<T>(serial_port: T) -> (Option<Version>, T)
where
    T: SerialPort + 'static,
{
    let (mut uart, _callbacks_rx) = make_uart(
        serial_port,
        CALLBACK_CHANNEL_SIZE,
        RESPONSE_CHANNEL_SIZE,
        PROTOCOL_VERSION,
    );

    let Some(current_version) = uart.get_current_version().await else {
        return (None, uart.terminate());
    };

    let serial_port = uart.terminate();
    (Some(current_version), serial_port)
}
