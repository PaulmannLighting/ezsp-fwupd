use ashv2::{BaudRate, open};
use serialport::{FlowControl, SerialPort};

#[derive(Debug)]
pub struct Tty {
    path: String,
    baud_rate: BaudRate,
    flow_control: FlowControl,
}

impl Tty {
    /// Create a new TTY configuration.
    pub const fn new(path: String, baud_rate: BaudRate, flow_control: FlowControl) -> Self {
        Self {
            path,
            baud_rate,
            flow_control,
        }
    }

    /// Open a new TTY.
    pub fn open(&self) -> serialport::Result<impl SerialPort + 'static> {
        open(self.path.clone(), self.baud_rate, self.flow_control)
    }
}
