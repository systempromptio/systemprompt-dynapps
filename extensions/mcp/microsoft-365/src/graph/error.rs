//! What can go wrong reaching Graph, and how each failure reads to the model.
//!
//! Messages never carry the request URL: Graph paths can name a mailbox or a
//! person, and reqwest errors are stripped of theirs before they get here.

use rmcp::ErrorData;
use serde::Deserialize;
use systemprompt::mcp::McpDomainError;
use systemprompt::models::net::OutboundUrlError;
use systemprompt_mcp_shared::reply;

const MAX_MESSAGE_CHARS: usize = 500;
const GENERIC_FAILURE: &str = "Graph API request failed";

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Validation error: {0}")]
    Invalid(String),
    #[error(
        "Microsoft 365 is not connected for this user: connect it on the Connectors page, then retry"
    )]
    NotConnected,
    #[error(
        "Microsoft 365 refused the access token: reconnect Microsoft 365 on the Connectors page"
    )]
    Unauthorized,
    #[error("Microsoft 365 credentials are unavailable: {0}")]
    Credentials(String),
    #[error("Microsoft Graph is throttling this user; retry after {retry_after_secs}s")]
    Throttled { retry_after_secs: u64 },
    #[error("Graph API error {status}: {message}")]
    Api {
        status: u16,
        code: Option<String>,
        message: String,
    },
    #[error("Graph response exceeds the {limit}-byte limit")]
    TooLarge { limit: usize },
    #[error("refusing a URL outside the configured Graph endpoint")]
    OffEndpoint,
    #[error("Graph request failed: {0}")]
    Transport(#[source] reqwest::Error),
    #[error("Graph returned a body that is not JSON: {0}")]
    Decode(#[source] serde_json::Error),
    #[error("Microsoft 365 server misconfigured: {0}")]
    Config(String),
    #[error("Microsoft 365 server misconfigured: graph url is not admissible: {0}")]
    Endpoint(#[from] OutboundUrlError),
    #[error("Validation error: graph path is not a valid URL: {0}")]
    Url(#[from] url::ParseError),
}

impl GraphError {
    pub fn api(status: u16, code: Option<String>, message: &str) -> Self {
        Self::Api {
            status,
            code,
            message: message.chars().take(MAX_MESSAGE_CHARS).collect(),
        }
    }

    pub fn rejects_option(&self, markers: &[&str]) -> bool {
        match self {
            Self::Api {
                status: 400,
                message,
                ..
            } => {
                let lower = message.to_lowercase();
                markers.iter().any(|marker| lower.contains(marker))
            },
            _ => false,
        }
    }

    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Invalid(_) | Self::Url(_) => "invalid",
            Self::NotConnected => "not_connected",
            Self::Unauthorized => "unauthorized",
            Self::Credentials(_) => "credentials",
            Self::Throttled { .. } => "throttled",
            Self::Api { .. } => "api",
            Self::TooLarge { .. } => "too_large",
            Self::OffEndpoint => "off_endpoint",
            Self::Transport(_) => "transport",
            Self::Decode(_) => "decode",
            Self::Config(_) | Self::Endpoint(_) => "config",
        }
    }
}

impl From<reqwest::Error> for GraphError {
    fn from(error: reqwest::Error) -> Self {
        Self::Transport(error.without_url())
    }
}

impl From<serde_json::Error> for GraphError {
    fn from(error: serde_json::Error) -> Self {
        Self::Decode(error)
    }
}

impl From<McpDomainError> for GraphError {
    fn from(error: McpDomainError) -> Self {
        match error {
            McpDomainError::ExternalAccountNotConnected { .. } => Self::NotConnected,
            other => Self::Credentials(other.to_string()),
        }
    }
}

impl From<GraphError> for ErrorData {
    fn from(error: GraphError) -> Self {
        let text = error.to_string();
        match error {
            GraphError::Invalid(_) | GraphError::Url(_) | GraphError::OffEndpoint => {
                reply::invalid(text)
            },
            GraphError::NotConnected
            | GraphError::Unauthorized
            | GraphError::Api { status: 403, .. } => reply::forbidden(text),
            GraphError::Api { status: 404, .. } => reply::not_found(text),
            GraphError::Api { status: 400, .. } => reply::invalid(text),
            GraphError::Credentials(_)
            | GraphError::Throttled { .. }
            | GraphError::Api { .. }
            | GraphError::TooLarge { .. }
            | GraphError::Transport(_)
            | GraphError::Decode(_)
            | GraphError::Config(_)
            | GraphError::Endpoint(_) => reply::failure(text),
        }
    }
}

#[derive(Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Deserialize)]
struct ErrorBody {
    code: Option<String>,
    message: Option<String>,
}

pub(super) fn api_error(status: u16, body: &[u8]) -> GraphError {
    match serde_json::from_slice::<ErrorEnvelope>(body) {
        Ok(envelope) => GraphError::api(
            status,
            envelope.error.code,
            envelope.error.message.as_deref().unwrap_or(GENERIC_FAILURE),
        ),
        Err(error) => {
            tracing::debug!(%error, status, "Graph error body is not an OData error");
            GraphError::api(status, None, GENERIC_FAILURE)
        },
    }
}
