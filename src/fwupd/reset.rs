use ashv2::HexSlice;
use log::{debug, info};
use serialport::SerialPort;
use std::time::Duration;

use crate::ignore_timeout::IgnoreTimeout;

/// Trait for resetting a device after a firmware update.
pub trait Reset {
    /// Reset the device and finalize the firmware update process.
    fn reset(&mut self, timeout: Option<Duration>) -> std::io::Result<()>;
}

impl<T> Reset for T
where
    T: SerialPort,
{
    fn reset(&mut self, timeout: Option<Duration>) -> std::io::Result<()> {
        let original_timeout = self.timeout();

        if let Some(timeout) = timeout {
            info!("Setting reset timeout to {timeout:?}");
            self.set_timeout(timeout)?;
        } else {
            info!("Using default timeout for reset");
        }

        info!("Resetting serial port...");
        self.flush()?;
        self.write_all(&[0x0A, 0x32])?;
        self.flush()?;

        let mut buffer = Vec::new();
        self.read_to_end(buffer.as_mut()).ignore_timeout()?;
        debug!("Read buffer after reset: {:#04X}", HexSlice::new(&buffer));

        self.set_timeout(original_timeout)?;
        Ok(())
    }
}
