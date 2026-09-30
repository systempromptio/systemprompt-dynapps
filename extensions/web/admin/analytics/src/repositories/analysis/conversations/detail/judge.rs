//! Manual judge requests: queue one or many conversations for the judge.
//!
//! Only contexts with a fact row are queued; re-queueing resets the analysis
//! row to `pending` with a fresh lease.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

pub async fn insert_manual_judgement(
    pool: &PgPool,
    context_id: &ContextId,
    requested_by: &UserId,
) -> Result<bool, sqlx::Error> {
    let user_id = sqlx::query_scalar!(
        r#"SELECT f.user_id AS "user_id!" FROM conversation_facts f WHERE f.context_id = $1"#,
        context_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    let Some(user_id) = user_id else {
        return Ok(false);
    };
    sqlx::query!(
        r#"INSERT INTO conversation_analyses (context_id, user_id, status, trigger, requested_by)
           VALUES ($1, $2, 'pending', 'manual', $3)
           ON CONFLICT (context_id) DO UPDATE
               SET status = 'pending', attempts = 0, next_attempt = clock_timestamp(),
                   lease_token = NULL, lease_until = NULL, trigger = 'manual',
                   requested_by = EXCLUDED.requested_by, updated_at = clock_timestamp()"#,
        context_id.as_str(),
        user_id,
        requested_by.as_str()
    )
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn insert_manual_judgements(
    pool: &PgPool,
    context_ids: &[String],
    requested_by: &UserId,
) -> Result<i64, sqlx::Error> {
    if context_ids.is_empty() {
        return Ok(0);
    }
    let queued = sqlx::query_scalar!(
        r#"WITH queued AS (
               INSERT INTO conversation_analyses (context_id, user_id, status, trigger, requested_by)
               SELECT f.context_id, f.user_id, 'pending', 'manual', $2
               FROM conversation_facts f WHERE f.context_id = ANY($1)
               ON CONFLICT (context_id) DO UPDATE
                   SET status = 'pending', attempts = 0, next_attempt = clock_timestamp(),
                       lease_token = NULL, lease_until = NULL, trigger = 'manual',
                       requested_by = EXCLUDED.requested_by, updated_at = clock_timestamp()
               RETURNING 1)
           SELECT COUNT(*)::bigint AS "queued!" FROM queued"#,
        context_ids,
        requested_by.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(queued)
}
