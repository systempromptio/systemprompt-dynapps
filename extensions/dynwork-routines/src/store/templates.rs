//! The template library: drafts, publishes with their revisions, closing,
//! and what `GET /templates` reads. A publish writes the template's next
//! revision and its copy in the history in one transaction; the row lock the
//! upsert takes orders concurrent publishes of one template.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use super::types::{RevisionRow, TemplateRow};
use super::{RevisionRecord, StoreError, TemplateRecord};
use crate::ids::TemplateId;
use crate::template::{TemplateBody, TemplateDraft, TemplateSource};

// Why: Postgres skips the upsert's update when the existing row is not a
// draft, so `None` tells the caller the template is published.
pub async fn save_template_draft(
    pool: &PgPool,
    draft: &TemplateDraft,
    source: TemplateSource,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<Option<TemplateRecord>, StoreError> {
    let row = sqlx::query_as!(
        TemplateRow,
        r#"INSERT INTO dynwork_routine_templates
               (id, revision, state, mandatory, plugins, source, body, created_by, updated_by, updated_at)
           VALUES ($1, 0, 'draft', $2, $3, $4, $5, $6, $6, $7)
           ON CONFLICT (id) DO UPDATE
           SET mandatory = EXCLUDED.mandatory, plugins = EXCLUDED.plugins,
               source = EXCLUDED.source, body = EXCLUDED.body,
               updated_by = EXCLUDED.updated_by, updated_at = EXCLUDED.updated_at
           WHERE dynwork_routine_templates.state = 'draft'
           RETURNING id, revision, state, retire_mode, mandatory, plugins, source,
                     body AS "body: Json<TemplateBody>", created_by, updated_by, updated_at"#,
        draft.id.as_str(),
        draft.mandatory,
        draft.plugins.as_slice(),
        source.as_str(),
        Json(&draft.body) as _,
        author.as_str(),
        now
    )
    .fetch_optional(pool)
    .await?;
    row.map(TemplateRecord::try_from).transpose()
}

// Why: retired templates take no revision (`None`); publishing a closed one
// reopens it to new activations.
pub async fn publish_template(
    pool: &PgPool,
    draft: &TemplateDraft,
    source: TemplateSource,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<Option<TemplateRecord>, StoreError> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as!(
        TemplateRow,
        r#"INSERT INTO dynwork_routine_templates
               (id, revision, state, mandatory, plugins, source, body, created_by, updated_by, updated_at)
           VALUES ($1, 1, 'published', $2, $3, $4, $5, $6, $6, $7)
           ON CONFLICT (id) DO UPDATE
           SET revision = dynwork_routine_templates.revision + 1,
               state = 'published',
               mandatory = EXCLUDED.mandatory, plugins = EXCLUDED.plugins,
               source = EXCLUDED.source, body = EXCLUDED.body,
               updated_by = EXCLUDED.updated_by, updated_at = EXCLUDED.updated_at
           WHERE dynwork_routine_templates.state <> 'retired'
           RETURNING id, revision, state, retire_mode, mandatory, plugins, source,
                     body AS "body: Json<TemplateBody>", created_by, updated_by, updated_at"#,
        draft.id.as_str(),
        draft.mandatory,
        draft.plugins.as_slice(),
        source.as_str(),
        Json(&draft.body) as _,
        author.as_str(),
        now
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    sqlx::query!(
        "INSERT INTO dynwork_routine_template_revisions
             (template_id, revision, mandatory, body, published_by, published_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
        row.id.as_str(),
        row.revision,
        row.mandatory,
        Json(&draft.body) as _,
        author.as_str(),
        now
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    TemplateRecord::try_from(row).map(Some)
}

pub async fn find_template(
    pool: &PgPool,
    template: &TemplateId,
) -> Result<Option<TemplateRecord>, StoreError> {
    let row = sqlx::query_as!(
        TemplateRow,
        r#"SELECT id, revision, state, retire_mode, mandatory, plugins, source,
                  body AS "body: Json<TemplateBody>", created_by, updated_by, updated_at
           FROM dynwork_routine_templates
           WHERE id = $1"#,
        template.as_str()
    )
    .fetch_optional(pool)
    .await?;
    row.map(TemplateRecord::try_from).transpose()
}

// Why: only a published template closes; `None` otherwise. Its linked
// routines keep following it.
pub async fn close_template(
    pool: &PgPool,
    template: &TemplateId,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<Option<TemplateRecord>, StoreError> {
    let row = sqlx::query_as!(
        TemplateRow,
        r#"UPDATE dynwork_routine_templates
           SET state = 'closed', updated_by = $2, updated_at = $3
           WHERE id = $1 AND state = 'published'
           RETURNING id, revision, state, retire_mode, mandatory, plugins, source,
                     body AS "body: Json<TemplateBody>", created_by, updated_by, updated_at"#,
        template.as_str(),
        author.as_str(),
        now
    )
    .fetch_optional(pool)
    .await?;
    row.map(TemplateRecord::try_from).transpose()
}

pub async fn list_templates(pool: &PgPool) -> Result<Vec<TemplateRecord>, StoreError> {
    let rows = sqlx::query_as!(
        TemplateRow,
        r#"SELECT id, revision, state, retire_mode, mandatory, plugins, source,
                  body AS "body: Json<TemplateBody>", created_by, updated_by, updated_at
           FROM dynwork_routine_templates
           ORDER BY id"#
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(TemplateRecord::try_from).collect()
}

pub async fn list_linked_template_ids(
    pool: &PgPool,
    user: &UserId,
) -> Result<Vec<TemplateId>, StoreError> {
    let ids = sqlx::query_scalar!(
        r#"SELECT DISTINCT template_id AS "template_id!"
           FROM dynwork_routines
           WHERE user_id = $1 AND deleted_at IS NULL AND template_id IS NOT NULL
           ORDER BY 1"#,
        user.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(ids.into_iter().map(TemplateId::from_stored).collect())
}

// Why: tombstones keep their template link, so a deleted task still counts.
pub async fn has_had_template(
    pool: &PgPool,
    user: &UserId,
    template: &TemplateId,
) -> Result<bool, StoreError> {
    let had = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM dynwork_routines WHERE user_id = $1 AND template_id = $2
           ) AS "had!""#,
        user.as_str(),
        template.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(had)
}

pub async fn find_template_revision(
    pool: &PgPool,
    template: &TemplateId,
    revision: i64,
) -> Result<Option<RevisionRecord>, StoreError> {
    let row = sqlx::query_as!(
        RevisionRow,
        r#"SELECT template_id, revision, mandatory, body AS "body: Json<TemplateBody>",
                  published_by, published_at
           FROM dynwork_routine_template_revisions
           WHERE template_id = $1 AND revision = $2"#,
        template.as_str(),
        revision
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(RevisionRecord::from))
}
