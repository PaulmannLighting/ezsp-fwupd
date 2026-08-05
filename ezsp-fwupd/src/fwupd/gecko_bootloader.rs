use std::io::{self, ErrorKind, Read, Write};
use std::time::Duration;

use log::{debug, trace};
use serialport::SerialPort;

use crate::ignore_timeout::IgnoreTimeout;

const BOOTLOADER_MENU_PROMPT: &[u8] = b"BL >";
const BOOTLOADER_WAKE_COMMAND: &[u8] = b"\r";
const MAX_BOOTLOADER_RESPONSE_SIZE: usize = 1024;
const RUN_APPLICATION_COMMAND: &[u8] = b"2";
const START_UPLOAD_COMMAND: &[u8] = b"1";
const XMODEM_CRC_REQUEST: &[u8] = b"C";

/// Commands supported by the interactive Silicon Labs Gecko standalone bootloader.
pub trait GeckoBootloader {
    /// Wakes the standalone bootloader and waits for its menu prompt.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the command cannot be written or the menu prompt cannot be read.
    fn wake_bootloader_menu(&mut self) -> io::Result<()>;

    /// Selects the GBL upload menu option and waits for the XMODEM-CRC request.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the command cannot be written or the XMODEM request cannot be
    /// read.
    fn start_xmodem_upload(&mut self) -> io::Result<()>;

    /// Selects the bootloader menu command that runs the application image.
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
        let response =
            write_command_and_read_until(self, BOOTLOADER_WAKE_COMMAND, BOOTLOADER_MENU_PROMPT)?;
        trace!("Received bootloader menu: {response:#04X?}");
        Ok(())
    }

    fn start_xmodem_upload(&mut self) -> io::Result<()> {
        debug!("Selecting the bootloader GBL upload command...");
        let response =
            write_command_and_read_until(self, START_UPLOAD_COMMAND, XMODEM_CRC_REQUEST)?;
        trace!("Received upload response: {response:#04X?}");
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

fn write_command_and_read_until<T>(
    io: &mut T,
    command: &[u8],
    terminator: &[u8],
) -> io::Result<Vec<u8>>
where
    T: Read + Write + ?Sized,
{
    io.write_all(command)?;
    io.flush()?;
    read_until(io, terminator)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Write};

    use super::{
        BOOTLOADER_MENU_PROMPT, BOOTLOADER_WAKE_COMMAND, MAX_BOOTLOADER_RESPONSE_SIZE,
        START_UPLOAD_COMMAND, XMODEM_CRC_REQUEST, write_command_and_read_until,
    };

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

        write_command_and_read_until(&mut io, BOOTLOADER_WAKE_COMMAND, BOOTLOADER_MENU_PROMPT)
            .expect("bootloader menu should be detected");

        assert_eq!(io.output, BOOTLOADER_WAKE_COMMAND);
    }

    #[test]
    fn starts_upload_on_first_crc_request() {
        let response = b"\r\nbegin upload\r\nCadditional bytes are not consumed";
        let mut io = MockIo::new(response);

        write_command_and_read_until(&mut io, START_UPLOAD_COMMAND, XMODEM_CRC_REQUEST)
            .expect("XMODEM-CRC request should be detected");

        assert_eq!(io.output, START_UPLOAD_COMMAND);
        assert_eq!(io.input.position(), 17);
    }

    #[test]
    fn rejects_unbounded_bootloader_output() {
        let response = vec![b'A'; MAX_BOOTLOADER_RESPONSE_SIZE];
        let mut io = MockIo::new(response);

        let error =
            write_command_and_read_until(&mut io, BOOTLOADER_WAKE_COMMAND, BOOTLOADER_MENU_PROMPT)
                .expect_err("missing prompt should fail");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
