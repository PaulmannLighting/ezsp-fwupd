use std::io::{ErrorKind, Read, Write};

use ashv2::HexSlice;
use log::{debug, error, trace};

use crate::ignore_timeout::IgnoreTimeout;
use crate::xmodem::frame::{ACK, Frame, NAK};
use crate::xmodem::send::MAX_RETRIES;

/// Sealed trait for sending XMODEM frames.
pub trait SendFrame: Read + Write {
    fn send_frame(&mut self, index: usize, frame: Frame) -> std::io::Result<()> {
        debug!("Sending frame #{index}...");

        let mut ctr: usize = 0;
        let bytes = frame.into_bytes();
        trace!("Sending frame #{index}: {:#04X}", HexSlice::new(&bytes));

        loop {
            match self.try_send_frame(&bytes) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    if ctr >= MAX_RETRIES {
                        error!("max retries exceeded for frame #{index}");
                        return Err(error);
                    }

                    debug!("Attempt {ctr} failed: {error}, retrying...");
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
        trace!("Received {response:#02X?}");
        let [byte] = response;
        let mut excess = Vec::new();
        self.read_to_end(&mut excess).ignore_timeout()?;
        trace!(
            "Received {} excess bytes: {:#04X}",
            excess.len(),
            HexSlice::new(&excess)
        );

        match byte {
            ACK => Ok(()),
            NAK => Err(std::io::Error::other("NAK received, retransmitting frame")),
            other => Err(std::io::Error::other(format!(
                "Received unexpected response: {other}"
            ))),
        }
    }
}

impl<T> SendFrame for T where T: Read + Write {}
