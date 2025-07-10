use ezsp::{GetValueExt, uart::Uart};
use log::error;
use semver::Version;
use serialport::SerialPort;

/// Extension trait for getting the current firmware version from a Zigbee device.
pub trait GetCurrentVersion {
    /// Get the current firmware version from the Zigbee device.
    fn get_current_version(&mut self) -> impl Future<Output = Option<Version>>;
}

impl<T> GetCurrentVersion for Uart<T>
where
    T: SerialPort + 'static,
{
    async fn get_current_version(&mut self) -> Option<Version> {
        match self.get_ember_version().await {
            Ok(result) => match result {
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
            },
            Err(error) => {
                error!("Failed to get version info: {error}");
                None
            }
        }
    }
}
