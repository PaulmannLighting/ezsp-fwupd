use std::ffi::OsStr;
use std::io::{ErrorKind, Write};
use std::process::{Command, Output, Stdio};

const BAUD_RATE: u32 = 115200;
const CTRL_C: u8 = 0x03; // ASCII Control-C
const Z3GATEWAY_HOST: &str = "/usr/bin/Z3GatewayHost";

/// Represents a host for the Z3 Gateway, which is used to communicate with Silicon Labs devices.
#[derive(Debug)]
pub struct Z3GatewayHost {
    command: Command,
}

impl Z3GatewayHost {
    /// Creates a new instance of `Z3GatewayHost`.
    pub fn new(binary: impl AsRef<OsStr>) -> Self {
        let mut command = Command::new(binary);
        command
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());
        Self { command }
    }

    /// Read out the status of the device connected to the specified TTY.
    pub fn status(mut self, tty: impl AsRef<OsStr>) -> std::io::Result<Output> {
        self.arg("-n")
            .arg(1.to_string())
            .arg("-b")
            .arg(BAUD_RATE.to_string())
            .arg("-f")
            .arg("x")
            .arg("-p")
            .arg(tty)
            .run()
    }

    /// Run the command and return the output.
    fn run(&mut self) -> std::io::Result<Output> {
        let mut child = self.command.spawn()?;

        let Some(stdin) = &mut child.stdin else {
            child.kill()?;
            return Err(std::io::Error::new(
                ErrorKind::Other,
                " Failed to open stdin",
            ))?;
        };

        stdin.write_all(&[CTRL_C])?;
        child.wait_with_output()
    }

    /// Add an argument to the command.
    fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.command.arg(arg);
        self
    }
}

impl Default for Z3GatewayHost {
    fn default() -> Self {
        Self::new(Z3GATEWAY_HOST)
    }
}
