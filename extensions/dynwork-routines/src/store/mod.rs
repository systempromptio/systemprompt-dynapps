//! The routines tables over Postgres, with compile-time checked queries.
//!
//! Every write takes its user's `users` row lock for the transaction and
//! numbers its change one past the user's highest `change_seq`. So a user's
//! changes commit in `change_seq` order, and a reader that has seen change
//! `n` has already seen every change below it: the feed loses nothing between
//! two pages. Every time the store compares against "now" it takes it as an
//! argument.

mod linked;
mod routines;
mod runs;
mod templates;
mod types;

use chrono::{DateTime, Utc};
use serde::Serialize;
// JSON: protocol boundary — the definition is DynWork's, stored as it sent it.
use serde_json::Value;
use systemprompt::identifiers::UserId;

pub use linked::{Retired, make_unclaimed, retire_template};
pub use routines::{delete_routine, list_changes, list_live_routines, put_routine, set_enabled};
pub use runs::{list_recent_runs, put_run};
pub use templates::{
    close_template, find_template, find_template_revision, has_had_template,
    list_linked_template_ids, list_templates, publish_template, save_template_draft,
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
    pub template_id: Option<TemplateId>,
    pub template_revision: Option<i64>,
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
/// `link` is the definition's `template`, if it has one.
#[derive(Debug, Clone)]
pub struct PutRoutine<'a> {
    pub user: &'a UserId,
    pub id: &'a RoutineId,
    pub expected: Option<i64>,
    pub owner_device: Option<&'a str>,
    pub owner_name: Option<&'a str>,
    pub definition: &'a Value,
    pub link: Option<PutLink<'a>>,
    pub now: DateTime<Utc>,
}

/// A delete of the revision `expected` (`None`: whichever is current), unless
/// the routine follows one of the `mandatory` templates.
#[derive(Debug, Clone)]
pub struct DeleteRoutine<'a> {
    pub user: &'a UserId,
    pub id: &'a RoutineId,
    pub expected: Option<i64>,
    pub mandatory: &'a [TemplateId],
    pub now: DateTime<Utc>,
}

/// A write's link to revision `revision` of `template`.
///
/// `keeps_locks`: the definition keeps the fields that revision locks.
/// `open`: the template takes a new link from this person (it is published
/// and they see it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PutLink<'a> {
    pub template: &'a TemplateId,
    pub revision: i64,
    pub keeps_locks: bool,
    pub open: bool,
}

// JSON: protocol boundary — the template's defaults in DynWork's shape.
/// The routine a mandatory template owes a person: its definition at the
/// template's `revision`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnclaimedRoutine<'a> {
    pub template: &'a TemplateId,
    pub revision: i64,
    pub definition: Value,
}

/// A routine write's result.
///
/// `Conflict` carries the current row: the routine exists on a create, is a
/// tombstone, or is at another revision. `AlreadyActivated` carries the
/// person's live routine of the template, and `TemplateLocked` the current
/// row, if there is one. `TemplateUnavailable`: a new link to a template that
/// is not published or not seen. `Mandatory`: a delete of a routine of a
/// template mandatory for its person.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteOutcome {
    Written(RoutineRecord),
    Conflict(RoutineRecord),
    NotFound,
    OwnerRequired,
    UnknownUser,
    AlreadyActivated(RoutineRecord),
    TemplateLocked(Option<RoutineRecord>),
    TemplateUnavailable,
    Mandatory,
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
