//! Persistence behind one trait: threads, messages and reply events.
//!
//! The server runs over [`postgres::PgChatStore`]; [`memory::MemoryChatStore`]
//! is the seam the suites of later phases use to run the HTTP layer without a
//! database, and both pass the same contract. Every method takes the owner's
//! [`UserId`], and no method reads across users. Every time the store stamps a
//! time it takes it as an argument, so the injected clock decides.

pub mod memory;
pub mod postgres;
mod types;

use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use crate::ids::{Cursor, MessageId, ThreadId};

pub use types::{
    AppendOutcome, ChangePage, CompletedReply, CreateOutcome, EventRecord, IdleThread,
    MessageRecord, ModelRef, NewEvent, NewMessage, NewThread, Role, ThreadEdit, ThreadPatch,
    ThreadRecord, WriteOutcome,
};

pub const MAX_PAGE: u32 = 500;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("chat store query failed")]
    Database(#[from] sqlx::Error),
    #[error("chat store row is invalid: {0}")]
    Corrupt(&'static str),
}

/// The chats storage. `#[async_trait]` because later phases hold it as an
/// `Arc<dyn ChatStore>` (Postgres in production, memory in tests).
///
/// Writes that change a thread (create, update, delete, a message, the end of
/// a reply) give it the next `change_seq`; only a change a person makes
/// (`update_thread`, `delete_thread`) also bumps `revision`. A deleted thread
/// is a tombstone: [`ChatStore::get_thread`] and [`ChatStore::changes_since`]
/// still show it, every other method treats it as absent.
///
/// `update_thread` applies an edit when the thread is at its
/// `expected_revision` (an empty patch writes nothing). `delete_thread` writes
/// the tombstone and removes the messages and events. `changes_since` lists
/// threads changed after a cursor, oldest first, at most [`MAX_PAGE`]; without
/// a cursor it is the full list of live threads. `list_messages` pages newest
/// first before a message id. `append_events` and `events_after` keep the
/// per-thread event log and do not change `change_seq`. `compact_reply` ends a
/// reply: final info and parts into its message, its events marked compacted,
/// the thread's next `change_seq`. `prune_events` drops events of replies
/// compacted before a time, and orphan events older than it. `idle_threads`
/// lists live threads of any user with no activity since a time, oldest
/// first, at most `limit`: neither the thread (`updated_at`) nor any of its
/// events changed since, so a reply that is still writing keeps its thread
/// out. With `prune_events` it is a read across users, for the retention job
/// only; the job deletes what it finds through `delete_thread`. Every method
/// that takes a thread answers `None` or `NotFound` when it is not the
/// caller's.
#[async_trait]
pub trait ChatStore: Send + Sync + Debug {
    async fn create_thread(
        &self,
        user: &UserId,
        thread: &NewThread,
        now: DateTime<Utc>,
    ) -> Result<CreateOutcome, StoreError>;

    async fn get_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
    ) -> Result<Option<ThreadRecord>, StoreError>;

    async fn update_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        edit: &ThreadEdit,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError>;

    async fn delete_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        expected_revision: i64,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError>;

    async fn changes_since(
        &self,
        user: &UserId,
        since: Option<Cursor>,
        limit: u32,
    ) -> Result<ChangePage, StoreError>;

    async fn append_message(
        &self,
        user: &UserId,
        thread: &ThreadId,
        message: &NewMessage,
        now: DateTime<Utc>,
    ) -> Result<AppendOutcome, StoreError>;

    async fn list_messages(
        &self,
        user: &UserId,
        thread: &ThreadId,
        before: Option<&MessageId>,
        limit: u32,
    ) -> Result<Option<Vec<MessageRecord>>, StoreError>;

    async fn append_events(
        &self,
        user: &UserId,
        thread: &ThreadId,
        events: &[NewEvent],
        now: DateTime<Utc>,
    ) -> Result<Option<Vec<EventRecord>>, StoreError>;

    async fn events_after(
        &self,
        user: &UserId,
        thread: &ThreadId,
        after: i64,
        limit: u32,
    ) -> Result<Option<Vec<EventRecord>>, StoreError>;

    async fn compact_reply(
        &self,
        user: &UserId,
        thread: &ThreadId,
        reply: &CompletedReply,
        now: DateTime<Utc>,
    ) -> Result<Option<MessageRecord>, StoreError>;

    async fn prune_events(&self, before: DateTime<Utc>) -> Result<u64, StoreError>;

    async fn idle_threads(
        &self,
        before: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<IdleThread>, StoreError>;
}

pub(crate) fn page_limit(limit: u32) -> u32 {
    limit.clamp(1, MAX_PAGE)
}
