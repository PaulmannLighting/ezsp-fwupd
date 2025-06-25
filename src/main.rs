use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

use ashv2::BaudRate;
use clap::Parser;
use fwupd::{Tty, update_firmware};
use log::error;
use serialport::FlowControl;

mod fwupd;

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
}

impl Args {
    pub fn firmware(&self) -> std::io::Result<Vec<u8>> {
        OpenOptions::new()
            .read(true)
            .open(&self.firmware)
            .and_then(|mut file| {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                Ok(buffer)
            })
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let firmware = args.firmware().expect("Failed to read firmware file");
    let tty = Tty::new(args.tty, BaudRate::RstCts, FlowControl::Software);

    update_firmware(tty, firmware).await.unwrap_or_else(|err| {
        error!("Firmware update failed: {err}");
        std::process::exit(1);
    });
}
