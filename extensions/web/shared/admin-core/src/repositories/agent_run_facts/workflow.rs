//! A workflow step's project. This instance has no workflow engine, so no
//! `workflow_runs` table exists and a `workflow_step` origin never appears
//! in `agent_runs`; the lookup is an empty map by construction. The origin
//! helpers stay so the facts view reads the same here and on an instance
//! that does run workflows.

use std::collections::HashMap;

use sqlx::PgPool;
use systemprompt_web_shared::ProjectId;

pub(super) const WORKFLOW_STEP_ORIGIN: &str = "workflow_step";

pub(super) fn workflow_run_id(origin_id: &str) -> Option<&str> {
    origin_id.split('/').next().filter(|s| !s.is_empty())
}

pub(super) async fn list_workflow_projects(
    _pool: &PgPool,
    _run_ids: &[String],
) -> Result<HashMap<String, ProjectId>, sqlx::Error> {
    Ok(HashMap::new())
}
