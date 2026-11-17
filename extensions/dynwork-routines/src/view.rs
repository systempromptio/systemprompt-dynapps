//! What the desktop API answers with: a routine (or its tombstone), a page of
//! the change feed, and the templates a person sees with their links. A run
//! answers as its [`crate::store::RunRecord`].

use chrono::{DateTime, Utc};
use serde::Serialize;
// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
use serde_json::Value;

use crate::ids::{RoutineId, TemplateId};
use crate::store::RoutineRecord;
use crate::template::{LinkState, RetireMode, TemplateView};

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

/// `GET /templates`: the published templates the person sees, and one link
/// per template they have a live task of, seen or not.
#[derive(Debug, Clone, Serialize)]
pub struct TemplatesPage {
    pub templates: Vec<TemplateView>,
    pub links: Vec<LinkView>,
}

/// `retire` says what retiring did to the person's task; it is set exactly
/// when `state` is `retired`.
#[derive(Debug, Clone, Serialize)]
pub struct LinkView {
    pub id: TemplateId,
    pub state: LinkState,
    pub retire: Option<RetireMode>,
}
