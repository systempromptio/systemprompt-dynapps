//! The JSON API behind `/admin/sync`: status of every source and plane,
//! a plane's drift, the writes, the export, and the source refresh.
//!
//! The planes core owns (access control, governance, gateway policies) are
//! read and written through core alone ([`CorePlanes`]): an administrator's
//! apply is core's entity merge, and an apply naming entities is core's
//! scoped apply. The install planes (groups, gateway routes) keep their two
//! directions. Reads are open to the console tier. The plane apply, the
//! per-entity "keep the database" decision and the source refresh sit in the
//! participant tier: an administrator applies a whole plane; a marketplace
//! participant must name the marketplaces they are syncing, every one inside
//! their participation, on the access-control plane alone. Every apply
//! carries a stated reason. The archive import stays in the manage
//! tier. Every write and every export leaves an activity row, awaited inline
//! so the audit trail exists by the time the response does.

mod archive;
mod core;
mod refresh;
mod status;

use crate::handlers::deps::PlaneDeps;
use std::sync::Arc;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

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
pub(crate) use status::status_handler;

use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::{NewActivity, PlaneApply};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::sync::core_planes::{CorePlanes, core_plane_meta};
use systemprompt_web_admin_core::repositories::sync::plane::{
    Actor, ApplyOutcome, PlaneDrift, SyncMode, SyncPlane,
};
use systemprompt_web_admin_core::repositories::sync::plane_ids;
use systemprompt_web_admin_core::repositories::sync::registry::find_plane;
use systemprompt_web_admin_core::types::UserContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ApplyMode {
    InsertOnly,
    Overwrite,
    Merge,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApplyRequest {
    pub mode: ApplyMode,
    #[serde(default)]
    pub entities: Vec<String>,
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

fn authorise_entities(user_ctx: &UserContext, entities: &[String]) -> AdminResult<()> {
    if user_ctx.is_console {
        return Ok(());
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
    Ok(())
}

#[derive(Debug, Serialize)]
pub(crate) struct ApplyResponse {
    pub plane: &'static str,
    pub mode: SyncMode,
    pub outcome: ApplyOutcome,
    pub drift_after: PlaneDrift,
}

fn plane_or_404(boot: &ExtensionBoot, id: &str) -> AdminResult<Box<dyn SyncPlane>> {
    find_plane(boot, id).ok_or_else(|| AdminError::NotFound(format!("no sync plane '{id}'")))
}

pub(crate) async fn drift_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(core_planes): Extension<CorePlanes>,
    Extension(boot): Extension<ExtensionBoot>,
    Path(plane): Path<String>,
) -> AdminResult<Response> {
    if core_plane_meta(&plane).is_some() {
        return Ok(Json(core_planes.diff(&plane).await?).into_response());
    }
    let plane = plane_or_404(&boot, &plane)?;
    Ok(Json(plane.drift(&pool).await?).into_response())
}

pub(crate) async fn apply_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    PlaneDeps { core_planes, boot }: PlaneDeps,
    Path(plane): Path<String>,
    Json(body): Json<ApplyRequest>,
) -> AdminResult<Response> {
    let reason = required_reason(body.reason.as_deref())?;
    if let Some(meta) = core_plane_meta(&plane) {
        authorise_entities(&user_ctx, &body.entities)?;
        if !body.entities.is_empty() && meta.id != plane_ids::ACCESS_CONTROL {
            return Err(AdminError::Unprocessable(format!(
                "the {} plane cannot be applied for single entities",
                meta.label
            )));
        }
        return core::apply_core(&pool, &core_planes, &user_ctx, meta, &body).await;
    }
    let plane = plane_or_404(&boot, &plane)?;
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden(
            "Applying this plane needs an administrator".to_owned(),
        ));
    }
    if !body.entities.is_empty() {
        return Err(AdminError::Unprocessable(format!(
            "the {} plane cannot be applied for single entities",
            plane.label()
        )));
    }
    let mode = match body.mode {
        ApplyMode::InsertOnly => SyncMode::InsertOnly,
        ApplyMode::Overwrite => SyncMode::Overwrite,
        ApplyMode::Merge => {
            return Err(AdminError::BadRequest(format!(
                "the {} plane applies insert_only or overwrite",
                plane.label()
            )));
        },
    };
    let outcome = plane
        .apply(&pool, mode, Actor::User(&user_ctx.user_id))
        .await?;
    activity::record(
        &pool,
        NewActivity::sync_applied(
            &user_ctx.user_id,
            PlaneApply {
                plane: plane.id(),
                label: plane.label(),
                mode,
                outcome: &outcome,
                reason: Some(reason),
            },
        ),
    )
    .await;
    let drift_after = plane.drift(&pool).await?;
    Ok(Json(ApplyResponse {
        plane: plane.id(),
        mode,
        outcome,
        drift_after,
    })
    .into_response())
}

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
    authorise_entities(&user_ctx, &keys)?;
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
    Extension(core_planes): Extension<CorePlanes>,
    Extension(boot): Extension<ExtensionBoot>,
    Path(plane): Path<String>,
) -> AdminResult<Response> {
    if let Some(meta) = core_plane_meta(&plane) {
        return core::export_core(&pool, &core_planes, &user_ctx, meta).await;
    }
    let plane = plane_or_404(&boot, &plane)?;
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
    Ok(file_response(
        export.content_type,
        export.filename,
        export.body,
    ))
}

fn file_response(content_type: &str, filename: &str, body: String) -> Response {
    let disposition = format!("attachment; filename=\"{filename}\"");
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, disposition.as_str()),
        ],
        body,
    )
        .into_response()
}
