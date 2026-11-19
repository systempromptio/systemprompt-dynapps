//! The `OpenCode` v2 session events a cloud reply emits, and their envelope.
//!
//! Every field is one the desktop fixtures carry and the phone's reducer
//! reads. A cloud thread has no directory, so `location` is
//! `{ "directory": "" }` and `subpath` is `""`. The store keeps each event's
//! `type` and `data`; the envelope is rebuilt from the row, so a replay and the
//! live copy are the same bytes: `id` is derived from the row's time and
//! `seq`, `created` is the row's time in epoch milliseconds, and `location`
//! and `durable` appear on exactly the types the desktop gives them (a text
//! delta and a usage update are not durable there, so they carry no
//! `durable` here either; their `seq` travels as the SSE `id`).

// JSON: an event's `data` and its envelope are `OpenCode` wire documents with
// a fixed shape per event type.
use serde_json::{Map, Value, json};

use crate::ids::{MessageId, ThreadId};
use crate::reply::model::ReplyError;
pub use crate::reply::tokens::Tokens;
use crate::store::{EventRecord, ModelRef, NewEvent};

pub const CREATED: &str = "session.created";
pub const EXECUTION_STARTED: &str = "session.execution.started";
pub const EXECUTION_SUCCEEDED: &str = "session.execution.succeeded";
pub const EXECUTION_FAILED: &str = "session.execution.failed";
pub const EXECUTION_INTERRUPTED: &str = "session.execution.interrupted";
pub const STEP_STARTED: &str = "session.step.started";
pub const STEP_ENDED: &str = "session.step.ended";
pub const USAGE_UPDATED: &str = "session.usage.updated";
pub const TEXT_STARTED: &str = "session.text.started";
pub const TEXT_DELTA: &str = "session.text.delta";
pub const TEXT_ENDED: &str = "session.text.ended";

pub const AGENT: &str = "chat";
pub const PROJECT: &str = "cloud";
pub const VERSION: &str = "cloud";
pub const TEXT_ORDINAL: u32 = 0;

const LOCATED: [&str; 6] = [
    CREATED,
    STEP_STARTED,
    STEP_ENDED,
    TEXT_STARTED,
    TEXT_DELTA,
    TEXT_ENDED,
];
const EPHEMERAL: [&str; 2] = [TEXT_DELTA, USAGE_UPDATED];
const TERMINAL: [&str; 3] = [EXECUTION_SUCCEEDED, EXECUTION_FAILED, EXECUTION_INTERRUPTED];

pub fn is_located(kind: &str) -> bool {
    LOCATED.contains(&kind)
}

pub fn is_durable(kind: &str) -> bool {
    !EPHEMERAL.contains(&kind)
}

pub fn is_terminal(kind: &str) -> bool {
    TERMINAL.contains(&kind)
}

pub fn model_json(model: &ModelRef) -> Value {
    json!({ "id": model.model_id, "providerID": model.provider_id })
}

pub fn error_json(error: &ReplyError) -> Value {
    json!({ "type": error.kind, "message": error.message })
}

fn event(reply: Option<&MessageId>, kind: &str, data: Value) -> NewEvent {
    NewEvent {
        message: reply.cloned(),
        kind: kind.to_owned(),
        data,
    }
}

pub fn session_created(thread: &ThreadId, title: &str, model: &ModelRef) -> NewEvent {
    event(
        None,
        CREATED,
        json!({
            "sessionID": thread,
            "slug": thread,
            "version": VERSION,
            "projectID": PROJECT,
            "location": { "directory": "" },
            "subpath": "",
            "title": title,
            "model": model_json(model),
        }),
    )
}

pub fn execution_started(thread: &ThreadId, reply: &MessageId) -> NewEvent {
    event(
        Some(reply),
        EXECUTION_STARTED,
        json!({ "sessionID": thread }),
    )
}

pub fn step_started(
    thread: &ThreadId,
    reply: &MessageId,
    model: &ModelRef,
    started: i64,
) -> NewEvent {
    event(
        Some(reply),
        STEP_STARTED,
        json!({
            "sessionID": thread,
            "agent": AGENT,
            "model": model_json(model),
            "assistantMessageID": reply,
            "started": started,
        }),
    )
}

pub fn text_started(thread: &ThreadId, reply: &MessageId) -> NewEvent {
    event(
        Some(reply),
        TEXT_STARTED,
        json!({ "sessionID": thread, "assistantMessageID": reply, "ordinal": TEXT_ORDINAL }),
    )
}

pub fn text_delta(thread: &ThreadId, reply: &MessageId, delta: &str) -> NewEvent {
    event(
        Some(reply),
        TEXT_DELTA,
        json!({
            "sessionID": thread,
            "assistantMessageID": reply,
            "ordinal": TEXT_ORDINAL,
            "delta": delta,
        }),
    )
}

pub fn text_ended(thread: &ThreadId, reply: &MessageId, text: &str) -> NewEvent {
    event(
        Some(reply),
        TEXT_ENDED,
        json!({
            "sessionID": thread,
            "assistantMessageID": reply,
            "ordinal": TEXT_ORDINAL,
            "text": text,
        }),
    )
}

pub fn step_ended(
    thread: &ThreadId,
    reply: &MessageId,
    finish: &str,
    raw_finish: &str,
    tokens: Tokens,
) -> NewEvent {
    event(
        Some(reply),
        STEP_ENDED,
        json!({
            "sessionID": thread,
            "assistantMessageID": reply,
            "finish": finish,
            "rawFinish": raw_finish,
            "tokens": tokens.to_json(),
            "cost": 0,
        }),
    )
}

pub fn usage_updated(thread: &ThreadId, reply: &MessageId, total: Tokens) -> NewEvent {
    event(
        Some(reply),
        USAGE_UPDATED,
        json!({ "sessionID": thread, "cost": 0, "tokens": total.to_json() }),
    )
}

pub fn execution_succeeded(thread: &ThreadId, reply: &MessageId) -> NewEvent {
    event(
        Some(reply),
        EXECUTION_SUCCEEDED,
        json!({ "sessionID": thread }),
    )
}

pub fn execution_failed(thread: &ThreadId, reply: &MessageId, error: &ReplyError) -> NewEvent {
    event(
        Some(reply),
        EXECUTION_FAILED,
        json!({ "sessionID": thread, "error": error_json(error) }),
    )
}

pub fn execution_interrupted(thread: &ThreadId, reply: &MessageId) -> NewEvent {
    event(
        Some(reply),
        EXECUTION_INTERRUPTED,
        json!({ "sessionID": thread, "reason": "user" }),
    )
}

pub fn event_id(record: &EventRecord) -> String {
    format!(
        "evt_{:012x}{:014}",
        record.created_at.timestamp_millis().max(0),
        record.seq.max(0)
    )
}

pub fn envelope(record: &EventRecord) -> Value {
    let mut event = Map::new();
    event.insert("id".to_owned(), Value::from(event_id(record)));
    event.insert(
        "created".to_owned(),
        Value::from(record.created_at.timestamp_millis()),
    );
    event.insert("type".to_owned(), Value::from(record.kind.as_str()));
    if is_located(&record.kind) {
        event.insert("location".to_owned(), json!({ "directory": "" }));
    }
    event.insert("data".to_owned(), record.data.clone());
    if is_durable(&record.kind) {
        event.insert(
            "durable".to_owned(),
            json!({ "aggregateID": record.thread, "seq": record.seq, "version": 1 }),
        );
    }
    Value::Object(event)
}
