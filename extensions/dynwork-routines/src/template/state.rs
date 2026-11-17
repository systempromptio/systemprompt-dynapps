//! A template's lifecycle as the library stores it, how it was retired,
//! where it came from, and what a person's link to it reports.

use std::str::FromStr;

use serde::Serialize;

/// `Draft` was never published. `Closed` takes no new activations, while the
/// tasks already linked keep receiving its revisions. `Retired` is final.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateState {
    Draft,
    Published,
    Closed,
    Retired,
}

/// What retiring a template does to the tasks linked to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RetireMode {
    Unlink,
    Pause,
    Remove,
}

/// Where the library got the template: the admin console or a YAML file
/// under `services/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSource {
    Console,
    Yaml,
}

/// A person's link to a template they have a live task of. `NoAccess`: the
/// template is still there but none of its anchor plugins admits them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkState {
    Published,
    Closed,
    Retired,
    NoAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a template {0}")]
pub struct UnknownValue(&'static str);

impl FromStr for TemplateState {
    type Err = UnknownValue;

    fn from_str(text: &str) -> Result<Self, UnknownValue> {
        Ok(match text {
            "draft" => Self::Draft,
            "published" => Self::Published,
            "closed" => Self::Closed,
            "retired" => Self::Retired,
            _ => return Err(UnknownValue("state")),
        })
    }
}

impl RetireMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unlink => "unlink",
            Self::Pause => "pause",
            Self::Remove => "remove",
        }
    }
}

impl FromStr for RetireMode {
    type Err = UnknownValue;

    fn from_str(text: &str) -> Result<Self, UnknownValue> {
        Ok(match text {
            "unlink" => Self::Unlink,
            "pause" => Self::Pause,
            "remove" => Self::Remove,
            _ => return Err(UnknownValue("retire mode")),
        })
    }
}

impl TemplateSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Console => "console",
            Self::Yaml => "yaml",
        }
    }
}

impl FromStr for TemplateSource {
    type Err = UnknownValue;

    fn from_str(text: &str) -> Result<Self, UnknownValue> {
        Ok(match text {
            "console" => Self::Console,
            "yaml" => Self::Yaml,
            _ => return Err(UnknownValue("source")),
        })
    }
}
