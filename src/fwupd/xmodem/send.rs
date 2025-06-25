use std::io::{ErrorKind, Read, Write};

use log::error;

use super::frame::{ACK, EOT, Frame, NAK};
use super::frames::Frames;

const MAX_RETRIES: usize = 10;

pub trait Send: Read + Write {
    /// Sends a file using the XMODEM protocol.
    fn send<T>(&mut self, data: T) -> std::io::Result<Box<[u8]>>
    where
        T: IntoIterator<Item = u8>,
    {
        for frame in Frames::new(data.into_iter()) {
            self.send_frame(frame)?;
        }

        self.write_all(&[EOT])?;
        self.flush()?;
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;
        self.write_all(&[0x0A, 0x32])?;
        Ok(buffer.into_boxed_slice())
    }

    fn send_frame(&mut self, frame: Frame) -> std::io::Result<()> {
        let mut ctr: usize = 0;
        let bytes = frame.into_bytes();

        loop {
            match self.try_send_frame(&bytes) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    if ctr >= MAX_RETRIES {
                        return Err(std::io::Error::new(
                            ErrorKind::TimedOut,
                            "Maximum retries exceeded",
                        ));
                    } else {
                        error!("Attempt {ctr} failed: {error}, retrying...");
                    }
                }
            }

            ctr += 1;
        }
    }

    fn try_send_frame(&mut self, frame: &[u8]) -> std::io::Result<()> {
        self.write_all(frame)?;
        self.flush()?;

        let mut response = [0];
        self.read_exact(&mut response)?;
        let [byte] = response;

        match byte {
            ACK => Ok(()),
            NAK => Err(std::io::Error::new(
                ErrorKind::Other,
                "NAK received, retransmitting frame",
            )),
            other => Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Received unexpected response: {other}"),
            )),
        }
    }
}

impl<T> Send for T where T: Read + Write {}
