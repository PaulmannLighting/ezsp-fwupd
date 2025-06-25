use log::{debug, error, info, warn};
use tokio::sync::mpsc::{Receiver, channel};

use ezsp::parameters::bootloader::handler::Handler;
use ezsp::uart::Uart;
use ezsp::{Bootloader, Callback};

pub use tty::Tty;
use xmodem::Send;

mod tty;
mod xmodem;

pub async fn update_firmware(tty: Tty, firmware: Vec<u8>) -> std::io::Result<Box<[u8]>> {
    let (callbacks_tx, callbacks_rx) = channel::<Callback>(8);
    let mut uart = Uart::new(tty.open()?, callbacks_tx, 8, 8);

    match uart
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

    spawn_listener_task(callbacks_rx);
    uart.launch_standalone_bootloader(0x00)
        .await
        .unwrap_or_else(|error| {
            error!("Failed to launch standalone bootloader: {error}");
        });

    drop(uart);
    tty.open()?.send(firmware)
}

fn spawn_listener_task(mut callbacks_rx: Receiver<Callback>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            info!("Awaiting callback...");
            if let Some(callback) = callbacks_rx.recv().await {
                debug!("Received callback: {callback:#?}");
                match callback {
                    Callback::Bootloader(message) => match message {
                        Handler::BootloadTransmitComplete(handler) => {
                            info!("Bootload transmit complete: {handler:#?}");
                            return;
                        }
                        Handler::IncomingBootloadMessage(handler) => {
                            info!("Incoming bootload message: {handler:#?}");
                            return;
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
