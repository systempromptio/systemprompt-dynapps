//! The core-owned planes on `/admin/sync`: their status from core's
//! `config_plane_state`, an apply through [`CorePlanes`], and the export of
//! the plane's own file form.

use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::{CorePlaneApply, NewActivity};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::sync::core_plane_state::{
    CorePlaneStateRow, find_core_plane_state,
};
use systemprompt_web_admin_core::repositories::sync::core_planes::{
    CorePlaneMeta, CorePlanes, core_plane_meta,
};
use systemprompt_web_admin_core::types::UserContext;

use super::ApplyRequest;

#[derive(Debug, Serialize)]
pub(crate) struct CoreApplyResponse {
    pub plane: &'static str,
    pub written: usize,
    pub conflicts: usize,
    pub drift: usize,
    pub outcome: systemprompt::extension::prelude::ApplyOutcome,
}

#[derive(Debug, Serialize)]
pub(crate) struct CorePlaneStatus {
    pub id: &'static str,
    pub label: &'static str,
    pub declared_path: String,
    pub declared_hash: Option<String>,
    pub drift_rows: usize,
    pub state: Option<CorePlaneStateRow>,
}

pub(crate) async fn core_plane_status(
    pool: &PgPool,
    core_planes: &CorePlanes,
    id: &str,
) -> AdminResult<CorePlaneStatus> {
    let meta =
        core_plane_meta(id).ok_or_else(|| AdminError::NotFound(format!("no core plane '{id}'")))?;
    let diff = core_planes.diff(id).await?;
    Ok(CorePlaneStatus {
        id: meta.id,
        label: meta.label,
        declared_path: diff.declared_path,
        declared_hash: diff.declared_hash,
        drift_rows: diff.rows.len(),
        state: find_core_plane_state(pool, id).await?,
    })
}

pub(super) async fn apply_core(
    pool: &PgPool,
    core_planes: &CorePlanes,
    user_ctx: &UserContext,
    meta: CorePlaneMeta,
    body: &ApplyRequest,
) -> AdminResult<Response> {
    let reason = super::required_reason(body.reason.as_deref())?;
    let entities = body.entities.as_slice();
    let (written, conflicts, drift, outcome) = if entities.is_empty() {
        let merged = core_planes
            .apply(meta.id, &user_ctx.user_id, reason)
            .await?;
        (
            merged.applied.len(),
            merged.conflicts.len(),
            merged.drift.len(),
            merged.outcome,
        )
    } else {
        let applied = core_planes
            .apply_entities(meta.id, entities, &user_ctx.user_id, reason)
            .await?;
        (applied.changes, 0, applied.remaining_drift, applied.outcome)
    };
    activity::record(
        pool,
        NewActivity::sync_core_applied(
            &user_ctx.user_id,
            CorePlaneApply {
                plane: meta.id,
                label: meta.label,
                entities,
                written,
                conflicts,
                drift,
                reason,
            },
        ),
    )
    .await;
    Ok(Json(CoreApplyResponse {
        plane: meta.id,
        written,
        conflicts,
        drift,
        outcome,
    })
    .into_response())
}

pub(super) async fn export_core(
    pool: &PgPool,
    core_planes: &CorePlanes,
    user_ctx: &UserContext,
    meta: CorePlaneMeta,
) -> AdminResult<Response> {
    let Some(body) = core_planes.export(meta.id).await? else {
        return Err(AdminError::NotFound(format!(
            "plane '{}' has no file form to export",
            meta.id
        )));
    };
    let diff = core_planes.diff(meta.id).await?;
    let filename = diff
        .declared_path
        .rsplit('/')
        .next()
        .unwrap_or(meta.id)
        .to_owned();
    activity::record(
        pool,
        NewActivity::sync_exported(&user_ctx.user_id, meta.id, meta.label, body.lines().count()),
    )
    .await;
    Ok(super::file_response(
        "application/yaml; charset=utf-8",
        &filename,
        body,
    ))
}
