use std::io::Read;
use std::time::Duration;

use ashv2::HexSlice;
use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};
use log::{debug, error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

use crate::ignore_timeout::IgnoreTimeout;
use clear_buffer::ClearBuffer;
pub use tty::Tty;
use xmodem::Send;

mod clear_buffer;
mod tty;
mod xmodem;

pub trait Fwupd {
    fn fwupd(
        &self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        prepare: bool,
        reset_only: bool,
    ) -> impl Future<Output = std::io::Result<()>>;
}

impl Fwupd for Tty {
    async fn fwupd(
        &self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        prepare: bool,
        reset_only: bool,
    ) -> std::io::Result<()> {
        if reset_only {
            info!("Only resetting the device...");
            return self.open()?.reset();
        }

        if prepare {
            info!("Preparing bootloader...");
            if let Err(error) = self.open()?.prepare_bootloader().await {
                self.open()?.reset()?;
                return Err(error);
            }
        }

        let mut serial_port = self.open()?;
        let original_timeout = serial_port.timeout();

        if let Some(timeout) = timeout {
            serial_port.set_timeout(timeout)?;
        }

        serial_port.clear_buffer()?;

        if let Err(error) = serial_port.transmit(firmware, Some(original_timeout)) {
            serial_port.reset()?;
            return Err(error);
        }

        serial_port.reset()
    }
}

pub trait Transmit {
    /// Transmit the firmware to the device using the XMODEM protocol.
    fn transmit(&mut self, firmware: Vec<u8>, timeout: Option<Duration>) -> std::io::Result<()>;
}

impl<T> Transmit for T
where
    T: SerialPort,
{
    fn transmit(&mut self, firmware: Vec<u8>, timeout: Option<Duration>) -> std::io::Result<()> {
        // TODO: What does this do?
        self.write_all(&[0x0A])?;
        let mut resp1 = [0; 69];
        info!("Waiting for initial response...");
        self.read_exact(&mut resp1).ignore_timeout()?;
        info!("Received initial response: {:#04X}", HexSlice::new(&resp1));

        // TODO: What does this do?
        info!("Sending start signal...");
        self.write_all(&[0x31])?;
        let mut resp2 = [0; 21];
        info!("Waiting for second response...");
        self.read_exact(&mut resp2).ignore_timeout()?;
        info!("Received second response: {:#04X}", HexSlice::new(&resp2));

        if let Some(timeout) = timeout {
            info!("Setting timeout to {timeout:?}");
            self.set_timeout(timeout)?;
        } else {
            info!("Using default timeout");
        }

        info!("Sending firmware...");
        let response = self.send(firmware)?;
        debug!("Firmware sent response: {:#04X}", HexSlice::new(&response));

        Ok(())
    }
}

pub trait Reset {
    /// Reset the device and finalize the firmware update process.
    fn reset(&mut self) -> std::io::Result<()>;
}

impl<T> Reset for T
where
    T: SerialPort,
{
    fn reset(&mut self) -> std::io::Result<()> {
        info!("Resetting serial port...");
        self.flush()?;
        self.write_all(&[0x0A, 0x32])?;
        let mut buffer = Vec::new();
        self.read_to_end(buffer.as_mut()).ignore_timeout()?;
        debug!("Read buffer after reset: {:#04X}", HexSlice::new(&buffer));
        self.flush()
    }
}

pub trait PrepareBootloader {
    /// Prepare the bootloader for firmware updates.
    fn prepare_bootloader(self) -> impl Future<Output = std::io::Result<()>>;
}

impl<T> PrepareBootloader for T
where
    T: SerialPort + 'static,
{
    async fn prepare_bootloader(self) -> std::io::Result<()> {
        let (callbacks_tx, _callbacks_rx) = channel::<Callback>(8);
        let mut uart = Uart::new(self, callbacks_tx, 8, 8);

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
}
