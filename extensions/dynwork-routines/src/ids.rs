//! The ids a desktop picks for its routines and their runs, and the states a
//! run reports.
//!
//! An id is 1 to 128 letters, digits, `-`, `_`, `.` or `:`, so it fits a path
//! segment as it is. It is scoped to its user: two people may use the same.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

const MAX_ID: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{0} is 1 to 128 letters, digits, '-', '_', '.' or ':'")]
pub struct IdError(&'static str);

fn valid(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_ID
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}

macro_rules! text_id {
    ($name:ident, $what:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(text: &str) -> Result<Self, IdError> {
                if valid(text) {
                    Ok(Self(text.to_owned()))
                } else {
                    Err(IdError($what))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) const fn from_stored(text: String) -> Self {
                Self(text)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::parse(&text).map_err(serde::de::Error::custom)
            }
        }
    };
}

text_id!(RoutineId, "a routine id");
text_id!(RunId, "a run id");

/// Where a run is, as `DynWork`'s routines contract names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunState {
    Queued,
    Running,
    Ok,
    Late,
    Failed,
    Blocked,
    Missed,
    Skipped,
    Cancelled,
}

impl RunState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Ok => "ok",
            Self::Late => "late",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Missed => "missed",
            Self::Skipped => "skipped",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for RunState {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, IdError> {
        Ok(match text {
            "queued" => Self::Queued,
            "running" => Self::Running,
            "ok" => Self::Ok,
            "late" => Self::Late,
            "failed" => Self::Failed,
            "blocked" => Self::Blocked,
            "missed" => Self::Missed,
            "skipped" => Self::Skipped,
            "cancelled" => Self::Cancelled,
            _ => return Err(IdError("a run state")),
        })
    }
}
