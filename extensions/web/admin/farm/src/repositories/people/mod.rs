//! Queries over the per-user run leaderboard, computed from `agent_farm_runs`
//! over a window with the same columns as the `agent_farm_runs_by_user` view.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::rows::RunsByUserRow;
use crate::types::FarmId;

pub async fn list_runs_by_user(
    pool: &PgPool,
    since: DateTime<Utc>,
    farm_id: Option<&FarmId>,
) -> Result<Vec<RunsByUserRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunsByUserRow,
        "src/repositories/people/list_runs_by_user.sql",
        since,
        farm_id.map(FarmId::as_str),
    )
    .fetch_all(pool)
    .await
}
