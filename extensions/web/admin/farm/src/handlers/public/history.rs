//! `GET /evals/history?farm=&agent=&since=` — per agent per day: eval runs,
//! case pass rate, mean judge score and cost, over the farms the caller may
//! use. `GET /evals/suites/{suite_id}/runs?limit=` — a suite's recent eval
//! runs, each with its trigger and tallies; a caller who neither owns the
//! farm nor holds the console sees only their own.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use super::shared::load_farm;
use crate::repositories::evals;
use crate::repositories::rows::EvalRunRow;
use crate::services::entitlement::{is_farm_owner, require_farm_access};
use crate::services::evals::{EvalHistoryPoint, HistoryScope, list_visible_history, parse_since};
use crate::state::FarmState;
use crate::types::{EvalSuiteId, FarmAgentId, FarmId};

const DEFAULT_SUITE_RUNS: i64 = 20;
const MAX_SUITE_RUNS: i64 = 200;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoryQuery {
    farm: Option<String>,
    agent: Option<String>,
    since: Option<String>,
}

#[derive(Debug, Serialize)]
struct HistoryResponse {
    since: DateTime<Utc>,
    points: Vec<EvalHistoryPoint>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SuiteRunsQuery {
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SuiteRunsResponse {
    suite_id: EvalSuiteId,
    runs: Vec<EvalRunRow>,
}

fn present(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

pub(crate) async fn eval_history(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<HistoryQuery>,
) -> AdminResult<Response> {
    let raw_since = query.since.unwrap_or_default();
    let since = parse_since(&raw_since, Utc::now()).ok_or_else(|| {
        AdminError::Unprocessable(format!(
            "since {raw_since:?} is not `<n>d`, `<n>h` or an RFC 3339 time"
        ))
    })?;
    let farm = present(query.farm).map(FarmId::new);
    let agent = present(query.agent).map(FarmAgentId::new);
    let points = list_visible_history(
        &state.pool,
        &state.services,
        &ctx,
        HistoryScope {
            farm: farm.as_ref(),
            agent: agent.as_ref(),
            since,
        },
    )
    .await?;
    Ok(Json(HistoryResponse { since, points }).into_response())
}

pub(crate) async fn list_suite_runs(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Path(suite_id): Path<EvalSuiteId>,
    Query(query): Query<SuiteRunsQuery>,
) -> AdminResult<Response> {
    let suite = evals::find_eval_suite(&state.pool, &suite_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("eval suite {}", suite_id.as_str())))?;
    let farm = load_farm(&state.pool, &suite.farm_id).await?;
    require_farm_access(&state.pool, &state.services, &ctx, &farm).await?;
    let sees_all = ctx.is_console || is_farm_owner(&ctx, &farm);
    let limit = query
        .limit
        .unwrap_or(DEFAULT_SUITE_RUNS)
        .clamp(1, MAX_SUITE_RUNS);
    let runs = evals::list_recent_eval_runs(&state.pool, Some(&farm.id), Some(&suite.id), limit)
        .await?
        .into_iter()
        .filter(|r| sees_all || r.user_id == ctx.user_id)
        .collect();
    Ok(Json(SuiteRunsResponse {
        suite_id: suite.id,
        runs,
    })
    .into_response())
}
