//! `GET /runs`, `GET /runs/{id}` and `POST /runs/{id}/cancel`.
//!
//! A console user lists every run and may filter by person; anyone else lists
//! only their own. A single run is readable by whoever launched it, console
//! users and the farm's owner.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use super::run_json::{RunJson, run_json};
use super::shared::load_visible_run;
use crate::repositories::filters::{FarmRunFilter, RowWindow, RunSort};
use crate::repositories::runs;
use crate::services::runs::cancel;
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId, FarmRunId, RunStatus};

const DEFAULT_LIMIT: i64 = 25;
const MAX_LIMIT: i64 = 100;
const SELF_ALIAS: &str = "me";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RunListQuery {
    farm: Option<FarmId>,
    agent: Option<FarmAgentId>,
    status: Option<String>,
    user: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Serialize)]
struct RunListResponse {
    runs: Vec<RunJson>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Serialize)]
struct CancelResponse {
    run_id: FarmRunId,
    cancel_requested: bool,
}

fn user_filter(ctx: &UserContext, requested: Option<&str>) -> AdminResult<Option<UserId>> {
    match requested.filter(|u| !u.is_empty()) {
        None if ctx.is_console => Ok(None),
        None | Some(SELF_ALIAS) => Ok(Some(ctx.user_id.clone())),
        Some(other) if other == ctx.user_id.as_str() => Ok(Some(ctx.user_id.clone())),
        Some(other) if ctx.is_console => Ok(Some(UserId::new(other))),
        Some(_) => Err(AdminError::Forbidden(
            "only console users may list another person's runs".to_owned(),
        )),
    }
}

fn status_filter(status: Option<&str>) -> AdminResult<Option<RunStatus>> {
    status
        .filter(|s| !s.is_empty())
        .map(|s| {
            RunStatus::parse(s)
                .ok_or_else(|| AdminError::Unprocessable(format!("unknown run status {s:?}")))
        })
        .transpose()
}

pub(crate) async fn list_runs(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Query(query): Query<RunListQuery>,
) -> AdminResult<Response> {
    let filter = FarmRunFilter {
        farm_id: query.farm,
        agent_id: query.agent,
        user_id: user_filter(&ctx, query.user.as_deref())?,
        status: status_filter(query.status.as_deref())?,
        ..FarmRunFilter::default()
    };
    let page = RowWindow {
        limit: query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
        offset: query.offset.unwrap_or(0).max(0),
    };
    let (rows, total) =
        runs::list_runs_paged(&state.pool, &filter, RunSort::CreatedDesc, page).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(run_json(&state.pool, row).await?);
    }
    Ok(Json(RunListResponse {
        runs: out,
        total,
        limit: page.limit,
        offset: page.offset,
    })
    .into_response())
}

pub(crate) async fn get_run(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Path(run_id): Path<FarmRunId>,
) -> AdminResult<Response> {
    let run = load_visible_run(&state.pool, &ctx, &run_id).await?;
    Ok(Json(run_json(&state.pool, run).await?).into_response())
}

pub(crate) async fn get_run_report(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Path(run_id): Path<FarmRunId>,
) -> AdminResult<Response> {
    let run = load_visible_run(&state.pool, &ctx, &run_id).await?;
    let report = crate::repositories::reports::find_run_report(&state.pool, &run.id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("report for run {}", run.id.as_str())))?;
    Ok(Json(report).into_response())
}

pub(crate) async fn cancel_run(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Path(run_id): Path<FarmRunId>,
) -> AdminResult<Response> {
    let run = load_visible_run(&state.pool, &ctx, &run_id).await?;
    let cancel_requested = cancel(&state, &run.id, &ctx.user_id).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(CancelResponse {
            run_id: run.id,
            cancel_requested,
        }),
    )
        .into_response())
}
