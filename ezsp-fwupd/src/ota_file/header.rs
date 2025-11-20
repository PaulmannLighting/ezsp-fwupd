use std::fmt::Display;

use le_stream::FromLeStream;
use le_stream::derive::FromLeStream;

use self::field_control::FieldControl;
use super::tag::Tag;

const HEADER_STRING_LENGTH: usize = 32;

mod field_control;

/// Represents the header of an OTA (Over-The-Air) file used for firmware updates.
#[derive(Clone, Debug, Eq, Hash, PartialEq, FromLeStream)]
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
    /// Return the version.
    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    /// Return the header's length.
    #[must_use]
    pub const fn length(&self) -> u16 {
        self.length
    }

    /// Return the field control flags.
    #[must_use]
    pub const fn field_control(&self) -> &FieldControl {
        &self.field_control
    }

    /// Return the manufacturer ID.
    #[must_use]
    pub const fn manufacturer_id(&self) -> u16 {
        self.manufacturer_id
    }

    /// Return the image type.
    #[must_use]
    pub const fn image_type(&self) -> u16 {
        self.image_type
    }

    /// Return the firmware version.
    #[must_use]
    pub const fn firmware_version(&self) -> u32 {
        self.firmware_version
    }

    /// Return the Zigbee stack version.
    #[must_use]
    pub const fn zigbee_stack_version(&self) -> u16 {
        self.zigbee_stack_version
    }

    /// Return the name of the OTA file.
    #[must_use]
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }

    /// Return the size of the image.
    #[must_use]
    pub const fn image_size(&self) -> u32 {
        self.image_size
    }

    /// Parse the tags from the OTA file.
    #[must_use]
    pub(crate) fn parse_tags<T>(&self, mut bytes: T) -> Vec<Tag>
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
        writeln!(f, "OTA image name:    {}", self.name())?;
        writeln!(f, "OTA image type:    {}", self.image_type())?;
        writeln!(f, "OTA image version: {}", self.version())?;
        writeln!(f, "OTA file version:  {}", self.firmware_version())?;
        writeln!(f, "OTA Zigbee stack:  {}", self.zigbee_stack_version())?;
        writeln!(f, "OTA manufacturer:  {}", self.manufacturer_id())?;
        write!(f, "OTA image size:    {}", self.image_size())
    }
}
