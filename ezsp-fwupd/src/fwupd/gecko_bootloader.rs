use std::io::{self, ErrorKind, Read, Write};
use std::time::Duration;

use log::{debug, trace};
use serialport::SerialPort;

use super::XMODEM_CRC_REQUEST;
use crate::ignore_timeout::IgnoreTimeout;

const BOOTLOADER_MENU_PROMPT: &[u8] = b"BL >";
const BOOTLOADER_WAKE_COMMAND: &[u8] = b"\r";
const MAX_BOOTLOADER_RESPONSE_SIZE: usize = 1024;
const RUN_APPLICATION_COMMAND: &[u8] = b"2";
const START_UPLOAD_COMMAND: &[u8] = b"1";

/// Drives the ASCII menu of a Silicon Labs Gecko standalone UART bootloader.
///
/// This trait covers only the bootloader console protocol. XMODEM framing and acknowledgement
/// handling belong to the `xmodem` crate used by [`crate::Fwupd`]. It is implemented for every
/// native [`SerialPort`].
pub trait GeckoBootloader {
    /// Sends a carriage return and waits for the `BL >` menu prompt.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the command cannot be written or the menu prompt cannot be read.
    fn wake_bootloader_menu(&mut self) -> io::Result<()>;

    /// Sends menu option `1` and waits for the bootloader's XMODEM-CRC request.
    ///
    /// The bootloader may emit an ASCII status line before its initial `C`. This method consumes
    /// that bounded console response through the `C`; the firmware transmitter replays the
    /// already-observed request to the XMODEM implementation when the transfer starts.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the command cannot be written or the XMODEM request cannot be
    /// read.
    fn start_xmodem_upload(&mut self) -> io::Result<()>;

    /// Waits for the standalone bootloader to display its `BL >` menu prompt.
    ///
    /// This is used after XMODEM has acknowledged the end of a transfer and the bootloader is
    /// preparing to accept another menu command.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the menu prompt cannot be read.
    fn wait_for_bootloader_menu(&mut self) -> io::Result<()>;

    /// Sends menu option `2` to reset into the application image.
    ///
    /// Any console output produced after the command is read until the configured serial timeout.
    /// A timeout while collecting that optional output is treated as normal completion.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the command or its response cannot be transmitted.
    fn run_application(&mut self, timeout: Option<Duration>) -> io::Result<()>;
}

impl<T> GeckoBootloader for T
where
    T: SerialPort,
{
    fn wake_bootloader_menu(&mut self) -> io::Result<()> {
        debug!("Waking the standalone bootloader menu...");
        write_command(self, BOOTLOADER_WAKE_COMMAND)?;
        let response = read_until(self, BOOTLOADER_MENU_PROMPT)?;
        trace!("Received bootloader menu: {response:#04X?}");
        Ok(())
    }

    fn start_xmodem_upload(&mut self) -> io::Result<()> {
        debug!("Selecting the bootloader GBL upload command...");
        write_command(self, START_UPLOAD_COMMAND)?;
        let response = read_until(self, &[XMODEM_CRC_REQUEST])?;
        trace!("Received upload response: {response:#04X?}");
        Ok(())
    }

    fn wait_for_bootloader_menu(&mut self) -> io::Result<()> {
        debug!("Waiting for the bootloader menu prompt...");
        let response = read_until(self, BOOTLOADER_MENU_PROMPT)?;
        trace!("Received bootloader menu: {response:#04X?}");
        Ok(())
    }

    fn run_application(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        let original_timeout = self.timeout();

        if let Some(timeout) = timeout {
            debug!("Setting run-command timeout to {timeout:?}");
            self.set_timeout(timeout)?;
        } else {
            debug!("Using default timeout for the run command");
        }

        debug!("Running the uploaded application...");
        self.flush()?;
        self.write_all(RUN_APPLICATION_COMMAND)?;
        self.flush()?;

        let mut response = Vec::new();
        self.read_to_end(&mut response).ignore_timeout()?;
        debug!("Read buffer after the run command: {response:#04X?}");

        self.set_timeout(original_timeout)?;
        Ok(())
    }
}

fn read_until<T>(reader: &mut T, terminator: &[u8]) -> io::Result<Vec<u8>>
where
    T: Read + ?Sized,
{
    let mut response = Vec::new();

    while response.len() < MAX_BOOTLOADER_RESPONSE_SIZE {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;
        response.push(byte[0]);

        if response.ends_with(terminator) {
            return Ok(response);
        }
    }

    Err(io::Error::new(
        ErrorKind::InvalidData,
        "bootloader response exceeded the maximum size",
    ))
}

fn write_command<T>(io: &mut T, command: &[u8]) -> io::Result<()>
where
    T: Write + ?Sized,
{
    io.write_all(command)?;
    io.flush()
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Write};

    use super::{
        BOOTLOADER_MENU_PROMPT, BOOTLOADER_WAKE_COMMAND, MAX_BOOTLOADER_RESPONSE_SIZE,
        START_UPLOAD_COMMAND, XMODEM_CRC_REQUEST, read_until, write_command,
    };

    const UPLOAD_START_RESPONSE: &[u8] = b"\r\nbegin upload\r\nC";

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
    fn wakes_menu_until_prompt() {
        let response = b"\r\nGecko Bootloader v2.01.02\r\n1. upload gbl\r\n2. run\r\nBL >";
        let mut io = MockIo::new(response);

        write_command(&mut io, BOOTLOADER_WAKE_COMMAND).expect("wake command should be written");
        read_until(&mut io, BOOTLOADER_MENU_PROMPT).expect("bootloader menu should be detected");

        assert_eq!(io.output, BOOTLOADER_WAKE_COMMAND);
    }

    #[test]
    fn starts_upload_on_first_crc_request() {
        let response = UPLOAD_START_RESPONSE
            .iter()
            .copied()
            .chain(b"additional bytes are not consumed".iter().copied())
            .collect::<Vec<_>>();
        let mut io = MockIo::new(response);

        write_command(&mut io, START_UPLOAD_COMMAND).expect("upload command should be written");
        let response = read_until(&mut io, &[XMODEM_CRC_REQUEST])
            .expect("XMODEM-CRC request should be detected");

        assert_eq!(io.output, START_UPLOAD_COMMAND);
        assert_eq!(response, UPLOAD_START_RESPONSE);
        assert_eq!(
            io.input.position(),
            u64::try_from(UPLOAD_START_RESPONSE.len()).expect("test response length should fit")
        );
    }

    #[test]
    fn rejects_unbounded_bootloader_output() {
        let response = vec![b'A'; MAX_BOOTLOADER_RESPONSE_SIZE];
        let mut io = MockIo::new(response);

        let error =
            read_until(&mut io, BOOTLOADER_MENU_PROMPT).expect_err("missing prompt should fail");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
