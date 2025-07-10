//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

use std::fs::read;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use ashv2::{BaudRate, open};
use clap::{Parser, Subcommand};
use ezsp::uart::Uart;
use ezsp::{Callback, GetValueExt};
use ezsp_fwupd::{FrameCount, Fwupd, OtaFile, Reset};
use indicatif::{ProgressBar, ProgressStyle};
use le_stream::FromLeStream;
use log::error;
use semver::Version;
use serialport::FlowControl;
use tokio::sync::mpsc::channel;

const DEFAULT_TIMEOUT: u64 = 1000; // Default timeout in milliseconds

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    #[clap(name = "flash", about = "Flash firmware onto the device")]
    Flash {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
        #[clap(index = 2, help = "the firmware file to upload")]
        firmware: PathBuf,
        #[clap(long, short, help = "serial port timeout in milliseconds", default_value_t = DEFAULT_TIMEOUT)]
        timeout: u64,
        #[clap(long, short, help = "offset in bytes to skip in the firmware file")]
        no_prepare: bool,
    },
    #[clap(name = "reset", about = "Reset the device")]
    Reset {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
        #[clap(long, short, help = "serial port timeout in milliseconds")]
        timeout: Option<u64>,
    },
    #[clap(name = "query", about = "Query the device for version info")]
    Query {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
    },
    #[clap(name = "ota", about = "Parse an OTA file")]
    Ota {
        #[clap(index = 1, help = "the firmware file to upload")]
        firmware: PathBuf,
        #[clap(long, short, help = "enable debug output")]
        debug: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let args = Args::parse();

    match args.action {
        Action::Flash {
            tty,
            firmware,
            timeout,
            no_prepare,
        } => {
            let firmware: Vec<u8> = read(firmware).expect("Failed to read firmware file");
            let ota_file = OtaFile::from_le_stream_exact(firmware.into_iter())
                .expect("Failed to read ota file")
                .validate()
                .expect("Failed to validate ota file");
            let firmware = ota_file.payload().to_vec();
            let progress_bar = ProgressBar::new(firmware.frame_count() as u64);
            progress_bar.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                )
                .unwrap()
                .progress_chars("##-"),
            );
            progress_bar.println("### Firmware update info ###");
            progress_bar.println(ota_file.to_string());

            let Ok(serial_port) = open(tty.clone(), BaudRate::RstCts, FlowControl::Software)
                .inspect_err(|error| error!("Failed to open serial port '{tty}': {error}"))
            else {
                return ExitCode::FAILURE;
            };

            let result = serial_port
                .fwupd(
                    firmware,
                    Some(Duration::from_millis(timeout)),
                    no_prepare,
                    Some(&progress_bar),
                )
                .await
                .map(drop);

            progress_bar.finish();

            if let Err(error) = result {
                error!("Firmware update failed: {error}");
                return ExitCode::FAILURE;
            };

            ExitCode::SUCCESS
        }
        Action::Reset { tty, timeout } => {
            let Ok(mut serial_port) = open(tty.clone(), BaudRate::RstCts, FlowControl::Software)
                .inspect_err(|error| error!("Failed to open serial port '{tty}': {error}"))
            else {
                return ExitCode::FAILURE;
            };

            if let Err(error) = serial_port.reset(timeout.map(Duration::from_millis)) {
                error!("Failed to reset device: {error}");
                return ExitCode::FAILURE;
            };

            ExitCode::SUCCESS
        }
        Action::Query { tty } => {
            let Ok(serial_port) = open(tty.clone(), BaudRate::RstCts, FlowControl::Software)
                .inspect_err(|error| error!("Failed to open serial port '{tty}': {error}"))
            else {
                return ExitCode::FAILURE;
            };

            let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
            let mut uart = Uart::new(serial_port, callbacks_tx, 8, 8);

            match uart.get_ember_version().await {
                Ok(result) => match result {
                    Ok(version_info) => {
                        println!("{version_info}");

                        if let Ok(semver) = Version::try_from(version_info) {
                            println!("Semver: {semver}");
                        };

                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        error!("Failed to parse version info: {error}");
                        ExitCode::FAILURE
                    }
                },
                Err(error) => {
                    error!("Failed to get version info: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Action::Ota { firmware, debug } => {
            let firmware: Vec<u8> = read(firmware).expect("Failed to read firmware file");
            let Ok(ota_file) = OtaFile::from_le_stream_exact(firmware.into_iter()) else {
                error!("Failed to read ota file");
                return ExitCode::FAILURE;
            };

            let Ok(ota_file) = ota_file.validate() else {
                error!("Failed to validate ota file");
                return ExitCode::FAILURE;
            };

            if debug {
                println!("Ota file:\n{ota_file:#04X?}");
            } else {
                println!("{ota_file}");
            }

            ExitCode::SUCCESS
        }
    }
}
