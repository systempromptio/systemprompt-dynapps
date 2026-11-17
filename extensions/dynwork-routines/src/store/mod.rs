//! The routines tables over Postgres, with compile-time checked queries.
//!
//! Every write takes its user's `users` row lock for the transaction and
//! numbers its change one past the user's highest `change_seq`. So a user's
//! changes commit in `change_seq` order, and a reader that has seen change
//! `n` has already seen every change below it: the feed loses nothing between
//! two pages. Every time the store compares against "now" it takes it as an
//! argument.

mod routines;
mod runs;
mod templates;
mod types;

use chrono::{DateTime, Utc};
use serde::Serialize;
// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
use serde_json::Value;
use systemprompt::identifiers::UserId;

pub use routines::{delete_routine, list_changes, list_live_routines, put_routine, set_enabled};
pub use runs::{list_recent_runs, put_run};
pub use templates::{
    find_template_revision, has_had_template, list_linked_template_ids, list_templates,
    publish_template, save_template_draft,
};

use crate::ids::{RoutineId, RunId, RunState, TemplateId};
pub use crate::template::TemplateState;
use crate::template::{RetireMode, TemplateBody, TemplateSource};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("routines store query failed")]
    Database(#[from] sqlx::Error),
    #[error("routines store row is invalid: {0}")]
    Corrupt(&'static str),
}

// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutineRecord {
    pub id: RoutineId,
    pub revision: i64,
    pub change_seq: i64,
    pub owner_device: String,
    pub owner_name: String,
    pub definition: Value,
    pub deleted_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunRecord {
    pub id: RunId,
    pub routine_id: RoutineId,
    pub state: RunState,
    pub reason: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
/// A create (`expected: None`) or an update of the revision `expected`.
#[derive(Debug, Clone)]
pub struct PutRoutine<'a> {
    pub user: &'a UserId,
    pub id: &'a RoutineId,
    pub expected: Option<i64>,
    pub owner_device: Option<&'a str>,
    pub owner_name: Option<&'a str>,
    pub definition: &'a Value,
    pub now: DateTime<Utc>,
}

/// A routine write's result. `Conflict` carries the current row: the routine
/// exists on a create, is a tombstone, or is at another revision.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteOutcome {
    Written(RoutineRecord),
    Conflict(RoutineRecord),
    NotFound,
    OwnerRequired,
    UnknownUser,
}

#[derive(Debug, Clone)]
pub struct PutRun<'a> {
    pub user: &'a UserId,
    pub id: &'a RunId,
    pub routine_id: &'a RoutineId,
    pub device: &'a str,
    pub state: RunState,
    pub reason: Option<&'a str>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub now: DateTime<Utc>,
}

/// A run write's result. `RoutineMismatch`: the run id is already a run of
/// another routine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Written(RunRecord),
    RoutineNotFound,
    NotOwnerDevice,
    RoutineMismatch,
}

/// A template in the library. Once published, `body`, `mandatory` and
/// `plugins` are those of revision `revision`; a draft is at revision 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateRecord {
    pub id: TemplateId,
    pub revision: i64,
    pub state: TemplateState,
    pub retire_mode: Option<RetireMode>,
    pub mandatory: bool,
    pub plugins: Vec<String>,
    pub source: TemplateSource,
    pub body: TemplateBody,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
}

/// A published revision as it was published, and who published it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionRecord {
    pub template_id: TemplateId,
    pub revision: i64,
    pub mandatory: bool,
    pub body: TemplateBody,
    pub published_by: UserId,
    pub published_at: DateTime<Utc>,
}
