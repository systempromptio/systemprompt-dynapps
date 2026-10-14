//! The four MCP error replies a tool hands back, so every bundled server
//! spells a refusal the same way and a client can tell a bad argument from
//! a missing row from a denied one.

use rmcp::ErrorData;

pub fn invalid(message: impl Into<String>) -> ErrorData {
    ErrorData::invalid_params(message.into(), None)
}

pub fn failure(message: impl Into<String>) -> ErrorData {
    ErrorData::internal_error(message.into(), None)
}

pub fn not_found(message: impl Into<String>) -> ErrorData {
    ErrorData::resource_not_found(message.into(), None)
}

// Why: MCP has no 403; `invalid_params` is the code clients surface verbatim
// to the model, which is where a scope refusal has to be read.
pub fn forbidden(message: impl Into<String>) -> ErrorData {
    ErrorData::invalid_params(format!("Forbidden: {}", message.into()), None)
}
