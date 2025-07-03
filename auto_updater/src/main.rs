//! A firmware auto updater for Zigbee devices using the `ezsp` protocol.

use std::fs::{read, read_to_string};
use std::process::ExitCode;

use ashv2::BaudRate;
use clap::Parser;
use ezsp::uart::Uart;
use ezsp::{Callback, GetValueExt};
use ezsp_fwupd::{Fwupd, OtaFile, Tty};
use le_stream::FromLeStream;
use log::{error, info};
use semver::Version;
use serialport::FlowControl;
use tokio::sync::mpsc::channel;
use tokio::time::sleep;

use args::Args;
use direction::Direction;
use manifest::Manifest;

mod args;
mod direction;
mod manifest;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let args = Args::parse();
    let tty = Tty::new(
        args.tty().to_string(),
        BaudRate::RstCts,
        FlowControl::Software,
    );

    let Some(current_version) = get_current_version(tty.clone()).await else {
        return ExitCode::FAILURE;
    };
    info!("Current version:  {current_version}");

    let Ok(json) = read_to_string(args.manifest())
        .inspect_err(|error| error!("Failed to read manifest file: {error}"))
    else {
        return ExitCode::FAILURE;
    };

    let Ok(manifest) = serde_json::from_str::<Manifest>(&json)
        .inspect_err(|error| error!("Failed to parse manifest file: {error}"))
    else {
        return ExitCode::FAILURE;
    };

    info!("Active version:   {}", manifest.active().version());

    let Some(direction) = Direction::parse(current_version, manifest.active().version().clone())
    else {
        info!("Firmware is up to date. No action required.");
        return ExitCode::SUCCESS;
    };

    let Ok(ota_file) = read(manifest.active().filename())
        .inspect_err(|error| error!("Failed to read firmware file: {error}"))
    else {
        return ExitCode::FAILURE;
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
        return ExitCode::FAILURE;
    };

    let header = ota_file.header();
    info!("OTA image name:   {}", header.name());
    info!("OTA image type:   {}", header.image_type());
    info!("OTA file version: {}", header.firmware_version());
    info!("OTA Zigbee stack: {}", header.zigbee_stack_version());
    info!("OTA manufacturer: {}", header.manufacturer_id());
    info!("OTA image size:   {}", header.image_size());

    info!("{} firmware...", direction.gerund());
    if let Err(error) = tty
        .fwupd(
            ota_file.payload().to_vec(),
            Some(args.timeout()),
            false,
            None,
        )
        .await
    {
        error!("Firmware {direction} failed: {error}");
        return ExitCode::FAILURE;
    }

    info!(
        "Firmware {direction} complete, waiting {}s for device to reboot...",
        args.reboot_grace_time().as_secs_f32()
    );
    sleep(args.reboot_grace_time()).await;

    info!("Validating firmware version.");
    let Some(new_version) = get_current_version(tty.clone()).await else {
        return ExitCode::FAILURE;
    };

    if new_version != *manifest.active().version() {
        error!(
            "Firmware {direction} failed: expected version {}, got {new_version}",
            manifest.active().version()
        );
        return ExitCode::FAILURE;
    }

    info!("Firmware {direction} successful. New version: {new_version}");
    ExitCode::SUCCESS
}

/// Get the current firmware version from the Zigbee device.
async fn get_current_version(tty: Tty) -> Option<Version> {
    let Ok(serial_port) = tty.open().inspect_err(|error| error!("{error}")) else {
        return None;
    };

    let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
    let mut uart = Uart::new(serial_port, callbacks_tx, 8, 8);

    match uart.get_ember_version().await {
        Ok(result) => match result {
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
        },
        Err(error) => {
            error!("Failed to get version info: {error}");
            None
        }
    }
}
