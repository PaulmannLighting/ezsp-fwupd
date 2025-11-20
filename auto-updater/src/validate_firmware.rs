use ezsp_fwupd::Reset;
use log::{error, info};
use semver::Version;
use serialport::SerialPort;

use crate::constants::{
    CALLBACK_CHANNEL_SIZE, PROTOCOL_VERSION, RESPONSE_CHANNEL_SIZE, RETRY_INTERVAL,
};
use crate::current_version::CurrentVersion;
use crate::direction::Direction;
use crate::make_uart::make_uart;

/// Validate the firmware version after the update.
pub async fn validate_firmware<T>(
    serial_port: T,
    version: &Version,
    direction: &Direction,
) -> Option<Version>
where
    T: SerialPort + 'static,
{
    let (mut uart, _callbacks_rx) = make_uart(
        serial_port,
        CALLBACK_CHANNEL_SIZE,
        RESPONSE_CHANNEL_SIZE,
        PROTOCOL_VERSION,
    );

    info!("Validating firmware version.");
    let Some(new_version) = uart.get_current_version().await else {
        error!("Failed to get new firmware version after update.");

        if let Err(error) = uart.terminate().reset(Some(RETRY_INTERVAL)) {
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
