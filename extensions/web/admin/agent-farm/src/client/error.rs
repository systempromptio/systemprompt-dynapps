//! What a farm-api call can fail with. No variant carries the token.

use systemprompt::models::net::OutboundUrlError;
use thiserror::Error;

use crate::models::ApiErrorBody;

#[derive(Debug, Error)]
pub enum FarmApiError {
    #[error("the console has no farm-api credential: set api_url in {} and the secret it names", crate::config::CONFIG_FILE)]
    NotConfigured,
    #[error("farm-api URL {url} refused: {source}")]
    BadUrl {
        url: String,
        #[source]
        source: OutboundUrlError,
    },
    #[error("farm-api did not answer: {0}")]
    Unreachable(#[source] reqwest::Error),
    #[error("farm-api answered {status}{}: {message}", code.as_deref().map(|c| format!(" {c}")).unwrap_or_default())]
    Status {
        status: u16,
        code: Option<String>,
        message: String,
    },
    #[error("farm-api sent a response the console could not read: {0}")]
    Decode(#[source] serde_json::Error),
    #[error("not a farm task id: {0}")]
    BadTaskId(String),
    #[error("not an agent slug: {0}")]
    BadSlug(String),
}

impl FarmApiError {
    /// The farm-api error code (`task_has_no_result`, `not_found`, …), if it
    /// answered with one.
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Status { code, .. } => code.as_deref(),
            _ => None,
        }
    }
}

/// The error a non-2xx answer becomes, with farm-api's code when its body
/// carries one.
pub fn status_error(status: u16, body: &[u8]) -> FarmApiError {
    match serde_json::from_slice::<ApiErrorBody>(body) {
        Ok(parsed) => FarmApiError::Status {
            status,
            code: Some(parsed.error.code),
            message: parsed.error.message,
        },
        Err(_) => FarmApiError::Status {
            status,
            code: None,
            message: "no error body".to_owned(),
        },
    }
}
