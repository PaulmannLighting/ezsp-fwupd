use std::time::Duration;

use crate::FlashProgress;
use crate::xmodem::Send;
use ashv2::HexSlice;
use indicatif::ProgressBar;
use log::{debug, trace};
use serialport::SerialPort;

const INIT_STAGE1: &[u8] = &[0x0A];
const INIT_STAGE1_RESPONSE_SIZE: usize = 69;
const INIT_STAGE2: &[u8] = &[0x31];
const INIT_STAGE2_RESPONSE_SIZE: usize = 21;

/// Trait for transmitting firmware to a device using the XMODEM protocol.
pub trait Transmit {
    /// Initialize the first stage of the firmware update process.
    fn init_stage1(&mut self) -> std::io::Result<()>;

    /// Initialize the second stage of the firmware update process.
    fn init_stage2(&mut self) -> std::io::Result<()>;

    /// Transmit the firmware to the device using the XMODEM protocol.
    fn transmit(
        &mut self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> std::io::Result<()>;
}

impl<T> Transmit for T
where
    T: SerialPort,
{
    fn init_stage1(&mut self) -> std::io::Result<()> {
        debug!("Firmware update stage 1 initialization...");
        self.write_all(INIT_STAGE1)?;
        let mut response = [0; INIT_STAGE1_RESPONSE_SIZE];
        debug!("Waiting for response...");
        self.read_exact(&mut response)?;
        trace!("Received response: {:#04X}", HexSlice::new(&response));
        Ok(())
    }

    fn init_stage2(&mut self) -> std::io::Result<()> {
        debug!("Firmware update stage 2 initialization...");
        self.write_all(INIT_STAGE2)?;
        let mut response = [0; INIT_STAGE2_RESPONSE_SIZE];
        debug!("Waiting for response...");
        self.read_exact(&mut response)?;
        trace!("Received response: {:#04X}", HexSlice::new(&response));
        Ok(())
    }

    fn transmit(
        &mut self,
        firmware: Vec<u8>,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> std::io::Result<()> {
        if let Some(timeout) = timeout {
            debug!("Setting timeout to {timeout:?}");
            self.set_timeout(timeout)?;
        } else {
            debug!("Using default timeout");
        }

        progress_bar.set_message("Flashing firmware...");
        let response = self.send(firmware, progress_bar)?;
        debug!("Firmware sent response: {:#04X}", HexSlice::new(&response));

        Ok(())
    }
}
