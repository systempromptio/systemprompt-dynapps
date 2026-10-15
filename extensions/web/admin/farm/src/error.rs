//! The farm domain's error type and its mapping onto the admin plane's
//! [`AdminError`], so handlers propagate with a bare `?`.

use systemprompt::agent_runs::AgentRunsError;
use systemprompt::identifiers::PluginId;
use systemprompt::identifiers::error::IdValidationError;
use systemprompt::marketplace::managed::ManagedError;
use systemprompt_web_admin_core::error::AdminError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FarmError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid: {0}")]
    Validation(String),

    #[error("Invalid: {at}: regex {pattern:?}: {source}")]
    InvalidRegex {
        at: String,
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error("Invalid: {path}: is not UTF-8: {source}")]
    NotUtf8 {
        path: String,
        #[source]
        source: std::str::Utf8Error,
    },

    #[error("Git source: {0}")]
    Git(String),

    #[error("Spawn: {0}")]
    Spawn(String),

    #[error("Git source: {source_ref}: {source}")]
    GitCapture {
        source_ref: String,
        #[source]
        source: ManagedError,
    },

    #[error("Task: {step}: {source}")]
    Task {
        step: &'static str,
        #[source]
        source: tokio::task::JoinError,
    },

    #[error("Gateway: {0}")]
    Gateway(String),

    #[error("JSON: {at}: {source}")]
    Json {
        at: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Config: plugin {plugin_id}: {path}: {source}")]
    PluginConfigRead {
        plugin_id: PluginId,
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Config: plugin {plugin_id}: {source}")]
    PluginConfigParse {
        plugin_id: PluginId,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Config: {0}")]
    Config(String),

    #[error("Runner: {0}")]
    Runner(String),

    #[error("Runner: {at}: {source}")]
    RunnerHttp {
        at: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Runner bundle file {path:?}: {source}")]
    BundleDecode {
        path: String,
        #[source]
        source: base64::DecodeError,
    },

    #[error("Agent run: {0}")]
    AgentRuns(#[from] AgentRunsError),

    #[error("Agent run: agent {agent}: {source}")]
    AgentHost {
        agent: String,
        #[source]
        source: Box<AgentRunsError>,
    },

    #[error("Invalid: {at}: {source}")]
    Identifier {
        at: &'static str,
        #[source]
        source: Box<IdValidationError>,
    },
}

pub type FarmResult<T> = Result<T, FarmError>;

impl From<FarmError> for AdminError {
    fn from(err: FarmError) -> Self {
        match err {
            FarmError::NotFound(msg) => Self::NotFound(msg),
            FarmError::Forbidden(msg) => Self::Forbidden(msg),
            FarmError::Validation(msg) => Self::Unprocessable(msg),
            err @ (FarmError::InvalidRegex { .. } | FarmError::NotUtf8 { .. }) => {
                Self::Unprocessable(err.to_string())
            },
            other => Self::Internal(Box::new(other)),
        }
    }
}
