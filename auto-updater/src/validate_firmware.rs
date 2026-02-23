use core::time::Duration;

use ashv2::TryCloneNative;
use ezsp_fwupd::{Reset, make_uart};
use log::{error, info};
use semver::Version;
use serialport::SerialPort;

use crate::current_version::CurrentVersion;
use crate::direction::Direction;
use crate::uart_params::UartParams;

/// Validate the firmware version after the update.
pub async fn validate_firmware<T>(
    serial_port: T,
    uart_params: &UartParams,
    retry_interval: Duration,
    max_retries: u8,
    version: &Version,
    direction: &Direction,
) -> Option<Version>
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

    info!("Validating firmware version.");
    let Some(new_version) = uart
        .await_current_version(retry_interval, max_retries)
        .await
    else {
        error!("Failed to get new firmware version after update.");
        let Ok(mut serial_port) = tasks
            .terminate()
            .await
            .inspect_err(|error| error!("Failed to terminate ASHv2 tasks: {error}"))
        else {
            return None;
        };

        if let Err(error) = serial_port.reset(Some(retry_interval)) {
            error!("Failed to reset device: {error}");
        }

        return None;
    };

    tasks.terminate().await.map_or_else(
        |error| error!("Failed to terminate ASHv2 tasks: {error}"),
        drop,
    );

    if new_version != *version {
        error!("Firmware {direction} failed: expected version {version}, got {new_version}",);
        return None;
    }

    Some(new_version)
}
