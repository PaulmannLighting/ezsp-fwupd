use std::io::{Error, ErrorKind, Result, Write};
use std::process::{Command, Output};

const CTRL_C: u8 = 0x03; // ASCII Control-C

pub trait CtrlCAndWaitWithOutput {
    /// Executes a command, sends Ctrl-C to its STDIN and waits for it to finish, and captures its output.
    fn ctrl_c_and_wait_with_output(&mut self) -> Result<Output>;
}

impl CtrlCAndWaitWithOutput for Command {
    fn ctrl_c_and_wait_with_output(&mut self) -> Result<Output> {
        let mut child = self.spawn()?;

        let Some(stdin) = &mut child.stdin else {
            child.kill()?;
            return Err(Error::new(ErrorKind::Other, " Failed to open stdin"))?;
        };

        stdin.write_all(&[CTRL_C])?;
        child.wait_with_output()
    }
}
