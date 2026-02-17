use le_stream::FromLeStream;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, FromLeStream)]
pub struct Tag {
    id: u16,
    length: u32,
}

impl Tag {
    pub const SIZE: u32 = 2 + 4;

    #[must_use]
    pub const fn id(self) -> u16 {
        self.id
    }

    #[must_use]
    pub const fn length(self) -> u32 {
        self.length
    }
}
