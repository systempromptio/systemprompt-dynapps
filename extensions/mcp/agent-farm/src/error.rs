//! The server's own error type.
//!
//! Every fallible step inside a tool carries
//! its typed cause here, and the one `From` onto rmcp's `ErrorData` decides
//! how each kind is reported to the client: a bad argument, a missing row, a
//! refused scope, or an internal failure that keeps its source.

use rmcp::ErrorData;
use systemprompt::agent_runs::AgentRunsError;
use systemprompt::identifiers::error::IdValidationError;
use systemprompt_mcp_shared::reply::{failure, forbidden, invalid, not_found};
use systemprompt_web_shared::error::MarketplaceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Failure(String),

    #[error("database read failed: {0}")]
    Db(#[from] sqlx::Error),

    #[error("agent run: {0}")]
    AgentRuns(#[from] AgentRunsError),

    #[error("invalid identifier: {0}")]
    Identifier(#[from] IdValidationError),

    #[error("catalog: {0}")]
    Catalog(#[from] MarketplaceError),

    #[error("serialisation: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "farm")]
    #[error("farm: {0}")]
    Farm(#[from] systemprompt_web_admin_farm::error::FarmError),

    #[error("authorization read: {0}")]
    Authz(#[from] systemprompt_web_admin_core::authz::AuthzReadError),

    #[error("{}", .0.message)]
    Mcp(ErrorData),
}

pub type ToolResult<T> = Result<T, ToolError>;

impl From<ErrorData> for ToolError {
    fn from(error: ErrorData) -> Self {
        Self::Mcp(error)
    }
}

impl From<ToolError> for ErrorData {
    fn from(error: ToolError) -> Self {
        match error {
            ToolError::Invalid(msg) => invalid(msg),
            ToolError::Forbidden(msg) => forbidden(msg),
            ToolError::NotFound(msg) => not_found(msg),
            ToolError::Identifier(_) => invalid(error.to_string()),
            #[cfg(feature = "farm")]
            ToolError::Farm(farm) => farm_error(farm),
            ToolError::Mcp(inner) => inner,
            ToolError::Failure(_)
            | ToolError::Db(_)
            | ToolError::AgentRuns(_)
            | ToolError::Catalog(_)
            | ToolError::Authz(_)
            | ToolError::Json(_) => failure(error.to_string()),
        }
    }
}

#[cfg(feature = "farm")]
fn farm_error(error: systemprompt_web_admin_farm::error::FarmError) -> ErrorData {
    use systemprompt_web_admin_farm::error::FarmError;
    match error {
        FarmError::NotFound(msg) => not_found(msg),
        FarmError::Forbidden(msg) => forbidden(msg),
        FarmError::Validation(msg) => invalid(msg),
        other => failure(other.to_string()),
    }
}
