use le_stream::derive::FromLeStream;
use std::fmt::Display;

#[derive(Debug, FromLeStream)]
pub struct Tag {
    id: u16,
    length: u32,
}

impl Tag {
    pub const SIZE: u32 = 2 + 4;

    #[must_use]
    pub const fn id(&self) -> u16 {
        self.id
    }

    #[must_use]
    pub const fn length(&self) -> u32 {
        self.length
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag(id: {}, length: {})", self.id(), self.length())
    }
}
