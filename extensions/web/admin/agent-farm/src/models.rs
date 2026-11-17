//! The farm-api response shapes the pages read.
//!
//! Only the fields shown are modelled; every one is optional or defaulted so a
//! field farm-api adds or drops never turns a page into an error. Shapes follow
//! `farm-api/farm_api/routers/tasks.py` (`wire`, `get_result`, `get_logs`) and
//! `routers/identity.py` (`whoami`).

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WhoAmI {
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub tenant: String,
    /// Only present for a credential with `farm:platform:read`.
    #[serde(default)]
    pub tenants: Vec<TenantRef>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TenantRef {
    pub tenant: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskList {
    #[serde(default)]
    pub items: Vec<TaskOut>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskOut {
    #[serde(rename = "task_id")]
    pub id: String,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default, rename = "agent_id")]
    pub agent: Option<String>,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub failure_reason: Option<String>,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub limits: Option<TaskLimits>,
    #[serde(default)]
    pub session_ref: Option<String>,
    #[serde(default)]
    pub resumable: Option<bool>,
    #[serde(default)]
    pub queue_position: Option<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub usage: Option<TaskUsage>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskLimits {
    #[serde(default)]
    pub cpu: Option<String>,
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub timeout_s: Option<i64>,
    #[serde(default)]
    pub ttl_s: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskUsage {
    #[serde(default)]
    pub input_tokens: Option<i64>,
    #[serde(default)]
    pub output_tokens: Option<i64>,
    #[serde(default)]
    pub cost_eur: Option<f64>,
    #[serde(default)]
    pub duration_s: Option<f64>,
    #[serde(default)]
    pub gateway_session_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskResult {
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub result: Option<ResultObject>,
}

/// The stored result object. The presigned `url` is deliberately not
/// modelled: it is a short-lived credential and the page never shows it.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ResultObject {
    #[serde(default)]
    pub size_bytes: Option<i64>,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskLogs {
    #[serde(default)]
    pub lines: Vec<String>,
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    pub error: ApiError,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentList {
    #[serde(default)]
    pub items: Vec<AgentOut>,
}

/// An agent of the catalogue (`routers/agents.py`, `AgentOut`). Secrets never
/// travel here: `mcp_servers` and `env_credentials` name credentials only.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentOut {
    pub slug: String,
    /// Present on every item of a list read with `?tenant=*`.
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub plugin_agent: Option<String>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerRef>,
    #[serde(default)]
    pub env_credentials: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub harness: Option<String>,
    #[serde(default)]
    pub requires_workspace: Option<bool>,
    #[serde(default)]
    pub capabilities: Option<AgentCapabilities>,
    #[serde(default)]
    pub default_limits: Option<TaskLimits>,
    #[serde(default)]
    pub model_alias: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub work_size: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct McpServerRef {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentCapabilities {
    #[serde(default)]
    pub credentials: Vec<String>,
    #[serde(default)]
    pub egress: Vec<String>,
    #[serde(default)]
    pub denied_tools: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScheduleList {
    #[serde(default)]
    pub items: Vec<ScheduleOut>,
}

/// A schedule (`routers/schedules.py`). The stored `task` body is not modelled
/// except for the agent it names: it carries the prompt.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScheduleOut {
    pub schedule_id: String,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub task: Option<ScheduleTask>,
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    #[serde(default)]
    pub overlap: Option<String>,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub next_fire_at: Option<String>,
    #[serde(default)]
    pub missed_runs: Option<i64>,
    #[serde(default)]
    pub runs_count: Option<i64>,
    #[serde(default)]
    pub last_fired_at: Option<String>,
    #[serde(default)]
    pub last_fire_state: Option<String>,
    #[serde(default)]
    pub last_fire_error: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScheduleTask {
    #[serde(default, rename = "agent_id")]
    pub agent: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Recurrence {
    #[serde(default)]
    pub cron: String,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub ends_at: Option<String>,
    #[serde(default)]
    pub max_runs: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CredentialList {
    #[serde(default)]
    pub items: Vec<CredentialOut>,
}

/// A credential's metadata.
///
/// By design there is no field for a value, the fingerprint or the free-form
/// metadata: whatever farm-api sends beyond these is dropped on
/// deserialisation and so can never reach a page.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CredentialOut {
    pub name: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}
