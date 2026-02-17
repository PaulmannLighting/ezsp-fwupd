use bitflags::bitflags;
use le_stream::FromLeStream;

/// Represents the field control flags in an OTA (Over-The-Air) file header.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, FromLeStream)]
pub struct FieldControl(u16);

bitflags! {
    impl FieldControl: u16 {
        const SECURITY_CREDENTIAL_VERSION_FIELD_PRESENT_MASK = 0b0000_0000_0000_0001;
        const DEVICE_SPECIFIC_FILE_PRESENT_MASK = 0b0000_0000_0000_0010;
        const HARDWARE_VERSIONS_PRESENT_MASK = 0b0000_0000_0000_0100;
    }
}

impl FieldControl {
    /// Returns whether the field control flags indicate that the security credentials version field is present.
    #[must_use]
    pub const fn has_security_credentials(self) -> bool {
        self.contains(Self::SECURITY_CREDENTIAL_VERSION_FIELD_PRESENT_MASK)
    }

    /// Returns whether the field control flags indicate that the device-specific file destination is present.
    #[must_use]
    pub const fn has_upgrade_file_destination(self) -> bool {
        self.contains(Self::DEVICE_SPECIFIC_FILE_PRESENT_MASK)
    }

    /// Returns whether the field control flags indicate that the hardware versions are present.
    #[must_use]
    pub const fn has_hardware_version(self) -> bool {
        self.contains(Self::HARDWARE_VERSIONS_PRESENT_MASK)
    }
}
