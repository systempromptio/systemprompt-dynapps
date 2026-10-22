//! The worker API a farm worker on a Fly Machine calls, nested at
//! `/api/public/agent-farm-worker/runs/{agent_run_id}/…`.
//!
//! No session or PAT reaches these routes: each request carries the run's
//! worker token as a bearer, and [`authorise`] admits it only for the run it
//! was minted for and only while this process waits on that run. A token for
//! a finished run, or any other run, is a 401.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use systemprompt::agent_runs::EventSink;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use crate::services::runner::fly::protocol::{
    Bundle, EventBatch, HeartbeatReply, OUTPUT_DIRS, SessionReport, WorkerResult, collect_bundle,
    write_bundle,
};
use crate::services::runner::fly::registry::{RemoteRun, RemoteRuns};

const BUNDLE_DIRS: [&str; 2] = ["workspace", "plugin"];

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn admit(runs: &RemoteRuns, headers: &HeaderMap, run_id: &str) -> AdminResult<Arc<RemoteRun>> {
    let run = bearer(headers)
        .and_then(|token| runs.authorise(run_id, token))
        .ok_or_else(|| AdminError::Unauthorized("no in-flight run for this token".to_owned()))?;
    run.touch();
    Ok(run)
}

fn is_output(path: &str) -> bool {
    OUTPUT_DIRS.iter().any(|dir| {
        path.strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
    })
}

pub(crate) async fn spec(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    Ok(Json(run.spec.clone()).into_response())
}

pub(crate) async fn bundle(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    let root = run.root.clone();
    let bundle = tokio::task::spawn_blocking(move || collect_bundle(&root, &BUNDLE_DIRS))
        .await
        .map_err(AdminError::internal)??;
    Ok(Json(bundle).into_response())
}

pub(crate) async fn events(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
    Json(batch): Json<EventBatch>,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    run.sink
        .emit(batch.events)
        .await
        .map_err(AdminError::internal)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn session(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
    Json(report): Json<SessionReport>,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    run.sink
        .session_observed(&report.session_id)
        .await
        .map_err(AdminError::internal)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(crate) async fn heartbeat(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    let reply = HeartbeatReply {
        cancel: run.cancel_requested(),
    };
    Ok(Json(reply).into_response())
}

pub(crate) async fn outputs(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
    Json(bundle): Json<Bundle>,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    if let Some(stray) = bundle.files.iter().find(|f| !is_output(&f.path)) {
        return Err(AdminError::Unprocessable(format!(
            "{:?} is not under {OUTPUT_DIRS:?}",
            stray.path
        )));
    }
    let root = run.root.clone();
    let written = tokio::task::spawn_blocking(move || write_bundle(&root, &bundle))
        .await
        .map_err(AdminError::internal)??;
    Ok(Json(serde_json::json!({ "written": written })).into_response())
}

pub(crate) async fn result(
    State(runs): State<Arc<RemoteRuns>>,
    Path(run_id): Path<String>,
    headers: HeaderMap,
    Json(result): Json<WorkerResult>,
) -> AdminResult<Response> {
    let run = admit(&runs, &headers, &run_id)?;
    if run.deliver(result) {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AdminError::Conflict(
            "this run's result was already delivered".to_owned(),
        ))
    }
}
