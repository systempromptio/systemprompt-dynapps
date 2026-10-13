//! The shapes this repository returns and the narrowing it accepts.
//!
//! `RunFacts` and `RunStats` derive `JsonSchema` so an MCP tool can hand
//! them back verbatim; `ProjectId` is a web-shared newtype without a schema
//! derive, so its field is described as the string it serialises to.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
// JSON: a run's structured output and diff summary are relayed verbatim
// from core's JSONB columns; their shape belongs to the caller and the host.
use serde_json::Value;
use systemprompt::identifiers::{AgentRunId, UserId};
use systemprompt_web_shared::ProjectId;

/// Who may see which runs: the runs they launched (`agent_runs.user_id`),
/// the runs whose farm belongs to one of their projects, or every run.
#[derive(Debug, Clone)]
pub enum RunScope {
    Own(UserId),
    Projects(Vec<ProjectId>),
    All,
}

/// Narrowing applied to the list and the stats alike.
#[derive(Debug, Clone, Default)]
pub struct RunFilter {
    pub origin_kind: Option<String>,
    pub origin_id: Option<String>,
    pub status: Option<String>,
    pub farm_id: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

/// Everything the tables recorded about one run.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunFacts {
    pub run_id: AgentRunId,
    pub origin_kind: String,
    pub origin_id: String,
    pub user_id: UserId,
    #[schemars(with = "Option<String>")]
    pub project_id: Option<ProjectId>,
    pub status: String,
    pub error_class: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub attempt: i32,
    pub turns: Option<i32>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars_reported: Option<i64>,
    pub cost_microdollars_metered: i64,
    pub tool_use_count: i64,
    pub tool_result_count: i64,
    pub tool_result_error_count: i64,
    pub tools_by_name: Vec<(String, i64)>,
    pub hook_allow: i64,
    pub hook_deny: i64,
    pub governance_allow: i64,
    pub governance_deny: i64,
    pub budget_exceeded: bool,
    pub timed_out: bool,
    // JSON: shaped by the request's own `output_schema` and by the host's
    // diff report, both relayed verbatim from core's JSONB columns.
    pub structured_output: Option<Value>,
    pub output_schema_valid: Option<bool>,
    pub diff_summary: Option<Value>,
}

/// Runs by `agent_runs.status`.
#[derive(Debug, Clone, Copy, Default, Serialize, JsonSchema)]
pub struct StatusCounts {
    pub queued: i64,
    pub preparing: i64,
    pub running: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub cancelled: i64,
    pub timed_out: i64,
}

/// Finished runs by `agent_runs.error_class`.
#[derive(Debug, Clone, Copy, Default, Serialize, JsonSchema)]
pub struct ErrorClassCounts {
    pub ok: i64,
    pub budget_exceeded: i64,
    pub timeout: i64,
    pub output_invalid: i64,
    pub hook_denied: i64,
    pub host_crashed: i64,
    pub connector_unavailable: i64,
}

/// One rollup: the whole scope, or one origin kind within it.
#[derive(Debug, Clone, Copy, Default, Serialize, JsonSchema)]
pub struct RunStatsBucket {
    pub total: i64,
    pub by_status: StatusCounts,
    pub by_error_class: ErrorClassCounts,
    pub cost_microdollars_reported: i64,
    pub cost_microdollars_metered: i64,
    pub p50_duration_ms: Option<f64>,
    pub p95_duration_ms: Option<f64>,
    pub tool_use_count: i64,
    pub hook_deny: i64,
}

/// `RunStatsBucket` for one `origin_kind`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OriginKindStats {
    pub origin_kind: String,
    #[serde(flatten)]
    pub stats: RunStatsBucket,
}

/// The scope's rollup and the same split by origin kind.
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct RunStats {
    #[serde(flatten)]
    pub totals: RunStatsBucket,
    pub by_origin_kind: Vec<OriginKindStats>,
}
