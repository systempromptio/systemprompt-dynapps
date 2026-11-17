//! The rows as the database returns them, and their conversion to the
//! store's records.

use chrono::{DateTime, Utc};
// JSON: the message info and parts and the event data are OpenCode documents
// the store keeps without interpreting.
use serde_json::Value;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use crate::ids::IdError;
use crate::store::{EventRecord, MessageRecord, ModelRef, StoreError, ThreadRecord};

pub(super) struct ThreadRow {
    pub(super) id: String,
    pub(super) owner: String,
    pub(super) title: String,
    pub(super) model: Option<Json<ModelRef>>,
    pub(super) revision: i64,
    pub(super) archived_at: Option<DateTime<Utc>>,
    pub(super) deleted_at: Option<DateTime<Utc>>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) change_seq: i64,
    pub(super) last_event_seq: i64,
}

pub(super) struct MessageRow {
    pub(super) id: String,
    pub(super) thread_id: String,
    pub(super) role: String,
    pub(super) info: Value,
    pub(super) parts: Value,
    pub(super) seq: i64,
    pub(super) created_at: DateTime<Utc>,
    pub(super) completed_at: Option<DateTime<Utc>>,
}

pub(super) struct EventRow {
    pub(super) thread_id: String,
    pub(super) seq: i64,
    pub(super) reply: Option<String>,
    pub(super) kind: String,
    pub(super) data: Value,
    pub(super) created_at: DateTime<Utc>,
}

pub(super) fn corrupt(column: &'static str) -> impl FnOnce(IdError) -> StoreError {
    move |error| {
        tracing::error!(%error, column, "dynwork chats row is invalid");
        StoreError::Corrupt(column)
    }
}

impl TryFrom<ThreadRow> for ThreadRecord {
    type Error = StoreError;

    fn try_from(row: ThreadRow) -> Result<Self, StoreError> {
        Ok(Self {
            id: row.id.parse().map_err(corrupt("dynwork_chat_threads.id"))?,
            user: UserId::new(row.owner),
            title: row.title,
            model: row.model.map(|Json(model)| model),
            revision: row.revision,
            archived_at: row.archived_at,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            change_seq: row.change_seq,
            last_event_seq: row.last_event_seq,
        })
    }
}

impl TryFrom<MessageRow> for MessageRecord {
    type Error = StoreError;

    fn try_from(row: MessageRow) -> Result<Self, StoreError> {
        Ok(Self {
            id: row
                .id
                .parse()
                .map_err(corrupt("dynwork_chat_messages.id"))?,
            thread: row
                .thread_id
                .parse()
                .map_err(corrupt("dynwork_chat_messages.thread_id"))?,
            role: row.role.parse().map_err(|()| {
                tracing::error!("dynwork chat message has an unknown role");
                StoreError::Corrupt("dynwork_chat_messages.role")
            })?,
            info: row.info,
            parts: row.parts,
            seq: row.seq,
            created_at: row.created_at,
            completed_at: row.completed_at,
        })
    }
}

impl TryFrom<EventRow> for EventRecord {
    type Error = StoreError;

    fn try_from(row: EventRow) -> Result<Self, StoreError> {
        Ok(Self {
            thread: row
                .thread_id
                .parse()
                .map_err(corrupt("dynwork_chat_events.thread_id"))?,
            seq: row.seq,
            message: row
                .reply
                .map(|id| {
                    id.parse()
                        .map_err(corrupt("dynwork_chat_events.message_id"))
                })
                .transpose()?,
            kind: row.kind,
            data: row.data,
            created_at: row.created_at,
        })
    }
}
