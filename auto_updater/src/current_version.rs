use std::array::TryFromSliceError;
use std::time::Duration;

use ezsp::{GetValueExt, ezsp::value::EmberVersion, uart::Uart};
use log::{debug, error};
use semver::Version;
use serialport::SerialPort;
use tokio::time::sleep;

/// Extension trait for getting the current firmware version from a Zigbee device.
pub trait CurrentVersion {
    /// Await the current firmware version from the Zigbee device.
    fn await_current_version(
        &mut self,
        retry_interval: Duration,
        max_retries: usize,
    ) -> impl Future<Output = Option<Version>>;

    /// Parse the version information from the device.
    fn parse_version(&self, result: Result<EmberVersion, TryFromSliceError>) -> Option<Version> {
        match result {
            Ok(version_info) => match version_info.try_into() {
                Ok(version) => Some(version),
                Err(error) => {
                    error!("Failed to parse version info: {error}");
                    None
                }
            },
            Err(error) => {
                error!("Failed to parse version info: {error}");
                None
            }
        }
    }
}

impl<T> CurrentVersion for Uart<T>
where
    T: SerialPort + 'static,
{
    async fn await_current_version(
        &mut self,
        retry_interval: Duration,
        mut max_retries: usize,
    ) -> Option<Version> {
        loop {
            match self.get_ember_version().await {
                Ok(result) => return self.parse_version(result),
                Err(error) => {
                    debug!("Failed to get version info: {error}");

                    if let Some(retries) = max_retries.checked_sub(1) {
                        max_retries = retries;
                    } else {
                        error!("Max retries reached: {error}");
                        return None;
                    }

                    sleep(retry_interval).await;
                }
            }
        }
    }
}
