//! The JSON API behind `/admin/sync`: status of every source and plane,
//! a plane's drift, the two writes, the export, and the source refresh.
//!
//! Reads are open to the console tier. The plane apply, the per-entity
//! "keep the database" decision and the source refresh sit in the
//! participant tier: an administrator applies a whole plane; a marketplace
//! participant must name the marketplaces they are syncing, every one inside
//! their participation, and the apply is narrowed to those entities. Every
//! apply carries a stated reason. The archive import stays in the manage
//! tier. Every write and every export leaves an activity row, awaited inline
//! so the audit trail exists by the time the response does.

mod archive;
mod refresh;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub(crate) use archive::{
    apply_import_handler, discard_import_handler, export_zip_handler, stage_import_handler,
};
pub(crate) use refresh::refresh_sources_handler;

use crate::activity::{self, NewActivity, PlaneApply};
use crate::error::{AdminError, AdminResult};
use crate::repositories::sync::plane::{
    Actor, ApplyOutcome, EntityScope, PlaneDrift, SyncMode, SyncPlane,
};
use crate::repositories::sync::registry::{find_plane, planes};
use crate::repositories::sync::sources::{SourcesView, build_sources};
use crate::repositories::sync::state::{SyncStateRow, find_sync_state};
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct ApplyRequest {
    pub mode: SyncMode,
    // Why: `<kind>/<id>` keys the apply is narrowed to. Required from a
    // participant, optional from an administrator.
    #[serde(default)]
    pub entities: Vec<String>,
    // Why: every apply is a person's decision to overwrite what the
    // database holds, and the trail records why they took it.
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct KeepRequest {
    pub entity: String,
    pub fingerprint: String,
    pub reason: String,
}

fn required_reason(reason: Option<&str>) -> AdminResult<&str> {
    reason
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .ok_or_else(|| AdminError::BadRequest("a reason is required to apply a sync".to_owned()))
}

// Why: the rule that makes a participant's apply safe: every key must be a
// marketplace inside their participation. Anything else — another kind, a
// marketplace they do not reach, or no scope at all — is refused before the
// plane is touched. An administrator's request passes through unchanged.
fn authorise_scope<'a>(
    user_ctx: &UserContext,
    entities: &'a [String],
) -> AdminResult<EntityScope<'a>> {
    if user_ctx.is_console {
        return Ok(if entities.is_empty() {
            EntityScope::All
        } else {
            EntityScope::Entities(entities)
        });
    }
    if entities.is_empty() {
        return Err(AdminError::Forbidden(
            "Name the marketplaces to sync; a plane-wide apply needs an administrator".to_owned(),
        ));
    }
    for key in entities {
        let allowed = key
            .strip_prefix("marketplace/")
            .is_some_and(|id| user_ctx.participation.includes_marketplace(id));
        if !allowed {
            return Err(AdminError::Forbidden(format!(
                "'{key}' is not a marketplace you participate in"
            )));
        }
    }
    Ok(EntityScope::Entities(entities))
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
    pub runtime_note: Option<&'static str>,
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
        runtime_note: plane.runtime_note(),
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
    let scope = authorise_scope(&user_ctx, &body.entities)?;
    let reason = required_reason(body.reason.as_deref())?;
    let outcome = plane
        .apply_scoped(&pool, body.mode, Actor::User(&user_ctx.user_id), scope)
        .await?;
    activity::record(
        &pool,
        NewActivity::sync_applied(
            &user_ctx.user_id,
            PlaneApply {
                plane: plane.id(),
                label: plane.label(),
                mode: body.mode,
                outcome: &outcome,
                reason: Some(reason),
            },
            scope,
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

// Why: "keep the database" writes no rule — the activity row is the whole
// decision, and the review holds the entity out of the to-do list for as
// long as its diff stays the one the decision was taken against.
pub(crate) async fn keep_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<KeepRequest>,
) -> AdminResult<Response> {
    let reason = required_reason(Some(&body.reason))?;
    if body.fingerprint.trim().is_empty() {
        return Err(AdminError::BadRequest("fingerprint required".to_owned()));
    }
    let keys = [body.entity.clone()];
    authorise_scope(&user_ctx, &keys)?;
    activity::record(
        &pool,
        NewActivity::sync_kept(&user_ctx.user_id, &body.entity, &body.fingerprint, reason),
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
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
