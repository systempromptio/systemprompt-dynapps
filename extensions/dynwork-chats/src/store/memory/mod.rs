//! [`ChatStore`] in memory, with the semantics of the Postgres one.
//!
//! The same outcomes, the same ordering and the same isolation between users.
//! It has no `users` table, so [`CreateOutcome::UnknownUser`] never happens
//! here.

mod ops;
mod state;

use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::{
    AppendOutcome, ChangePage, ChatStore, CompletedReply, CreateOutcome, EventRecord, IdleThread,
    MessageRecord, NewEvent, NewMessage, NewThread, StoreError, ThreadEdit, ThreadRecord,
    WriteOutcome,
};
use crate::ids::{Cursor, MessageId, ThreadId};
use ops::{append, compact, create, delete, idle, update};
use state::{State, StoredEvent, usize_from};

#[derive(Debug, Default)]
pub struct MemoryChatStore {
    state: Mutex<State>,
}

impl MemoryChatStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn with<R>(&self, work: impl FnOnce(&mut State) -> R) -> R {
        let mut guard = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        work(&mut guard)
    }
}

#[async_trait]
impl ChatStore for MemoryChatStore {
    async fn create_thread(
        &self,
        user: &UserId,
        thread: &NewThread,
        now: DateTime<Utc>,
    ) -> Result<CreateOutcome, StoreError> {
        Ok(self.with(|state| create(state, user, thread, now)))
    }

    async fn get_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
    ) -> Result<Option<ThreadRecord>, StoreError> {
        Ok(self.with(|state| {
            state
                .threads
                .get(id)
                .filter(|entry| entry.record.user == *user)
                .map(|entry| entry.record.clone())
        }))
    }

    async fn update_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        edit: &ThreadEdit,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError> {
        Ok(self.with(|state| update(state, user, id, edit, now)))
    }

    async fn delete_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        expected_revision: i64,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError> {
        Ok(self.with(|state| delete(state, user, id, expected_revision, now)))
    }

    async fn changes_since(
        &self,
        user: &UserId,
        since: Option<Cursor>,
        limit: u32,
    ) -> Result<ChangePage, StoreError> {
        Ok(self.with(|state| {
            let after = since.map_or(0, Cursor::seq);
            let mut rows: Vec<&ThreadRecord> = state
                .threads
                .values()
                .map(|entry| &entry.record)
                .filter(|record| record.user == *user && record.change_seq > after)
                .filter(|record| since.is_some() || record.deleted_at.is_none())
                .collect();
            rows.sort_by_key(|record| record.change_seq);
            let limit = usize_from(limit);
            let has_more = rows.len() > limit;
            rows.truncate(limit);
            let cursor = rows
                .last()
                .and_then(|record| Cursor::from_seq(record.change_seq))
                .or(since)
                .unwrap_or(Cursor::ZERO);
            ChangePage {
                threads: rows.into_iter().cloned().collect(),
                cursor,
                has_more,
            }
        }))
    }

    async fn append_message(
        &self,
        user: &UserId,
        thread: &ThreadId,
        message: &NewMessage,
        now: DateTime<Utc>,
    ) -> Result<AppendOutcome, StoreError> {
        Ok(self.with(|state| append(state, user, thread, message, now)))
    }

    async fn list_messages(
        &self,
        user: &UserId,
        thread: &ThreadId,
        before: Option<&MessageId>,
        limit: u32,
    ) -> Result<Option<Vec<MessageRecord>>, StoreError> {
        Ok(self.with(|state| {
            let entry = state.live(user, thread)?;
            let ceiling = match before {
                None => Some(i64::MAX),
                Some(id) => entry
                    .messages
                    .iter()
                    .find(|stored| stored.id == *id)
                    .map(|stored| stored.seq),
            };
            let Some(ceiling) = ceiling else {
                return Some(Vec::new());
            };
            Some(
                entry
                    .messages
                    .iter()
                    .rev()
                    .filter(|stored| stored.seq < ceiling)
                    .take(usize_from(limit))
                    .cloned()
                    .collect(),
            )
        }))
    }

    async fn append_events(
        &self,
        user: &UserId,
        thread: &ThreadId,
        events: &[NewEvent],
        now: DateTime<Utc>,
    ) -> Result<Option<Vec<EventRecord>>, StoreError> {
        Ok(self.with(|state| {
            let entry = state.live(user, thread)?;
            let mut written = Vec::with_capacity(events.len());
            for event in events {
                entry.record.last_event_seq += 1;
                let record = EventRecord {
                    thread: thread.clone(),
                    seq: entry.record.last_event_seq,
                    message: event.message.clone(),
                    kind: event.kind.clone(),
                    data: event.data.clone(),
                    created_at: now,
                };
                entry.events.push(StoredEvent {
                    record: record.clone(),
                    compacted_at: None,
                });
                written.push(record);
            }
            Some(written)
        }))
    }

    async fn events_after(
        &self,
        user: &UserId,
        thread: &ThreadId,
        after: i64,
        limit: u32,
    ) -> Result<Option<Vec<EventRecord>>, StoreError> {
        Ok(self.with(|state| {
            let entry = state.live(user, thread)?;
            Some(
                entry
                    .events
                    .iter()
                    .filter(|stored| stored.record.seq > after)
                    .take(usize_from(limit))
                    .map(|stored| stored.record.clone())
                    .collect(),
            )
        }))
    }

    async fn compact_reply(
        &self,
        user: &UserId,
        thread: &ThreadId,
        reply: &CompletedReply,
        now: DateTime<Utc>,
    ) -> Result<Option<MessageRecord>, StoreError> {
        Ok(self.with(|state| compact(state, user, thread, reply, now)))
    }

    async fn prune_events(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        Ok(self.with(|state| {
            let mut removed = 0u64;
            for entry in state.threads.values_mut() {
                let kept = entry.events.len();
                entry.events.retain(|event| {
                    let expired = match (&event.record.message, event.compacted_at) {
                        (None, _) => event.record.created_at < before,
                        (Some(_), Some(compacted_at)) => compacted_at < before,
                        (Some(_), None) => false,
                    };
                    !expired
                });
                removed += u64::try_from(kept - entry.events.len()).unwrap_or(0);
            }
            removed
        }))
    }

    async fn idle_threads(
        &self,
        before: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<IdleThread>, StoreError> {
        Ok(self.with(|state| idle(state, before, usize_from(limit))))
    }
}
