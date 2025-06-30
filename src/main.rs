//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

use std::fs::read;
use std::path::PathBuf;
use std::time::Duration;

use ashv2::BaudRate;
use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use log::error;
use serialport::FlowControl;

use fwupd::{FrameCount, Fwupd, Reset, Tty};

mod fwupd;
mod ignore_timeout;

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
        #[clap(
            long,
            short,
            help = "the offset in the firmware file to start uploading from",
            default_value_t = 0
        )]
        offset: usize,
        #[clap(long, short, help = "serial port timeout in milliseconds")]
        timeout: Option<u64>,
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
            offset,
            timeout,
        } => {
            let firmware: Vec<u8> =
                read(firmware).expect("Failed to read firmware file")[offset..].to_vec();
            let progress_bar = ProgressBar::new(firmware.frame_count() as u64);

            Tty::new(args.tty, BaudRate::RstCts, FlowControl::Software)
                .fwupd(
                    firmware,
                    timeout.map(Duration::from_millis),
                    Some(progress_bar),
                )
                .await
                .unwrap_or_else(|err| {
                    error!("Firmware update failed: {err}");
                    std::process::exit(1);
                });
        }
    }
}
