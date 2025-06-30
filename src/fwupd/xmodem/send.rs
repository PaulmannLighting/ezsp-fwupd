use std::io::{ErrorKind, Read, Write};

use ashv2::HexSlice;
use indicatif::ProgressBar;
use log::{error, info};

use super::frame::{ACK, EOT, Frame, NAK};
use super::frames::Frames;
use crate::ignore_timeout::IgnoreTimeout;

const MAX_RETRIES: usize = 10;

pub trait Send: Read + Write {
    /// Sends a file using the XMODEM protocol.
    fn send<T>(&mut self, data: T, progress_bar: Option<&ProgressBar>) -> std::io::Result<Box<[u8]>>
    where
        T: IntoIterator<Item = u8>,
    {
        info!("Starting XMODEM file transfer...");

        for (index, frame) in Frames::new(data.into_iter()).enumerate() {
            self.send_frame(index, frame)?;

            if let Some(ref progress_bar) = progress_bar {
                progress_bar.inc(1);
            }
        }

        self.write_all(&[EOT])?;
        self.flush()?;
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer).ignore_timeout()?;
        Ok(buffer.into_boxed_slice())
    }

    fn send_frame(&mut self, index: usize, frame: Frame) -> std::io::Result<()> {
        info!("Sending frame #{index}...");

        let mut ctr: usize = 0;
        let bytes = frame.into_bytes();
        info!("Sending frame #{index}: {:#04X}", HexSlice::new(&bytes));

        loop {
            match self.try_send_frame(&bytes) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    if ctr >= MAX_RETRIES {
                        return Err(std::io::Error::new(
                            ErrorKind::TimedOut,
                            "Maximum retries exceeded",
                        ));
                    }

                    error!("Attempt {ctr} failed: {error}, retrying...");
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
        info!("Received {response:#02X?}");
        let [byte] = response;
        let mut excess = Vec::new();
        self.read_to_end(&mut excess).ignore_timeout()?;
        info!(
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

impl<T> Send for T where T: Read + Write {}
