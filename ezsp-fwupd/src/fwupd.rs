use std::time::Duration;

use indicatif::ProgressBar;
use log::{debug, error, info};
use serialport::{FlowControl, SerialPort};

pub use self::gecko_bootloader::GeckoBootloader;
use self::transmit::Transmit;
use crate::launch_bootloader::LaunchBootloader;
use crate::{ClearBuffer, FlashProgress};

mod gecko_bootloader;
mod transmit;

/// Performs a complete Gecko standalone-bootloader firmware update on a native serial port.
///
/// The implementation temporarily turns the port into an asynchronous ASHv2/EZSP connection to
/// launch the bootloader. It then recovers the port, disables serial flow control, drives the Gecko
/// ASCII menu, and sends the supplied GBL bytes with XMODEM-CRC. The original timeout and flow
/// control settings are restored before the returned future completes.
pub trait Fwupd: Sized {
    /// Uploads `firmware` and asks the standalone bootloader to run the resulting application.
    ///
    /// `firmware` must yield the raw GBL byte stream expected by the Gecko bootloader. When
    /// `timeout` is present, it is used for the synchronous bootloader and XMODEM stages. Progress
    /// is advanced once for each 128-byte block read by the XMODEM implementation.
    ///
    /// The future returns the same serial port value, with its original timeout and flow-control
    /// settings restored.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if EZSP bootloader entry, serial reconfiguration, a Gecko menu
    /// operation, the XMODEM transfer, or restoration of the serial settings fails.
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
    T: SerialPort + Send + 'static,
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
        let original_flow_control = self.flow_control()?;
        self = self.launch_bootloader().await?;
        self.set_flow_control(FlowControl::None)?;
        let original_timeout = self.timeout();

        let update_result = (|| {
            if let Some(timeout) = timeout {
                self.set_timeout(timeout)?;
            }

            self.clear_buffer()?;

            debug!("Waking the bootloader menu...");
            self.wake_bootloader_menu()?;

            debug!("Starting the XMODEM upload...");
            self.start_xmodem_upload()?;

            debug!("Transmitting firmware...");
            self.transmit(firmware, progress_bar)?;

            debug!("Waiting for the bootloader menu...");
            self.wait_for_bootloader_menu()?;

            progress_bar.set_message("Firmware update complete, resetting device...");
            self.run_application(timeout)
        })();
        let restore_result =
            restore_serial_settings(&mut self, original_timeout, original_flow_control);

        match (update_result, restore_result) {
            (Ok(()), Ok(())) => Ok(self),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(update_error), Err(restore_error)) => {
                error!("Failed to restore serial settings: {restore_error}");
                Err(update_error)
            }
        }
    }
}

fn restore_serial_settings<T>(
    serial_port: &mut T,
    timeout: Duration,
    flow_control: FlowControl,
) -> std::io::Result<()>
where
    T: SerialPort,
{
    let timeout_result = serial_port
        .set_timeout(timeout)
        .map_err(std::io::Error::from);
    let flow_control_result = serial_port
        .set_flow_control(flow_control)
        .map_err(std::io::Error::from);

    match (timeout_result, flow_control_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(timeout_error), Err(flow_control_error)) => {
            error!("Failed to restore serial flow control: {flow_control_error}");
            Err(timeout_error)
        }
    }
}
