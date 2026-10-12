//! `GET /admin/api/farm/evals/runs/{eval_run_id}` — the eval run page's KPI
//! strip and case rows as an HTML fragment, rendered through the
//! `farm/eval-run-live` partial from the same view the page embeds. The
//! page's live list polls it while the eval run is queued or running.
//!
//! A non-console caller is refused with 403 rather than redirected: the
//! caller is a script, and a fetch that followed a login redirect would swap
//! a login page into the table.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::UserContext;

use crate::handlers::ssr::farm_eval_run::{eval_run_live, parse_eval_run_id};
use crate::repositories::evals::{find_eval_run, list_eval_results};

const LIVE_PARTIAL: &str = "farm/eval-run-live";

pub(crate) async fn eval_run_fragment(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(eval_run_id): Path<String>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }
    let id = parse_eval_run_id(&eval_run_id)?;
    let run = find_eval_run(&pool, &id)
        .await?
        .ok_or_else(|| AdminError::NotFound("Eval run not found.".to_owned()))?;
    let results = list_eval_results(&pool, &id).await?;
    let html = engine.render(LIVE_PARTIAL, &eval_run_live(&run, &results))?;
    Ok((StatusCode::OK, [(CACHE_CONTROL, "no-store")], Html(html)).into_response())
}
