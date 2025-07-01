use std::time::Duration;

use ashv2::HexSlice;
use indicatif::ProgressBar;
use log::debug;
use serialport::SerialPort;

use crate::fwupd::xmodem::Send;
use crate::ignore_timeout::IgnoreTimeout;

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
        self.write_all(INIT_STAGE1)?;
        let mut resp1 = [0; INIT_STAGE1_RESPONSE_SIZE];
        debug!("Waiting for initial response...");
        self.read_exact(&mut resp1).ignore_timeout()?;
        debug!("Received initial response: {:#04X}", HexSlice::new(&resp1));
        Ok(())
    }

    fn init_stage2(&mut self) -> std::io::Result<()> {
        debug!("Sending start signal...");
        self.write_all(INIT_STAGE2)?;
        let mut resp2 = [0; INIT_STAGE2_RESPONSE_SIZE];
        debug!("Waiting for second response...");
        self.read_exact(&mut resp2).ignore_timeout()?;
        debug!("Received second response: {:#04X}", HexSlice::new(&resp2));
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

        debug!("Sending firmware...");
        let response = self.send(firmware, progress_bar)?;
        debug!("Firmware sent response: {:#04X}", HexSlice::new(&response));

        Ok(())
    }
}
