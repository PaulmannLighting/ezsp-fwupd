const ONES_COMPLEMENT: u8 = 0xFF; // 255 in decimal, used for checksum calculations
const SOH: u8 = 0x01; // Start of Header
pub const EOT: u8 = 0x04; // End of Transmission
pub const ACK: u8 = 0x06; // Acknowledge
pub const NAK: u8 = 0x15; // Negative Acknowledge
pub const PAYLOAD_SIZE: usize = 128; // Size of the payload in bytes
pub const PACKET_SIZE: usize = PAYLOAD_SIZE + 4; // Total size of the packet (SOH, block number, complement, payload, checksum)
pub type Payload = heapless::Vec<u8, PAYLOAD_SIZE>;
pub type PacketBytes = heapless::Vec<u8, PACKET_SIZE>;

/// Represents an Xmodem packet structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Xmodem {
    soh: u8,
    blk: u8,
    cmp: u8,
    data: Payload,
    chk: u8,
}

impl Xmodem {
    /// Creates a new Xmodem packet with the given block number and data.
    pub fn new(blk: u8, data: Payload) -> Self {
        Self {
            soh: SOH,
            blk,
            cmp: blk ^ ONES_COMPLEMENT,
            chk: data.iter().fold(0, |acc, x| acc.wrapping_add(*x)),
            data,
        }
    }
}

impl From<Xmodem> for PacketBytes {
    fn from(packet: Xmodem) -> Self {
        let mut vec = heapless::Vec::new();
        vec.push(packet.soh)
            .expect("Buffer overflow. This is a bug.");
        vec.push(packet.blk)
            .expect("Buffer overflow. This is a bug.");
        vec.push(packet.cmp)
            .expect("Buffer overflow. This is a bug.");
        vec.extend(packet.data);
        vec.push(packet.chk)
            .expect("Buffer overflow. This is a bug.");
        vec
    }
}

impl From<Xmodem> for heapless::Vec<u8, 255> {
    fn from(packet: Xmodem) -> Self {
        let mut vec = Self::new();
        vec.extend(PacketBytes::from(packet));
        vec
    }
}
