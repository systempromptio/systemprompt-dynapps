//! `capabilities` on the document tools: the agent declares what a live
//! HTML page calls when it creates, imports, edits or updates the page.
//!
//! Core's tools know nothing of it, so the argument is taken off the call
//! before core sees it. A declaration is checked first, as the agent's own
//! person (`LivePages::check`: servers, policy, the tools that person sees);
//! a refused one stops the call before anything is written. Once core's tool
//! has written, the declaration is recorded on the document if it is an HTML
//! page the caller may edit, and the result says what happened. An update
//! that carries nothing but `capabilities` does not reach core.
//! `documents_guide` gains the `live_pages` topic, and the tool list
//! advertises the argument.

use std::sync::Arc;

use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock, JsonObject, Tool};
// JSON: MCP-protocol boundary, tool arguments, schemas and results.
use serde_json::{Value, json};
use systemprompt::identifiers::DocumentId;
use systemprompt_web_admin_platform::artifacts::manifest::{Capabilities, DeclarationRefused};

pub(super) const ARGUMENT: &str = "capabilities";
pub(super) const GUIDE_TOOL: &str = "documents_guide";
pub(super) const GUIDE_TOPIC: &str = "live_pages";
const UPDATE_TOOL: &str = "documents_update";
const DECLARING_TOOLS: [&str; 4] = [
    "documents_create",
    "documents_import",
    "documents_edit",
    UPDATE_TOOL,
];

pub(super) const INSTRUCTIONS_HINT: &str = "A live HTML page, one that calls MCP tools as the person \
viewing it, declares them with `capabilities` on documents_create, documents_import, documents_edit or \
documents_update; read documents_guide topic live_pages before writing one.";

pub(super) const GUIDE: &str = r#"## live_pages
An HTML page can call MCP tools as the person viewing it, through the viewer the Drive serves it in. The page declares what it calls with `capabilities` on documents_create, documents_import, documents_edit or documents_update:
  {"mcp": {"servers": [{"server": "<server>", "tools": ["<tool>", ...]}]}, "artifact": {}, "user": {}}
List only the tools the page calls. Each server must be on this instance and each tool one you can see on it and one the instance's live-pages policy allows; otherwise the declaration is refused and nothing is written. `{}` clears the declaration; leaving `capabilities` out keeps it. Only an HTML page you can edit declares.
In the page, `await window.governance.use("mcp")` resolves the capability (`window.claude.use` is the same object, with the same calls and error codes), or null where it cannot run: a public link, an invitee who has not signed in, any copy outside the viewer. Render the page without it first and light the live parts up when it resolves.
- `mcp.callTool(server, tool, input)` for an action; `mcp.watchTool(server, tool, input, handler)` for data that stays current; `mcp.listTools()`; `mcp.describeTool(server, tool)`.
- Each viewer allows each server once per page. A tool that does not declare readOnlyHint: true asks the viewer to approve every call.
- Branch on the error `code`: approval_required, not_in_manifest, blocked_by_policy, server_not_connected, server_unavailable (retry once), tool_error.
- With `artifact` declared, `(await window.governance.use("artifact")).publish(html)` saves a whole new page as a new version of this document, as the viewer, when they may edit it (`not_writer` otherwise, `conflict` when someone saved first).
- The page has no network and no session of its own: everything live goes through use()."#;

pub(super) fn advertise(mut tools: Vec<Tool>) -> Vec<Tool> {
    for tool in &mut tools {
        if DECLARING_TOOLS.contains(&tool.name.as_ref()) {
            let mut schema = (*tool.input_schema).clone();
            if let Some(Value::Object(properties)) = schema.get_mut("properties") {
                properties.insert(ARGUMENT.to_owned(), schema_of_capabilities());
            }
            tool.input_schema = Arc::new(schema);
        }
    }
    tools
}

// JSON: MCP tool `inputSchema` of the `capabilities` argument; its shape is
// the guide's, so the tool list stays inside its size budget.
fn schema_of_capabilities() -> Value {
    json!({ "type": "object", "description": "Live page. Guide: live_pages." })
}

pub(super) fn refusal(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message)])
}

pub(super) fn refused(refused: &DeclarationRefused) -> CallToolResult {
    refusal(format!(
        "capabilities refused, nothing was written: {refused}"
    ))
}

// JSON: MCP-protocol boundary, the call's arguments.
pub(super) fn take(
    request: &mut CallToolRequestParams,
) -> Result<Option<Capabilities>, DeclarationRefused> {
    if !DECLARING_TOOLS.contains(&request.name.as_ref()) {
        return Ok(None);
    }
    let Some(declared) = request
        .arguments
        .as_mut()
        .and_then(|arguments| arguments.remove(ARGUMENT))
    else {
        return Ok(None);
    };
    Capabilities::parse(&declared).map(Some)
}

pub(super) fn only_declares(request: &CallToolRequestParams) -> bool {
    request.name == UPDATE_TOOL
        && request
            .arguments
            .as_ref()
            .is_some_and(|arguments| arguments.keys().all(|key| key == "document_id"))
}

pub(super) fn named_document(
    request: &CallToolRequestParams,
    result: &CallToolResult,
) -> Option<DocumentId> {
    let from =
        |object: Option<&JsonObject>| object?.get("document_id")?.as_str().map(DocumentId::new);
    from(request.arguments.as_ref()).or_else(|| {
        let structured = result
            .structured_content
            .as_ref()
            .and_then(Value::as_object);
        from(structured).or_else(|| {
            let text = result
                .content
                .first()?
                .as_text()?
                .text
                .lines()
                .last()?
                .to_owned();
            from(serde_json::from_str::<Value>(&text).ok()?.as_object())
        })
    })
}

// JSON: MCP-protocol boundary, the result the agent reads.
pub(super) fn noted(mut result: CallToolResult, recorded: bool, note: &str) -> CallToolResult {
    if let Some(Value::Object(structured)) = result.structured_content.as_mut() {
        structured.insert(
            ARGUMENT.to_owned(),
            json!({ "recorded": recorded, "note": note }),
        );
    }
    result.content.push(ContentBlock::text(note.to_owned()));
    result
}

// JSON: MCP-protocol boundary, the guide call's topics.
pub(super) fn wants_guide(request: &mut CallToolRequestParams) -> Option<bool> {
    if request.name != GUIDE_TOOL {
        return None;
    }
    let topics = request
        .arguments
        .as_mut()?
        .get_mut("topics")?
        .as_array_mut()?;
    let asked = topics.iter().any(|topic| topic == GUIDE_TOPIC);
    if !asked {
        return None;
    }
    topics.retain(|topic| topic != GUIDE_TOPIC);
    Some(topics.is_empty())
}
