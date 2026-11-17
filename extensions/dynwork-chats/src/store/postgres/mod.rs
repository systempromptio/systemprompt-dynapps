//! [`ChatStore`] over Postgres.
//!
//! The queries are the checked `sqlx::query!` macros, so they have entries in
//! the workspace's `.sqlx/` cache; the Postgres suite runs every one of them
//! against the real schema.
//!
//! Every write that changes a thread starts by locking the owner's `users`
//! row (`FOR NO KEY UPDATE`, which does not block the foreign-key checks of
//! other inserts). `change_seq` is taken after that lock, so for one user the
//! sequence order is the commit order and a cursor never skips a change. The
//! thread row is then locked for the compare of `revision` and for the
//! per-thread `seq` counters.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::{
    AppendOutcome, ChangePage, ChatStore, CompletedReply, CreateOutcome, EventRecord, IdleThread,
    MessageRecord, NewEvent, NewMessage, NewThread, StoreError, ThreadEdit, ThreadRecord,
    WriteOutcome,
};
use crate::ids::{Cursor, MessageId, ThreadId};

mod messages;
mod support;
mod threads;
mod types;

#[derive(Debug, Clone)]
pub struct PgChatStore {
    pool: Arc<PgPool>,
}

impl PgChatStore {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChatStore for PgChatStore {
    async fn create_thread(
        &self,
        user: &UserId,
        thread: &NewThread,
        now: DateTime<Utc>,
    ) -> Result<CreateOutcome, StoreError> {
        self.run_create_thread(user, thread, now).await
    }

    async fn get_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
    ) -> Result<Option<ThreadRecord>, StoreError> {
        self.run_get_thread(user, id).await
    }

    async fn update_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        edit: &ThreadEdit,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError> {
        self.run_update_thread(user, id, edit, now).await
    }

    async fn delete_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        expected_revision: i64,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError> {
        self.run_delete_thread(user, id, expected_revision, now)
            .await
    }

    async fn changes_since(
        &self,
        user: &UserId,
        since: Option<Cursor>,
        limit: u32,
    ) -> Result<ChangePage, StoreError> {
        self.run_changes_since(user, since, limit).await
    }

    async fn append_message(
        &self,
        user: &UserId,
        thread: &ThreadId,
        message: &NewMessage,
        now: DateTime<Utc>,
    ) -> Result<AppendOutcome, StoreError> {
        self.run_append_message(user, thread, message, now).await
    }

    async fn list_messages(
        &self,
        user: &UserId,
        thread: &ThreadId,
        before: Option<&MessageId>,
        limit: u32,
    ) -> Result<Option<Vec<MessageRecord>>, StoreError> {
        self.run_list_messages(user, thread, before, limit).await
    }

    async fn append_events(
        &self,
        user: &UserId,
        thread: &ThreadId,
        events: &[NewEvent],
        now: DateTime<Utc>,
    ) -> Result<Option<Vec<EventRecord>>, StoreError> {
        self.run_append_events(user, thread, events, now).await
    }

    async fn events_after(
        &self,
        user: &UserId,
        thread: &ThreadId,
        after: i64,
        limit: u32,
    ) -> Result<Option<Vec<EventRecord>>, StoreError> {
        self.run_events_after(user, thread, after, limit).await
    }

    async fn compact_reply(
        &self,
        user: &UserId,
        thread: &ThreadId,
        reply: &CompletedReply,
        now: DateTime<Utc>,
    ) -> Result<Option<MessageRecord>, StoreError> {
        self.run_compact_reply(user, thread, reply, now).await
    }

    async fn prune_events(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        self.run_prune_events(before).await
    }

    async fn idle_threads(
        &self,
        before: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<IdleThread>, StoreError> {
        self.run_idle_threads(before, limit).await
    }
}
