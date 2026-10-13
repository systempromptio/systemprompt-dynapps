//! The `secrets.external` block for `kind: onepassword`, parsed from the keys
//! core leaves in `ExternalSecretsConfig::config`.
//!
//! The token is never configuration: the block names the environment variable
//! that holds it (`token_env`), and a literal `token` key is refused outright
//! so a profile can never carry one.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use systemprompt::config::ExternalSecretsConfig;

use crate::error::OnePasswordError;
use crate::reference::OpReference;

pub const DEFAULT_OP_BIN: &str = "op";
pub const DEFAULT_CLI_TOKEN_ENV: &str = "OP_SERVICE_ACCOUNT_TOKEN";
pub const DEFAULT_CONNECT_TOKEN_ENV: &str = "OP_CONNECT_TOKEN";
pub const DEFAULT_TIMEOUT_SECS: u64 = 10;
pub const MAX_TIMEOUT_SECS: u64 = 120;
pub const DEFAULT_RETRIES: u32 = 3;
pub const MAX_RETRIES: u32 = 10;

const FORBIDDEN_KEYS: [&str; 2] = ["token", "connect_token"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Cli,
    Connect,
}

impl BackendKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Connect => "connect",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mapping {
    Items(BTreeMap<String, OpReference>),
    SingleItem { item: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnePasswordConfig {
    pub backend: BackendKind,
    pub op_bin: String,
    pub connect_url: Option<String>,
    pub token_env: String,
    pub vault: String,
    pub mapping: Mapping,
    pub timeout_secs: u64,
    pub retries: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    backend: Option<BackendKind>,
    #[serde(default)]
    op_bin: Option<String>,
    #[serde(default)]
    connect_url: Option<String>,
    #[serde(default)]
    token_env: Option<String>,
    vault: String,
    #[serde(default)]
    items: Option<BTreeMap<String, String>>,
    #[serde(default)]
    item: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
    #[serde(default)]
    retries: Option<u32>,
}

fn invalid(msg: impl Into<String>) -> OnePasswordError {
    OnePasswordError::Config(msg.into())
}

impl OnePasswordConfig {
    pub fn from_external(cfg: &ExternalSecretsConfig) -> Result<Self, OnePasswordError> {
        Self::from_map(&cfg.config)
    }

    // JSON: core hands an external secrets provider its profile block as an
    // opaque map; it is typed here, at the boundary, through `RawConfig`.
    pub fn from_map(
        map: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Self, OnePasswordError> {
        if let Some(key) = FORBIDDEN_KEYS.iter().find(|k| map.contains_key(**k)) {
            return Err(invalid(format!(
                "'{key}' is not accepted in the profile; put the token in the environment \
                 variable named by token_env"
            )));
        }
        let raw: RawConfig = serde_json::from_value(serde_json::Value::Object(map.clone()))
            .map_err(|e| invalid(e.to_string()))?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawConfig) -> Result<Self, OnePasswordError> {
        let backend = raw.backend.unwrap_or(BackendKind::Cli);
        let vault = non_empty("vault", raw.vault)?;
        let connect_url = match (backend, raw.connect_url) {
            (BackendKind::Connect, Some(url)) => Some(validate_url(url)?),
            (BackendKind::Connect, None) => {
                return Err(invalid("backend: connect requires connect_url"));
            },
            (BackendKind::Cli, Some(_)) => {
                return Err(invalid("connect_url is only valid with backend: connect"));
            },
            (BackendKind::Cli, None) => None,
        };
        let token_env = match raw.token_env {
            Some(name) => non_empty("token_env", name)?,
            None => match backend {
                BackendKind::Cli => DEFAULT_CLI_TOKEN_ENV.to_owned(),
                BackendKind::Connect => DEFAULT_CONNECT_TOKEN_ENV.to_owned(),
            },
        };
        let op_bin = non_empty(
            "op_bin",
            raw.op_bin.unwrap_or_else(|| DEFAULT_OP_BIN.to_owned()),
        )?;
        let timeout_secs = raw.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
        if !(1..=MAX_TIMEOUT_SECS).contains(&timeout_secs) {
            return Err(invalid(format!(
                "timeout_secs must be between 1 and {MAX_TIMEOUT_SECS}"
            )));
        }
        let retries = raw.retries.unwrap_or(DEFAULT_RETRIES);
        if retries > MAX_RETRIES {
            return Err(invalid(format!("retries must be at most {MAX_RETRIES}")));
        }
        let mapping = mapping(raw.items, raw.item)?;
        Ok(Self {
            backend,
            op_bin,
            connect_url,
            token_env,
            vault,
            mapping,
            timeout_secs,
            retries,
        })
    }

    #[must_use]
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

fn mapping(
    items: Option<BTreeMap<String, String>>,
    item: Option<String>,
) -> Result<Mapping, OnePasswordError> {
    match (items, item) {
        (Some(_), Some(_)) => Err(invalid("set either items or item, not both")),
        (None, None) => Err(invalid("one of items or item is required")),
        (None, Some(item)) => Ok(Mapping::SingleItem {
            item: non_empty("item", item)?,
        }),
        (Some(items), None) => {
            if items.is_empty() {
                return Err(invalid("items is empty"));
            }
            let mut parsed = BTreeMap::new();
            for (key, raw) in items {
                if key.trim().is_empty() {
                    return Err(invalid("items has an empty key"));
                }
                parsed.insert(key, OpReference::parse(&raw)?);
            }
            Ok(Mapping::Items(parsed))
        },
    }
}

fn non_empty(field: &str, value: String) -> Result<String, OnePasswordError> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} must not be empty")));
    }
    Ok(value)
}

fn validate_url(url: String) -> Result<String, OnePasswordError> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(invalid("connect_url must be an http:// or https:// URL"));
    }
    Ok(url.trim_end_matches('/').to_owned())
}
