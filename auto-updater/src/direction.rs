use std::cmp::Ordering;
use std::fmt::Display;

use semver::Version;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Upgrade,
    Downgrade,
    Unknown,
}

impl Direction {
    /// Parses the direction from two versions.
    #[must_use]
    pub fn from_versions(current: Option<&Version>, new: &Version) -> Option<Self> {
        let Some(current) = current else {
            return Some(Self::Unknown);
        };

        match current.cmp(new) {
            Ordering::Less => Some(Self::Upgrade),
            Ordering::Greater => Some(Self::Downgrade),
            Ordering::Equal => None,
        }
    }

    /// Returns the present participle form of the direction.
    #[must_use]
    pub const fn present_participle(self) -> &'static str {
        match self {
            Self::Upgrade => "Upgrading",
            Self::Downgrade => "Downgrading",
            Self::Unknown => "Flashing",
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Upgrade => write!(f, "upgrade"),
            Self::Downgrade => write!(f, "downgrade"),
            Self::Unknown => write!(f, "flashing"),
        }
    }
}
