use std::sync::Arc;

use ezsp::parameters::bootloader::handler::Handler;
use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};
use log::{debug, error, info, warn};
use serialport::SerialPort;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, channel};

use crate::silabs2::xmodem::EOT;
use xmodem_frames::XmodemFrames;

mod xmodem;
mod xmodem_frames;

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

    pub async fn update_firmware(mut self, firmware: Vec<u8>) -> std::io::Result<()> {
        let callbacks_rx = self.receiver.take().expect("Receiver already taken");
        let uart = Arc::new(Mutex::new(self.uart));

        match uart
            .lock()
            .await
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

        let mut xmodem_frames = XmodemFrames::new(firmware.into_iter());
        let first_frame = xmodem_frames.next().expect("Firmware must not be empty");
        let task = spawn_listener_task(uart.clone(), xmodem_frames, callbacks_rx);

        uart.lock()
            .await
            .launch_standalone_bootloader(0x01)
            .await
            .unwrap_or_else(|error| {
                error!("Failed to launch bootloader: {error}");
            });
        uart.lock()
            .await
            .send_bootload_message(
                false,
                [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].into(),
                first_frame.into(),
            )
            .await
            .unwrap_or_else(|error| {
                error!("Failed to send bootload message: {error}");
            });

        task.await.unwrap_or_else(|error| {
            error!("Task failed: {error}");
        });

        Ok(())
    }
}

fn spawn_listener_task<T>(
    uart: Arc<Mutex<Uart>>,
    mut frames: XmodemFrames<T>,
    mut callbacks_rx: Receiver<Callback>,
) -> tokio::task::JoinHandle<()>
where
    T: Iterator<Item = u8> + Send + 'static,
{
    tokio::spawn(async move {
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

                            if let Some(frame) = frames.next() {
                                info!("Sending next frame: {frame:#?}");
                                if let Err(error) = uart
                                    .lock()
                                    .await
                                    .send_bootload_message(
                                        false,
                                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].into(),
                                        frame.into(),
                                    )
                                    .await
                                {
                                    error!("Failed to send bootload message: {error}");
                                    return;
                                }
                            } else {
                                info!("Sending EOT.");

                                if let Err(error) = uart
                                    .lock()
                                    .await
                                    .send_bootload_message(
                                        false,
                                        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].into(),
                                        {
                                            let mut vec = heapless::Vec::<u8, 255>::new();
                                            vec.push(EOT).unwrap();
                                            vec
                                        },
                                    )
                                    .await
                                {
                                    error!("Failed to send EOT: {error}");
                                }

                                info!("All frames sent, exiting bootloader.");
                                return;
                            }
                        }
                    },
                    other => {
                        warn!("Unhandled callback: {other:#?}");
                    }
                }
            }
        }
    })
}
