use ezsp::Callback;
use log::debug;
use tokio::spawn;
use tokio::sync::mpsc::Receiver;

/// Discard all callbacks received on the given channel.
pub fn discard_callbacks(mut callbacks: Receiver<Callback>) {
    spawn(async move {
        while let Some(callback) = callbacks.recv().await {
            debug!("Discarding callback: {callback:?}");
        }
    });
}
