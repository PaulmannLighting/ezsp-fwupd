use ashv2::{BaudRate, open};
pub use firmware_updater::FirmwareUpdater;
use log::error;
use serialport::FlowControl;
use silabs::MGM210P22A;
use silabs2::Fwupd;

mod firmware_updater;
mod silabs;
mod silabs2;

#[tokio::main]
async fn main() {
    env_logger::init();

    match open("/dev/ttymxc3", BaudRate::RstCts, FlowControl::Software) {
        Ok(serial_port) => {
            let fwupd = Fwupd::new(serial_port);
            fwupd.update_firmware(&[]).await.unwrap();
        }
        Err(error) => error!("{error}"),
    }
}
