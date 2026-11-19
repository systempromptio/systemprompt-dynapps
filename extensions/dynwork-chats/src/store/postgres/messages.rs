//! The store's message and event operations on Postgres.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::PgChatStore;
use super::support::{
    fetch_message, limit_param, live_thread_exists, lock_live_thread, lock_user, touch_thread,
};
use super::types::{EventRow, MessageRow};
use crate::ids::{MessageId, ThreadId};
use crate::store::{
    AppendOutcome, CompletedReply, EventRecord, MessageRecord, NewEvent, NewMessage, StoreError,
};

impl PgChatStore {
    pub(super) async fn run_append_message(
        &self,
        user: &UserId,
        thread: &ThreadId,
        message: &NewMessage,
        now: DateTime<Utc>,
    ) -> Result<AppendOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, user).await? {
            return Ok(AppendOutcome::NoThread);
        }
        if lock_live_thread(&mut tx, user, thread).await?.is_none() {
            return Ok(AppendOutcome::NoThread);
        }
        if let Some(existing) = fetch_message(&mut tx, thread, &message.id).await? {
            return Ok(AppendOutcome::Duplicate(existing));
        }
        let seq = sqlx::query_scalar!(
            "UPDATE dynwork_chat_threads SET last_message_seq = last_message_seq + 1
             WHERE id = $1 RETURNING last_message_seq",
            thread.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as!(
            MessageRow,
            "INSERT INTO dynwork_chat_messages
                 (id, thread_id, role, info, parts, seq, created_at, completed_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, thread_id, role, info, parts, seq, created_at, completed_at",
            message.id.as_str(),
            thread.as_str(),
            message.role.as_str(),
            message.info,
            message.parts,
            seq,
            now,
            message.completed_at
        )
        .fetch_one(&mut *tx)
        .await?;
        touch_thread(&mut tx, thread, now).await?;
        let record = MessageRecord::try_from(row)?;
        tx.commit().await?;
        Ok(AppendOutcome::Appended(record))
    }

    pub(super) async fn run_list_messages(
        &self,
        user: &UserId,
        thread: &ThreadId,
        before: Option<&MessageId>,
        limit: u32,
    ) -> Result<Option<Vec<MessageRecord>>, StoreError> {
        if !live_thread_exists(&self.pool, user, thread).await? {
            return Ok(None);
        }
        let rows = sqlx::query_as!(
            MessageRow,
            "SELECT id, thread_id, role, info, parts, seq, created_at, completed_at
             FROM dynwork_chat_messages
             WHERE thread_id = $1
               AND ($2::text IS NULL OR seq < (
                   SELECT seq FROM dynwork_chat_messages WHERE thread_id = $1 AND id = $2))
             ORDER BY seq DESC
             LIMIT $3",
            thread.as_str(),
            before.map(MessageId::as_str),
            limit_param(limit)
        )
        .fetch_all(&*self.pool)
        .await?;
        rows.into_iter()
            .map(MessageRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub(super) async fn run_append_events(
        &self,
        user: &UserId,
        thread: &ThreadId,
        events: &[NewEvent],
        now: DateTime<Utc>,
    ) -> Result<Option<Vec<EventRecord>>, StoreError> {
        let count = i64::try_from(events.len()).map_err(|source| {
            tracing::error!(error = %source, "event batch too large");
            StoreError::Corrupt("event batch")
        })?;
        let mut tx = self.pool.begin().await?;
        let last = sqlx::query_scalar!(
            "UPDATE dynwork_chat_threads SET last_event_seq = last_event_seq + $3
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
             RETURNING last_event_seq",
            thread.as_str(),
            user.as_str(),
            count
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(last) = last else {
            return Ok(None);
        };
        let mut written = Vec::with_capacity(events.len());
        for (seq, event) in (last - count + 1..).zip(events) {
            let row = sqlx::query_as!(
                EventRow,
                "INSERT INTO dynwork_chat_events (thread_id, seq, message_id, type, data, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING thread_id, seq, message_id AS reply, type AS kind, data, created_at",
                thread.as_str(),
                seq,
                event.message.as_ref().map(MessageId::as_str),
                event.kind,
                event.data,
                now
            )
            .fetch_one(&mut *tx)
            .await?;
            written.push(EventRecord::try_from(row)?);
        }
        tx.commit().await?;
        Ok(Some(written))
    }

    pub(super) async fn run_events_after(
        &self,
        user: &UserId,
        thread: &ThreadId,
        after: i64,
        limit: u32,
    ) -> Result<Option<Vec<EventRecord>>, StoreError> {
        if !live_thread_exists(&self.pool, user, thread).await? {
            return Ok(None);
        }
        let rows = sqlx::query_as!(
            EventRow,
            "SELECT thread_id, seq, message_id AS reply, type AS kind, data, created_at
             FROM dynwork_chat_events
             WHERE thread_id = $1 AND seq > $2
             ORDER BY seq
             LIMIT $3",
            thread.as_str(),
            after,
            limit_param(limit)
        )
        .fetch_all(&*self.pool)
        .await?;
        rows.into_iter()
            .map(EventRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub(super) async fn run_compact_reply(
        &self,
        user: &UserId,
        thread: &ThreadId,
        reply: &CompletedReply,
        now: DateTime<Utc>,
    ) -> Result<Option<MessageRecord>, StoreError> {
        let message = &reply.message;
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, user).await? {
            return Ok(None);
        }
        if lock_live_thread(&mut tx, user, thread).await?.is_none() {
            return Ok(None);
        }
        let row = sqlx::query_as!(
            MessageRow,
            "UPDATE dynwork_chat_messages SET info = $3, parts = $4, completed_at = $5
             WHERE thread_id = $1 AND id = $2
             RETURNING id, thread_id, role, info, parts, seq, created_at, completed_at",
            thread.as_str(),
            message.as_str(),
            reply.info,
            reply.parts,
            now
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        sqlx::query!(
            "UPDATE dynwork_chat_events SET compacted_at = $3
             WHERE thread_id = $1 AND message_id = $2 AND compacted_at IS NULL",
            thread.as_str(),
            message.as_str(),
            now
        )
        .execute(&mut *tx)
        .await?;
        touch_thread(&mut tx, thread, now).await?;
        let record = MessageRecord::try_from(row)?;
        tx.commit().await?;
        Ok(Some(record))
    }

    pub(super) async fn run_prune_events(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        let done = sqlx::query!(
            "DELETE FROM dynwork_chat_events
             WHERE (message_id IS NULL AND created_at < $1) OR compacted_at < $1",
            before
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }
}
