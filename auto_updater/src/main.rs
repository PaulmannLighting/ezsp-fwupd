//! A firmware auto updater for Zigbee devices using the `ezsp` protocol.

use std::fs::{read, read_to_string};
use std::io::ErrorKind;
use std::process::ExitCode;

use args::Args;
use ashv2::{BaudRate, open};
use clap::Parser;
use direction::Direction;
use ezsp::{Callback, uart::Uart};
use ezsp_fwupd::{Fwupd, OtaFile};
use get_current_version::GetCurrentVersion;
use le_stream::FromLeStream;
use log::{error, info};
use manifest::Manifest;
use serialport::{FlowControl, SerialPort};
use tokio::sync::mpsc::Receiver;
use tokio::{sync::mpsc::channel, time::sleep};

mod args;
mod direction;
mod get_current_version;
mod manifest;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let args = Args::parse();

    let Ok(serial_port) = open(
        args.tty().to_string(),
        BaudRate::RstCts,
        FlowControl::Software,
    )
    .inspect_err(|error| error!("Failed to open serial port '{}': {error}", args.tty())) else {
        return ExitCode::FAILURE;
    };

    let (mut uart, _callbacks_rx) = make_uart(serial_port, 8, 8, 8);

    let Some(current_version) = uart.get_current_version().await else {
        return ExitCode::FAILURE;
    };

    let serial_port = uart.terminate();
    info!("Current version:  {current_version}");

    let json = match read_to_string(args.manifest()) {
        Ok(json) => json,
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                info!("No manifest file found at '{}'.", args.manifest().display());
                return ExitCode::SUCCESS;
            }

            error!(
                "Failed to read manifest file '{}': {error}",
                args.manifest().display()
            );
            return ExitCode::FAILURE;
        }
    };

    let Ok(manifest) = serde_json::from_str::<Manifest>(&json)
        .inspect_err(|error| error!("Failed to parse manifest file: {error}"))
    else {
        return ExitCode::FAILURE;
    };

    let Some(active) = manifest.active() else {
        info!("No active firmware version found in the manifest.");
        return ExitCode::SUCCESS;
    };

    info!("Active version:   {}", active.version());

    let Some(direction) = Direction::from_versions(current_version, active.version().clone())
    else {
        info!("Firmware is up to date. No action required.");
        return ExitCode::SUCCESS;
    };

    let Ok(ota_file) = read(active.filename())
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

    info!("{} firmware...", direction.present_participle());
    let serial_port = match serial_port
        .fwupd(
            ota_file.payload().to_vec(),
            Some(args.timeout()),
            false,
            None,
        )
        .await
    {
        Ok(serial_port) => serial_port,
        Err(error) => {
            error!("Firmware {direction} failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    info!(
        "Firmware {direction} complete, waiting {}s for device to reboot...",
        args.reboot_grace_time().as_secs_f32()
    );
    sleep(args.reboot_grace_time()).await;

    let (mut uart, _callbacks_rx) = make_uart(serial_port, 8, 8, 8);

    info!("Validating firmware version.");
    let Some(new_version) = uart.get_current_version().await else {
        return ExitCode::FAILURE;
    };

    if new_version != *active.version() {
        error!(
            "Firmware {direction} failed: expected version {}, got {new_version}",
            active.version()
        );
        return ExitCode::FAILURE;
    }

    info!("Firmware {direction} successful. New version: {new_version}");
    ExitCode::SUCCESS
}

fn make_uart<T>(
    serial_port: T,
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
) -> (Uart<T>, Receiver<Callback>)
where
    T: SerialPort + 'static,
{
    let (callbacks_tx, callbacks_rx) = channel::<Callback>(callback_channel_size);
    (
        Uart::new(
            serial_port,
            callbacks_tx,
            protocol_version,
            response_channel_size,
        ),
        callbacks_rx,
    )
}
