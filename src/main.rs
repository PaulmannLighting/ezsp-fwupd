//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

use std::fs::read;
use std::path::PathBuf;
use std::time::Duration;

use ashv2::BaudRate;
use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use le_stream::FromLeStream;
use log::{error, info};
use serialport::FlowControl;

use crate::ota_file::OtaFile;
use fwupd::{FrameCount, Fwupd, Reset, Tty};

const DEFAULT_TIMEOUT: u64 = 1000; // Default timeout in milliseconds

mod clear_buffer;
mod fwupd;
mod ignore_timeout;
mod ota_file;
mod xmodem;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    Reset {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
        #[clap(long, short, help = "serial port timeout in milliseconds")]
        timeout: Option<u64>,
    },
    Update {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
        #[clap(index = 2, help = "the firmware file to upload")]
        firmware: PathBuf,
        #[clap(long, short, help = "serial port timeout in milliseconds", default_value_t = DEFAULT_TIMEOUT)]
        timeout: u64,
        #[clap(long, short, help = "offset in bytes to skip in the firmware file")]
        no_prepare: bool,
    },
    Ota {
        #[clap(index = 1, help = "the firmware file to upload")]
        firmware: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    match args.action {
        Action::Reset { tty, timeout } => {
            Tty::new(tty, BaudRate::RstCts, FlowControl::Software)
                .open()
                .unwrap_or_else(|err| {
                    error!("Failed to open serial port: {err}");
                    std::process::exit(1);
                })
                .reset(timeout.map(Duration::from_millis))
                .unwrap_or_else(|err| {
                    error!("Failed to reset device: {err}");
                    std::process::exit(1);
                });
        }
        Action::Update {
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
            info!("{ota_file}");
            let firmware = ota_file.payload().to_vec();
            let progress_bar = ProgressBar::new(firmware.frame_count() as u64);

            Tty::new(tty, BaudRate::RstCts, FlowControl::Software)
                .fwupd(
                    firmware,
                    Some(Duration::from_millis(timeout)),
                    no_prepare,
                    Some(progress_bar),
                )
                .await
                .unwrap_or_else(|err| {
                    error!("Firmware update failed: {err}");
                    std::process::exit(1);
                });
        }
        Action::Ota { firmware } => {
            let firmware: Vec<u8> = read(firmware).expect("Failed to read firmware file");
            let ota_file = OtaFile::from_le_stream_exact(firmware.into_iter())
                .expect("Failed to read ota file")
                .validate()
                .expect("Failed to validate ota file");
            println!("OTA file: {ota_file:#04X?}");
        }
    }
}
