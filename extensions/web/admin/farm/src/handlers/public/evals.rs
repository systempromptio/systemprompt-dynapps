//! `POST /evals/{suite_id}/run` launches an eval suite as the caller;
//! `GET /evals/runs/{eval_run_id}` reads the eval run and its per-case
//! results.
//!
//! The judge is the configured model through the process's `AiService` when
//! inference is configured; without one, rubric cases are scored on their
//! assertions alone.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use systemprompt::ai::AiService;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use super::shared::{load_farm, triggered_via};
use crate::repositories::evals;
use crate::repositories::rows::{EvalResultRow, EvalRunRow};
use crate::services::entitlement::{is_farm_owner, require_farm_access};
use crate::services::evals::{EvalLaunch, start_eval_run};
use crate::state::FarmState;
use crate::types::{EvalRunId, EvalSuiteId, FarmId};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvalLaunchBody {
    #[serde(alias = "farm_id")]
    farm: FarmId,
    cases: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct EvalLaunched {
    eval_run_id: EvalRunId,
    status: String,
}

#[derive(Debug, Serialize)]
struct EvalRunResponse {
    eval_run: EvalRunRow,
    results: Vec<EvalResultRow>,
}

pub(crate) async fn run_eval_suite(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    ai: Option<Extension<Option<Arc<AiService>>>>,
    Path(suite_id): Path<EvalSuiteId>,
    Json(body): Json<EvalLaunchBody>,
) -> AdminResult<Response> {
    let farm = load_farm(&state.pool, &body.farm).await?;
    require_farm_access(&state.pool, &ctx, &farm).await?;
    let cases = body.cases.filter(|c| !c.is_empty());
    let judge = state
        .judge
        .judge_for(ai.and_then(|Extension(ai)| ai), &ctx.user_id);
    let eval_run = start_eval_run(
        Arc::clone(&state),
        judge,
        EvalLaunch {
            farm_id: farm.id,
            suite_id,
            user_id: ctx.user_id.clone(),
            triggered_via: triggered_via(&ctx),
            case_filter: cases,
        },
    )
    .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(EvalLaunched {
            eval_run_id: eval_run.id,
            status: eval_run.status,
        }),
    )
        .into_response())
}

pub(crate) async fn get_eval_run(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Path(eval_run_id): Path<EvalRunId>,
) -> AdminResult<Response> {
    let not_found = || AdminError::NotFound(format!("eval run {}", eval_run_id.as_str()));
    let eval_run = evals::find_eval_run(&state.pool, &eval_run_id)
        .await?
        .ok_or_else(not_found)?;
    let visible = ctx.is_console
        || eval_run.user_id == ctx.user_id
        || load_farm(&state.pool, &eval_run.farm_id)
            .await
            .is_ok_and(|farm| is_farm_owner(&ctx, &farm));
    if !visible {
        return Err(not_found());
    }
    let results = evals::list_eval_results(&state.pool, &eval_run.id).await?;
    Ok(Json(EvalRunResponse { eval_run, results }).into_response())
}
