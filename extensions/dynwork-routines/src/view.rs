//! What the desktop API answers with: a routine (or its tombstone) and a
//! page of the change feed. A run answers as its [`crate::store::RunRecord`].

use chrono::{DateTime, Utc};
use serde::Serialize;
// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
use serde_json::Value;

use crate::ids::RoutineId;
use crate::store::RoutineRecord;

// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
#[derive(Debug, Clone, Serialize)]
pub struct RoutineView {
    pub id: RoutineId,
    pub revision: i64,
    pub owner_device: String,
    pub owner_name: String,
    pub definition: Option<Value>,
    pub deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl From<RoutineRecord> for RoutineView {
    fn from(record: RoutineRecord) -> Self {
        let deleted = record.deleted_at.is_some();
        Self {
            id: record.id,
            revision: record.revision,
            owner_device: record.owner_device,
            owner_name: record.owner_name,
            definition: (!deleted).then_some(record.definition),
            deleted,
            deleted_at: record.deleted_at,
            updated_at: record.updated_at,
        }
    }
}

/// One page of the feed. `cursor` is opaque to the desktop: it sends it back
/// as `since`. `more` says another page is waiting now.
#[derive(Debug, Clone, Serialize)]
pub struct ChangesPage {
    pub routines: Vec<RoutineView>,
    pub cursor: String,
    pub more: bool,
}
