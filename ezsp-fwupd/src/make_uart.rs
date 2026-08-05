use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::NonZero;

use ashv2::ezsp::Receiver;
use async_serialport::AsyncSerialPort;
use ezsp::{Client, Connection};
use serialport::SerialPort;
use tokio::spawn;
use tokio::sync::mpsc::channel;
use tokio::task::{JoinError, JoinHandle};

use crate::discard_callbacks;

const ASYNC_SERIAL_CHANNEL_SIZE: usize = 8;

/// Tasks that drive an ASHv2-backed EZSP connection.
pub struct Tasks<T> {
    worker: JoinHandle<T>,
    actors: [JoinHandle<()>; 4],
}

impl<T> Tasks<T> {
    /// Terminates the protocol actors and returns the underlying serial port.
    ///
    /// # Errors
    ///
    /// Returns a [`JoinError`] if an actor or serial-port worker task panicked.
    pub async fn terminate(self) -> Result<T, JoinError> {
        let Self { worker, actors } = self;

        for actor in &actors {
            actor.abort();
        }

        let mut actor_error = None;
        for actor in actors {
            if let Err(error) = actor.await
                && !error.is_cancelled()
            {
                actor_error.get_or_insert(error);
            }
        }

        let serial_port = worker.await?;
        actor_error.map_or(Ok(serial_port), Err)
    }
}

/// An error encountered while creating an ASHv2-backed EZSP connection.
#[derive(Debug)]
pub enum MakeUartError {
    /// The requested EZSP protocol version was zero.
    InvalidProtocolVersion,

    /// EZSP version negotiation failed.
    Connect(ezsp::Error),
}

impl Display for MakeUartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtocolVersion => write!(f, "EZSP protocol version must not be zero"),
            Self::Connect(error) => write!(f, "failed to connect to EZSP: {error}"),
        }
    }
}

impl Error for MakeUartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidProtocolVersion => None,
            Self::Connect(error) => Some(error),
        }
    }
}

/// Creates an EZSP connection over `ASHv2` using the specified serial port and channel sizes.
///
/// # Errors
///
/// Returns a [`MakeUartError`] if the protocol version is zero or EZSP version negotiation fails.
///
/// # Panics
///
/// Panics if either channel size is zero.
pub async fn make_uart<T>(
    serial_port: T,
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
) -> Result<(Tasks<T>, Connection), MakeUartError>
where
    T: SerialPort + Send + 'static,
{
    let protocol_version =
        NonZero::new(protocol_version).ok_or(MakeUartError::InvalidProtocolVersion)?;
    let (reader, writer, worker) = serial_port.split(ASYNC_SERIAL_CHANNEL_SIZE);
    let worker = spawn(worker);
    let (response_tx, response_rx) = channel(response_channel_size);
    let (ash_transmitter, ash_futures) = ashv2::start(reader, writer, response_tx);
    let ash_transmitter_task = spawn(ash_futures.transmitter);
    let ash_receiver_task = spawn(ash_futures.receiver);
    let ash_receiver = Receiver::new(response_rx);
    let (client, ezsp_futures) = Client::run(ash_transmitter, ash_receiver, callback_channel_size);
    let ezsp_transmitter_task = spawn(ezsp_futures.transmitter);
    let ezsp_receiver_task = spawn(ezsp_futures.receiver);
    let tasks = Tasks {
        worker,
        actors: [
            ezsp_transmitter_task,
            ezsp_receiver_task,
            ash_transmitter_task,
            ash_receiver_task,
        ],
    };

    match client.connect(protocol_version).await {
        Ok((connection, callbacks)) => {
            discard_callbacks(callbacks);
            Ok((tasks, connection))
        }
        Err(error) => {
            if let Ok(serial_port) = tasks.terminate().await {
                drop(serial_port);
            }
            Err(MakeUartError::Connect(error))
        }
    }
}
