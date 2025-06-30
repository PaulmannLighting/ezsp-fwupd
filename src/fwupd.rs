use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};
use log::{error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

use clear_buffer::ClearBuffer;
pub use tty::Tty;
use xmodem::Send;

mod clear_buffer;
mod tty;
mod xmodem;

pub async fn update_firmware(tty: Tty, firmware: Vec<u8>) -> std::io::Result<Box<[u8]>> {
    prepare_bootloader(tty.open()?).await?;
    let mut serial_port = tty.open()?;
    serial_port.clear_buffer()?;
    serial_port.send(firmware)
}

async fn prepare_bootloader(serial_port: impl SerialPort + 'static) -> std::io::Result<()> {
    let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
    let mut uart = Uart::new(serial_port, callbacks_tx, 8, 8);

    match uart
        .get_standalone_bootloader_version_plat_micro_phy()
        .await
    {
        Ok(info) => {
            info!("Bootloader info: {info:#?}");
        }
        Err(error) => {
            error!("Failed to get bootloader info: {error}");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
        }
    }

    uart.launch_standalone_bootloader(0x00)
        .await
        .unwrap_or_else(|error| {
            error!("Failed to launch standalone bootloader: {error}");
        });
    Ok(())
}
