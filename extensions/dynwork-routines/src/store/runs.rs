//! Run summaries. Only the routine's owner device writes them; a run keeps
//! the routine it was first written for.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::types::RunRow;
use super::{PutRun, RunOutcome, RunRecord, StoreError};

pub async fn put_run(pool: &PgPool, run: &PutRun<'_>) -> Result<RunOutcome, StoreError> {
    let mut tx = pool.begin().await?;
    let owner = sqlx::query_scalar!(
        "SELECT owner_device FROM dynwork_routines
         WHERE user_id = $1 AND id = $2 AND deleted_at IS NULL
         FOR SHARE",
        run.user.as_str(),
        run.routine_id.as_str()
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(owner) = owner else {
        return Ok(RunOutcome::RoutineNotFound);
    };
    if owner != run.device {
        return Ok(RunOutcome::NotOwnerDevice);
    }
    let row = sqlx::query_as!(
        RunRow,
        "INSERT INTO dynwork_routine_runs
             (user_id, id, routine_id, state, reason, started_at, ended_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (user_id, id) DO UPDATE
         SET state = EXCLUDED.state, reason = EXCLUDED.reason,
             started_at = EXCLUDED.started_at, ended_at = EXCLUDED.ended_at,
             updated_at = EXCLUDED.updated_at
         WHERE dynwork_routine_runs.routine_id = EXCLUDED.routine_id
         RETURNING id, routine_id, state, reason, started_at, ended_at, updated_at",
        run.user.as_str(),
        run.id.as_str(),
        run.routine_id.as_str(),
        run.state.as_str(),
        run.reason,
        run.started_at,
        run.ended_at,
        run.now
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        return Ok(RunOutcome::RoutineMismatch);
    };
    tx.commit().await?;
    Ok(RunOutcome::Written(row.try_into()?))
}

pub async fn list_recent_runs(
    pool: &PgPool,
    user: &UserId,
    per_routine: i64,
) -> Result<Vec<RunRecord>, StoreError> {
    let rows = sqlx::query_as!(
        RunRow,
        r#"SELECT id AS "id!", routine_id AS "routine_id!", state AS "state!", reason,
                  started_at, ended_at, updated_at AS "updated_at!"
           FROM (
               SELECT runs.*, row_number() OVER (
                   PARTITION BY routine_id
                   ORDER BY COALESCE(started_at, updated_at) DESC, id DESC
               ) AS position
               FROM dynwork_routine_runs runs
               WHERE user_id = $1
           ) ranked
           WHERE position <= $2
           ORDER BY routine_id, COALESCE(started_at, updated_at) DESC, id DESC"#,
        user.as_str(),
        per_routine
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(RunRecord::try_from).collect()
}
