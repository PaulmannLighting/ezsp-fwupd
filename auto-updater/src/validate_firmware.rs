use core::time::Duration;

use ezsp_fwupd::Reset;
use log::{error, info};
use semver::Version;
use serialport::SerialPort;

use crate::current_version::CurrentVersion;
use crate::direction::Direction;
use crate::make_uart::make_uart;

/// Validate the firmware version after the update.
pub async fn validate_firmware<T>(
    serial_port: T,
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
    retry_interval: Duration,
    max_retries: u8,
    version: &Version,
    direction: &Direction,
) -> Option<Version>
where
    T: SerialPort + 'static,
{
    let (mut uart, _callbacks_rx) = make_uart(
        serial_port,
        callback_channel_size,
        response_channel_size,
        protocol_version,
    );

    info!("Validating firmware version.");
    let Some(new_version) = uart
        .await_current_version(retry_interval, max_retries)
        .await
    else {
        error!("Failed to get new firmware version after update.");

        if let Err(error) = uart.terminate().reset(Some(retry_interval)) {
            error!("Failed to reset device: {error}");
        }

        return None;
    };

    if new_version != *version {
        error!("Firmware {direction} failed: expected version {version}, got {new_version}",);
        return None;
    }

    Some(new_version)
}
