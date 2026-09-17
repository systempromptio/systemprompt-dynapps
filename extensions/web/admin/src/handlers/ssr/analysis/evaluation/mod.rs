//! `/admin/analysis/evaluation` — what the judge makes of the instance: the
//! AI-written report and the scores behind it (`overview`), the same for one
//! marketplace (`marketplace`), every report ever written (`reports`), and
//! the judge's own runs as the audit trail: prompts, response, tokens, cost,
//! and the gateway request behind each.

pub(crate) mod actions;
pub(crate) mod marketplace;
pub(crate) mod overview;
mod report_view;
pub(crate) mod reports;
pub(crate) mod trigger;
pub(crate) mod views;

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared::get_baked_services_path;
use crate::handlers::ssr::list_view::Pagination;
use crate::repositories::config::evaluation::{LoadedEvaluation, load_evaluation};
use crate::repositories::evaluation::reports::EvaluationReportRow;
use crate::repositories::evaluation::runs::{self, RunsFilter};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use views::{RunDetailView, RunView};

const PAGE_SIZE: i64 = 50;
pub(crate) fn load_config() -> Result<Option<LoadedEvaluation>, AdminError> {
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let path = get_baked_services_path()?;
    Ok(load_evaluation(&path, &services.providers, &services.ai)?)
}

pub(super) fn owner_plugin() -> Result<Option<String>, AdminError> {
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    Ok(services
        .plugins
        .values()
        .find(|p| p.enabled && p.hooks.evaluation)
        .map(|p| p.id.to_string()))
}

// Why: a report that is queued or has failed, as the pages say so.
#[derive(Debug, Serialize)]
pub(crate) struct PendingView {
    pub status: String,
    pub tone: &'static str,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub trigger: String,
    pub created_relative: String,
}

impl PendingView {
    pub(crate) fn of(row: &EvaluationReportRow) -> Self {
        Self {
            tone: if row.status == "failed" {
                "err"
            } else {
                "info"
            },
            status: row.status.clone(),
            attempts: row.attempts,
            last_error: row.last_error.clone(),
            trigger: row.trigger.clone(),
            created_relative: crate::handlers::ssr::format::relative_time(row.created_at),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunsQuery {
    criterion: Option<String>,
    status: Option<String>,
    host: Option<String>,
    session: Option<String>,
    page: Option<u32>,
}

impl RunsQuery {
    fn url(&self, page: i64) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        for (key, value) in [
            ("criterion", self.criterion.clone()),
            ("status", self.status.clone()),
            ("host", self.host.clone()),
            ("session", self.session.clone()),
        ] {
            if let Some(value) = value.filter(|v| !v.is_empty()) {
                pairs.push((key, value));
            }
        }
        if page > 1 {
            pairs.push(("page", page.to_string()));
        }
        let query = pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={}", urlencoding::encode(&value)))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            "/admin/analysis/evaluation/runs".to_owned()
        } else {
            format!("/admin/analysis/evaluation/runs?{query}")
        }
    }
}

fn clean(value: Option<&String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

#[derive(Serialize)]
struct RunsContext {
    page: &'static str,
    title: &'static str,
    query: RunsQuery,
    runs: Vec<RunView>,
    has_runs: bool,
    pagination: Pagination,
    criteria: [&'static str; 4],
    statuses: [&'static str; 2],
    export: crate::export::ExportView,
}

pub(crate) async fn runs_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RunsQuery>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let page = i64::from(query.page.unwrap_or(1).max(1));
    let filter = RunsFilter {
        criterion: clean(query.criterion.as_ref()),
        status: clean(query.status.as_ref()),
        host: clean(query.host.as_ref()),
        session: clean(query.session.as_ref()),
        limit: PAGE_SIZE,
        offset: (page - 1) * PAGE_SIZE,
    };
    let (rows, total) = runs::list_runs_paged(&pool, &filter).await?;
    let shown = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let total_pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let pagination = Pagination {
        current_page: page.min(total_pages),
        total_pages,
        first_row: if shown == 0 {
            0
        } else {
            (page - 1) * PAGE_SIZE + 1
        },
        last_row: (page - 1) * PAGE_SIZE + shown,
        total_rows: total,
        noun: "runs",
        has_prev: page > 1,
        has_next: page < total_pages,
        prev_url: (page > 1).then(|| query.url(page - 1)),
        next_url: (page < total_pages).then(|| query.url(page + 1)),
    };
    let export = crate::export::ExportView::single(
        "evaluation-runs",
        &crate::export::view::query_string(&[
            ("criterion", query.criterion.as_deref()),
            ("status", query.status.as_deref()),
            ("host", query.host.as_deref()),
            ("session", query.session.as_deref()),
        ]),
    );
    let context = RunsContext {
        page: "analysis-evaluation-runs",
        title: "Judge runs",
        query,
        export,
        has_runs: !rows.is_empty(),
        runs: rows.iter().map(RunView::of).collect(),
        pagination,
        criteria: ["intent", "quality", "speed", "cost"],
        statuses: ["succeeded", "failed"],
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluation-runs",
        &context,
        &user,
        &marketplace,
    ))
}

#[derive(Serialize)]
struct RunContext {
    page: &'static str,
    title: String,
    detail: RunDetailView,
    runs_url: &'static str,
}

pub(crate) async fn run_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let detail = runs::find_run(&pool, id.trim())
        .await?
        .ok_or_else(|| AdminError::NotFound("No judge run has that id.".to_owned()))?;
    let detail = RunDetailView::of(&detail);
    let context = RunContext {
        page: "analysis-evaluation-run",
        title: format!(
            "Judge run · {} · {}",
            detail.run.criterion, detail.run.id_short
        ),
        detail,
        runs_url: "/admin/analysis/evaluation/runs",
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluation-run",
        &context,
        &user,
        &marketplace,
    ))
}
