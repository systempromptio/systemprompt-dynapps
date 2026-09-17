//! The Evaluation page's two buttons.
//!
//! A click runs the judge inline and answers when the runs are recorded —
//! the same drain the scheduler uses, with the admin on the run row. The
//! judge is the process's one `AiService`, handed to this router by core; an
//! instance without inference configured has none, and every trigger says so
//! instead of queueing work nobody will do.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Redirect;
use sqlx::PgPool;

use super::trigger::JudgeRequest;
use crate::error::{AdminError, AdminHtmlResult};
use crate::evaluation::{Trigger, drain};
use crate::repositories::config::evaluation::CriterionName;
use crate::repositories::evaluation::queue;

const EVALUATION_PAGE: &str = "/admin/analysis/evaluation";
const RUNS_PAGE: &str = "/admin/analysis/evaluation/runs";

pub(crate) async fn run_pending(
    request: JudgeRequest,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Redirect> {
    request.require_console_write()?;
    let judge = request.judge()?;
    let leases = queue::claim_leases(
        &pool,
        judge.config.worker.batch_size,
        judge.config.worker.lease_seconds,
    )
    .await?;
    let trigger = Trigger::Manual {
        requested_by: request.user.user_id.clone(),
    };
    let report = drain(&pool, &judge, &leases, &trigger).await?;
    tracing::info!(
        requested_by = %request.user.user_id,
        leased = report.leased,
        succeeded = report.succeeded,
        failed = report.failed,
        "Evaluation queue drained from the console"
    );
    Ok(Redirect::to(RUNS_PAGE))
}

pub(crate) async fn queue_closed(
    request: JudgeRequest,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Redirect> {
    request.require_console_write()?;
    let config = super::load_config()?
        .filter(|c| c.enabled)
        .ok_or_else(|| AdminError::BadRequest("Evaluation is not configured.".to_owned()))?;
    let close = queue::CloseParams {
        quiet_minutes: config.session_close.quiet_minutes,
        min_user_prompts: config.session_close.min_user_prompts,
        lookback_hours: config.session_close.lookback_hours,
        criteria: &CriterionName::ALL,
    };
    let enqueued = queue::insert_closed_sessions(&pool, &close).await?;
    tracing::info!(requested_by = %request.user.user_id, enqueued, "Closed sessions queued from the console");
    Ok(Redirect::to(EVALUATION_PAGE))
}
