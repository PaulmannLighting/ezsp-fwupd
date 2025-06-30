//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

use std::fs::read;
use std::path::PathBuf;
use std::time::Duration;

use crate::fwupd::FrameCount;
use ashv2::BaudRate;
use clap::Parser;
use fwupd::{Fwupd, Tty};
use indicatif::ProgressBar;
use log::error;
use serialport::FlowControl;

mod fwupd;
mod ignore_timeout;

#[derive(Debug, Parser)]
struct Args {
    #[clap(index = 1, help = "the serial port to use for firmware update")]
    tty: String,
    #[clap(index = 2, help = "the firmware file to upload")]
    firmware: PathBuf,
    #[clap(
        long,
        short,
        help = "the offset in the firmware file to start uploading from",
        default_value_t = 0
    )]
    offset: usize,
    #[clap(
        long,
        short,
        help = "do not prepare the bootloader before uploading the firmware"
    )]
    no_prepare: bool,
    #[clap(
        long,
        short,
        help = "the timeout in milliseconds for the firmware update"
    )]
    timeout: Option<u64>,
    #[clap(long, help = "only reset the device")]
    reset: bool,
}

impl Args {
    pub fn firmware(&self) -> std::io::Result<Vec<u8>> {
        read(&self.firmware)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let firmware: Vec<u8> =
        args.firmware().expect("Failed to read firmware file")[args.offset..].to_vec();
    let frame_count = firmware.frame_count();
    let progress_bar = ProgressBar::new(frame_count as u64);

    Tty::new(args.tty, BaudRate::RstCts, FlowControl::Software)
        .fwupd(
            firmware,
            args.timeout.map(Duration::from_millis),
            !args.no_prepare,
            args.reset,
            Some(progress_bar),
        )
        .await
        .unwrap_or_else(|err| {
            error!("Firmware update failed: {err}");
            std::process::exit(1);
        });
}
