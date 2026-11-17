//! Routine writes and reads. A write locks its user, reads the current row,
//! checks the revision and writes the next one with the user's next
//! `change_seq`, all in one transaction.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

use super::types::RoutineRow;
use super::{PutRoutine, RoutineRecord, StoreError, WriteOutcome};
use crate::ids::RoutineId;

type Tx<'a> = Transaction<'a, Postgres>;

pub async fn put_routine(pool: &PgPool, put: &PutRoutine<'_>) -> Result<WriteOutcome, StoreError> {
    let Some((mut tx, current)) = open(pool, put.user, put.id).await? else {
        return Ok(WriteOutcome::UnknownUser);
    };
    let seq = next_seq(&mut tx, put.user).await?;
    let row = match (put.expected, current) {
        (None, Some(current)) => return Ok(WriteOutcome::Conflict(current)),
        (Some(_), None) => return Ok(WriteOutcome::NotFound),
        (Some(expected), Some(current))
            if current.deleted_at.is_some() || current.revision != expected =>
        {
            return Ok(WriteOutcome::Conflict(current));
        },
        (None, None) => {
            let Some(owner_device) = put.owner_device else {
                return Ok(WriteOutcome::OwnerRequired);
            };
            sqlx::query_as!(
                RoutineRow,
                "INSERT INTO dynwork_routines
                     (user_id, id, revision, change_seq, owner_device, owner_name, definition, updated_at)
                 VALUES ($1, $2, 1, $3, $4, $5, $6, $7)
                 RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                           deleted_at, updated_at",
                put.user.as_str(),
                put.id.as_str(),
                seq,
                owner_device,
                put.owner_name.unwrap_or_default(),
                put.definition,
                put.now
            )
            .fetch_one(&mut *tx)
            .await?
        },
        (Some(_), Some(_)) => {
            sqlx::query_as!(
                RoutineRow,
                "UPDATE dynwork_routines
                 SET revision = revision + 1, change_seq = $3,
                     owner_device = COALESCE($4, owner_device),
                     owner_name = COALESCE($5, owner_name),
                     definition = $6, updated_at = $7
                 WHERE user_id = $1 AND id = $2
                 RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                           deleted_at, updated_at",
                put.user.as_str(),
                put.id.as_str(),
                seq,
                put.owner_device,
                put.owner_name,
                put.definition,
                put.now
            )
            .fetch_one(&mut *tx)
            .await?
        },
    };
    tx.commit().await?;
    Ok(WriteOutcome::Written(row.into()))
}

// Why: `expected: None` deletes whatever revision is current, for the
// "My routines" page, which has no revision to name.
pub async fn delete_routine(
    pool: &PgPool,
    user: &UserId,
    id: &RoutineId,
    expected: Option<i64>,
    now: DateTime<Utc>,
) -> Result<WriteOutcome, StoreError> {
    let Some((mut tx, current)) = open(pool, user, id).await? else {
        return Ok(WriteOutcome::UnknownUser);
    };
    let Some(current) = current else {
        return Ok(WriteOutcome::NotFound);
    };
    if current.deleted_at.is_some() || expected.is_some_and(|rev| rev != current.revision) {
        return Ok(WriteOutcome::Conflict(current));
    }
    let seq = next_seq(&mut tx, user).await?;
    let row = sqlx::query_as!(
        RoutineRow,
        "UPDATE dynwork_routines
         SET revision = revision + 1, change_seq = $3, definition = '{}'::jsonb,
             deleted_at = $4, updated_at = $4
         WHERE user_id = $1 AND id = $2
         RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                   deleted_at, updated_at",
        user.as_str(),
        id.as_str(),
        seq,
        now
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(WriteOutcome::Written(row.into()))
}

pub async fn set_enabled(
    pool: &PgPool,
    user: &UserId,
    id: &RoutineId,
    enabled: bool,
    now: DateTime<Utc>,
) -> Result<WriteOutcome, StoreError> {
    let Some((mut tx, current)) = open(pool, user, id).await? else {
        return Ok(WriteOutcome::UnknownUser);
    };
    let Some(current) = current else {
        return Ok(WriteOutcome::NotFound);
    };
    if current.deleted_at.is_some() {
        return Ok(WriteOutcome::NotFound);
    }
    let seq = next_seq(&mut tx, user).await?;
    let row = sqlx::query_as!(
        RoutineRow,
        "UPDATE dynwork_routines
         SET revision = revision + 1, change_seq = $3,
             definition = jsonb_set(definition, '{enabled}', to_jsonb($4::boolean)),
             updated_at = $5
         WHERE user_id = $1 AND id = $2
         RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                   deleted_at, updated_at",
        user.as_str(),
        id.as_str(),
        seq,
        enabled,
        now
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(WriteOutcome::Written(row.into()))
}

pub async fn list_changes(
    pool: &PgPool,
    user: &UserId,
    since: i64,
    limit: i64,
) -> Result<Vec<RoutineRecord>, StoreError> {
    let rows = sqlx::query_as!(
        RoutineRow,
        "SELECT id, revision, change_seq, owner_device, owner_name, definition,
                deleted_at, updated_at
         FROM dynwork_routines
         WHERE user_id = $1 AND change_seq > $2
         ORDER BY change_seq
         LIMIT $3",
        user.as_str(),
        since,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(RoutineRecord::from).collect())
}

pub async fn list_live_routines(
    pool: &PgPool,
    user: &UserId,
) -> Result<Vec<RoutineRecord>, StoreError> {
    let rows = sqlx::query_as!(
        RoutineRow,
        "SELECT id, revision, change_seq, owner_device, owner_name, definition,
                deleted_at, updated_at
         FROM dynwork_routines
         WHERE user_id = $1 AND deleted_at IS NULL
         ORDER BY lower(definition->>'name'), id",
        user.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(RoutineRecord::from).collect())
}

async fn open<'a>(
    pool: &'a PgPool,
    user: &UserId,
    id: &RoutineId,
) -> Result<Option<(Tx<'a>, Option<RoutineRecord>)>, StoreError> {
    let mut tx = pool.begin().await?;
    let locked = sqlx::query_scalar!(
        "SELECT id FROM users WHERE id = $1 FOR UPDATE",
        user.as_str()
    )
    .fetch_optional(&mut *tx)
    .await?;
    if locked.is_none() {
        return Ok(None);
    }
    let current = sqlx::query_as!(
        RoutineRow,
        "SELECT id, revision, change_seq, owner_device, owner_name, definition,
                deleted_at, updated_at
         FROM dynwork_routines
         WHERE user_id = $1 AND id = $2",
        user.as_str(),
        id.as_str()
    )
    .fetch_optional(&mut *tx)
    .await?;
    Ok(Some((tx, current.map(RoutineRecord::from))))
}

async fn next_seq(tx: &mut Tx<'_>, user: &UserId) -> Result<i64, StoreError> {
    let seq = sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(change_seq), 0) + 1 AS "next!" FROM dynwork_routines WHERE user_id = $1"#,
        user.as_str()
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(seq)
}
