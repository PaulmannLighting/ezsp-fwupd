//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

use std::fs::read;
use std::path::PathBuf;
use std::time::Duration;

use ashv2::BaudRate;
use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use le_stream::FromLeStream;
use log::error;
use serialport::FlowControl;

use crate::ota_file::OtaFile;
use fwupd::{FrameCount, Fwupd, Reset, Tty};

mod clear_buffer;
mod fwupd;
mod ignore_timeout;
mod ota_file;
mod xmodem;

#[derive(Debug, Parser)]
struct Args {
    #[clap(index = 1, help = "the serial port to use for firmware update")]
    tty: String,
    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    Reset {
        #[clap(long, short, help = "serial port timeout in milliseconds")]
        timeout: Option<u64>,
    },
    Update {
        #[clap(index = 1, help = "the firmware file to upload")]
        firmware: PathBuf,
        #[clap(long, short, help = "serial port timeout in milliseconds")]
        timeout: Option<u64>,
        #[clap(long, short, help = "offset in bytes to skip in the firmware file")]
        offset: usize,
        #[clap(
            long,
            short,
            help = "do not prepare the bootloader before firmware update"
        )]
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
        Action::Reset { timeout } => {
            Tty::new(args.tty, BaudRate::RstCts, FlowControl::Software)
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
            firmware,
            timeout,
            offset,
            no_prepare,
        } => {
            let firmware: Vec<u8> =
                read(firmware).expect("Failed to read firmware file")[offset..].to_vec();
            let progress_bar = ProgressBar::new(firmware.frame_count() as u64);

            Tty::new(args.tty, BaudRate::RstCts, FlowControl::Software)
                .fwupd(
                    firmware,
                    timeout.map(Duration::from_millis),
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
            println!(
                "Raw first 66 bytes: {:#04X}",
                ashv2::HexSlice::new(&firmware[..66])
            );
            let ota_file = OtaFile::from_le_stream_exact(firmware.into_iter())
                .expect("Failed to read ota file");
            let header = ota_file.header();
            println!("OTA header: {header:?}");
            println!("OTA footer: {:?}", ota_file.footer());
            println!("Header string: {}", header.header_string());
            println!(
                "Size: {} / {}",
                header.image_size(),
                ota_file.payload().len()
            );
        }
    }
}
