//! The extension's settings, from `<services>/config/dynwork-chats.yaml`.
//!
//! The file is optional and every key in it too: without a key the default
//! applies. `inactive_days` is the inactivity window of the retention job
//! (decision D4): unset keeps threads until a person deletes them. It must be
//! at least 1, so a typo can never read as "delete everything idle now".
//! `web_search_per_thread` is the per-thread search ceiling stage 3 enforces;
//! it is read and exposed here and nothing uses it yet. Core validates the
//! file at startup through [`ChatsConfig::from_json`], and the retention job
//! reads it again on every run, so a change reaches the next run without a
//! restart.

use std::io;
use std::path::Path;

use chrono::Duration;
use serde::Deserialize;
// JSON: core hands an extension its config block as a JSON document.
use serde_json::Value;

pub const CONFIG_PREFIX: &str = "dynwork-chats";

/// The key core adds when it mounts an extension's router: it calls `validate_config` a second time with the profile's
/// paths as `{"paths": ...}` instead of the config file's block. It is never one of our keys, so it is set aside before
/// the strict check (without that the server refuses to start: "the chats config is not a valid block").
const MOUNT_KEY: &str = "paths";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatsConfig {
    #[serde(default)]
    pub inactive_days: Option<u32>,
    #[serde(default)]
    pub web_search_per_thread: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("the chats config {path} could not be read")]
    Read {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("the chats config is not valid YAML of the expected shape")]
    Yaml(#[from] serde_yaml::Error),
    #[error("the chats config is not a valid block")]
    Json(#[from] serde_json::Error),
    #[error("inactive_days must be at least 1 (leave it unset to keep threads)")]
    ZeroDays,
}

impl ChatsConfig {
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let parsed: Option<Self> = serde_yaml::from_str(text)?;
        parsed.unwrap_or_default().checked()
    }

    pub fn from_json(block: &Value) -> Result<Self, ConfigError> {
        if block.is_null() {
            return Ok(Self::default());
        }
        let mut own = block.clone();
        if let Value::Object(keys) = &mut own {
            keys.remove(MOUNT_KEY);
        }
        serde_json::from_value::<Self>(own)?.checked()
    }

    pub async fn load(services_root: &Path) -> Result<Self, ConfigError> {
        let path = services_root
            .join("config")
            .join(format!("{CONFIG_PREFIX}.yaml"));
        match tokio::fs::read_to_string(&path).await {
            Ok(text) => Self::parse(&text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Read {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    pub fn inactivity(&self) -> Option<Duration> {
        self.inactive_days
            .map(|days| Duration::days(i64::from(days)))
    }

    const fn checked(self) -> Result<Self, ConfigError> {
        match self.inactive_days {
            Some(0) => Err(ConfigError::ZeroDays),
            _ => Ok(self),
        }
    }
}
