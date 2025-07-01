//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

use std::fs::read;
use std::path::PathBuf;
use std::time::Duration;

use crate::ota_file::OtaFile;
use ashv2::BaudRate;
use clap::{Parser, Subcommand};
use ezsp::uart::Uart;
use ezsp::{Callback, Ezsp};
use fwupd::{FrameCount, Fwupd, Reset, Tty};
use indicatif::{ProgressBar, ProgressStyle};
use le_stream::FromLeStream;
use log::error;
use serialport::FlowControl;
use tokio::sync::mpsc::channel;

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
    Reset {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
        #[clap(long, short, help = "serial port timeout in milliseconds")]
        timeout: Option<u64>,
    },
    Query {
        #[clap(index = 1, help = "the serial port to use for firmware update")]
        tty: String,
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
            let firmware = ota_file.payload().to_vec();
            let progress_bar = ProgressBar::new(firmware.frame_count() as u64);
            progress_bar.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                )
                .unwrap()
                .progress_chars("##-"),
            );
            progress_bar.println(ota_file.to_string());

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
        Action::Query { tty } => {
            let tty = Tty::new(tty, BaudRate::RstCts, FlowControl::Software)
                .open()
                .unwrap_or_else(|err| {
                    error!("Failed to open serial port: {err}");
                    std::process::exit(1);
                });
            let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
            let mut uart = Uart::new(tty, callbacks_tx, 8, 8);

            match uart.init().await {
                Ok(response) => {
                    println!("EZSP version:  {:#04X}", response.protocol_version());
                    println!("Stack type:    {:#04X}", response.stack_type());
                    println!("Stack version: {}", response.stack_version());
                }
                Err(error) => {
                    error!("Failed to get version info: {error}");
                }
            }
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
