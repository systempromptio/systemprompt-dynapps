//! Row shapes as the queries return them, before conversion into the
//! store's records.

use std::str::FromStr;

use chrono::{DateTime, Utc};
// JSON: protocol boundary — the JSONB definition column.
use serde_json::Value;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use super::{RevisionRecord, RoutineRecord, RunRecord, StoreError, TemplateRecord};
use crate::ids::{RoutineId, RunId, TemplateId};
use crate::template::{TemplateBody, UnknownValue};

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

pub(super) struct TemplateRow {
    pub(super) id: String,
    pub(super) revision: i64,
    pub(super) state: String,
    pub(super) retire_mode: Option<String>,
    pub(super) mandatory: bool,
    pub(super) plugins: Vec<String>,
    pub(super) source: String,
    pub(super) body: Json<TemplateBody>,
    pub(super) created_by: String,
    pub(super) updated_by: String,
    pub(super) updated_at: DateTime<Utc>,
}

impl TryFrom<TemplateRow> for TemplateRecord {
    type Error = StoreError;

    fn try_from(row: TemplateRow) -> Result<Self, StoreError> {
        Ok(Self {
            id: TemplateId::from_stored(row.id),
            revision: row.revision,
            state: parsed(&row.state, "dynwork_routine_templates.state")?,
            retire_mode: row
                .retire_mode
                .map(|mode| parsed(&mode, "dynwork_routine_templates.retire_mode"))
                .transpose()?,
            mandatory: row.mandatory,
            plugins: row.plugins,
            source: parsed(&row.source, "dynwork_routine_templates.source")?,
            body: row.body.0,
            created_by: UserId::new(row.created_by),
            updated_by: UserId::new(row.updated_by),
            updated_at: row.updated_at,
        })
    }
}

fn parsed<T: FromStr<Err = UnknownValue>>(
    text: &str,
    column: &'static str,
) -> Result<T, StoreError> {
    text.parse().map_err(|error| {
        tracing::error!(%error, column, "dynwork routine template row has an unknown value");
        StoreError::Corrupt(column)
    })
}

pub(super) struct RevisionRow {
    pub(super) template_id: String,
    pub(super) revision: i64,
    pub(super) mandatory: bool,
    pub(super) body: Json<TemplateBody>,
    pub(super) published_by: String,
    pub(super) published_at: DateTime<Utc>,
}

impl From<RevisionRow> for RevisionRecord {
    fn from(row: RevisionRow) -> Self {
        Self {
            template_id: TemplateId::from_stored(row.template_id),
            revision: row.revision,
            mandatory: row.mandatory,
            body: row.body.0,
            published_by: UserId::new(row.published_by),
            published_at: row.published_at,
        }
    }
}
