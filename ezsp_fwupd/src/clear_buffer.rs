use std::io::Read;

use log::debug;

use crate::ignore_timeout::IgnoreTimeout;

/// Trait to clear the read buffer of a type that implements `Read`.
pub trait ClearBuffer: Read {
    /// Clears the read buffer by reading all available data until EOF.
    ///
    /// Ignores `TimedOut` errors.
    fn clear_buffer(&mut self) -> std::io::Result<()> {
        let mut discard = Vec::new();
        self.read_to_end(&mut discard).ignore_timeout()?;
        debug!("Cleared buffer containing: {discard:#04X?}");
        Ok(())
    }
}

impl<T> ClearBuffer for T where T: Read {}
