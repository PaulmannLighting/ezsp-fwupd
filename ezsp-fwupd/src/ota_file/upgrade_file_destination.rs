use std::fmt::Display;

use ezsp::ember::Eui64;

/// Represents a Thread device identifier (Thread ID).
pub type ThreadId = [u8; 32];

/// Represents the destination for an OTA (Over-The-Air) upgrade file.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UpgradeFileDestination {
    /// Destination for a Zigbee device, identified by its EUI-64 address.
    Zigbee(Eui64),
    /// Destination for a Thread device, identified by its Thread ID.
    Thread(Box<ThreadId>),
}

impl Display for UpgradeFileDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zigbee(eui64) => write!(f, "Zigbee: {eui64}"),
            Self::Thread(thread_id) => {
                write!(f, "Thread: {thread_id:010X?}")
            }
        }
    }
}
