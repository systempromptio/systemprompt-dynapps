//! The JSON API behind `/admin/sync`: status of every source and plane,
//! a plane's drift, the two writes, the export, and the source refresh.
//!
//! Reads are open to the console tier so a project manager can audit; the
//! writes (`POST …/apply`, `POST sources/refresh`) sit in the manage tier.
//! Every write and every export leaves an activity row, awaited inline so
//! the audit trail exists by the time the response does.

mod refresh;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub(crate) use refresh::refresh_sources_handler;

use crate::activity::{self, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories::sync::plane::{ApplyOutcome, PlaneDrift, SyncMode, SyncPlane};
use crate::repositories::sync::registry::{find_plane, planes};
use crate::repositories::sync::sources::{SourcesView, build_sources};
use crate::repositories::sync::state::{SyncStateRow, find_sync_state};
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct ApplyRequest {
    pub mode: SyncMode,
}

#[derive(Debug, Serialize)]
pub(crate) struct ApplyResponse {
    pub plane: &'static str,
    pub mode: SyncMode,
    pub outcome: ApplyOutcome,
    pub drift_after: PlaneDrift,
}

#[derive(Debug, Serialize)]
pub(crate) struct PlaneStatus {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub projection: &'static str,
    pub drift: PlaneDrift,
    pub state: Option<SyncStateRow>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StatusResponse {
    pub sources: SourcesView,
    pub planes: Vec<PlaneStatus>,
}

fn plane_or_404(id: &str) -> AdminResult<Box<dyn SyncPlane>> {
    find_plane(id).ok_or_else(|| AdminError::NotFound(format!("no sync plane '{id}'")))
}

pub(crate) async fn plane_status(pool: &PgPool, plane: &dyn SyncPlane) -> AdminResult<PlaneStatus> {
    let drift = plane.drift(pool).await?;
    let state = find_sync_state(pool, plane.id()).await?;
    Ok(PlaneStatus {
        id: plane.id(),
        label: plane.label(),
        source_file: plane.source_file(),
        projection: plane.projection(),
        drift,
        state,
    })
}

pub(crate) async fn status_handler(State(pool): State<Arc<PgPool>>) -> AdminResult<Response> {
    let sources = build_sources()?;
    let mut out = Vec::new();
    for plane in planes() {
        out.push(plane_status(&pool, plane.as_ref()).await?);
    }
    Ok(Json(StatusResponse {
        sources,
        planes: out,
    })
    .into_response())
}

pub(crate) async fn drift_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plane): Path<String>,
) -> AdminResult<Response> {
    let plane = plane_or_404(&plane)?;
    Ok(Json(plane.drift(&pool).await?).into_response())
}

pub(crate) async fn apply_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(plane): Path<String>,
    Json(body): Json<ApplyRequest>,
) -> AdminResult<Response> {
    let plane = plane_or_404(&plane)?;
    let outcome = plane.apply(&pool, body.mode, &user_ctx.user_id).await?;
    activity::record(
        &pool,
        NewActivity::sync_applied(
            &user_ctx.user_id,
            plane.id(),
            plane.label(),
            body.mode,
            &outcome,
        ),
    )
    .await;
    let drift_after = plane.drift(&pool).await?;
    Ok(Json(ApplyResponse {
        plane: plane.id(),
        mode: body.mode,
        outcome,
        drift_after,
    })
    .into_response())
}

pub(crate) async fn export_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(plane): Path<String>,
) -> AdminResult<Response> {
    let plane = plane_or_404(&plane)?;
    let Some(export) = plane.export(&pool).await? else {
        return Err(AdminError::NotFound(format!(
            "plane '{}' has no file form to export",
            plane.id()
        )));
    };
    activity::record(
        &pool,
        NewActivity::sync_exported(
            &user_ctx.user_id,
            plane.id(),
            plane.label(),
            export.row_count,
        ),
    )
    .await;
    let disposition = format!("attachment; filename=\"{}\"", export.filename);
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, export.content_type),
            (header::CONTENT_DISPOSITION, disposition.as_str()),
        ],
        export.body,
    )
        .into_response())
}
