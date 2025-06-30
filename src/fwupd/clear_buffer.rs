use log::debug;
use std::io::{ErrorKind, Read};

/// Trait to clear the read buffer of a type that implements `Read`.
pub trait ClearBuffer: Read {
    /// Clears the read buffer by reading all available data until EOF.
    ///
    /// Ignores `TimedOut` errors.
    fn clear_buffer(&mut self) -> std::io::Result<()> {
        let mut discard = Vec::new();
        let result = self.read_to_end(&mut discard);
        debug!("Cleared buffer containing: {discard:#04X?}");

        if let Err(error) = result {
            if error.kind() == ErrorKind::TimedOut {
                return Ok(());
            }

            return Err(error);
        }

        Ok(())
    }
}

impl<T> ClearBuffer for T where T: Read {}
