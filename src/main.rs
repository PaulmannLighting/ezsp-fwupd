use ashv2::{BaudRate, open};
use clap::Parser;
pub use firmware_updater::FirmwareUpdater;
use log::error;
use serialport::FlowControl;
use silabs2::Fwupd;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

mod firmware_updater;
mod silabs;
mod silabs2;

#[derive(Debug, Parser)]
struct Args {
    tty: String,
    firmware: PathBuf,
}

impl Args {
    pub fn tty(&self) -> &str {
        &self.tty
    }

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

    match open(
        args.tty().to_string(),
        BaudRate::RstCts,
        FlowControl::Software,
    ) {
        Ok(serial_port) => {
            let fwupd = Fwupd::new(serial_port);
            fwupd
                .update_firmware(args.firmware().expect("Invalid data"))
                .await
                .unwrap();
        }
        Err(error) => error!("{error}"),
    }
}
