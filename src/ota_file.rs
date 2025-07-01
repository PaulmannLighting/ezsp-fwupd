use le_stream::FromLeStream;
use le_stream::derive::FromLeStream;

const EMBER_AF_OTA_MAX_HEADER_STRING_LENGTH: usize = 32;

#[derive(Debug)]
pub struct OtaFile {
    header: OtaHeader,
    footer: Option<Footer>,
    payload: Vec<u8>,
}

impl OtaFile {
    #[must_use]
    pub const fn header(&self) -> &OtaHeader {
        &self.header
    }

    #[must_use]
    pub const fn footer(&self) -> Option<&Footer> {
        self.footer.as_ref()
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl FromLeStream for OtaFile {
    fn from_le_stream<T>(mut bytes: T) -> Option<Self>
    where
        T: Iterator<Item = u8>,
    {
        let header = OtaHeader::from_le_stream(&mut bytes)?;
        let footer = Footer::from_le_stream(&mut bytes)?;
        let payload = bytes.collect::<Vec<_>>();
        Some(Self {
            header,
            footer: Some(footer),
            payload,
        })
    }
}

#[derive(Debug, FromLeStream)]
pub struct OtaHeader {
    version: u16,
    length: u16,
    field_control: u16,
    manufacturer_id: u16,
    image_type: u16,
    firmware_version: u32,
    zigbee_stack_version: u16,
    header_string: [u8; EMBER_AF_OTA_MAX_HEADER_STRING_LENGTH + 1],
    image_size: u32,
}

impl OtaHeader {
    #[must_use]
    pub fn header_string(&self) -> String {
        String::from_utf8_lossy(&self.header_string).to_string()
    }

    #[must_use]
    pub const fn image_size(&self) -> u32 {
        self.image_size
    }
}

#[derive(Debug, FromLeStream)]
pub struct Footer {
    security_credentials: u8,
    upgrade_file_destination: [u8; 8],
    minimum_hardware_version: u16,
    maximum_hardware_version: u16,
}
