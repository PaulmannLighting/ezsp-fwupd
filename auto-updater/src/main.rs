//! A firmware auto updater for Zigbee devices using the `ezsp` protocol.

use std::process::ExitCode;

use ashv2::{BaudRate, open};
use clap::Parser;
use log::{error, info};
use serialport::FlowControl;

use self::args::Args;
use self::current_version::get_current_version;
use self::direction::Direction;
use self::manifest::get_metadata;
use self::update_firmware::update_firmware;
use self::validate_ota_file::validate_ota_file;
use crate::validate_firmware::validate_firmware;

mod args;
mod current_version;
mod direction;
mod make_uart;
mod manifest;
mod update_firmware;
mod validate_firmware;
mod validate_ota_file;

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

    let (current_version, serial_port) = get_current_version(
        serial_port,
        args.callback_channel_size(),
        args.response_channel_size(),
        args.protocol_version(),
    )
    .await;

    if let Some(current_version) = &current_version {
        info!("Current version:  {current_version}");
    } else {
        error!("Failed to get current firmware version.");
    }

    let Some(direction) = Direction::from_versions(current_version.as_ref(), metadata.version())
    else {
        info!("Firmware is up to date. No action required.");
        return ExitCode::SUCCESS;
    };

    info!("Active version:   {}", metadata.version());

    match update_firmware(
        serial_port,
        &ota_file,
        direction,
        args.timeout(),
        args.reboot_grace_time(),
    )
    .await
    {
        Ok(serial_port) => validate_firmware(
            serial_port,
            args.callback_channel_size(),
            args.response_channel_size(),
            args.protocol_version(),
            args.timeout(),
            args.max_retries(),
            metadata.version(),
            &direction,
        )
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
