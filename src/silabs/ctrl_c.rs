use std::io::{Error, ErrorKind, Result, Write};
use std::process::Child;

const CTRL_C: u8 = 0x03; // ASCII Control-C

pub trait CtrlC: Sized {
    /// Sends Ctrl-C to the STDIN.
    fn ctrl_c(self) -> Result<Self>;
}

impl CtrlC for Child {
    fn ctrl_c(mut self) -> Result<Self> {
        let Some(ref mut stdin) = self.stdin else {
            self.kill()?;
            return Err(Error::new(ErrorKind::Other, "Failed to open STDIN"))?;
        };

        stdin.write_all(&[CTRL_C])?;
        Ok(self)
    }
}
