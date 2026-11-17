//! The memory store's writes, one function per operation, run under its lock.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::state::{Entry, State};
use crate::ids::ThreadId;
use crate::store::{
    AppendOutcome, CompletedReply, CreateOutcome, IdleThread, MessageRecord, NewMessage, NewThread,
    ThreadEdit, ThreadRecord, WriteOutcome,
};

pub(super) fn create(
    state: &mut State,
    user: &UserId,
    thread: &NewThread,
    now: DateTime<Utc>,
) -> CreateOutcome {
    if let Some(existing) = state.threads.get(&thread.id) {
        return if existing.record.user == *user {
            CreateOutcome::Exists(existing.record.clone())
        } else {
            CreateOutcome::IdTaken
        };
    }
    let record = ThreadRecord {
        id: thread.id.clone(),
        user: user.clone(),
        title: thread.title.clone(),
        model: thread.model.clone(),
        revision: 1,
        archived_at: None,
        deleted_at: None,
        created_at: now,
        updated_at: now,
        change_seq: state.next_change(),
        last_event_seq: 0,
    };
    state.threads.insert(
        thread.id.clone(),
        Entry {
            record: record.clone(),
            last_message_seq: 0,
            messages: Vec::new(),
            events: Vec::new(),
        },
    );
    CreateOutcome::Created(record)
}

pub(super) fn update(
    state: &mut State,
    user: &UserId,
    id: &ThreadId,
    edit: &ThreadEdit,
    now: DateTime<Utc>,
) -> WriteOutcome {
    let seq = state.change_seq + 1;
    let Some(entry) = state.live(user, id) else {
        return WriteOutcome::NotFound;
    };
    if entry.record.revision != edit.expected_revision {
        return WriteOutcome::Conflict(entry.record.clone());
    }
    let patch = &edit.patch;
    if patch.is_empty() {
        return WriteOutcome::Done(entry.record.clone());
    }
    let record = &mut entry.record;
    if let Some(title) = &patch.title {
        record.title.clone_from(title);
    }
    if let Some(model) = &patch.model {
        record.model = Some(model.clone());
    }
    match patch.archived {
        Some(true) => {
            record.archived_at.get_or_insert(now);
        },
        Some(false) => record.archived_at = None,
        None => {},
    }
    record.revision += 1;
    record.updated_at = now;
    record.change_seq = seq;
    let done = record.clone();
    state.change_seq = seq;
    WriteOutcome::Done(done)
}

pub(super) fn delete(
    state: &mut State,
    user: &UserId,
    id: &ThreadId,
    expected_revision: i64,
    now: DateTime<Utc>,
) -> WriteOutcome {
    let seq = state.change_seq + 1;
    let Some(entry) = state.live(user, id) else {
        return WriteOutcome::NotFound;
    };
    if entry.record.revision != expected_revision {
        return WriteOutcome::Conflict(entry.record.clone());
    }
    entry.messages.clear();
    entry.events.clear();
    let record = &mut entry.record;
    record.title.clear();
    record.deleted_at = Some(now);
    record.revision += 1;
    record.updated_at = now;
    record.change_seq = seq;
    let done = record.clone();
    state.change_seq = seq;
    WriteOutcome::Done(done)
}

pub(super) fn append(
    state: &mut State,
    user: &UserId,
    thread: &ThreadId,
    message: &NewMessage,
    now: DateTime<Utc>,
) -> AppendOutcome {
    let seq = state.change_seq + 1;
    let Some(entry) = state.live(user, thread) else {
        return AppendOutcome::NoThread;
    };
    if let Some(existing) = entry.messages.iter().find(|stored| stored.id == message.id) {
        return AppendOutcome::Duplicate(existing.clone());
    }
    entry.last_message_seq += 1;
    let record = MessageRecord {
        id: message.id.clone(),
        thread: thread.clone(),
        role: message.role,
        info: message.info.clone(),
        parts: message.parts.clone(),
        seq: entry.last_message_seq,
        created_at: now,
        completed_at: message.completed_at,
    };
    entry.messages.push(record.clone());
    entry.record.updated_at = now;
    entry.record.change_seq = seq;
    state.change_seq = seq;
    AppendOutcome::Appended(record)
}

pub(super) fn compact(
    state: &mut State,
    user: &UserId,
    thread: &ThreadId,
    reply: &CompletedReply,
    now: DateTime<Utc>,
) -> Option<MessageRecord> {
    let seq = state.change_seq + 1;
    let entry = state.live(user, thread)?;
    let stored = entry
        .messages
        .iter_mut()
        .find(|stored| stored.id == reply.message)?;
    stored.info.clone_from(&reply.info);
    stored.parts.clone_from(&reply.parts);
    stored.completed_at = Some(now);
    let done = stored.clone();
    for event in &mut entry.events {
        if event.record.message.as_ref() == Some(&reply.message) && event.compacted_at.is_none() {
            event.compacted_at = Some(now);
        }
    }
    entry.record.updated_at = now;
    entry.record.change_seq = seq;
    state.change_seq = seq;
    Some(done)
}

pub(super) fn idle(state: &State, before: DateTime<Utc>, limit: usize) -> Vec<IdleThread> {
    let mut found: Vec<&Entry> = state
        .threads
        .values()
        .filter(|entry| entry.record.deleted_at.is_none() && entry.record.updated_at < before)
        .filter(|entry| {
            entry
                .events
                .iter()
                .all(|event| event.record.created_at < before)
        })
        .collect();
    found.sort_by(|a, b| {
        (a.record.updated_at, a.record.id.as_str())
            .cmp(&(b.record.updated_at, b.record.id.as_str()))
    });
    found
        .into_iter()
        .take(limit)
        .map(|entry| IdleThread {
            user: entry.record.user.clone(),
            id: entry.record.id.clone(),
            revision: entry.record.revision,
        })
        .collect()
}
