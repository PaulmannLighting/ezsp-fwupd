//! A firmware auto updater for Zigbee devices using the `ezsp` protocol.

use std::collections::BTreeMap;
use std::fs::read;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use ashv2::BaudRate;
use clap::Parser;
use ezsp::uart::Uart;
use ezsp::{Callback, GetValueExt};
use ezsp_fwupd::{Fwupd, OtaFile, Tty, VersionFromFilename};
use le_stream::FromLeStream;
use log::{error, info, warn};
use semver::Version;
use serialport::FlowControl;
use tokio::sync::mpsc::channel;

const DEFAULT_TIMEOUT: u64 = 1000; // Default timeout in milliseconds

#[derive(Debug, Parser)]
struct Args {
    #[clap(index = 1, help = "the serial port to use for firmware update")]
    tty: String,
    #[clap(long, short, help = "the firmware files' base directory")]
    base_dir: PathBuf,
    #[clap(long, short, help = "serial port timeout in milliseconds", default_value_t = DEFAULT_TIMEOUT)]
    timeout: u64,
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let args = Args::parse();
    let tty = Tty::new(args.tty, BaudRate::RstCts, FlowControl::Software);
    let Some(current_version) = get_current_version(tty.clone()).await else {
        return ExitCode::FAILURE;
    };

    info!("Current version: {current_version}");
    let firmware_files = list_firmware_files(&args.base_dir);

    for version in firmware_files.keys() {
        info!("Available firmware version: {version}");
    }

    let Some((latest_version, file)) = firmware_files.last_key_value() else {
        error!(
            "No valid firmware files found in the specified directory: {}",
            args.base_dir.display()
        );
        return ExitCode::FAILURE;
    };

    if current_version > *latest_version {
        info!(
            "No firmware update needed. Current version: {current_version} >= Latest version: {latest_version}"
        );
        return ExitCode::SUCCESS;
    }

    let Ok(ota_file) =
        read(file).inspect_err(|error| error!("Failed to read firmware file: {error}"))
    else {
        return ExitCode::FAILURE;
    };

    let Ok(ota_file) = OtaFile::from_le_stream_exact(ota_file.into_iter())
        .inspect_err(|error| error!("Failed to parse OTA file: {error}"))
    else {
        return ExitCode::FAILURE;
    };

    let Ok(ota_file) = ota_file.validate().inspect_err(|error| {
        error!("Invalid OTA file magic: {error:#04X?}");
    }) else {
        return ExitCode::FAILURE;
    };

    let header = ota_file.header();
    info!("OTA image name:   {}", header.name());
    info!("OTA image type:   {}", header.image_type());
    info!("OTA file version: {}", header.firmware_version());
    info!("OTA Zigbee stack: {}", header.zigbee_stack_version());
    info!("OTA manufacturer: {}", header.manufacturer_id());
    info!("OTA image size:   {}", header.image_size());

    info!("Updating firmware...");
    if let Err(error) = tty
        .fwupd(
            ota_file.payload().to_vec(),
            Some(Duration::from_millis(args.timeout)),
            false,
            None,
        )
        .await
    {
        error!("Firmware update failed: {error}");
        return ExitCode::FAILURE;
    }

    let Some(current_version_after_update) = get_current_version(tty.clone()).await else {
        return ExitCode::FAILURE;
    };

    info!("validating firmware version.");
    if current_version_after_update != *latest_version {
        error!(
            "Firmware update failed: expected version {latest_version}, got {current_version_after_update}"
        );
        return ExitCode::FAILURE;
    }

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

/// List all firmware files in the specified directory.
///
/// Valid firmware files must have the `.ota` extension and contain a valid semver in their filename.
fn list_firmware_files(base_dir: &PathBuf) -> BTreeMap<Version, PathBuf> {
    let mut files = BTreeMap::new();

    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().canonicalize() {
                if path.is_file() {
                    if path.extension().is_some_and(|ext| ext == "ota") {
                        if let Some(version) = path.version_from_filename() {
                            files.insert(version, path);
                        } else {
                            warn!("Failed to extract version from file: {}", path.display());
                        }
                    } else {
                        warn!("File does not have a valid extension: {}", path.display());
                    }
                }
            }
        }
    }

    files
}
