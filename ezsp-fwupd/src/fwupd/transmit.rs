use std::io;
use std::time::Duration;

use indicatif::ProgressBar;
use log::debug;
use serialport::SerialPort;

use crate::FlashProgress;
use crate::xmodem::Send;

/// Trait for transmitting firmware to a device using the XMODEM protocol.
pub trait Transmit {
    /// Transmit the firmware to the device using the XMODEM protocol.
    fn transmit<F>(
        &mut self,
        firmware: F,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> io::Result<()>
    where
        F: IntoIterator<Item = u8>;
}

impl<T> Transmit for T
where
    T: SerialPort,
{
    fn transmit<F>(
        &mut self,
        firmware: F,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> io::Result<()>
    where
        F: IntoIterator<Item = u8>,
    {
        if let Some(timeout) = timeout {
            debug!("Setting timeout to {timeout:?}");
            self.set_timeout(timeout)?;
        } else {
            debug!("Using default timeout");
        }

        progress_bar.set_message("Flashing firmware...");
        let response = self.send(firmware, progress_bar)?;
        debug!("Firmware sent response: {response:#04X?}");

        Ok(())
    }
}
