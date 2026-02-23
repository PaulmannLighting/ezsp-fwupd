use std::array::TryFromSliceError;
use std::time::Duration;

use ashv2::TryCloneNative;
use ezsp::GetValueExt;
use ezsp::ezsp::value::EmberVersion;
use ezsp::uart::Uart;
use ezsp_fwupd::make_uart;
use log::{debug, error};
use semver::Version;
use serialport::SerialPort;
use tokio::time::sleep;

use crate::uart_params::UartParams;

/// Extension trait for getting the current firmware version from a Zigbee device.
pub trait CurrentVersion {
    /// Await the current firmware version from the Zigbee device.
    fn get_current_version(&mut self) -> impl Future<Output = Option<Version>>;

    /// Await the current firmware version from the Zigbee device.
    async fn await_current_version(&mut self, interval: Duration, retries: u8) -> Option<Version> {
        for attempt in 0..retries {
            if let Some(version) = self.get_current_version().await {
                debug!("Retrieved current version on attempt #{attempt}: {version}");
                return Some(version);
            }

            sleep(interval).await;
        }

        error!("Exceeded maximum retries to get current version.");
        None
    }
}

impl CurrentVersion for Uart {
    async fn get_current_version(&mut self) -> Option<Version> {
        match self.get_ember_version().await {
            Ok(result) => parse_version(result),
            Err(error) => {
                debug!("Failed to get version info: {error}");
                None
            }
        }
    }
}

/// Get the current firmware version from the Zigbee device.
pub async fn get_current_version<T>(
    serial_port: T,
    uart_params: &UartParams,
) -> (Option<Version>, T)
where
    T: SerialPort + TryCloneNative + Send + Sync + 'static,
{
    let (tasks, mut uart) = make_uart(
        serial_port,
        uart_params.callback_channel_size(),
        uart_params.response_channel_size(),
        uart_params.protocol_version(),
    )
    .expect("Failed to create uart");

    let current_version = uart.get_current_version().await;
    let serial_port = tasks.terminate().await.expect("Failed to terminate tasks");
    (current_version, serial_port)
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
