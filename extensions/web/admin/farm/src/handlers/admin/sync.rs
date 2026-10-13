//! `POST /agent-farm/farms/{id}/sync`: capture the farm's Git source at its
//! configured ref, project it into the farm tables, record the action in the
//! activity feed, and return the sync report.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::PgPool;
use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::{ActivityAction, ActivityCategory, NewActivity};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::types::UserContext;

use crate::services::farm_sync::{
    FarmSourceCapture, FarmSyncReport, NativeCapture, sync_farm as run_sync,
};
use crate::state::FarmState;
use crate::types::FarmId;

const SCRATCH_DIR: &str = "scratch";

fn farm_synced(ctx: &UserContext, farm_id: &FarmId, report: &FarmSyncReport) -> NewActivity {
    NewActivity {
        user_id: ctx.user_id.clone(),
        category: ActivityCategory::UserManagement,
        action: ActivityAction::Imported,
        entity: None,
        description: format!(
            "Synced agent farm {} at {}: {} agents, {} eval suites",
            farm_id.as_str(),
            report.commit,
            report.agents,
            report.suites
        ),
        // JSON: JSONB column — the farm and the commit the sync captured.
        metadata: serde_json::json!({ "farm_id": farm_id.as_str(), "commit": report.commit }),
    }
}

pub(crate) async fn sync_farm(
    State(pool): State<Arc<PgPool>>,
    Extension(ctx): Extension<UserContext>,
    Extension(state): Extension<Arc<FarmState>>,
    Path(farm_id): Path<FarmId>,
) -> AdminResult<Response> {
    let capture: Arc<dyn FarmSourceCapture> =
        Arc::new(NativeCapture::new(state.paths.data_root.join(SCRATCH_DIR)));
    let report = run_sync(&state, capture, &farm_id, Some(&ctx.user_id)).await?;
    activity::record(&pool, farm_synced(&ctx, &farm_id, &report)).await;
    Ok(Json(report).into_response())
}
