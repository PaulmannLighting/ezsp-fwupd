use std::fmt::Display;

use ashv2::HexSlice;
use ezsp::ember::Eui64;

#[derive(Debug)]
pub enum UpgradeFileDestination {
    Zigbee(Eui64),
    Thread(Box<[u8; 32]>),
}

impl Display for UpgradeFileDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zigbee(eui64) => write!(f, "Zigbee: {eui64}"),
            Self::Thread(thread_id) => {
                write!(f, "Thread: {:010X}", HexSlice::new(thread_id.as_slice()))
            }
        }
    }
}
