/// Parameters for UART communication with the EZSP device.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UartParams {
    callback_channel_size: usize,
    response_channel_size: usize,
    protocol_version: u8,
}

impl UartParams {
    /// Create new UART parameters.
    #[must_use]
    pub const fn new(
        callback_channel_size: usize,
        response_channel_size: usize,
        protocol_version: u8,
    ) -> Self {
        Self {
            callback_channel_size,
            response_channel_size,
            protocol_version,
        }
    }

    /// Return the callback channel size.
    #[must_use]
    pub const fn callback_channel_size(&self) -> usize {
        self.callback_channel_size
    }

    /// Return the response channel size.
    #[must_use]
    pub const fn response_channel_size(&self) -> usize {
        self.response_channel_size
    }

    /// Return the EZSP protocol version to use.
    #[must_use]
    pub const fn protocol_version(&self) -> u8 {
        self.protocol_version
    }
}
