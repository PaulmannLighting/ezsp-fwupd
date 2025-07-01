use std::fmt::Display;

use bitflags::bitflags;
use le_stream::derive::FromLeStream;

#[derive(Debug, FromLeStream)]
pub struct FieldControl(u16);

bitflags! {
    impl FieldControl: u16 {
        const SECURITY_CREDENTIAL_VERSION_FIELD_PRESENT_MASK = 0b0000_0000_0000_0001;
        const DEVICE_SPECIFIC_FILE_PRESENT_MASK = 0b0000_0000_0000_0010;
        const HARDWARE_VERSIONS_PRESENT_MASK = 0b0000_0000_0000_0100;
    }
}

impl FieldControl {
    #[must_use]
    pub const fn has_security_credentials(&self) -> bool {
        self.contains(Self::SECURITY_CREDENTIAL_VERSION_FIELD_PRESENT_MASK)
    }

    #[must_use]
    pub const fn has_upgrade_file_destination(&self) -> bool {
        self.contains(Self::DEVICE_SPECIFIC_FILE_PRESENT_MASK)
    }

    #[must_use]
    pub const fn has_hardware_version(&self) -> bool {
        self.contains(Self::HARDWARE_VERSIONS_PRESENT_MASK)
    }
}

impl Display for FieldControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        if self.has_security_credentials() {
            flags.push("Security Credentials");
        }

        if self.has_upgrade_file_destination() {
            flags.push("Upgrade File Destination");
        }

        if self.has_hardware_version() {
            flags.push("Hardware Versions");
        }

        write!(f, "FieldControl({})", flags.join(", "))
    }
}
