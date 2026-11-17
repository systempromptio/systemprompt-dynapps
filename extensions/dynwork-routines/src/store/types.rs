//! Row shapes as the queries return them, before conversion into the
//! store's records.

use chrono::{DateTime, Utc};
// JSON: protocol boundary — the JSONB definition column.
use serde_json::Value;

use super::{RoutineRecord, RunRecord, StoreError};
use crate::ids::{RoutineId, RunId};

// JSON: protocol boundary — the JSONB definition column.
pub(super) struct RoutineRow {
    pub(super) id: String,
    pub(super) revision: i64,
    pub(super) change_seq: i64,
    pub(super) owner_device: String,
    pub(super) owner_name: String,
    pub(super) definition: Value,
    pub(super) deleted_at: Option<DateTime<Utc>>,
    pub(super) updated_at: DateTime<Utc>,
}

impl From<RoutineRow> for RoutineRecord {
    fn from(row: RoutineRow) -> Self {
        Self {
            id: RoutineId::from_stored(row.id),
            revision: row.revision,
            change_seq: row.change_seq,
            owner_device: row.owner_device,
            owner_name: row.owner_name,
            definition: row.definition,
            deleted_at: row.deleted_at,
            updated_at: row.updated_at,
        }
    }
}

pub(super) struct RunRow {
    pub(super) id: String,
    pub(super) routine_id: String,
    pub(super) state: String,
    pub(super) reason: Option<String>,
    pub(super) started_at: Option<DateTime<Utc>>,
    pub(super) ended_at: Option<DateTime<Utc>>,
    pub(super) updated_at: DateTime<Utc>,
}

impl TryFrom<RunRow> for RunRecord {
    type Error = StoreError;

    fn try_from(row: RunRow) -> Result<Self, StoreError> {
        let state = row.state.parse().map_err(|error| {
            tracing::error!(%error, "dynwork routine run row has an unknown state");
            StoreError::Corrupt("dynwork_routine_runs.state")
        })?;
        Ok(Self {
            id: RunId::from_stored(row.id),
            routine_id: RoutineId::from_stored(row.routine_id),
            state,
            reason: row.reason,
            started_at: row.started_at,
            ended_at: row.ended_at,
            updated_at: row.updated_at,
        })
    }
}
