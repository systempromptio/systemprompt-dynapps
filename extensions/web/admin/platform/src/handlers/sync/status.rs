//! `GET /sync/status`: every source, every install plane's drift and last
//! apply, and every core plane's state.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::sync::core_planes::{CORE_PLANE_IDS, CorePlanes};
use systemprompt_web_admin_core::repositories::sync::plane::{PlaneDrift, SyncPlane};
use systemprompt_web_admin_core::repositories::sync::registry::planes;
use systemprompt_web_admin_core::repositories::sync::sources::{SourcesView, build_sources};
use systemprompt_web_admin_core::repositories::sync::state::{SyncStateRow, find_sync_state};

use super::core::{CorePlaneStatus, core_plane_status};

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
    pub core_planes: Vec<CorePlaneStatus>,
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

pub(crate) async fn status_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(core_planes): Extension<CorePlanes>,
) -> AdminResult<Response> {
    let sources = build_sources()?;
    let mut out = Vec::new();
    for plane in planes() {
        out.push(plane_status(&pool, plane.as_ref()).await?);
    }
    let mut core = Vec::new();
    for id in CORE_PLANE_IDS {
        core.push(core_plane_status(&pool, &core_planes, id).await?);
    }
    Ok(Json(StatusResponse {
        sources,
        planes: out,
        core_planes: core,
    })
    .into_response())
}
