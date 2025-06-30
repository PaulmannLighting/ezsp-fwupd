use std::io::{Read, Write};

use ashv2::HexSlice;
use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};
use log::{error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

use crate::ignore_timeout::IgnoreTimeout;
use clear_buffer::ClearBuffer;
pub use tty::Tty;
use xmodem::Send;

mod clear_buffer;
mod tty;
mod xmodem;

pub async fn update_firmware(tty: Tty, firmware: Vec<u8>) -> std::io::Result<Box<[u8]>> {
    info!("Preparing bootloader...");
    prepare_bootloader(tty.open()?).await?;
    let mut serial_port = tty.open()?;
    info!("Clearing buffer...");
    serial_port.clear_buffer()?;
    serial_port.set_timeout(std::time::Duration::from_millis(1000))?;

    // TODO: What does this do?
    serial_port.write_all(&[0x0A])?;
    let mut resp1 = [0; 69];
    info!("Waiting for initial response...");
    serial_port.read_exact(&mut resp1).ignore_timeout()?;
    info!("Received initial response: {:#04X}", HexSlice::new(&resp1));

    // TODO: What does this do?
    info!("Sending start signal...");
    serial_port.write_all(&[0x31])?;
    let mut resp2 = [0; 21];
    info!("Waiting for second response...");
    serial_port.read_exact(&mut resp2).ignore_timeout()?;
    info!("Received second response: {:#04X}", HexSlice::new(&resp2));

    info!("Sending firmware...");
    serial_port.send(firmware)
}

async fn prepare_bootloader(serial_port: impl SerialPort + 'static) -> std::io::Result<()> {
    let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
    let mut uart = Uart::new(serial_port, callbacks_tx, 8, 8);

    info!("Getting bootloader version...");
    match uart
        .get_standalone_bootloader_version_plat_micro_phy()
        .await
    {
        Ok(info) => {
            info!("Bootloader info: {info:#?}");
        }
        Err(error) => {
            error!("Failed to get bootloader info: {error}");
            return Err(std::io::Error::other(error));
        }
    }

    info!("Launching standalone bootloader...");
    uart.launch_standalone_bootloader(0x00)
        .await
        .unwrap_or_else(|error| {
            error!("Failed to launch standalone bootloader: {error}");
        });
    Ok(())
}
