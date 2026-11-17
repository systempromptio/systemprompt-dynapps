//! View models of what systemprompt recorded for a task (its model calls,
//! their totals by model and its conversation). Pure: the handlers read
//! [`crate::sp`] and hand the outcome here.
//!
//! Money is shown in USD, as systemprompt stores it (`cost_microdollars`); it
//! is never converted.

use serde::Serialize;
use systemprompt::identifiers::ContextId;
use systemprompt_web_admin_core::ssr::entity_urls::context_detail_url;

use crate::sp::{MAX_CALLS, SpCall, SpModelTotal};
use crate::views::{format_time, state_tone};

const NONE: &str = "—";
const MICROS_PER_DOLLAR: f64 = 1_000_000.0;
/// From one cent up, two decimals are enough; below it, four.
const ONE_CENT_MICROS: i64 = 10_000;

/// USD from systemprompt's `cost_microdollars`: `$1.25`, `$0.0024`, `$0.00`.
pub fn format_usd(micros: i64) -> String {
    let dollars = micros as f64 / MICROS_PER_DOLLAR;
    if micros == 0 || micros.abs() >= ONE_CENT_MICROS {
        format!("${dollars:.2}")
    } else {
        format!("${dollars:.4}")
    }
}

/// `450 ms` under a second, `8.2 s` from a second up.
pub fn format_latency(ms: Option<i32>) -> String {
    match ms {
        Some(ms) if ms < 0 => NONE.to_owned(),
        Some(ms) if ms < 1000 => format!("{ms} ms"),
        Some(ms) => format!("{:.1} s", f64::from(ms) / 1000.0),
        None => NONE.to_owned(),
    }
}

pub fn count_label(n: i64, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

fn tokens(value: Option<i32>) -> String {
    value.map_or_else(|| NONE.to_owned(), |v| v.to_string())
}

/// The status of a call, toned like the rest of the console.
fn call_tone(status: &str) -> &'static str {
    match status {
        "completed" | "succeeded" | "success" => "ok",
        other => state_tone(other),
    }
}

#[derive(Debug, Serialize)]
pub struct CallRow {
    pub time: String,
    pub model: String,
    pub status: String,
    pub status_tone: &'static str,
    pub kind: String,
    pub input_tokens: String,
    pub output_tokens: String,
    pub cache_read_tokens: String,
    pub cache_creation_tokens: String,
    pub cost: String,
    pub latency: String,
}

#[derive(Debug, Serialize)]
pub struct ModelRow {
    pub model: String,
    pub calls: i64,
    pub failed: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost: String,
}

#[derive(Debug, Serialize)]
pub struct ConversationLink {
    #[serde(rename = "context_id")]
    pub context: String,
    pub href: String,
}

/// The task page's systemprompt section.
#[derive(Debug, Serialize)]
pub struct SpTaskView {
    pub gateway_session_id: Option<String>,
    /// Why there is nothing to list, when there is not.
    pub note: Option<String>,
    pub calls: Vec<CallRow>,
    pub has_calls: bool,
    pub truncated: Option<String>,
    pub totals: Vec<ModelRow>,
    pub total_calls: String,
    pub total_cost: String,
    pub conversations: Vec<ConversationLink>,
    pub conversation_note: Option<String>,
}

/// What reading a task's calls gave.
#[derive(Debug)]
pub enum CallsRead {
    /// farm-api has no gateway session for the task: nothing to look up.
    NoSession,
    /// systemprompt could not be read (the fault is on the page).
    Failed,
    Read {
        calls: Vec<SpCall>,
        totals: Vec<SpModelTotal>,
    },
}

/// What reading a task's conversation gave.
#[derive(Debug)]
pub enum ContextsRead {
    /// farm-api has no Claude Code session for the task.
    NoSession,
    Failed,
    Read {
        client_session_id: String,
        contexts: Vec<String>,
    },
}

/// The Claude Code session of a task: its `session_ref` without `sess_`.
pub fn client_session_id(session_ref: Option<&str>) -> Option<String> {
    let raw = session_ref?.trim();
    let id = raw.strip_prefix("sess_").unwrap_or(raw);
    let valid =
        !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    valid.then(|| id.to_owned())
}

/// A gateway session id as farm-api reports it (`sess_<uuid>`), or nothing.
pub fn gateway_session_id(raw: Option<&str>) -> Option<String> {
    let id = raw?.trim();
    let body = id.strip_prefix("sess_")?;
    let valid = !body.is_empty()
        && id.len() <= 64
        && body.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    valid.then(|| id.to_owned())
}

fn call_row(call: &SpCall) -> CallRow {
    CallRow {
        time: format_time(Some(&call.created_at.to_rfc3339())),
        model: call.model.clone().unwrap_or_else(|| NONE.to_owned()),
        status_tone: call_tone(&call.status),
        status: call.status.clone(),
        kind: call.request_kind.clone(),
        input_tokens: tokens(call.input_tokens),
        output_tokens: tokens(call.output_tokens),
        cache_read_tokens: tokens(call.cache_read_tokens),
        cache_creation_tokens: tokens(call.cache_creation_tokens),
        cost: format_usd(call.cost_microdollars),
        latency: format_latency(call.latency_ms),
    }
}

fn model_row(total: &SpModelTotal) -> ModelRow {
    ModelRow {
        model: total.model.clone().unwrap_or_else(|| NONE.to_owned()),
        calls: total.calls,
        failed: total.failed,
        input_tokens: total.input_tokens,
        output_tokens: total.output_tokens,
        cache_read_tokens: total.cache_read_tokens,
        cache_creation_tokens: total.cache_creation_tokens,
        cost: format_usd(total.cost_microdollars),
    }
}

fn conversation_part(contexts: ContextsRead) -> (Vec<ConversationLink>, Option<String>) {
    match contexts {
        ContextsRead::NoSession => (
            Vec::new(),
            Some("No conversation: farm-api has no Claude Code session for this task.".to_owned()),
        ),
        ContextsRead::Failed => (
            Vec::new(),
            Some("The conversation could not be read; the notice above says why.".to_owned()),
        ),
        ContextsRead::Read {
            client_session_id,
            contexts,
        } => {
            let links: Vec<ConversationLink> = contexts
                .into_iter()
                .filter_map(|c| {
                    ContextId::try_new(c.as_str())
                        .ok()
                        .map(|id| ConversationLink {
                            href: context_detail_url(&id),
                            context: c,
                        })
                })
                .collect();
            let note = links.is_empty().then(|| {
                format!(
                    "No conversation in systemprompt for Claude Code session {client_session_id}."
                )
            });
            (links, note)
        },
    }
}

pub fn sp_task_view(
    gateway_session_id: Option<&str>,
    calls: CallsRead,
    contexts: ContextsRead,
) -> SpTaskView {
    let (conversations, conversation_note) = conversation_part(contexts);
    let mut view = SpTaskView {
        gateway_session_id: gateway_session_id.map(ToOwned::to_owned),
        note: None,
        calls: Vec::new(),
        has_calls: false,
        truncated: None,
        totals: Vec::new(),
        total_calls: count_label(0, "call", "calls"),
        total_cost: format_usd(0),
        conversations,
        conversation_note,
    };
    match calls {
        CallsRead::NoSession => {
            view.note = Some(
                "This task has no systemprompt gateway session in farm-api: it ran on another \
                 gateway, stopped before its first model call, or its usage was not reported. \
                 Its model calls cannot be linked."
                    .to_owned(),
            );
        },
        CallsRead::Failed => {
            view.note =
                Some("systemprompt usage could not be read; the notice above says why.".to_owned());
        },
        CallsRead::Read { calls, totals } => {
            if calls.is_empty() {
                view.note = Some(format!(
                    "systemprompt has no model calls for gateway session {}. Calls older than \
                     systemprompt's retention are deleted.",
                    gateway_session_id.unwrap_or(NONE)
                ));
            }
            if calls.len() as i64 >= MAX_CALLS {
                view.truncated = Some(format!(
                    "Only the first {MAX_CALLS} calls are listed; the totals cover all of them."
                ));
            }
            view.total_calls = count_label(totals.iter().map(|t| t.calls).sum(), "call", "calls");
            view.total_cost = format_usd(totals.iter().map(|t| t.cost_microdollars).sum());
            view.has_calls = !calls.is_empty();
            view.calls = calls.iter().map(call_row).collect();
            view.totals = totals.iter().map(model_row).collect();
        },
    }
    view
}
