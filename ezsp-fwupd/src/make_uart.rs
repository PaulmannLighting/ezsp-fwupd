use ashv2::{Actor, Tasks, TryCloneNative};
use ezsp::Callback;
use ezsp::uart::Uart;
use log::debug;
use serialport::SerialPort;
use tokio::spawn;
use tokio::sync::mpsc::{Receiver, channel};

/// Creates a new `Uart` instance with the specified serial port and channel sizes.
///
/// # Errors
///
/// Returns a [`serialport::Error`] if the serial port cannot be cloned or if the actor cannot be created.
pub fn make_uart<T>(
    serial_port: T,
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
) -> Result<(Tasks<T>, Uart), serialport::Error>
where
    T: SerialPort + TryCloneNative + Send + Sync + 'static,
{
    let (response_tx, response_rx) = channel(response_channel_size);
    let (tasks, proxy) = Actor::new(serial_port, response_tx, 8)?.spawn();
    let (callbacks_tx, callbacks_rx) = channel(callback_channel_size);
    discard_callbacks(callbacks_rx);
    Ok((
        tasks,
        Uart::new(proxy, response_rx, callbacks_tx, protocol_version, 8),
    ))
}

/// Discard all callbacks received on the given channel.
fn discard_callbacks(mut callbacks: Receiver<Callback>) {
    spawn(async move {
        while let Some(callback) = callbacks.recv().await {
            debug!("Discarding callback: {callback:?}");
        }
    });
}
