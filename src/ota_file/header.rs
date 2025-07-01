use std::fmt::Display;

use field_control::FieldControl;
use le_stream::FromLeStream;
use le_stream::derive::FromLeStream;

use super::tag::Tag;

const HEADER_STRING_LENGTH: usize = 32;

mod field_control;

#[derive(Debug, FromLeStream)]
pub struct Header {
    version: u16,
    length: u16,
    field_control: FieldControl,
    manufacturer_id: u16,
    image_type: u16,
    firmware_version: u32,
    zigbee_stack_version: u16,
    name: [u8; HEADER_STRING_LENGTH],
    image_size: u32,
}

impl Header {
    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn length(&self) -> u16 {
        self.length
    }

    #[must_use]
    pub const fn field_control(&self) -> &FieldControl {
        &self.field_control
    }

    #[must_use]
    pub const fn manufacturer_id(&self) -> u16 {
        self.manufacturer_id
    }

    #[must_use]
    pub const fn image_type(&self) -> u16 {
        self.image_type
    }

    #[must_use]
    pub const fn firmware_version(&self) -> u32 {
        self.firmware_version
    }

    #[must_use]
    pub const fn zigbee_stack_version(&self) -> u16 {
        self.zigbee_stack_version
    }

    #[must_use]
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }

    #[must_use]
    pub const fn image_size(&self) -> u32 {
        self.image_size
    }

    #[must_use]
    pub fn tags<T>(&self, mut bytes: T) -> Vec<Tag>
    where
        T: Iterator<Item = u8>,
    {
        let mut tags = Vec::new();
        let mut limit = self.image_size() - u32::from(self.length());

        while limit > 0 {
            if let Some(tag) = Tag::from_le_stream(&mut bytes) {
                limit = limit.saturating_sub(tag.length()).saturating_sub(Tag::SIZE);
                tags.push(tag);
            } else {
                return tags;
            }
        }

        tags
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Header {{ version: {}, length: {}, field_control: {}, manufacturer_id: {}, image_type: {}, firmware_version: {}, zigbee_stack_version: {}, name: '{}', image_size: {} }}",
            self.version(),
            self.length(),
            self.field_control(),
            self.manufacturer_id(),
            self.image_type(),
            self.firmware_version(),
            self.zigbee_stack_version(),
            self.name(),
            self.image_size()
        )
    }
}
