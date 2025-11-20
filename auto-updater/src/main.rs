//! A firmware auto updater for Zigbee devices using the `ezsp` protocol.

use std::fs::read;
use std::io;
use std::process::ExitCode;
use std::time::Duration;

use ashv2::{BaudRate, open};
use clap::Parser;
use ezsp_fwupd::{Fwupd, OtaFile, Reset};
use le_stream::FromLeStream;
use log::{error, info};
use semver::Version;
use serialport::{FlowControl, SerialPort};
use tokio::time::sleep;

use self::args::Args;
use self::current_version::CurrentVersion;
use self::direction::Direction;
use self::make_uart::make_uart;
use self::manifest::{Metadata, get_metadata};

mod args;
mod current_version;
mod direction;
mod make_uart;
mod manifest;

const CALLBACK_CHANNEL_SIZE: usize = 8;
const RESPONSE_CHANNEL_SIZE: usize = 8;
const PROTOCOL_VERSION: u8 = 8;
const MAX_RETRIES: usize = 5;
const RETRY_INTERVAL: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let args = Args::parse();

    let metadata = match get_metadata(args.manifest()) {
        Ok(Some(metadata)) => metadata,
        Ok(None) => {
            info!("No active firmware version configured.");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            error!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let Some(ota_file) = validate_ota_file(&metadata) else {
        return ExitCode::FAILURE;
    };

    let Ok(serial_port) = open(
        args.tty().to_string(),
        BaudRate::RstCts,
        FlowControl::Software,
    )
    .inspect_err(|error| error!("Failed to open serial port '{}': {error}", args.tty())) else {
        return ExitCode::FAILURE;
    };

    let Some((serial_port, current_version)) = get_current_version(serial_port).await else {
        return ExitCode::FAILURE;
    };

    info!("Active version:   {}", metadata.version());

    let Some(direction) = Direction::from_versions(&current_version, metadata.version()) else {
        info!("Firmware is up to date. No action required.");
        return ExitCode::SUCCESS;
    };

    match update_firmware(
        serial_port,
        ota_file,
        direction,
        args.timeout(),
        args.reboot_grace_time(),
    )
    .await
    {
        Ok(serial_port) => validate_firmware(serial_port, metadata.version(), &direction)
            .await
            .map_or(ExitCode::FAILURE, |new_version| {
                info!("Firmware {direction} successful. New version: {new_version}");
                ExitCode::SUCCESS
            }),
        Err(error) => {
            error!("Firmware update failed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Get the current firmware version from the Zigbee device.
async fn get_current_version<T>(serial_port: T) -> Option<(T, Version)>
where
    T: SerialPort + 'static,
{
    let (mut uart, _callbacks_rx) = make_uart(
        serial_port,
        CALLBACK_CHANNEL_SIZE,
        RESPONSE_CHANNEL_SIZE,
        PROTOCOL_VERSION,
    );

    let Some(current_version) = uart
        .await_current_version(RETRY_INTERVAL, MAX_RETRIES)
        .await
    else {
        error!("Failed to get current firmware version.");

        if let Err(error) = uart.terminate().reset(Some(RETRY_INTERVAL)) {
            error!("Failed to reset device: {error}");
        }

        return None;
    };

    let serial_port = uart.terminate();
    info!("Current version:  {current_version}");
    Some((serial_port, current_version))
}

/// Validate the OTA file by reading it and checking its contents.
fn validate_ota_file(metadata: &Metadata) -> Option<OtaFile> {
    let Ok(ota_file) = read(metadata.filename())
        .inspect_err(|error| error!("Failed to read firmware file: {error}"))
    else {
        return None;
    };

    let Ok(ota_file) = OtaFile::from_le_stream_exact(ota_file.into_iter())
        .inspect_err(|error| error!("Failed to parse OTA file: {error}"))
        .map_err(drop)
        .and_then(|ota_file| {
            ota_file
                .validate()
                .inspect_err(|error| {
                    error!("Invalid OTA file magic: {error:#04X?}");
                })
                .map_err(drop)
        })
    else {
        return None;
    };

    let header = ota_file.header();
    info!("OTA image name:   {}", header.name());
    info!("OTA image type:   {}", header.image_type());
    info!("OTA file version: {}", header.firmware_version());
    info!("OTA Zigbee stack: {}", header.zigbee_stack_version());
    info!("OTA manufacturer: {}", header.manufacturer_id());
    info!("OTA image size:   {}", header.image_size());
    Some(ota_file)
}

/// Update the firmware of the Zigbee device.
async fn update_firmware<T>(
    serial_port: T,
    ota_file: OtaFile,
    direction: Direction,
    timeout: Duration,
    reboot_grace_time: Duration,
) -> io::Result<T>
where
    T: SerialPort + 'static,
{
    info!("{} firmware...", direction.present_participle());
    let serial_port = serial_port
        .fwupd(ota_file.payload().to_vec(), Some(timeout), false, None)
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

/// Validate the firmware version after the update.
async fn validate_firmware<T>(
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
    let Some(new_version) = uart
        .await_current_version(RETRY_INTERVAL, MAX_RETRIES)
        .await
    else {
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
