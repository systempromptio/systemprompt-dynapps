//! The writes a template makes on the routines linked to it: the link rules
//! a routine write checks, the unclaimed routine a mandatory template owes a
//! person, and retiring a template, which may pause or remove every live
//! routine linked to it. Each runs under the user row lock of the person (or
//! people) whose routines it writes, like every other routine write.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use super::routines::{Tx, lock_user, next_seq};
use super::types::{RoutineRow, TemplateRow};
use super::{
    PutLink, PutRoutine, RoutineRecord, StoreError, TemplateRecord, UnclaimedRoutine, WriteOutcome,
};
use crate::ids::TemplateId;
use crate::template::{RetireMode, TemplateBody};

// Why: a routine already linked to the template follows it, whether or not
// the person still sees it, and may move to a later revision but never back
// to an earlier one; a new link needs a template open to the person and no
// other live routine of it.
pub(super) async fn refused_link(
    tx: &mut Tx<'_>,
    put: &PutRoutine<'_>,
    link: &PutLink<'_>,
    current: Option<&RoutineRecord>,
) -> Result<Option<WriteOutcome>, StoreError> {
    let locked = || WriteOutcome::TemplateLocked(current.cloned());
    let followed = current
        .filter(|current| current.template_id.as_ref() == Some(link.template))
        .and_then(|current| current.template_revision);
    if let Some(followed) = followed {
        return Ok((!link.keeps_locks || link.revision < followed).then(locked));
    }
    if !link.open {
        return Ok(Some(WriteOutcome::TemplateUnavailable));
    }
    let other = sqlx::query_as!(
        RoutineRow,
        "SELECT id, revision, change_seq, owner_device, owner_name, definition,
                deleted_at, updated_at, template_id, template_revision
         FROM dynwork_routines
         WHERE user_id = $1 AND template_id = $2 AND deleted_at IS NULL AND id <> $3",
        put.user.as_str(),
        link.template.as_str(),
        put.id.as_str()
    )
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(other) = other {
        return Ok(Some(WriteOutcome::AlreadyActivated(other.into())));
    }
    Ok((!link.keeps_locks).then(locked))
}


// Why: concurrent reads of one person serialise on their user row, and each
// insert re-checks for a live routine of the template (and that the template
// is still at that revision, published and mandatory), so however many
// desktops ask at once a template makes one routine. Returns how many it made.
pub async fn make_unclaimed(
    pool: &PgPool,
    user: &UserId,
    routines: &[UnclaimedRoutine<'_>],
    now: DateTime<Utc>,
) -> Result<u64, StoreError> {
    let mut tx = pool.begin().await?;
    if !lock_user(&mut tx, user).await? {
        return Ok(0);
    }
    let mut made = 0;
    for routine in routines {
        let seq = next_seq(&mut tx, user).await?;
        made += sqlx::query!(
            "INSERT INTO dynwork_routines
                 (user_id, id, revision, change_seq, owner_device, owner_name, definition,
                  updated_at, template_id, template_revision)
             SELECT $1, 'tpl_' || gen_random_uuid()::text, 1, $3, '', '', $4, $5, t.id, t.revision
             FROM dynwork_routine_templates t
             WHERE t.id = $2 AND t.revision = $6 AND t.state = 'published' AND t.mandatory
               AND NOT EXISTS (
                   SELECT 1 FROM dynwork_routines r
                   WHERE r.user_id = $1 AND r.template_id = $2 AND r.deleted_at IS NULL
               )
             ON CONFLICT DO NOTHING",
            user.as_str(),
            routine.template.as_str(),
            seq,
            routine.definition,
            now,
            routine.revision
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }
    tx.commit().await?;
    Ok(made)
}

/// A retired template and the people whose routines the retirement wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retired {
    pub template: TemplateRecord,
    pub users: Vec<UserId>,
}

// Why: a published or closed template retires once (`None` otherwise). Pause
// and remove write every live linked routine in the same transaction, each
// as its person's next change, so a desktop that is off finds it in the feed;
// the people are locked in id order, as one person's write locks theirs.
pub async fn retire_template(
    pool: &PgPool,
    template: &TemplateId,
    mode: RetireMode,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<Option<Retired>, StoreError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as!(
        TemplateRow,
        r#"UPDATE dynwork_routine_templates
           SET state = 'retired', retire_mode = $2, updated_by = $3, updated_at = $4
           WHERE id = $1 AND state IN ('published', 'closed')
           RETURNING id, revision, state, retire_mode, mandatory, plugins, source,
                     body AS "body: Json<TemplateBody>", created_by, updated_by, updated_at"#,
        template.as_str(),
        mode.as_str(),
        author.as_str(),
        now
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let users = if mode == RetireMode::Unlink {
        Vec::new()
    } else {
        sqlx::query_scalar!(
            "SELECT id FROM users
             WHERE id IN (
                 SELECT user_id FROM dynwork_routines
                 WHERE template_id = $1 AND deleted_at IS NULL
             )
             ORDER BY id
             FOR UPDATE",
            template.as_str()
        )
        .fetch_all(&mut *tx)
        .await?
    };
    match mode {
        RetireMode::Unlink => {},
        RetireMode::Pause => {
            sqlx::query!(
                "UPDATE dynwork_routines r
                 SET revision = r.revision + 1,
                     change_seq = (SELECT MAX(m.change_seq) + 1 FROM dynwork_routines m
                                   WHERE m.user_id = r.user_id),
                     definition = jsonb_set(r.definition, '{enabled}', 'false'::jsonb),
                     updated_at = $3
                 WHERE r.template_id = $1 AND r.deleted_at IS NULL AND r.user_id = ANY($2)",
                template.as_str(),
                users.as_slice(),
                now
            )
            .execute(&mut *tx)
            .await?;
        },
        RetireMode::Remove => {
            sqlx::query!(
                "UPDATE dynwork_routines r
                 SET revision = r.revision + 1,
                     change_seq = (SELECT MAX(m.change_seq) + 1 FROM dynwork_routines m
                                   WHERE m.user_id = r.user_id),
                     definition = '{}'::jsonb, deleted_at = $3, updated_at = $3
                 WHERE r.template_id = $1 AND r.deleted_at IS NULL AND r.user_id = ANY($2)",
                template.as_str(),
                users.as_slice(),
                now
            )
            .execute(&mut *tx)
            .await?;
        },
    }
    tx.commit().await?;
    Ok(Some(Retired {
        template: TemplateRecord::try_from(row)?,
        users: users.into_iter().map(UserId::new).collect(),
    }))
}
