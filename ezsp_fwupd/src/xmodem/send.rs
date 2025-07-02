use indicatif::ProgressBar;
use log::debug;

use super::frame::EOT;
use super::frames::Frames;
use crate::FlashProgress;
use crate::IgnoreTimeout;
use send_frame::SendFrame;

mod send_frame;

const MAX_RETRIES: usize = 10;

/// Trait for sending data using the XMODEM protocol.
pub trait Send: SendFrame {
    /// Sends a file using the XMODEM protocol.
    fn send<T>(&mut self, data: T, progress_bar: Option<&ProgressBar>) -> std::io::Result<Box<[u8]>>
    where
        T: IntoIterator<Item = u8>,
    {
        debug!("Starting XMODEM file transfer...");

        for (index, frame) in Frames::new(data.into_iter()).enumerate() {
            self.send_frame(index, frame)?;
            progress_bar.increase();
        }

        progress_bar.println("Transfer complete, sending EOT...");
        self.write_all(&[EOT])?;
        self.flush()?;
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer).ignore_timeout()?;
        Ok(buffer.into_boxed_slice())
    }
}

impl<T> Send for T where T: SendFrame {}
