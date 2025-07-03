use semver::Version;
use std::fmt::Display;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Upgrade,
    Downgrade,
}

impl Direction {
    /// Parses the direction from two versions.
    #[must_use]
    pub fn from_versions(src: Version, dst: Version) -> Option<Self> {
        if src < dst {
            Some(Direction::Upgrade)
        } else if src > dst {
            Some(Direction::Downgrade)
        } else {
            None
        }
    }

    /// Returns the english gerund form of the direction.
    #[must_use]
    pub const fn gerund(self) -> &'static str {
        match self {
            Direction::Upgrade => "Upgrading",
            Direction::Downgrade => "Downgrading",
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Upgrade => write!(f, "upgrade"),
            Direction::Downgrade => write!(f, "downgrade"),
        }
    }
}
