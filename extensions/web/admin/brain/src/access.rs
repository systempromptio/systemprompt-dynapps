//! The Access tab of an article, and the form that changes it.
//!
//! It lists what the article's own grants say and what each subject ends up
//! with after inheritance, and a form adds, changes or removes one own grant
//! through the Brain's `set_grants`, as the signed-in admin. A proxy instance
//! answers that its permissions are managed at the source.

use axum::extract::Path;
use axum::response::Redirect;
use axum::{Extension, Form};
use dynbrain::access::{Level, SubjectKind};
use dynbrain::brain::args::{GrantInput, GrantTarget};
use dynbrain::brain::views::GrantView;
use dynbrain::console::Console;
use dynbrain::ids::ArticleId;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::source::{Source, require_admin};

#[derive(Debug, Serialize)]
pub struct GrantRow {
    pub who: String,
    pub kind: &'static str,
    pub level: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AccessView {
    pub article: i64,
    pub lang: String,
    pub editable: bool,
    pub notice: Option<String>,
    pub inherits: Option<bool>,
    pub own: Vec<GrantRow>,
    pub effective: Vec<GrantRow>,
}

#[derive(Debug, Deserialize)]
pub struct GrantForm {
    pub kind: String,
    pub subject: String,
    pub level: String,
    pub lang: Option<String>,
}

pub async fn view(console: &Console, admin: &UserId, article: i64, lang: &str) -> AccessView {
    let mut view = AccessView {
        article,
        lang: lang.to_owned(),
        editable: false,
        notice: None,
        inherits: None,
        own: Vec::new(),
        effective: Vec::new(),
    };
    let target = GrantTarget::Article(ArticleId(article));
    match console.brain.list_grants(admin, target).await {
        Ok(grants) => {
            view.editable = true;
            view.inherits = grants.inherit_access;
            view.own = grants.own.iter().map(row).collect();
            view.effective = grants.access.iter().map(row).collect();
        },
        Err(fault) => view.notice = Some(fault.to_string()),
    }
    view
}

fn row(grant: &GrantView) -> GrantRow {
    GrantRow {
        who: grant.id.clone(),
        kind: match grant.kind {
            SubjectKind::User => "User",
            SubjectKind::Role => "Role",
            SubjectKind::Group => "Group",
        },
        level: match grant.level {
            Level::NoAccess => "No access",
            Level::Read => "Can read",
            Level::Write => "Can edit",
            Level::Manage => "Can manage",
        },
    }
}

pub async fn set_access(
    shell: Page,
    Extension(source): Extension<Source>,
    Path(article): Path<i64>,
    Form(form): Form<GrantForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&shell)?;
    let console = source
        .console()
        .ok_or_else(|| AdminError::NotFound("the Brain is not running".to_owned()))?;
    let admin = &shell.user.user_id;
    let kind = parse_kind(&form.kind)?;
    let subject = form.subject.trim().to_owned();
    if subject.is_empty() {
        return Err(AdminError::BadRequest("Name who the grant is for.".to_owned()).into());
    }
    let level = parse_level(&form.level)?;
    let target = GrantTarget::Article(ArticleId(article));
    let current = console
        .brain
        .list_grants(admin, target)
        .await
        .map_err(|fault| AdminError::ConflictingState {
            context: "the Brain refused the access change",
            source: fault.into(),
        })?;
    let mut grants: Vec<GrantInput> = current
        .own
        .iter()
        .filter(|grant| !(grant.kind == kind && grant.id == subject))
        .map(|grant| GrantInput {
            kind: grant.kind,
            id: grant.id.clone(),
            level: grant.level,
        })
        .collect();
    if let Some(level) = level {
        grants.push(GrantInput {
            kind,
            id: subject.clone(),
            level,
        });
    }
    console
        .brain
        .set_grants(admin, target, &grants)
        .await
        .map_err(|fault| AdminError::ConflictingState {
            context: "the Brain refused the access change",
            source: fault.into(),
        })?;
    if let Some(native) = &console.native {
        native
            .repo
            .record_admin_action(admin.as_str(), "set_access", Some(&subject), &form.level)
            .await
            .map_err(AdminError::internal)?;
    }
    let lang: String = form
        .lang
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    Ok(Redirect::to(&format!(
        "/admin/brain/articles/{article}?tab=access&lang={lang}"
    )))
}

fn parse_kind(kind: &str) -> Result<SubjectKind, AdminError> {
    match kind {
        "user" => Ok(SubjectKind::User),
        "role" => Ok(SubjectKind::Role),
        "group" => Ok(SubjectKind::Group),
        _ => Err(AdminError::BadRequest(
            "Pick user, role or group.".to_owned(),
        )),
    }
}

fn parse_level(level: &str) -> Result<Option<Level>, AdminError> {
    match level {
        "none" => Ok(Some(Level::NoAccess)),
        "read" => Ok(Some(Level::Read)),
        "write" => Ok(Some(Level::Write)),
        "manage" => Ok(Some(Level::Manage)),
        "remove" => Ok(None),
        _ => Err(AdminError::BadRequest("Pick a level.".to_owned())),
    }
}
