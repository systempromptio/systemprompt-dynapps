//! Failures of the 1Password source. Every variant names the vault, item,
//! field or status that failed and never a secret value or the token.

use systemprompt::config::SecretsBootstrapError;

pub const KIND: &str = "onepassword";

const MAX_STDERR_CHARS: usize = 200;

#[derive(Debug, thiserror::Error)]
pub enum OnePasswordError {
    #[error("invalid onepassword config: {0}")]
    Config(String),

    #[error("token environment variable '{env}' is unset or empty")]
    TokenMissing { env: String },

    #[error("vault '{0}' not found")]
    VaultNotFound(String),

    #[error("item '{item}' not found in vault '{vault}'")]
    ItemNotFound { vault: String, item: String },

    #[error(
        "field '{field}' not found on item '{item}' (only CONCEALED and STRING fields are read)"
    )]
    FieldNotFound { item: String, field: String },

    #[error("'{name}' matches more than one {what}; reference it by id")]
    AmbiguousItem { what: &'static str, name: String },

    #[error("op exited with {status}: {stderr_excerpt}")]
    Cli {
        status: String,
        stderr_excerpt: String,
    },

    #[error("could not run op: {0}")]
    Spawn(String),

    #[error("connect request failed: {0}")]
    Http(String),

    #[error("connect answered HTTP {code} for {path}")]
    Status { code: u16, path: String },

    #[error("unreadable response: {0}")]
    InvalidResponse(String),

    #[error("timed out after {0} s")]
    Timeout(u64),

    #[error("this backend cannot read single references")]
    Unsupported,
}

impl OnePasswordError {
    #[must_use]
    pub fn cli(status: impl Into<String>, stderr: &[u8]) -> Self {
        let text = String::from_utf8_lossy(stderr);
        Self::Cli {
            status: status.into(),
            stderr_excerpt: text.trim().chars().take(MAX_STDERR_CHARS).collect(),
        }
    }

    #[must_use]
    pub const fn allows_reference_fallback(&self) -> bool {
        matches!(
            self,
            Self::Cli { .. } | Self::ItemNotFound { .. } | Self::InvalidResponse(_)
        )
    }
}

impl From<OnePasswordError> for SecretsBootstrapError {
    fn from(error: OnePasswordError) -> Self {
        Self::External {
            kind: KIND.to_owned(),
            source: Box::new(error),
        }
    }
}
