//! The records the store returns and the commands it takes.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// JSON: the message info and parts and the event data are OpenCode documents
// the store keeps without interpreting.
use serde_json::Value;
use systemprompt::identifiers::UserId;

use crate::ids::{Cursor, MessageId, ThreadId};

/// The model a thread answers with, as `OpenCode` names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

impl Role {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        match text {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A thread row. A tombstone has `deleted_at` set and nothing else worth
/// reading: its title is empty and its messages are gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadRecord {
    pub id: ThreadId,
    pub user: UserId,
    pub title: String,
    pub model: Option<ModelRef>,
    pub revision: i64,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub change_seq: i64,
    pub last_event_seq: i64,
}

#[derive(Debug, Clone)]
pub struct NewThread {
    pub id: ThreadId,
    pub title: String,
    pub model: Option<ModelRef>,
}

/// What a rename, an archive or a model change sets. `None` leaves a field as
/// it is; a model cannot be cleared once set.
#[derive(Debug, Clone, Default)]
pub struct ThreadPatch {
    pub title: Option<String>,
    pub model: Option<ModelRef>,
    pub archived: Option<bool>,
}

impl ThreadPatch {
    pub const fn is_empty(&self) -> bool {
        self.title.is_none() && self.model.is_none() && self.archived.is_none()
    }
}

/// An edit of a thread: the revision the caller last saw (`If-Match`) and what
/// to change.
#[derive(Debug, Clone)]
pub struct ThreadEdit {
    pub expected_revision: i64,
    pub patch: ThreadPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateOutcome {
    Created(ThreadRecord),
    Exists(ThreadRecord),
    IdTaken,
    UnknownUser,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    Done(ThreadRecord),
    Conflict(ThreadRecord),
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangePage {
    pub threads: Vec<ThreadRecord>,
    pub cursor: Cursor,
    pub has_more: bool,
}

#[derive(Debug, Clone)]
pub struct NewMessage {
    pub id: MessageId,
    pub role: Role,
    pub info: Value,
    pub parts: Value,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageRecord {
    pub id: MessageId,
    pub thread: ThreadId,
    pub role: Role,
    pub info: Value,
    pub parts: Value,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppendOutcome {
    Appended(MessageRecord),
    Duplicate(MessageRecord),
    NoThread,
}

#[derive(Debug, Clone)]
pub struct NewEvent {
    pub message: Option<MessageId>,
    pub kind: String,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    pub thread: ThreadId,
    pub seq: i64,
    pub message: Option<MessageId>,
    pub kind: String,
    pub data: Value,
    pub created_at: DateTime<Utc>,
}

/// The final form of a reply, folded by the caller from its events.
#[derive(Debug, Clone)]
pub struct CompletedReply {
    pub message: MessageId,
    pub info: Value,
    pub parts: Value,
}
