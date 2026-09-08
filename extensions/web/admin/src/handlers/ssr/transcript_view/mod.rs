//! The transcript view-models, shared by the admin context-detail page and the
//! owner-facing conversation page under `/admin/history`.
//!
//! [`build_conversation`] is the reader's model: one turn per human prompt,
//! assembled from the latest request's stored history so a ten-turn
//! conversation renders ten turns rather than the fifty-five messages the
//! gateway persisted. [`build_transcript`] is the older flat, per-request
//! interleaving. Both differ only in how text is treated on the way out, which
//! is what [`TranscriptOptions`] carries: the owner-facing page strips the
//! gateway's `=== USER PROMPT ===` framing and runs every body through the
//! credential redactor, the admin page shows the stored text as it is.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use serde::Serialize;
// JSON: per-tool payloads pretty-printed for the transcript view.
use serde_json::Value;
use systemprompt::identifiers::AiRequestId;

use crate::handlers::ssr::entity_urls::request_detail_url;
use crate::handlers::ssr::format::format_cost;
use crate::repositories::analytics::context_detail::{
    ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};
use crate::repositories::analytics::conversations::{redact_text, strip_gateway_markers};

mod conversation;
mod markers;
mod thread;

pub use conversation::{
    ConversationView, SideCallRowView, SideCallsView, StepView, ThreadView, ToolChipView,
    TurnView, build_conversation,
};
pub use markers::{ParsedAssistant, ToolUseMarker, parse_assistant};

const TRANSCRIPT_PREVIEW_CHARS: usize = 4000;

#[derive(Debug, Serialize)]
pub(crate) struct TranscriptEntryView {
    // Why: display DTO; request id carried as string from the transcript grouping key
    pub(crate) request_id: AiRequestId,
    pub(crate) request_id_short: String,
    pub(crate) request_url: String,
    // Why: Telemetry for the request that carried this turn. `None` when the
    // transcript row outlives its `ai_requests` row (retention trims the
    // request rollup before the messages), so the turn still renders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) meta: Option<TranscriptMetaView>,
    pub(crate) ts_local: String,
    pub(crate) ts_full: String,
    pub(crate) kind: &'static str,
    pub(crate) role: String,
    pub(crate) is_user: bool,
    pub(crate) is_assistant: bool,
    pub(crate) is_system: bool,
    pub(crate) is_tool: bool,
    pub(crate) content_preview: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_input_pretty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_result_pretty: Option<String>,
}

// Why: The per-turn telemetry rail: how the turn was served and what it cost.
// Cloned onto every turn of the same request, so all of a request's messages
// state the same numbers rather than only the last one.
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptMetaView {
    pub model: String,
    pub status: String,
    pub is_error: bool,
    pub latency_display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_display: Option<String>,
    pub cost_display: String,
}

// Why: how message bodies are treated on the way into the view.
#[derive(Debug, Clone, Copy, Default)]
pub struct TranscriptOptions {
    // Why: cut the gateway's `=== USER PROMPT ===` /
    // Why: `=== ASSISTANT ANSWER ===` framing out of user turns before display.
    pub strip_markers: bool,
    // Why: run every body through the credential redactor.
    pub redact: bool,
}

impl TranscriptOptions {
    // Why: what the owner-facing conversation page uses.
    pub const fn owner_facing() -> Self {
        Self {
            strip_markers: true,
            redact: true,
        }
    }
}

// Why: the transcript plus how many credential redactions were applied
// Why: building it, so a page can say that redaction occurred rather than
// Why: silently altering the text.
pub(crate) struct BuiltTranscript {
    pub(crate) entries: Vec<TranscriptEntryView>,
    pub(crate) redaction_count: u32,
}

#[derive(Clone)]
struct Entry {
    seq: i32,
    ts: DateTime<Utc>,
    kind: &'static str,
    role: String,
    content: String,
    tool_name: Option<String>,
    // JSON: MCP tool arguments and results are per-tool shaped and only ever
    // JSON: pretty-printed for display
    tool_input: Option<Value>,
    tool_result: Option<Value>,
}

pub(crate) fn build_transcript(
    messages: &[ContextMessageRow],
    tool_calls: &[ContextToolCallRow],
    requests: &[ContextRequestRow],
    opts: TranscriptOptions,
) -> BuiltTranscript {
    let mut by_request: BTreeMap<(DateTime<Utc>, String), Vec<Entry>> = BTreeMap::new();

    for m in messages {
        by_request
            .entry((m.created_at, m.request_id.as_str().to_owned()))
            .or_default()
            .push(Entry {
                seq: m.sequence_number,
                ts: m.created_at,
                kind: "message",
                role: m.role.clone(),
                content: m.content.clone(),
                tool_name: None,
                tool_input: None,
                tool_result: None,
            });
    }
    for t in tool_calls {
        by_request
            .entry((t.created_at, t.request_id.as_str().to_owned()))
            .or_default()
            .push(Entry {
                seq: t.sequence_number,
                ts: t.created_at,
                kind: "tool_call",
                role: "tool".into(),
                content: String::new(),
                tool_name: Some(t.tool_name.clone()),
                tool_input: Some(t.tool_input.clone()),
                tool_result: t.tool_result_payload.clone(),
            });
    }

    let by_id: HashMap<&str, &ContextRequestRow> =
        requests.iter().map(|r| (r.id.as_str(), r)).collect();

    let mut redaction_count: u32 = 0;
    let mut out = Vec::new();
    for ((ts, request_id), mut entries) in by_request {
        let meta = by_id.get(request_id.as_str()).copied().map(meta_view);
        let request_id = AiRequestId::new(&request_id);
        entries.sort_by_key(|e| e.seq);
        for e in entries {
            let (body, redactions) = display_body(&e.content, opts);
            redaction_count = redaction_count.saturating_add(redactions);
            out.push(TranscriptEntryView {
                request_id: request_id.clone(),
                request_id_short: short_id(request_id.as_str()),
                request_url: request_detail_url(&request_id),
                meta: meta.clone(),
                ts_local: e
                    .ts
                    .with_timezone(&chrono::Local)
                    .format("%H:%M:%S")
                    .to_string(),
                ts_full: ts.to_rfc3339(),
                kind: e.kind,
                is_user: e.role == "user",
                is_assistant: e.role == "assistant",
                is_system: e.role == "system",
                is_tool: e.kind == "tool_call",
                role: e.role,
                content_preview: preview(&body),
                tool_name: e.tool_name,
                tool_input_pretty: e.tool_input.as_ref().map(pretty_json),
                tool_result_pretty: e.tool_result.as_ref().map(pretty_json),
            });
        }
    }
    BuiltTranscript {
        entries: out,
        redaction_count,
    }
}

pub(crate) fn display_body(content: &str, opts: TranscriptOptions) -> (String, u32) {
    let stripped = if opts.strip_markers {
        strip_gateway_markers(content)
    } else {
        content.to_owned()
    };
    if opts.redact {
        redact_text(&stripped)
    } else {
        (stripped, 0)
    }
}

pub fn meta_view(r: &ContextRequestRow) -> TranscriptMetaView {
    TranscriptMetaView {
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        latency_display: format_latency(r.latency_ms),
        token_display: format_tokens(r.input_tokens, r.output_tokens),
        cost_display: format_cost(r.cost_microdollars),
    }
}

pub(crate) fn format_latency(latency_ms: Option<i32>) -> String {
    latency_ms.map_or_else(|| "—".to_owned(), |ms| format!("{ms}ms"))
}

pub(crate) fn format_tokens(input: Option<i32>, output: Option<i32>) -> Option<String> {
    match (input, output) {
        (None, None) => None,
        (i, o) => Some(format!(
            "{} in / {} out",
            i.unwrap_or_default(),
            o.unwrap_or_default()
        )),
    }
}

// JSON: pretty-prints the per-tool payloads above for the transcript view
pub(crate) fn pretty_json(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

pub fn preview(s: &str) -> String {
    if s.chars().count() > TRANSCRIPT_PREVIEW_CHARS {
        let mut iter = s.chars();
        let head: String = (&mut iter).take(TRANSCRIPT_PREVIEW_CHARS).collect();
        format!("{head}…")
    } else {
        s.to_owned()
    }
}

pub fn short_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_owned()
    }
}
