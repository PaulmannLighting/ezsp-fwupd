use std::io::{self, ErrorKind, Read, Write};

use indicatif::ProgressBar;
use log::debug;
use serialport::SerialPort;
use xmodem::{BlockLength, Error as XmodemError, Xmodem};

use crate::FlashProgress;

const XMODEM_MAX_ERRORS: u32 = 10;

#[derive(Debug)]
struct FirmwareReader<'a, T> {
    bytes: T,
    progress_bar: Option<&'a ProgressBar>,
}

impl<'a, T> FirmwareReader<'a, T> {
    const fn new(bytes: T, progress_bar: Option<&'a ProgressBar>) -> Self {
        Self {
            bytes,
            progress_bar,
        }
    }
}

impl<T> Read for FirmwareReader<'_, T>
where
    T: Iterator<Item = u8>,
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut count = 0;

        for destination in buffer {
            let Some(byte) = self.bytes.next() else {
                break;
            };

            *destination = byte;
            count += 1;
        }

        if count > 0 {
            self.progress_bar.increase();
        }

        Ok(count)
    }
}

/// Trait for transmitting firmware to a device using the XMODEM protocol.
pub trait Transmit {
    /// Transmit the firmware to the device using the XMODEM protocol.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the firmware cannot be read or the XMODEM transfer fails.
    fn transmit<F>(&mut self, firmware: F, progress_bar: Option<&ProgressBar>) -> io::Result<()>
    where
        F: IntoIterator<Item = u8>;
}

impl<T> Transmit for T
where
    T: SerialPort,
{
    fn transmit<F>(&mut self, firmware: F, progress_bar: Option<&ProgressBar>) -> io::Result<()>
    where
        F: IntoIterator<Item = u8>,
    {
        progress_bar.set_message("Flashing firmware...");
        let bytes_sent = send_xmodem(self, firmware.into_iter(), progress_bar)?;
        debug!("XMODEM transferred {bytes_sent} firmware bytes");

        Ok(())
    }
}

fn map_xmodem_error(error: XmodemError) -> io::Error {
    match error {
        XmodemError::Io(error) => error,
        XmodemError::ExhaustedRetries => io::Error::other("XMODEM exhausted its retry limit"),
        XmodemError::Canceled => io::Error::new(
            ErrorKind::ConnectionAborted,
            "XMODEM transfer was cancelled by the bootloader",
        ),
        XmodemError::Invalid => {
            io::Error::new(ErrorKind::InvalidData, "XMODEM received invalid data")
        }
        XmodemError::SequenceMismatch => io::Error::new(
            ErrorKind::InvalidData,
            "XMODEM received a mismatched block sequence",
        ),
        XmodemError::Checksum => io::Error::new(
            ErrorKind::InvalidData,
            "XMODEM received an invalid checksum",
        ),
    }
}

fn send_xmodem<D, T>(
    device: &mut D,
    firmware: T,
    progress_bar: Option<&ProgressBar>,
) -> io::Result<usize>
where
    D: Read + Write,
    T: Iterator<Item = u8>,
{
    let mut firmware = FirmwareReader::new(firmware, progress_bar);
    let mut xmodem = Xmodem::new();
    xmodem.block_length = BlockLength::Standard;
    xmodem.max_errors = XMODEM_MAX_ERRORS;
    xmodem.send(device, &mut firmware).map_err(map_xmodem_error)
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Read, Write};

    use indicatif::ProgressBar;

    use super::send_xmodem;

    const ACK: u8 = 0x06;
    const CRC_REQUEST: u8 = b'C';
    const DATA: u8 = 0xA5;
    const EOT: u8 = 0x04;
    const STANDARD_FRAME_SIZE: usize = 133;

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
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.input.read(buffer)
        }
    }

    impl Write for MockIo {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.output.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn sends_firmware_with_xmodem_crate() {
        let mut device = MockIo::new([CRC_REQUEST, ACK, ACK]);
        let progress_bar = ProgressBar::hidden();

        let bytes_sent = send_xmodem(&mut device, [DATA].into_iter(), Some(&progress_bar))
            .expect("XMODEM transfer should complete");

        assert_eq!(bytes_sent, 1);
        assert_eq!(device.output.len(), STANDARD_FRAME_SIZE + 1);
        assert_eq!(device.output.last(), Some(&EOT));
        assert_eq!(progress_bar.position(), 1);
    }
}
