use std::time::Duration;

pub const CALLBACK_CHANNEL_SIZE: usize = 8;
pub const RESPONSE_CHANNEL_SIZE: usize = 8;
pub const PROTOCOL_VERSION: u8 = 8;
pub const RETRY_INTERVAL: Duration = Duration::from_secs(1);
pub const MAX_RETRIES: u8 = 5;
