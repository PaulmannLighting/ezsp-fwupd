use ashv2::{BaudRate, open};
use serialport::{FlowControl, SerialPort};

/// Represents a TTY serial port for firmware updates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tty {
    path: String,
    baud_rate: BaudRate,
    flow_control: FlowControl,
}

impl Tty {
    /// Create a new TTY configuration.
    #[must_use]
    pub const fn new(path: String, baud_rate: BaudRate, flow_control: FlowControl) -> Self {
        Self {
            path,
            baud_rate,
            flow_control,
        }
    }

    /// Open a new TTY.
    ///
    /// # Errors
    ///
    /// If the serial port cannot be opened, a [`serialport::Error`] is returned.
    pub fn open(&self) -> serialport::Result<impl SerialPort + 'static> {
        open(self.path.clone(), self.baud_rate, self.flow_control)
    }
}
