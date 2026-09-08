//! Error type for the knowledge-bank MCP server.

use axum::http::StatusCode;
use systemprompt::traits::ExtensionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KnowledgeBankError {
    #[error("Knowledge backend not configured: {0}")]
    NotConfigured(String),

    #[error("Invalid knowledge-bank configuration: {0}")]
    Config(String),

    #[error("Unknown knowledge source: {0}")]
    UnknownSource(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error(
        "Atlassian is not available to this sync: {0}. If your Atlassian account is not linked yet, connect it at /admin/profile#connected-accounts"
    )]
    AtlassianUnavailable(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ExtensionError for KnowledgeBankError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured(_) => "BACKEND_NOT_CONFIGURED",
            Self::Config(_) => "INVALID_CONFIG",
            Self::UnknownSource(_) => "UNKNOWN_SOURCE",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::AtlassianUnavailable(_) => "ATLASSIAN_UNAVAILABLE",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::Backend(_) => "BACKEND_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Http(_) => "HTTP_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::UnknownSource(_) => StatusCode::NOT_FOUND,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::NotConfigured(_) | Self::AtlassianUnavailable(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            },
            Self::Config(_)
            | Self::Backend(_)
            | Self::Database(_)
            | Self::Http(_)
            | Self::Serialization(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Backend(_) | Self::Database(_) | Self::Http(_) | Self::AtlassianUnavailable(_)
        )
    }
}
