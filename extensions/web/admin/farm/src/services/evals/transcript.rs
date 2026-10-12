//! Renders a run's event stream as the plain-text transcript the judge
//! reads, and pulls the final result text out of it.

use crate::repositories::rows::RunEventRow;
use crate::types::RunEventPayload;

pub const TRANSCRIPT_CAP_BYTES: usize = 60 * 1024;

fn decode(events: &[RunEventRow]) -> impl Iterator<Item = RunEventPayload> + '_ {
    events
        .iter()
        .filter_map(|e| serde_json::from_value::<RunEventPayload>(e.payload.clone()).ok())
}

#[must_use]
pub fn final_result_text(events: &[RunEventRow]) -> Option<String> {
    decode(events)
        .filter_map(|p| match p {
            RunEventPayload::Result { text, .. } => text,
            _ => None,
        })
        .last()
}

#[must_use]
pub fn render_transcript(prompt: &str, events: &[RunEventRow]) -> String {
    let mut out = format!("PROMPT:\n{prompt}\n\n");
    for payload in decode(events) {
        let line = match payload {
            RunEventPayload::AssistantText { text } => format!("ASSISTANT:\n{text}\n\n"),
            RunEventPayload::ToolUse { name, .. } => format!("TOOL: {name}\n"),
            RunEventPayload::Result {
                text: Some(text), ..
            } => format!("\nFINAL RESULT:\n{text}\n"),
            _ => continue,
        };
        out.push_str(&line);
    }
    truncate_utf8(out, TRANSCRIPT_CAP_BYTES)
}

#[must_use]
pub fn truncate_utf8(mut text: String, cap: usize) -> String {
    if text.len() <= cap {
        return text;
    }
    let mut end = cap;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    text.push_str("\n[truncated]");
    text
}
