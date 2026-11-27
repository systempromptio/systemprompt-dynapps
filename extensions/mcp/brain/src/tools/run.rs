//! From a tool call to the operation of the Brain and back to its reply.
//!
//! The tool's name is the operation's; its arguments go to
//! [`dynbrain::brain::Brain::run`] as they came, so a native instance reads
//! them into the operation's arguments and a proxy one passes them to its
//! source.

use dynbrain::brain::Brain;
use dynbrain::brain::operation::Operation;
use rmcp::ErrorData as McpError;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::reply;

use super::Json;
use super::fault::{tool_error, undeclared_filter};

// Why: the person is the one the authenticated request names, never one an
// argument names; a call that carries none is refused before any operation.
pub async fn run(
    brain: &dyn Brain,
    context: &RequestContext,
    name: &str,
    arguments: Json,
) -> Result<Json, McpError> {
    let caller = &context
        .user
        .as_ref()
        .ok_or_else(|| reply::forbidden("a signed-in person is required"))?
        .id;
    let operation = Operation::from_tool(name)
        .ok_or_else(|| reply::invalid(format!("Unknown tool: {name}")))?;
    let operation = first_upload_call(operation, &arguments);
    match brain.run(caller, context, operation, arguments).await {
        Ok(reply) => Ok(reply),
        Err(fault) if operation == Operation::Search => undeclared_filter(fault, "results"),
        Err(fault) if operation == Operation::List => undeclared_filter(fault, "documents"),
        Err(fault) => Err(tool_error(&fault)),
    }
}

// Why: the file contract is two calls to one tool; the first one, with no
// `upload_id`, is the request for the URL the bytes are sent to.
fn first_upload_call(operation: Operation, arguments: &Json) -> Operation {
    let has_upload = arguments
        .get("upload_id")
        .is_some_and(|upload| !upload.is_null());
    if operation == Operation::AssetUpload && !has_upload {
        return Operation::RequestUpload;
    }
    operation
}
