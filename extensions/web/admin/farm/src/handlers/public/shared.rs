//! Lookups every public handler shares: the farm a request names, the run a
//! caller may see, and how the caller reached the API.

use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use crate::repositories::rows::{FarmRow, RunRow};
use crate::repositories::{farms, runs};
use crate::services::entitlement::may_view_run;
use crate::types::{FarmId, FarmRunId, TriggeredVia};

pub(super) const fn triggered_via(ctx: &UserContext) -> TriggeredVia {
    if ctx.session_id.is_some() {
        TriggeredVia::Dashboard
    } else {
        TriggeredVia::Api
    }
}

pub(super) async fn load_farm(pool: &PgPool, farm_id: &FarmId) -> AdminResult<FarmRow> {
    farms::find_farm(pool, farm_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("farm {}", farm_id.as_str())))
}

pub(super) async fn load_visible_run(
    pool: &PgPool,
    ctx: &UserContext,
    run_id: &FarmRunId,
) -> AdminResult<RunRow> {
    let not_found = || AdminError::NotFound(format!("run {}", run_id.as_str()));
    let run = runs::find_run(pool, run_id).await?.ok_or_else(not_found)?;
    let farm = if ctx.is_console || run.user_id == ctx.user_id {
        None
    } else {
        farms::find_farm(pool, &run.farm_id).await?
    };
    if may_view_run(ctx, &run, farm.as_ref()) {
        Ok(run)
    } else {
        Err(not_found())
    }
}
