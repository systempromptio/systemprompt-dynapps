//! The run as the public API returns it: every farm column, the executor's
//! live figures from its `agent_runs` row, the cost in dollars, and the links
//! to its transcript, dashboard page and event stream.
//!
//! `claude_session_id` is kept as the key scripts already read and carries
//! the host session id for every host; `host_session_id` is the same value
//! under its host-neutral name.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::agent_runs::{AgentRunRepository, AgentRunRow};
use systemprompt::identifiers::{ApiKeyId, HostSessionId, SessionId};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::numeric::to_f64;

use crate::repositories::rows::RunRow;
use crate::repositories::runs;

const MICRODOLLARS_PER_DOLLAR: f64 = 1_000_000.0;

// JSON: protocol boundary — the diff summary and the structured answer are
// relayed as the `agent_runs` JSONB columns hold them.
#[derive(Debug, Default, Serialize)]
pub(super) struct ExecutorJson {
    pub run_dir: Option<String>,
    pub gateway_session_id: Option<SessionId>,
    pub api_key_id: Option<ApiKeyId>,
    pub claude_session_id: Option<HostSessionId>,
    pub host_session_id: Option<HostSessionId>,
    pub pid: Option<i32>,
    pub claimed_by: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub num_turns: Option<i32>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub diff_summary: Option<serde_json::Value>,
    pub structured_output: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunJson {
    #[serde(flatten)]
    pub run: RunRow,
    #[serde(flatten)]
    pub executor: ExecutorJson,
    pub cost_usd: Option<f64>,
    pub transcript_url: Option<String>,
    pub dashboard_url: String,
    pub events_url: String,
}

fn executor_json(row: AgentRunRow) -> ExecutorJson {
    ExecutorJson {
        run_dir: row.run_dir,
        gateway_session_id: row.gateway_session_id,
        api_key_id: row.api_key_id,
        claude_session_id: row.host_session_id.clone(),
        host_session_id: row.host_session_id,
        pid: row.pid,
        claimed_by: row.claimed_by,
        lease_until: row.lease_until,
        duration_ms: row.duration_ms,
        num_turns: row.turns,
        input_tokens: row.input_tokens,
        output_tokens: row.output_tokens,
        exit_code: row.exit_code,
        error: row.error,
        diff_summary: row.diff_summary,
        structured_output: row.structured_output,
    }
}

pub(super) async fn run_json(pool: &Arc<PgPool>, run: RunRow) -> AdminResult<RunJson> {
    let agent_run = AgentRunRepository::from_pool(Arc::clone(pool))
        .find_run(&run.agent_run_id)
        .await
        .map_err(AdminError::internal)?;
    let host_session = agent_run.as_ref().and_then(|r| r.host_session_id.clone());
    let context_id = match (&run.context_id, &host_session) {
        (Some(id), _) => Some(id.as_str().to_owned()),
        (None, Some(session)) => runs::find_run_context_id(pool, &run.user_id, session)
            .await?
            .map(|id| id.as_str().to_owned()),
        (None, None) => None,
    };
    let id = run.id.as_str().to_owned();
    let cost = run.cost_microdollars;
    Ok(RunJson {
        executor: agent_run.map(executor_json).unwrap_or_default(),
        cost_usd: (cost > 0 || run.finished_at.is_some())
            .then(|| to_f64(cost) / MICRODOLLARS_PER_DOLLAR),
        transcript_url: context_id.map(|c| format!("/admin/contexts/{c}")),
        dashboard_url: format!("/admin/farm/runs/{id}"),
        events_url: format!("/api/public/agent-farm/runs/{id}/events"),
        run,
    })
}
