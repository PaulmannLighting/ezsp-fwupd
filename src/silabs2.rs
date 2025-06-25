mod xmodem;
mod xmodem_frames;

use ezsp::parameters::bootloader::handler::Handler;
use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback, Configuration, Utilities};
use log::{debug, error, info};
use serialport::SerialPort;
use tokio::sync::mpsc::{Receiver, channel};

#[derive(Debug)]
pub struct Fwupd {
    uart: Uart,
    receiver: Option<Receiver<Callback>>,
}

impl Fwupd {
    pub fn new<T>(serial_port: T) -> Self
    where
        T: SerialPort + 'static,
    {
        let (callbacks_tx, callbacks_rx) = channel::<Callback>(8);
        Self {
            uart: Uart::new(serial_port, callbacks_tx, 8, 8),
            receiver: Some(callbacks_rx),
        }
    }

    pub async fn update_firmware(mut self, firmware: &[u8]) -> std::io::Result<()> {
        let mut callbacks_rx = self.receiver.take().expect("Receiver already taken");

        match self
            .uart
            .get_standalone_bootloader_version_plat_micro_phy()
            .await
        {
            Ok(info) => {
                info!("Bootloader info: {info:#?}");
            }
            Err(error) => {
                error!("Failed to get bootloader info: {error}");
                return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
            }
        };

        let task = tokio::spawn(async move {
            loop {
                if let Some(callback) = callbacks_rx.recv().await {
                    debug!("Received callback: {callback:#?}");
                    match callback {
                        Callback::Bootloader(message) => match message {
                            Handler::BootloadTransmitComplete(handler) => {
                                info!("Bootload transmit complete: {handler:#?}");
                            }
                            Handler::IncomingBootloadMessage(handler) => {
                                info!("Incoming bootload message: {handler:#?}");
                            }
                        },
                        other => {
                            debug!("Unhandled callback: {other:#?}");
                        }
                    }
                }
            }
        });

        self.uart
            .launch_standalone_bootloader(0x01)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch bootloader: {error}");
            });

        task.await.unwrap_or_else(|error| {
            error!("Task failed: {error}");
        });

        Ok(())
    }
}
