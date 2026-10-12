//! A run's stream-json events as the rail and the timeline draw them.
//!
//! Every `payload` is decoded here, once, into [`RunEventPayload`]; a row
//! that does not decode is still shown, as its raw JSON, so a payload shape
//! the farm has not met yet never drops an event from the record.
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

use crate::repositories::rows::RunEventRow;
use crate::types::RunEventPayload;
use pairing::{PairKey, pair_seqs};
use text::{EventText, PREVIEW_CHARS, event_text, first_line, is_long, raw_text};

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

struct Decoded<'a> {
    row: &'a RunEventRow,
    payload: Option<RunEventPayload>,
}

impl Decoded<'_> {
    fn tool_use_id(&self) -> Option<&str> {
        match &self.payload {
            Some(
                RunEventPayload::ToolUse { tool_use_id, .. }
                | RunEventPayload::ToolResult { tool_use_id, .. },
            ) => Some(tool_use_id.as_str()),
            Some(_) => None,
            None => self.row.tool_use_id.as_deref(),
        }
    }

    fn is_use(&self) -> bool {
        self.payload.as_ref().map_or_else(
            || self.row.kind == "tool_use",
            |p| matches!(p, RunEventPayload::ToolUse { .. }),
        )
    }

    fn tool_name(&self) -> Option<String> {
        match &self.payload {
            Some(RunEventPayload::ToolUse { name, .. }) => Some(name.clone()),
            _ => self.row.tool_name.clone(),
        }
    }
}

pub fn event_views(events: &[RunEventRow], started_at: Option<DateTime<Utc>>) -> Vec<RunEventView> {
    let decoded: Vec<Decoded<'_>> = events
        .iter()
        .map(|row| Decoded {
            row,
            payload: serde_json::from_value::<RunEventPayload>(row.payload.clone()).ok(),
        })
        .collect();
    let keys: Vec<PairKey<'_>> = decoded
        .iter()
        .map(|d| PairKey {
            seq: d.row.seq,
            tool_use_id: d.tool_use_id(),
            is_use: d.is_use(),
        })
        .collect();
    let pairs = pair_seqs(&keys);
    let origin = started_at.or_else(|| events.iter().map(|e| e.created_at).min());
    let names: HashMap<i64, String> = decoded
        .iter()
        .filter_map(|d| d.tool_name().map(|n| (d.row.seq, n)))
        .collect();
    decoded
        .iter()
        .zip(pairs)
        .map(|(d, paired_seq)| {
            let tool_name = d
                .tool_name()
                .or_else(|| paired_seq.and_then(|s| names.get(&s).cloned()));
            let text = d.payload.as_ref().map_or_else(
                || raw_text(&d.row.kind, &d.row.payload),
                |p| event_text(p, tool_name.as_deref()),
            );
            view(d, text, tool_name, paired_seq, origin)
        })
        .collect()
}

fn view(
    d: &Decoded<'_>,
    text: EventText,
    tool_name: Option<String>,
    paired_seq: Option<i64>,
    origin: Option<DateTime<Utc>>,
) -> RunEventView {
    let offset_ms = origin.map_or(0, |o| (d.row.created_at - o).num_milliseconds().max(0));
    let kind = d
        .payload
        .as_ref()
        .map_or_else(|| d.row.kind.clone(), |p| p.kind().as_str().to_owned());
    RunEventView {
        seq: d.row.seq,
        kind,
        glyph: text.glyph,
        offset_ms,
        offset_label: format!("+{}", format_duration_ms(offset_ms)),
        tool_name,
        title: text.title,
        preview: first_line(&text.body, PREVIEW_CHARS),
        body_is_long: is_long(&text.body),
        body: text.body,
        is_error: text.is_error,
        is_tool_use: d.is_use(),
        paired_seq,
        created_local: local_time(d.row.created_at),
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
