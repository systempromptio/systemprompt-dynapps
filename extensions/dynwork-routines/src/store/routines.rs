//! Routine writes and reads. A write locks its user, reads the current row,
//! checks the revision and writes the next one with the user's next
//! `change_seq`, all in one transaction. A write also keeps the routine's
//! link to a template (`template_id`, `template_revision`, from the
//! definition's `template`) and enforces it in the same transaction: one live
//! routine per person and template, and the locked fields of the revision it
//! names.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

use super::types::RoutineRow;
use super::{DeleteRoutine, PutRoutine, RoutineRecord, StoreError, WriteOutcome};
use crate::ids::RoutineId;

pub(super) type Tx<'a> = Transaction<'a, Postgres>;

pub async fn put_routine(pool: &PgPool, put: &PutRoutine<'_>) -> Result<WriteOutcome, StoreError> {
    let Some((mut tx, current)) = open(pool, put.user, put.id).await? else {
        return Ok(WriteOutcome::UnknownUser);
    };
    if let Some(refused) = refused(put, current.as_ref()) {
        return Ok(refused);
    }
    if let Some(link) = &put.link
        && let Some(refused) =
            super::linked::refused_link(&mut tx, put, link, current.as_ref()).await?
    {
        return Ok(refused);
    }
    let seq = next_seq(&mut tx, put.user).await?;
    let template = put.link.as_ref().map(|link| link.template.as_str());
    let template_revision = put.link.as_ref().map(|link| link.revision);
    let row = if current.is_some() {
        sqlx::query_as!(
            RoutineRow,
            "UPDATE dynwork_routines
             SET revision = revision + 1, change_seq = $3,
                 owner_device = COALESCE($4, owner_device),
                 owner_name = COALESCE($5, owner_name),
                 definition = $6, updated_at = $7,
                 template_id = $8, template_revision = $9
             WHERE user_id = $1 AND id = $2
             RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                       deleted_at, updated_at, template_id, template_revision",
            put.user.as_str(),
            put.id.as_str(),
            seq,
            put.owner_device,
            put.owner_name,
            put.definition,
            put.now,
            template,
            template_revision
        )
        .fetch_one(&mut *tx)
        .await?
    } else {
        sqlx::query_as!(
            RoutineRow,
            "INSERT INTO dynwork_routines
                 (user_id, id, revision, change_seq, owner_device, owner_name, definition,
                  updated_at, template_id, template_revision)
             VALUES ($1, $2, 1, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                       deleted_at, updated_at, template_id, template_revision",
            put.user.as_str(),
            put.id.as_str(),
            seq,
            put.owner_device.unwrap_or_default(),
            put.owner_name.unwrap_or_default(),
            put.definition,
            put.now,
            template,
            template_revision
        )
        .fetch_one(&mut *tx)
        .await?
    };
    tx.commit().await?;
    Ok(WriteOutcome::Written(row.into()))
}

// Why: `expected: None` deletes whatever revision is current, for the
// "My routines" page, which has no revision to name. The tombstone keeps the
// template link.
pub async fn delete_routine(
    pool: &PgPool,
    delete: &DeleteRoutine<'_>,
) -> Result<WriteOutcome, StoreError> {
    let user = delete.user;
    let Some((mut tx, current)) = open(pool, user, delete.id).await? else {
        return Ok(WriteOutcome::UnknownUser);
    };
    let Some(current) = current else {
        return Ok(WriteOutcome::NotFound);
    };
    if current.deleted_at.is_some() || delete.expected.is_some_and(|rev| rev != current.revision) {
        return Ok(WriteOutcome::Conflict(current));
    }
    if current
        .template_id
        .as_ref()
        .is_some_and(|template| delete.mandatory.contains(template))
    {
        return Ok(WriteOutcome::Mandatory);
    }
    let seq = next_seq(&mut tx, user).await?;
    let row = sqlx::query_as!(
        RoutineRow,
        "UPDATE dynwork_routines
         SET revision = revision + 1, change_seq = $3, definition = '{}'::jsonb,
             deleted_at = $4, updated_at = $4
         WHERE user_id = $1 AND id = $2
         RETURNING id, revision, change_seq, owner_device, owner_name, definition,
                   deleted_at, updated_at, template_id, template_revision",
        user.as_str(),
        delete.id.as_str(),
        seq,
        delete.now
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
                   deleted_at, updated_at, template_id, template_revision",
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
                deleted_at, updated_at, template_id, template_revision
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
                deleted_at, updated_at, template_id, template_revision
         FROM dynwork_routines
         WHERE user_id = $1 AND deleted_at IS NULL
         ORDER BY lower(definition->>'name'), id",
        user.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(RoutineRecord::from).collect())
}

fn refused(put: &PutRoutine<'_>, current: Option<&RoutineRecord>) -> Option<WriteOutcome> {
    match (put.expected, current) {
        (None, Some(current)) => Some(WriteOutcome::Conflict(current.clone())),
        (Some(_), None) => Some(WriteOutcome::NotFound),
        (Some(expected), Some(current))
            if current.deleted_at.is_some() || current.revision != expected =>
        {
            Some(WriteOutcome::Conflict(current.clone()))
        },
        (None, None) if put.owner_device.is_none() => Some(WriteOutcome::OwnerRequired),
        _ => None,
    }
}

async fn open<'a>(
    pool: &'a PgPool,
    user: &UserId,
    id: &RoutineId,
) -> Result<Option<(Tx<'a>, Option<RoutineRecord>)>, StoreError> {
    let mut tx = pool.begin().await?;
    if !lock_user(&mut tx, user).await? {
        return Ok(None);
    }
    let current = sqlx::query_as!(
        RoutineRow,
        "SELECT id, revision, change_seq, owner_device, owner_name, definition,
                deleted_at, updated_at, template_id, template_revision
         FROM dynwork_routines
         WHERE user_id = $1 AND id = $2",
        user.as_str(),
        id.as_str()
    )
    .fetch_optional(&mut *tx)
    .await?;
    Ok(Some((tx, current.map(RoutineRecord::from))))
}

pub(super) async fn lock_user(tx: &mut Tx<'_>, user: &UserId) -> Result<bool, StoreError> {
    let locked = sqlx::query_scalar!(
        "SELECT id FROM users WHERE id = $1 FOR UPDATE",
        user.as_str()
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(locked.is_some())
}

pub(super) async fn next_seq(tx: &mut Tx<'_>, user: &UserId) -> Result<i64, StoreError> {
    let seq = sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(change_seq), 0) + 1 AS "next!" FROM dynwork_routines WHERE user_id = $1"#,
        user.as_str()
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(seq)
}
