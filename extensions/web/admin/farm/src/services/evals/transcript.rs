//! Renders a run's event stream as the plain-text transcript the judge
//! reads, and pulls the final answer (the last assistant text) out of it.

use systemprompt::agent_runs::{RunEvent, RunOutcome};

use crate::types::RunEventPayload;

pub const TRANSCRIPT_CAP_BYTES: usize = 60 * 1024;

#[must_use]
pub fn final_result_text(events: &[RunEvent]) -> Option<String> {
    let payloads: Vec<RunEventPayload> = events.iter().map(|e| e.payload.clone()).collect();
    RunOutcome::last_assistant_text(&payloads)
}

#[must_use]
pub fn render_transcript(prompt: &str, events: &[RunEvent]) -> String {
    let mut out = format!("PROMPT:\n{prompt}\n\n");
    for event in events {
        let line = match &event.payload {
            RunEventPayload::AssistantText { text } => format!("ASSISTANT:\n{text}\n\n"),
            RunEventPayload::ToolUse { name, .. } => format!("TOOL: {name}\n"),
            _ => continue,
        };
        out.push_str(&line);
    }
    if let Some(text) = final_result_text(events) {
        out.push_str(&format!("\nFINAL RESULT:\n{text}\n"));
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
