//! The two writes besides access: Reindex a document and Retry a failed job.
//!
//! Both only queue work for the indexer; neither touches the content. A
//! proxy instance has no queue here, so both answer that the index lives at
//! the source.

use axum::extract::Path;
use axum::response::Redirect;
use axum::{Extension, Form};
use dynbrain::console::Native;
use dynbrain::ids::ArticleId;
use serde::Deserialize;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::source::{Source, require_admin};
use crate::views::AT_SOURCE;

#[derive(Debug, Deserialize)]
pub struct ReindexForm {
    pub lang: String,
}

pub async fn reindex(
    shell: Page,
    Extension(source): Extension<Source>,
    Path(article): Path<i64>,
    Form(form): Form<ReindexForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&shell)?;
    let native = native_of(&source)?;
    let queued = native
        .repo
        .requeue_variant(ArticleId(article), &form.lang)
        .await
        .map_err(AdminError::internal)?;
    if queued == 0 {
        return Err(AdminError::NotFound(format!("article {article} in {}", form.lang)).into());
    }
    native
        .repo
        .record_admin_action(
            shell.user.user_id.as_str(),
            "reindex",
            Some(&article.to_string()),
            &form.lang,
        )
        .await
        .map_err(AdminError::internal)?;
    let lang: String = form
        .lang
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    Ok(Redirect::to(&format!(
        "/admin/brain/articles/{article}?tab=ingest&lang={lang}"
    )))
}

pub async fn retry(
    shell: Page,
    Extension(source): Extension<Source>,
    Path(job): Path<i64>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&shell)?;
    let native = native_of(&source)?;
    let queued = native
        .repo
        .retry_job(job)
        .await
        .map_err(AdminError::internal)?;
    if queued == 0 {
        return Err(AdminError::NotFound(format!("failed job {job}")).into());
    }
    native
        .repo
        .record_admin_action(
            shell.user.user_id.as_str(),
            "retry",
            Some(&job.to_string()),
            "",
        )
        .await
        .map_err(AdminError::internal)?;
    Ok(Redirect::to("/admin/brain/ingest"))
}

fn native_of(source: &Source) -> Result<Native, AdminError> {
    let console = source
        .console()
        .ok_or_else(|| AdminError::NotFound("the Brain is not running".to_owned()))?;
    console
        .native
        .ok_or_else(|| AdminError::Conflict(AT_SOURCE.to_owned()))
}
