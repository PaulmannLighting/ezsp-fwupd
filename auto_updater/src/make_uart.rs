use ezsp::Callback;
use ezsp::uart::Uart;
use serialport::SerialPort;
use tokio::sync::mpsc::{Receiver, channel};

/// Creates a new `Uart` instance with the specified serial port and channel sizes.
pub fn make_uart<T>(
    serial_port: T,
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
) -> (Uart<T>, Receiver<Callback>)
where
    T: SerialPort + 'static,
{
    let (callbacks_tx, callbacks_rx) = channel::<Callback>(callback_channel_size);
    (
        Uart::new(
            serial_port,
            callbacks_tx,
            protocol_version,
            response_channel_size,
        ),
        callbacks_rx,
    )
}
