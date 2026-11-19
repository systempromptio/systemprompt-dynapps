//! The locks and small queries the store's operations share.

use chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

use super::types::{MessageRow, ThreadRow};
use crate::ids::{MessageId, ThreadId};
use crate::store::{MessageRecord, ModelRef, StoreError, ThreadRecord, page_limit};

pub(super) fn limit_param(limit: u32) -> i64 {
    i64::from(page_limit(limit))
}

pub(super) async fn lock_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
) -> Result<bool, StoreError> {
    let row = sqlx::query_scalar!(
        "SELECT id FROM users WHERE id = $1 FOR NO KEY UPDATE",
        user.as_str()
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.is_some())
}

pub(super) async fn lock_live_thread(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    id: &ThreadId,
) -> Result<Option<ThreadRecord>, StoreError> {
    sqlx::query_as!(
        ThreadRow,
        r#"SELECT id, user_id AS owner, title, model AS "model?: Json<ModelRef>", revision, archived_at,
                  deleted_at, created_at, updated_at, change_seq, last_event_seq
           FROM dynwork_chat_threads
           WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL FOR UPDATE"#,
        id.as_str(),
        user.as_str()
    )
    .fetch_optional(&mut **tx)
    .await?
    .map(ThreadRecord::try_from)
    .transpose()
}

pub(super) async fn live_thread_exists(
    pool: &PgPool,
    user: &UserId,
    id: &ThreadId,
) -> Result<bool, StoreError> {
    let found = sqlx::query_scalar!(
        "SELECT id FROM dynwork_chat_threads
         WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        id.as_str(),
        user.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub(super) async fn fetch_message(
    tx: &mut Transaction<'_, Postgres>,
    thread: &ThreadId,
    message: &MessageId,
) -> Result<Option<MessageRecord>, StoreError> {
    sqlx::query_as!(
        MessageRow,
        "SELECT id, thread_id, role, info, parts, seq, created_at, completed_at
         FROM dynwork_chat_messages WHERE thread_id = $1 AND id = $2",
        thread.as_str(),
        message.as_str()
    )
    .fetch_optional(&mut **tx)
    .await?
    .map(MessageRecord::try_from)
    .transpose()
}

pub(super) async fn touch_thread(
    tx: &mut Transaction<'_, Postgres>,
    id: &ThreadId,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE dynwork_chat_threads
         SET updated_at = $2, change_seq = nextval('dynwork_chat_change_seq')
         WHERE id = $1",
        id.as_str(),
        now
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
