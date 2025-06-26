use std::io::{ErrorKind, Read};

/// Trait to clear the read buffer of a type that implements `Read`.
pub trait ClearBuffer: Read {
    /// Clears the read buffer by reading all available data until EOF.
    ///
    /// Ignores `TimedOut` errors.
    fn clear_buffer(&mut self) -> std::io::Result<()> {
        let mut discard = Vec::new();

        if let Err(error) = self.read_to_end(&mut discard) {
            if error.kind() == ErrorKind::TimedOut {
                return Ok(());
            }

            return Err(error);
        }

        Ok(())
    }
}

impl<T> ClearBuffer for T where T: Read {}
