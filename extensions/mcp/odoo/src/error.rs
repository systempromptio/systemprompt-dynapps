//! Error type for the `odoo` MCP server.

use axum::http::StatusCode;
use systemprompt::traits::ExtensionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OdooError {
    #[error("Odoo is not configured: {0}")]
    NotConfigured(String),

    #[error("No Odoo credential: {0}")]
    NotLinked(String),

    #[error("Odoo app not installed: {0}")]
    AppMissing(String),

    #[error("Odoo denied access: {0}")]
    AccessDenied(String),

    #[error("Odoo rejected the call: {0}")]
    Odoo(String),

    #[error("Could not resolve an Odoo record: {0}")]
    Unresolved(String),

    #[error("Odoo transport error: {0}")]
    Transport(String),

    #[error("Failed to build the HTTP client: {0}")]
    HttpClient(#[source] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ExtensionError for OdooError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured(_) => "NOT_CONFIGURED",
            Self::NotLinked(_) => "NOT_LINKED",
            Self::AppMissing(_) => "APP_NOT_INSTALLED",
            Self::AccessDenied(_) => "ODOO_ACCESS_DENIED",
            Self::Odoo(_) => "ODOO_REJECTED",
            Self::Unresolved(_) => "ODOO_UNRESOLVED",
            Self::Transport(_) => "UPSTREAM_UNAVAILABLE",
            Self::HttpClient(_) => "HTTP_CLIENT_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotConfigured(_) | Self::Transport(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotLinked(_) | Self::AccessDenied(_) => StatusCode::FORBIDDEN,
            Self::AppMissing(_) => StatusCode::NOT_IMPLEMENTED,
            Self::Odoo(_) => StatusCode::BAD_GATEWAY,
            Self::Unresolved(_) => StatusCode::BAD_REQUEST,
            Self::HttpClient(_) | Self::Serialization(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }

    // Why: a transport failure is the only variant where the same call might
    // succeed unchanged; everything else needs the caller, the credential, or
    // the deployment to change first.
    fn is_retryable(&self) -> bool {
        matches!(self, Self::Transport(_))
    }
}

impl From<OdooError> for rmcp::ErrorData {
    fn from(err: OdooError) -> Self {
        match err {
            // Why: MCP clients show these four to the caller as the remedy —
            // attaching a credential, installing an app, or being granted
            // rights — so they must not be flattened into an internal error.
            OdooError::NotLinked(msg)
            | OdooError::NotConfigured(msg)
            | OdooError::AppMissing(msg)
            | OdooError::AccessDenied(msg) => Self::invalid_request(msg, None),
            // Why: MCP invalid_params keeps the list of what would have matched
            // in front of the model; a name that matched nothing is a bad
            // argument.
            OdooError::Unresolved(msg) => Self::invalid_params(msg, None),
            other => Self::internal_error(other.to_string(), None),
        }
    }
}
