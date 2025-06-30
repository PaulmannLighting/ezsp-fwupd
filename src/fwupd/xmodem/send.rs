use super::frame::{ACK, EOT, Frame, NAK};
use super::frames::Frames;
use crate::ignore_timeout::IgnoreTimeout;
use ashv2::HexSlice;
use log::{error, info};
use std::io::{ErrorKind, Read, Write};

const MAX_RETRIES: usize = 10;

pub trait Send: Read + Write {
    /// Sends a file using the XMODEM protocol.
    fn send<T>(&mut self, data: T) -> std::io::Result<Box<[u8]>>
    where
        T: IntoIterator<Item = u8>,
    {
        info!("Starting XMODEM file transfer...");

        // TODO: Does this belong here?
        self.write_all(&[0x0A])?;
        let mut resp1 = [0; 69];
        info!("Waiting for initial response...");
        self.read_exact(&mut resp1).ignore_timeout()?;
        info!("Received initial response: {:#04X}", HexSlice::new(&resp1));

        // TODO: Does this belong here?
        info!("Sending start signal...");
        self.write_all(&[0x31])?;
        let mut resp2 = [0; 21];
        info!("Waiting for second response...");
        self.read_exact(&mut resp2).ignore_timeout()?;
        info!("Received second response: {:#04X}", HexSlice::new(&resp2));

        for (index, frame) in Frames::new(data.into_iter()).enumerate() {
            self.send_frame(index, frame)?;
        }

        self.write_all(&[EOT])?;
        self.flush()?;
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;
        self.write_all(&[0x0A, 0x32])?;
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
        let excess = Vec::new();
        let amount = self.read(&mut response)?;
        info!(
            "Received {amount} excess bytes: {:#04X}",
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
