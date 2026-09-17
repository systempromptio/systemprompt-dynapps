//! Trace explorer queries.
//!
//! A "trace" here is keyed on `session_id` (which all four governance / gateway
//! / events tables share) and surfaces the `trace_id` from `ai_requests` when
//! one exists. [`list_traces`] returns one summary row per session in the
//! window; [`list_trace_spans`] returns the union of per-table rows for a
//! single session, normalised into a [`Span`] shape and ordered by start time.
//!
//! A Claude Code session carries two ids: the bridge session (`sess_…`) on the
//! gateway rows and Claude Code's own uuid on the hook and governance rows.
//! `ai_requests.client_session_id` holds the uuid beside the bridge session,
//! so every query here folds uuid-keyed rows into the bridge session; a
//! hook-only session with no gateway row keeps the uuid as its key.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{AgentId, ArtifactId, ContextId, SessionId, TraceId, UserId};

mod list;
mod list_row;
mod resolve;
mod spans;
mod stats;

pub use list::{TracePage, list_traces};
pub use resolve::resolve_trace_session;
pub use spans::list_trace_spans;
pub use stats::get_trace_stats;

#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub session_id: SessionId,
    pub trace_id: Option<TraceId>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub active_ms: i64,
    // Why: First-to-last event span. A client may reuse a session id for hours, so
    // this is context for `active_ms`, never a substitute for it.
    pub window_ms: i64,
    pub user_id: Option<UserId>,
    pub user_label: Option<String>,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub span_count: i64,
    pub request_count: i64,
    pub tool_call_count: i64,
    pub governance_count: i64,
    pub deny_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub total_latency_ms: i64,
    pub cache_hit_any: bool,
    pub top_tool: Option<String>,
    pub has_error: bool,
    // Why: Failed gateway requests on the trace — the count behind `has_error`.
    pub error_count: i64,
    pub has_deny: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceFilter<'a> {
    pub user_id: Option<&'a str>,
    // Why: the caller's resolved `SubjectScope::as_sql()`; NULL = every user.
    pub subject_ids: Option<&'a [String]>,
    pub agent_id: Option<&'a str>,
    pub agent_scope: Option<&'a str>,
    pub policy: Option<&'a str>,
    pub decision: Option<&'a str>,
    pub error_only: bool,
    pub deny_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortColumn {
    StartedAt,
    Duration,
    SpanCount,
    Cost,
    Tokens,
}

impl TraceSortColumn {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::StartedAt => "started_at",
            Self::Duration => "duration",
            Self::SpanCount => "span_count",
            Self::Cost => "cost",
            Self::Tokens => "tokens",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortDir {
    Asc,
    Desc,
}

impl TraceSortDir {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TraceSort {
    pub column: TraceSortColumn,
    pub dir: TraceSortDir,
}

impl Default for TraceSort {
    fn default() -> Self {
        Self {
            column: TraceSortColumn::StartedAt,
            dir: TraceSortDir::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TraceStats {
    pub total_traces: i64,
    pub error_count: i64,
    pub deny_count: i64,
    // Why: Percentiles over `active_ms` of the traces that issued at least one
    // request; governance-only traces have no latency to contribute and would
    // otherwise drag every percentile to zero.
    pub p50_active_ms: i64,
    pub p95_active_ms: i64,
    pub p99_active_ms: i64,
    pub total_cost_microdollars: i64,
    pub total_tokens: i64,
}

/// One waterfall span — normalised across the four span sources.
#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub id: String,
    pub kind: SpanKind,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status: SpanStatus,
    pub identity_label: Option<String>,
    // Why: only a model span carries these; a governance or tool span has no
    // provider to name and no cost of its own, so the waterfall table renders
    // an em dash rather than a zero that reads as "free".
    pub provider: Option<String>,
    pub model: Option<String>,
    pub cost_microdollars: Option<i64>,
    pub latency_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub error_message: Option<String>,
    // Why: the gateway stamps one `trace_id` on a request and on every
    // governance decision it took for that request, so it is the key that
    // folds a turn's checks back under the turn.
    pub trace_id: Option<String>,
    pub context_id: Option<ContextId>,
    pub user_id: UserId,
    pub client_kind: Option<String>,
    // Why: a governance span's policy and verdict; the name carries the tool.
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub reason: Option<String>,
    pub stages: Vec<PolicyStage>,
    // Why: a hook event that reported a tool result links to the artifact the
    // ingest stored for it; every other span has none.
    pub artifact_id: Option<ArtifactId>,
}

/// One stage of a governance chain evaluation, as the policy audited it.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyStage {
    pub policy_id: String,
    pub result: String,
    pub detail: String,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    Gateway,
    Governance,
    Tool,
    Model,
    Spawn,
}

impl SpanKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Governance => "governance",
            Self::Tool => "tool",
            Self::Model => "model",
            Self::Spawn => "spawn",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    Ok,
    Deny,
    // Why: a policy that let the call through while flagging it; the row
    // reads as a caution, not a refusal and not a clean pass.
    Warn,
    Error,
    Pending,
    // Why: a gateway-rejected request never reached a provider, so it has no
    // provider or model to name; conflating it with Error would report a
    // policy refusal as a fault the operator should chase upstream.
    Rejected,
}

impl SpanStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Deny => "deny",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Pending => "pending",
            Self::Rejected => "rejected",
        }
    }
}
