//! The admin side of the template library: save a draft, publish it, close
//! it and retire it.
//!
//! Only a Governance admin writes the library: the author's roles, read from
//! the database with `find_user_access_profile`, must grant management
//! (`roles_grant_manage`), the rule behind the admin console's `is_admin`.
//! The console and the YAML sync call these, never the store directly. A
//! publish, a close or a retire tells every connected desktop, of every
//! person, through the relay's `templates.changed` hint; a retire that paused
//! or removed someone's routine also sends them `routines.changed`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::repositories::principal::role::find_user_access_profile;
use systemprompt_web_admin_core::types::roles_grant_manage;

use crate::ids::TemplateId;
use crate::store::{self, StoreError, TemplateRecord};
use crate::template::{self, Refusal, RetireMode, TemplateDraft, TemplateSource};

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("only a Governance admin writes the template library")]
    NotAdmin,
    #[error("the template cannot be published: {0}")]
    Refused(Refusal),
    #[error("a published template changes only by publishing a new revision")]
    NotDraft,
    #[error("a retired template takes no new revision")]
    Retired,
    #[error("no template that this change applies to has this id")]
    Missing,
    #[error("template library store failure")]
    Store(#[from] StoreError),
}

pub async fn save_draft(
    pool: &PgPool,
    draft: &TemplateDraft,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<TemplateRecord, LibraryError> {
    require_admin(pool, author).await?;
    template::check_draft(draft).map_err(LibraryError::Refused)?;
    store::save_template_draft(pool, draft, TemplateSource::Console, author, now)
        .await?
        .ok_or(LibraryError::NotDraft)
}

pub async fn publish(
    pool: &PgPool,
    draft: &TemplateDraft,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<TemplateRecord, LibraryError> {
    require_admin(pool, author).await?;
    template::check_publishable(draft).map_err(LibraryError::Refused)?;
    let published = store::publish_template(pool, draft, TemplateSource::Console, author, now)
        .await?
        .ok_or(LibraryError::Retired)?;
    tracing::info!(
        template = %published.id,
        revision = published.revision,
        author = %author,
        "dynwork routine template published"
    );
    crate::http::templates_hint(pool).await;
    Ok(published)
}

// Why: only a published template closes; its linked tasks keep following it.
pub async fn close(
    pool: &PgPool,
    template: &TemplateId,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<TemplateRecord, LibraryError> {
    require_admin(pool, author).await?;
    let closed = store::close_template(pool, template, author, now)
        .await?
        .ok_or(LibraryError::Missing)?;
    tracing::info!(template = %closed.id, author = %author, "dynwork routine template closed");
    crate::http::templates_hint(pool).await;
    Ok(closed)
}

// Why: a published or closed template retires once. `Unlink` changes only
// the template; `Pause` and `Remove` also write every live linked task, so
// they apply even to a computer that is off.
pub async fn retire(
    pool: &PgPool,
    template: &TemplateId,
    mode: RetireMode,
    author: &UserId,
    now: DateTime<Utc>,
) -> Result<TemplateRecord, LibraryError> {
    require_admin(pool, author).await?;
    let retired = store::retire_template(pool, template, mode, author, now)
        .await?
        .ok_or(LibraryError::Missing)?;
    tracing::info!(
        template = %retired.template.id,
        mode = mode.as_str(),
        people = retired.users.len(),
        author = %author,
        "dynwork routine template retired"
    );
    for user in &retired.users {
        crate::http::hint(pool, user, None).await;
    }
    crate::http::templates_hint(pool).await;
    Ok(retired.template)
}

async fn require_admin(pool: &PgPool, author: &UserId) -> Result<(), LibraryError> {
    let profile = find_user_access_profile(pool, author)
        .await
        .map_err(StoreError::from)?;
    if !profile.is_some_and(|profile| roles_grant_manage(&profile.roles)) {
        return Err(LibraryError::NotAdmin);
    }
    Ok(())
}
