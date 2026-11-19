//! The JSON shapes the chats routes speak.
//!
//! A thread is the subset of `OpenCode`'s session info the phone reducer reads
//! (`id`, `title`, `time.created`, `time.updated`, `time.archived` in epoch
//! milliseconds) plus `revision`, `model` (`{ providerID, id }`, `OpenCode`'s
//! model ref) and `deleted`. It also carries the routines feed's RFC 3339
//! `created_at`, `updated_at`, `archived_at` and `deleted_at`, which the
//! desktop's Chats view reads. A message is the stored `OpenCode` message
//! document, with its `id` and `type` taken from the row.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// JSON: a message is an `OpenCode` document the server stores and serves
// without interpreting it.
use serde_json::{Map, Value};

use crate::store::{MessageRecord, ModelRef, ThreadRecord};

/// A model as `OpenCode` names it on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireModel {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    pub id: String,
}

impl From<&ModelRef> for WireModel {
    fn from(model: &ModelRef) -> Self {
        Self {
            provider_id: model.provider_id.clone(),
            id: model.model_id.clone(),
        }
    }
}

impl From<WireModel> for ModelRef {
    fn from(model: WireModel) -> Self {
        Self {
            provider_id: model.provider_id,
            model_id: model.id,
        }
    }
}

/// The epoch-millisecond times of a thread, as `OpenCode`'s session info.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WireTimes {
    pub created: i64,
    pub updated: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<i64>,
}

/// A thread row as the routes answer it; a tombstone has `deleted: true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WireThread {
    pub id: String,
    pub title: String,
    pub model: Option<WireModel>,
    pub revision: i64,
    pub time: WireTimes,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<&ThreadRecord> for WireThread {
    fn from(record: &ThreadRecord) -> Self {
        Self {
            id: record.id.as_str().to_owned(),
            title: record.title.clone(),
            model: record.model.as_ref().map(WireModel::from),
            revision: record.revision,
            time: WireTimes {
                created: record.created_at.timestamp_millis(),
                updated: record.updated_at.timestamp_millis(),
                archived: record.archived_at.map(|at| at.timestamp_millis()),
            },
            deleted: record.deleted_at.is_some(),
            created_at: record.created_at,
            updated_at: record.updated_at,
            archived_at: record.archived_at,
            deleted_at: record.deleted_at,
        }
    }
}

/// A page of the changes feed: `cursor` is opaque, `more` says another page
/// is ready now.
#[derive(Debug, Clone, Serialize)]
pub struct WireChanges {
    pub threads: Vec<WireThread>,
    pub cursor: String,
    pub more: bool,
}

/// A page of history, newest first; `before` is the id to page on from.
#[derive(Debug, Clone, Serialize)]
pub struct WireMessages {
    pub messages: Vec<Value>,
    pub more: bool,
    pub before: Option<String>,
}

pub fn message(record: &MessageRecord) -> Value {
    let mut document = match &record.info {
        Value::Object(fields) => fields.clone(),
        _ => Map::new(),
    };
    document.insert("id".to_owned(), Value::from(record.id.as_str()));
    document.insert("type".to_owned(), Value::from(record.role.as_str()));
    Value::Object(document)
}
