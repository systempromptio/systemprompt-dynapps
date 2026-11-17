//! The store's threads operations on Postgres.

use chrono::{DateTime, Utc};
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use super::PgChatStore;
use super::support::{limit_param, lock_live_thread, lock_user};
use super::types::ThreadRow;
use crate::ids::{Cursor, ThreadId};
use crate::store::{
    ChangePage, CreateOutcome, ModelRef, NewThread, StoreError, ThreadEdit, ThreadRecord,
    WriteOutcome,
};

impl PgChatStore {
    #[expect(
        trivial_casts,
        reason = "the `as` is sqlx's bind-type override: jsonb binds a `Value` unless told otherwise"
    )]
    pub(super) async fn run_create_thread(
        &self,
        user: &UserId,
        thread: &NewThread,
        now: DateTime<Utc>,
    ) -> Result<CreateOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, user).await? {
            return Ok(CreateOutcome::UnknownUser);
        }
        let inserted = sqlx::query_as!(
            ThreadRow,
            r#"INSERT INTO dynwork_chat_threads
                   (id, user_id, title, model, revision, created_at, updated_at, change_seq)
               VALUES ($1, $2, $3, $4, 1, $5, $5, nextval('dynwork_chat_change_seq'))
               ON CONFLICT (id) DO NOTHING
               RETURNING id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                       deleted_at, created_at, updated_at, change_seq, last_event_seq"#,
            thread.id.as_str(),
            user.as_str(),
            thread.title,
            thread.model.as_ref().map(Json) as Option<Json<&ModelRef>>,
            now
        )
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(row) = inserted {
            let record = ThreadRecord::try_from(row)?;
            tx.commit().await?;
            return Ok(CreateOutcome::Created(record));
        }
        let existing = sqlx::query_as!(
            ThreadRow,
            r#"SELECT id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                    deleted_at, created_at, updated_at, change_seq, last_event_seq
               FROM dynwork_chat_threads WHERE id = $1"#,
            thread.id.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;
        if existing.owner == user.as_str() {
            Ok(CreateOutcome::Exists(ThreadRecord::try_from(existing)?))
        } else {
            Ok(CreateOutcome::IdTaken)
        }
    }

    pub(super) async fn run_get_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
    ) -> Result<Option<ThreadRecord>, StoreError> {
        sqlx::query_as!(
            ThreadRow,
            r#"SELECT id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                    deleted_at, created_at, updated_at, change_seq, last_event_seq
               FROM dynwork_chat_threads WHERE id = $1 AND user_id = $2"#,
            id.as_str(),
            user.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?
        .map(ThreadRecord::try_from)
        .transpose()
    }

    #[expect(
        trivial_casts,
        reason = "the `as` is sqlx's bind-type override: jsonb binds a `Value` unless told otherwise"
    )]
    pub(super) async fn run_update_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        edit: &ThreadEdit,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError> {
        let patch = &edit.patch;
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, user).await? {
            return Ok(WriteOutcome::NotFound);
        }
        let Some(current) = lock_live_thread(&mut tx, user, id).await? else {
            return Ok(WriteOutcome::NotFound);
        };
        if current.revision != edit.expected_revision {
            return Ok(WriteOutcome::Conflict(current));
        }
        if patch.is_empty() {
            return Ok(WriteOutcome::Done(current));
        }
        let row = sqlx::query_as!(
            ThreadRow,
            r#"UPDATE dynwork_chat_threads SET
                   title = COALESCE($3, title),
                   model = COALESCE($4, model),
                   archived_at = CASE
                       WHEN $5::boolean IS NULL THEN archived_at
                       WHEN $5 THEN COALESCE(archived_at, $6)
                       ELSE NULL
                   END,
                   revision = revision + 1,
                   updated_at = $6,
                   change_seq = nextval('dynwork_chat_change_seq')
               WHERE id = $1 AND user_id = $2
               RETURNING id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                       deleted_at, created_at, updated_at, change_seq, last_event_seq"#,
            id.as_str(),
            user.as_str(),
            patch.title.as_deref(),
            patch.model.as_ref().map(Json) as Option<Json<&ModelRef>>,
            patch.archived,
            now
        )
        .fetch_one(&mut *tx)
        .await?;
        let record = ThreadRecord::try_from(row)?;
        tx.commit().await?;
        Ok(WriteOutcome::Done(record))
    }

    pub(super) async fn run_delete_thread(
        &self,
        user: &UserId,
        id: &ThreadId,
        expected_revision: i64,
        now: DateTime<Utc>,
    ) -> Result<WriteOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, user).await? {
            return Ok(WriteOutcome::NotFound);
        }
        let Some(current) = lock_live_thread(&mut tx, user, id).await? else {
            return Ok(WriteOutcome::NotFound);
        };
        if current.revision != expected_revision {
            return Ok(WriteOutcome::Conflict(current));
        }
        sqlx::query!(
            "DELETE FROM dynwork_chat_messages WHERE thread_id = $1",
            id.as_str()
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM dynwork_chat_events WHERE thread_id = $1",
            id.as_str()
        )
        .execute(&mut *tx)
        .await?;
        let row = sqlx::query_as!(
            ThreadRow,
            r#"UPDATE dynwork_chat_threads SET
                   title = '',
                   deleted_at = $3,
                   revision = revision + 1,
                   updated_at = $3,
                   change_seq = nextval('dynwork_chat_change_seq')
               WHERE id = $1 AND user_id = $2
               RETURNING id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                       deleted_at, created_at, updated_at, change_seq, last_event_seq"#,
            id.as_str(),
            user.as_str(),
            now
        )
        .fetch_one(&mut *tx)
        .await?;
        let record = ThreadRecord::try_from(row)?;
        tx.commit().await?;
        Ok(WriteOutcome::Done(record))
    }

    pub(super) async fn run_changes_since(
        &self,
        user: &UserId,
        since: Option<Cursor>,
        limit: u32,
    ) -> Result<ChangePage, StoreError> {
        let limit = limit_param(limit);
        let mut rows = sqlx::query_as!(
            ThreadRow,
            r#"SELECT id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                    deleted_at, created_at, updated_at, change_seq, last_event_seq
               FROM dynwork_chat_threads
               WHERE user_id = $1 AND change_seq > $2 AND ($3 OR deleted_at IS NULL)
               ORDER BY change_seq
               LIMIT $4"#,
            user.as_str(),
            since.map_or(0, Cursor::seq),
            since.is_some(),
            limit + 1
        )
        .fetch_all(&*self.pool)
        .await?;
        let has_more = i64::try_from(rows.len()).is_ok_and(|len| len > limit);
        rows.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
        let threads = rows
            .into_iter()
            .map(ThreadRecord::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let cursor = threads
            .last()
            .and_then(|thread| Cursor::from_seq(thread.change_seq))
            .or(since)
            .unwrap_or(Cursor::ZERO);
        Ok(ChangePage {
            threads,
            cursor,
            has_more,
        })
    }
}
