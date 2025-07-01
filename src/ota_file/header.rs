use le_stream::derive::FromLeStream;

use field_control::FieldControl;

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
}
