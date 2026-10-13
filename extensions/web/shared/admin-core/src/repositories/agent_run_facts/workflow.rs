//! A workflow step's project, read from `workflow_runs` where that table
//! exists.
//!
//! `workflow_runs` is an astound table; this shared crate also builds on
//! instances without it, so the sqlx macros (checked against the local
//! schema) cannot name it. The lookup is the crate's one unchecked read and
//! is listed in `scripts/check-sqlx.sh` for that reason. Postgres rejects a
//! statement naming a missing relation at parse time, so the existence
//! probe and the read are two statements.

use std::collections::HashMap;

use sqlx::{PgPool, Row};
use systemprompt_web_shared::ProjectId;

pub(super) const WORKFLOW_STEP_ORIGIN: &str = "workflow_step";

pub(super) fn workflow_run_id(origin_id: &str) -> Option<&str> {
    origin_id.split('/').next().filter(|s| !s.is_empty())
}

pub(super) async fn list_workflow_projects(
    pool: &PgPool,
    run_ids: &[String],
) -> Result<HashMap<String, ProjectId>, sqlx::Error> {
    let present: bool =
        sqlx::query_scalar("SELECT to_regclass('public.workflow_runs') IS NOT NULL")
            .fetch_one(pool)
            .await?;
    if !present {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        "SELECT run_id, project_id FROM workflow_runs \
         WHERE run_id = ANY($1) AND project_id IS NOT NULL",
    )
    .bind(run_ids)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let run_id: String = row.try_get("run_id")?;
            let project = ProjectId::new(row.try_get::<String, _>("project_id")?);
            Ok((run_id, project))
        })
        .collect()
}
