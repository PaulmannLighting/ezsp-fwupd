use std::io;
use std::time::Duration;

use ezsp_fwupd::{Fwupd, OtaFile};
use log::{error, info};
use serialport::SerialPort;
use tokio::time::sleep;

use crate::direction::Direction;

/// Update the firmware of the Zigbee device.
pub async fn update_firmware<T>(
    serial_port: T,
    ota_file: &OtaFile,
    direction: Direction,
    timeout: Duration,
    reboot_grace_time: Duration,
) -> io::Result<T>
where
    T: SerialPort + 'static,
{
    info!("{} firmware...", direction.present_participle());
    let serial_port = serial_port
        .fwupd(ota_file.payload().to_vec(), Some(timeout), None)
        .await
        .inspect_err(|error| {
            error!("Firmware {direction} failed: {error}");
        })?;
    info!(
        "Firmware {direction} complete, waiting {}s for device to reboot...",
        reboot_grace_time.as_secs_f32()
    );
    sleep(reboot_grace_time).await;
    Ok(serial_port)
}
