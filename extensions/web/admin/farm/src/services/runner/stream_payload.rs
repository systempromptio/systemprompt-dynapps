//! Maps decoded stream-json events onto the stored [`RunEventPayload`],
//! capping free text at 64 KiB so one verbose tool result cannot bloat the
//! events table.

use super::stream_json::StreamEvent;
use crate::types::RunEventPayload;

pub const TEXT_CAP_BYTES: usize = 64 * 1024;

#[must_use]
pub fn cap_text(text: &str) -> String {
    if text.len() <= TEXT_CAP_BYTES {
        return text.to_owned();
    }
    let mut end = TEXT_CAP_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…[truncated {} bytes]", &text[..end], text.len() - end)
}

#[must_use]
pub fn to_payload(event: &StreamEvent) -> RunEventPayload {
    match event {
        StreamEvent::Init {
            session_id,
            model,
            tools,
            cwd,
        } => RunEventPayload::Init {
            session_id: session_id.clone(),
            model: model.clone(),
            tools: tools.clone(),
            cwd: cwd.clone(),
        },
        StreamEvent::Hook { name, outcome } => RunEventPayload::Hook {
            name: name.clone(),
            decision: outcome.clone(),
        },
        StreamEvent::AssistantText(text) => RunEventPayload::AssistantText {
            text: cap_text(text),
        },
        StreamEvent::ToolUse { id, name, input } => RunEventPayload::ToolUse {
            tool_use_id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        },
        StreamEvent::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => RunEventPayload::ToolResult {
            tool_use_id: tool_use_id.clone(),
            content: cap_text(content),
            is_error: *is_error,
        },
        StreamEvent::Result(summary) => RunEventPayload::Result {
            subtype: summary.subtype.clone().unwrap_or_default(),
            is_error: summary.is_error.unwrap_or(false),
            num_turns: summary.num_turns.unwrap_or(0),
            duration_ms: summary.duration_ms.unwrap_or(0),
            cost_usd: summary.total_cost_usd,
            text: summary.result.as_deref().map(cap_text),
        },
    }
}
