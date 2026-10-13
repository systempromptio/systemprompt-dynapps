//! A run's `agent_run_events` as the rail and the timeline draw them.
//!
//! Core's repository decodes every row into a typed [`RunEvent`], so this
//! module only words and pairs them; the host that produced an event
//! (Claude Code, Codex CLI, …) makes no difference here.
//!
//! - [`text`] — titles, bodies and folding.
//! - [`pairing`] — `tool_use` ⇄ `tool_result` by `tool_use_id`.
//! - [`timeline`] — the SVG strip over the run's own window.

pub(crate) mod pairing;
pub(crate) mod text;
pub(crate) mod timeline;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::{format_duration_ms, local_time};

use systemprompt::agent_runs::{RunEvent, RunEventPayload};

use pairing::{PairKey, pair_seqs};
use text::{EventText, PREVIEW_CHARS, event_text, first_line, is_long};

pub(crate) use timeline::{RunTimelineView, timeline_view};

pub const FRAGMENT_EVENT_CAP: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunEventView {
    pub seq: i64,
    pub kind: String,
    pub glyph: &'static str,
    pub offset_ms: i64,
    pub offset_label: String,
    pub tool_name: Option<String>,
    pub title: String,
    pub preview: String,
    pub body: String,
    pub body_is_long: bool,
    pub is_error: bool,
    pub is_tool_use: bool,
    pub paired_seq: Option<i64>,
    pub created_local: String,
}

fn tool_name(event: &RunEvent) -> Option<String> {
    match &event.payload {
        RunEventPayload::ToolUse { name, .. } => Some(name.clone()),
        _ => event.tool_name.clone(),
    }
}

const fn is_use(event: &RunEvent) -> bool {
    matches!(event.payload, RunEventPayload::ToolUse { .. })
}

pub fn event_views(events: &[RunEvent], started_at: Option<DateTime<Utc>>) -> Vec<RunEventView> {
    let keys: Vec<PairKey<'_>> = events
        .iter()
        .map(|e| PairKey {
            seq: e.seq,
            tool_use_id: e.payload.tool_use_id().or(e.tool_use_id.as_deref()),
            is_use: is_use(e),
        })
        .collect();
    let pairs = pair_seqs(&keys);
    let origin = started_at.or_else(|| events.iter().map(|e| e.created_at).min());
    let names: HashMap<i64, String> = events
        .iter()
        .filter_map(|e| tool_name(e).map(|n| (e.seq, n)))
        .collect();
    events
        .iter()
        .zip(pairs)
        .map(|(e, paired_seq)| {
            let name = tool_name(e).or_else(|| paired_seq.and_then(|s| names.get(&s).cloned()));
            let text = event_text(&e.payload, name.as_deref());
            view(e, text, name, paired_seq, origin)
        })
        .collect()
}

fn view(
    e: &RunEvent,
    text: EventText,
    tool_name: Option<String>,
    paired_seq: Option<i64>,
    origin: Option<DateTime<Utc>>,
) -> RunEventView {
    let offset_ms = origin.map_or(0, |o| (e.created_at - o).num_milliseconds().max(0));
    RunEventView {
        seq: e.seq,
        kind: e.kind.as_str().to_owned(),
        glyph: text.glyph,
        offset_ms,
        offset_label: format!("+{}", format_duration_ms(offset_ms)),
        tool_name,
        title: text.title,
        preview: first_line(&text.body, PREVIEW_CHARS),
        body_is_long: is_long(&text.body),
        body: text.body,
        is_error: text.is_error,
        is_tool_use: is_use(e),
        paired_seq,
        created_local: local_time(e.created_at),
    }
}

pub fn events_after(views: &[RunEventView], after: i64) -> Vec<RunEventView> {
    views
        .iter()
        .filter(|v| v.seq > after)
        .take(FRAGMENT_EVENT_CAP)
        .cloned()
        .collect()
}

pub fn transcript_views(views: &[RunEventView]) -> Vec<RunEventView> {
    views
        .iter()
        .filter(|v| v.kind == "assistant_text" || v.kind == "result")
        .cloned()
        .collect()
}
