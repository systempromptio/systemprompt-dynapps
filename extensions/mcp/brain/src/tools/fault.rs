//! What a refused operation becomes: a tool error, or the empty reply with its
//! `error` that the `docs_*` search and list give for a filter naming a
//! property no space declares.

use dynbrain::brain::fault::{BrainFault, SourceFault};
use dynbrain::properties::PropertyError;
use rmcp::ErrorData as McpError;
use serde_json::json;
use systemprompt_mcp_shared::reply;

use super::Json;

pub fn tool_error(fault: &BrainFault) -> McpError {
    match fault {
        BrainFault::NotFound => reply::not_found("not found"),
        BrainFault::Forbidden | BrainFault::Source(SourceFault::Refused(_)) => {
            reply::forbidden(fault.to_string())
        },
        BrainFault::Invalid(_)
        | BrainFault::Refused(_)
        | BrainFault::Property(_)
        | BrainFault::TooLarge(_)
        | BrainFault::Arguments(_)
        | BrainFault::ManagedAtSource
        | BrainFault::Source(SourceFault::Invalid(_)) => reply::invalid(fault.to_string()),
        BrainFault::Unavailable(message) => reply::failure(*message),
        BrainFault::Source(SourceFault::NotFound(message)) => reply::not_found(message.clone()),
        BrainFault::Source(SourceFault::Unavailable(_)) => reply::failure(fault.to_string()),
        BrainFault::Encode(source) => {
            tracing::error!(error = %source, "a Brain reply could not be encoded");
            reply::failure(fault.to_string())
        },
        BrainFault::Internal(source) => {
            tracing::error!(error = %source, "the Brain's store failed");
            reply::failure(fault.to_string())
        },
    }
}

// Why: `docs_search` and `docs_list` answer a `where` that names a property
// nobody declares with `{error, <rows>: [], total: 0}`, not with a tool error;
// every other fault is a tool error.
pub fn undeclared_filter(fault: BrainFault, rows: &str) -> Result<Json, McpError> {
    match fault {
        BrainFault::Property(error @ PropertyError::NotDeclared { .. }) => {
            Ok(json!({"error": error.to_string(), rows: [], "total": 0}))
        },
        other => Err(tool_error(&other)),
    }
}
