//! `/admin/brain/ingest`: the global indexing queue, with its errors.

use axum::extract::Query;
use axum::response::Response;
use axum::Extension;
use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::source::{Source, require_admin};
use crate::views::{AT_SOURCE, NOT_MOUNTED, crumbs, keep, state_tone, when};

const STATES: [&str; 4] = ["pending", "running", "done", "failed"];
const SHOWN: i64 = 200;

#[derive(Debug, Default, Deserialize)]
pub struct IngestQuery {
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QueueRow {
    pub id: i64,
    pub article: i64,
    pub title: String,
    pub lang: String,
    pub state: String,
    pub tone: &'static str,
    pub attempts: i32,
    pub error: String,
    pub updated: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct StateChoice {
    pub label: &'static str,
    pub selected: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct IngestPage {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub unavailable: Option<&'static str>,
    pub proxy_note: Option<&'static str>,
    pub states: Vec<StateChoice>,
    pub rows: Vec<QueueRow>,
    pub has_rows: bool,
}

pub async fn ingest_page(
    shell: Page,
    Extension(source): Extension<Source>,
    Query(query): Query<IngestQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let mut faults = Faults::new();
    let mut page = IngestPage {
        page: "brain-ingest",
        title: "Ingest",
        breadcrumbs: crumbs("Ingest"),
        ..IngestPage::default()
    };
    let wanted = query
        .state
        .as_deref()
        .filter(|state| STATES.contains(state));
    page.states = STATES
        .into_iter()
        .map(|label| StateChoice {
            label,
            selected: Some(label) == wanted,
        })
        .collect();
    match source.console() {
        None => page.unavailable = Some(NOT_MOUNTED),
        Some(console) => match &console.native {
            None => page.proxy_note = Some(AT_SOURCE),
            Some(native) => {
                let jobs = native.repo.list_index_jobs(wanted, SHOWN).await;
                page.rows = keep(&mut faults, "Queue", jobs)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|job| QueueRow {
                        id: job.id,
                        article: job.article_id,
                        title: job.title.unwrap_or_default(),
                        lang: job.lang,
                        tone: state_tone(&job.state),
                        retryable: job.state == "failed",
                        state: job.state,
                        attempts: job.attempts,
                        error: job.last_error.unwrap_or_default(),
                        updated: when(job.updated_at),
                    })
                    .collect();
                page.has_rows = !page.rows.is_empty();
            },
        },
    }
    Ok(render_page_with_faults(
        &shell.engine,
        "brain-ingest",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
