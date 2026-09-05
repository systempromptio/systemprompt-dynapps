//! The project a user belongs to: one of the two AD-group-backed workstreams
//! the directory places every person in. Two values, closed set, so it is an
//! enum rather than a table.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Project {
    Commerce,
    Core,
}

impl Project {
    pub const ALL: [Self; 2] = [Self::Commerce, Self::Core];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Commerce => "commerce",
            Self::Core => "core",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Commerce => "Commerce",
            Self::Core => "Core",
        }
    }

    #[must_use]
    pub fn parse_opt(value: Option<&str>) -> Option<Self> {
        value.and_then(|v| v.parse().ok())
    }
}

impl FromStr for Project {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|p| p.as_str() == s).ok_or(())
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
