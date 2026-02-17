use std::time::Duration;

use ashv2::{Actor, TryCloneNative};
use ezsp::Bootloader;
use ezsp::uart::Uart;
use indicatif::ProgressBar;
use log::{debug, error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::channel;

pub use self::reset::Reset;
use self::transmit::Transmit;
pub use crate::xmodem::FrameCount;
use crate::{ClearBuffer, FlashProgress, discard_callbacks};

mod reset;
mod transmit;

const MODE: u8 = 0x00;

/// Trait for firmware update operations using a serial port.
pub trait Fwupd: Sized {
    /// Performs a firmware update operation.
    fn fwupd<F>(
        self,
        firmware: F,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> impl Future<Output = std::io::Result<Self>>
    where
        F: IntoIterator<Item = u8>;
}

impl<T> Fwupd for T
where
    T: SerialPort + TryCloneNative + Send + Sync + 'static,
{
    async fn fwupd<F>(
        mut self,
        firmware: F,
        timeout: Option<Duration>,
        progress_bar: Option<&ProgressBar>,
    ) -> std::io::Result<Self>
    where
        F: IntoIterator<Item = u8>,
    {
        info!("Preparing bootloader...");
        let (response_tx, response_rx) = channel(8);
        let (actor, proxy) = Actor::new(self.try_clone_native()?, response_tx, 8)?;
        let (tx_handle, rx_handle) = actor.spawn();
        let (callbacks_tx, callbacks_rx) = channel(8);
        discard_callbacks(callbacks_rx);
        let mut uart = Uart::new(proxy, response_rx, callbacks_tx, 8, 8);
        debug!("Launching standalone bootloader...");
        /*
        uart.launch_standalone_bootloader(MODE)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch standalone bootloader: {error}");
            });*/

        let original_timeout = self.timeout();

        if let Some(timeout) = timeout {
            self.set_timeout(timeout)?;
        }

        self.clear_buffer()?;

        debug!("Initializing stage 1...");
        self.init_stage1()?;

        debug!("Initializing stage 2...");
        self.init_stage2()?;

        debug!("Transmitting firmware...");
        self.transmit(firmware, Some(original_timeout), progress_bar)?;

        progress_bar.set_message("Firmware update complete, resetting device...");
        self.reset(timeout)?;
        Ok(self)
    }
}
