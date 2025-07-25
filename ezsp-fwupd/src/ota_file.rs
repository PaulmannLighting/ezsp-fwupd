use std::fmt::Display;

use ezsp::ember::Eui64;
use header::Header;
use le_stream::FromLeStream;
use tag::Tag;
use upgrade_file_destination::UpgradeFileDestination;

const MAGIC: [u8; 4] = [0x1E, 0xF1, 0xEE, 0x0B];
const HEADER_VERSION_ZIGBEE: u16 = 0x0100;
const HEADER_VERSION_THREAD: u16 = 0x0200;

mod header;
mod tag;
mod upgrade_file_destination;

/// Represents an OTA (Over-The-Air) file used for firmware updates.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    /// Return the OTA file's header magic.
    #[must_use]
    pub const fn magic(&self) -> &[u8; 4] {
        &self.magic
    }

    /// Return the OTA file's header.
    #[must_use]
    pub const fn header(&self) -> &Header {
        &self.header
    }

    /// Return the OTA file's security credentials, if present.
    #[must_use]
    pub const fn security_credentials(&self) -> Option<u8> {
        self.security_credentials
    }

    /// Return the OTA file's upgrade file destination, if present.
    #[must_use]
    pub const fn upgrade_file_destination(&self) -> Option<&UpgradeFileDestination> {
        self.upgrade_file_destination.as_ref()
    }

    /// Return the OTA file's hardware versions, if present.
    ///
    /// The first value is the minimum hardware version, and the second value is the maximum hardware version.
    #[must_use]
    pub const fn hardware_versions(&self) -> Option<(u16, u16)> {
        self.hardware_versions
    }

    /// Return the OTA file's tags.
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Return the OTA file's payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Validate the OTA file's magic number.
    ///
    /// # Returns
    ///
    /// If the magic number matches, returns `Ok(Self)`.
    ///
    /// # Errors
    ///
    /// If the magic number does not match, returns `Err([u8; 4])` with the faulty magic number.
    pub fn validate(self) -> Result<Self, [u8; 4]> {
        if self.magic == MAGIC {
            Ok(self)
        } else {
            Err(self.magic)
        }
    }

    /// Convert the OTA file into a payload vector.
    #[must_use]
    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}

impl Display for OtaFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)
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
                    <[u8; 32]>::from_le_stream(&mut bytes)?.into(),
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
