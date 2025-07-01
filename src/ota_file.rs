use ashv2::HexSlice;
use ezsp::ember::Eui64;
use header::Header;
use le_stream::FromLeStream;
use std::fmt::Display;
use tag::Tag;
use upgrade_file_destination::UpgradeFileDestination;

const MAGIC: [u8; 4] = [0x1E, 0xF1, 0xEE, 0x0B];
const HEADER_VERSION_ZIGBEE: u16 = 0x0100;
const HEADER_VERSION_THREAD: u16 = 0x0200;

mod header;
mod tag;
mod upgrade_file_destination;

#[derive(Debug)]
pub struct OtaFile {
    magic: [u8; 4],
    header: Header,
    security_credentials: Option<u8>,
    upgrade_file_destination: Option<UpgradeFileDestination>,
    hardware_versions: Option<(u16, u16)>,
    tags: Vec<Tag>,
    payload: Vec<u8>,
}

impl OtaFile {
    #[must_use]
    pub const fn magic(&self) -> &[u8; 4] {
        &self.magic
    }

    #[must_use]
    pub const fn header(&self) -> &Header {
        &self.header
    }

    #[must_use]
    pub const fn security_credentials(&self) -> Option<u8> {
        self.security_credentials
    }

    #[must_use]
    pub const fn upgrade_file_destination(&self) -> Option<&UpgradeFileDestination> {
        self.upgrade_file_destination.as_ref()
    }

    #[must_use]
    pub const fn hardware_versions(&self) -> Option<(u16, u16)> {
        self.hardware_versions
    }

    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn validate(self) -> Result<Self, [u8; 4]> {
        if self.magic == MAGIC {
            Ok(self)
        } else {
            Err(self.magic)
        }
    }
}

impl Display for OtaFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OtaFile {{ magic: {:#04X}, header: {}, security_credentials: {}, upgrade_file_destination: {}, hardware_versions: {:?}, tags: {:?}, payload: {:#04X?} }}",
            HexSlice::new(self.magic()),
            self.header(),
            self.security_credentials().map_or_else(
                || "-".to_string(),
                |security_credentials| format!("{security_credentials:#04X}")
            ),
            self.upgrade_file_destination().map_or_else(
                || "-".to_string(),
                |upgrade_file_destination| format!("{upgrade_file_destination}")
            ),
            self.hardware_versions(),
            self.tags(),
            self.payload()
        )
    }
}

impl FromLeStream for OtaFile {
    fn from_le_stream<T>(mut bytes: T) -> Option<Self>
    where
        T: Iterator<Item = u8>,
    {
        let magic = <[u8; 4]>::from_le_stream(&mut bytes)?;
        let header = Header::from_le_stream(&mut bytes)?;
        let field_control = header.field_control();

        let security_credentials = if field_control.has_security_credentials() {
            Some(u8::from_le_stream(&mut bytes)?)
        } else {
            None
        };

        let upgrade_file_destination = if field_control.has_upgrade_file_destination() {
            match header.version() {
                HEADER_VERSION_ZIGBEE => Some(UpgradeFileDestination::Zigbee(
                    Eui64::from_le_stream(&mut bytes)?,
                )),
                HEADER_VERSION_THREAD => Some(UpgradeFileDestination::Thread(
                    <[u8; 32]>::from_le_stream(&mut bytes)?,
                )),
                _ => None,
            }
        } else {
            None
        };

        let hardware_versions = if field_control.has_hardware_version() {
            Some((
                u16::from_le_stream(&mut bytes)?,
                u16::from_le_stream(&mut bytes)?,
            ))
        } else {
            None
        };

        let tags = header.tags(&mut bytes);
        let payload = bytes.collect::<Vec<_>>();

        Some(Self {
            magic,
            header,
            security_credentials,
            upgrade_file_destination,
            hardware_versions,
            tags,
            payload,
        })
    }
}
