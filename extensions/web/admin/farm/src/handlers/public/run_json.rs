//! The run as the public API returns it: every column, the cost in dollars,
//! and the links to its transcript, dashboard page and event stream.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::numeric::to_f64;

use crate::repositories::rows::RunRow;
use crate::repositories::runs;

const MICRODOLLARS_PER_DOLLAR: f64 = 1_000_000.0;

#[derive(Debug, Serialize)]
pub(super) struct RunJson {
    #[serde(flatten)]
    pub run: RunRow,
    pub cost_usd: Option<f64>,
    pub transcript_url: Option<String>,
    pub dashboard_url: String,
    pub events_url: String,
}

pub(super) async fn run_json(pool: &PgPool, run: RunRow) -> AdminResult<RunJson> {
    let context_id = match (&run.context_id, &run.claude_session_id) {
        (Some(id), _) => Some(id.as_str().to_owned()),
        (None, Some(session)) => runs::find_run_context_id(pool, &run.user_id, session)
            .await?
            .map(|id| id.as_str().to_owned()),
        (None, None) => None,
    };
    let id = run.id.as_str().to_owned();
    Ok(RunJson {
        cost_usd: run
            .cost_microdollars
            .map(|m| to_f64(m) / MICRODOLLARS_PER_DOLLAR),
        transcript_url: context_id.map(|c| format!("/admin/contexts/{c}")),
        dashboard_url: format!("/admin/farm/runs/{id}"),
        events_url: format!("/api/public/agent-farm/runs/{id}/events"),
        run,
    })
}
