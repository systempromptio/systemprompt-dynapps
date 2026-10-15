//! The runner machine a run executed on, when it ran on a Fly Machine
//! rather than as a child of the API process.

use sqlx::PgPool;

use crate::types::FarmRunId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerMachineRow {
    pub runner_machine_id: Option<String>,
    pub runner_region: Option<String>,
}

pub async fn update_run_runner(
    pool: &PgPool,
    id: &FarmRunId,
    machine_id: &str,
    region: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/runs/update_run_runner.sql",
        id.as_str(),
        machine_id,
        region,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn find_run_runner(
    pool: &PgPool,
    id: &FarmRunId,
) -> Result<Option<RunnerMachineRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunnerMachineRow,
        "src/repositories/runs/find_run_runner.sql",
        id.as_str()
    )
    .fetch_optional(pool)
    .await
}
