use le_stream::FromLeStream;

/// Header describing one sub-element in a Zigbee OTA image.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, FromLeStream)]
pub struct Tag {
    id: u16,
    length: u32,
}

impl Tag {
    /// Encoded size of the tag identifier and payload length fields.
    pub const SIZE: u32 = 2 + 4;

    /// Returns the Zigbee OTA tag identifier.
    #[must_use]
    pub const fn id(self) -> u16 {
        self.id
    }

    /// Returns the declared payload length for this tag.
    #[must_use]
    pub const fn length(self) -> u32 {
        self.length
    }
}
