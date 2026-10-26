//! Core's `config_plane_state` row for a plane the console shows: the last
//! reconcile pass and the last explicit apply. Core writes the table; this
//! only reads it.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub struct CorePlaneStateRow {
    pub plane: String,
    pub authority: String,
    pub declared_hash: Option<String>,
    pub drift_rows: i32,
    pub outcome: String,
    pub reconciled_at: DateTime<Utc>,
    pub last_apply_declared_hash: Option<String>,
    pub last_apply_actor: Option<String>,
    pub last_apply_mode: Option<String>,
    pub last_apply_at: Option<DateTime<Utc>>,
}

pub async fn find_core_plane_state(
    pool: &PgPool,
    plane: &str,
) -> Result<Option<CorePlaneStateRow>, sqlx::Error> {
    sqlx::query_as!(
        CorePlaneStateRow,
        r"SELECT plane, authority, declared_hash, drift_rows, outcome, reconciled_at,
                 last_apply_declared_hash, last_apply_actor, last_apply_mode, last_apply_at
            FROM config_plane_state
           WHERE plane = $1",
        plane
    )
    .fetch_optional(pool)
    .await
}
