use std::io::{self, ErrorKind, Read, Write};

use indicatif::ProgressBar;
use log::{debug, trace};

use self::send_frame::SendFrame;
use super::frame::{ACK, CAN, EOT, NAK};
use super::frames::Frames;
use crate::{FlashProgress, IgnoreTimeout};

mod send_frame;

const MAX_RETRIES: usize = 10;

/// Trait for sending data using the XMODEM protocol.
pub trait Send: SendFrame {
    /// Sends a file using the XMODEM protocol.
    fn send<T>(&mut self, data: T, progress_bar: Option<&ProgressBar>) -> io::Result<Box<[u8]>>
    where
        T: IntoIterator<Item = u8>,
    {
        debug!("Starting XMODEM file transfer...");

        for (index, frame) in Frames::new(data.into_iter()).enumerate() {
            self.send_frame(index, frame)?;
            progress_bar.increase();
        }

        progress_bar.println("Transfer complete, sending EOT...");
        finish_transmission(self)
    }
}

impl<T> Send for T where T: SendFrame {}

fn finish_transmission<T>(io: &mut T) -> io::Result<Box<[u8]>>
where
    T: Read + Write + ?Sized,
{
    let mut retries = 0;

    loop {
        io.write_all(&[EOT])?;
        io.flush()?;

        let mut response = [0];
        io.read_exact(&mut response)?;
        let [response] = response;
        trace!("Received XMODEM completion response: {response:#04X}");

        match response {
            ACK => break,
            NAK if retries < MAX_RETRIES => {
                retries += 1;
                debug!("EOT was not acknowledged, retrying ({retries}/{MAX_RETRIES})...");
            }
            NAK => {
                return Err(io::Error::other(
                    "maximum retries exceeded while sending EOT",
                ));
            }
            CAN => {
                return Err(io::Error::new(
                    ErrorKind::ConnectionAborted,
                    "XMODEM transfer was cancelled by the bootloader",
                ));
            }
            other => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected XMODEM completion response: {other:#04X}"),
                ));
            }
        }
    }

    let mut response = Vec::new();
    io.read_to_end(&mut response).ignore_timeout()?;
    Ok(response.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Write};

    use super::finish_transmission;
    use crate::xmodem::frame::{ACK, CAN, EOT, NAK};

    const COMPLETION_MESSAGE: &[u8] = b"\r\nSerial upload complete\r\n";

    #[derive(Debug)]
    struct MockIo {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl MockIo {
        fn new(input: impl Into<Vec<u8>>) -> Self {
            Self {
                input: Cursor::new(input.into()),
                output: Vec::new(),
            }
        }
    }

    impl Read for MockIo {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            self.input.read(buffer)
        }
    }

    impl Write for MockIo {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.output.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn requires_ack_for_eot() {
        let input: Vec<u8> = [ACK]
            .into_iter()
            .chain(COMPLETION_MESSAGE.iter().copied())
            .collect();
        let mut io = MockIo::new(input);

        let response = finish_transmission(&mut io).expect("ACK should complete the transfer");

        assert_eq!(io.output, [EOT]);
        assert_eq!(&*response, COMPLETION_MESSAGE);
    }

    #[test]
    fn retries_eot_after_nak() {
        let mut io = MockIo::new([NAK, ACK]);

        finish_transmission(&mut io).expect("ACK after NAK should complete the transfer");

        assert_eq!(io.output, [EOT, EOT]);
    }

    #[test]
    fn reports_cancelled_transfer() {
        let mut io = MockIo::new([CAN]);

        let error = finish_transmission(&mut io).expect_err("CAN should abort the transfer");

        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionAborted);
    }
}
