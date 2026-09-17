//! The report pages and the Regenerate button: every generation per scope,
//! one report with the digest it was written from, and the click that
//! queues a scope and writes it inline.

use std::sync::Arc;

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::response::{Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::PendingView;
use super::report_view::ReportView;
use super::trigger::JudgeRequest;
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis_urls::{ANALYSIS_EVALUATION_URL, analysis_evaluation_url};
use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::evaluation::reports::{
    ReportScope, claim_report_leases, find_report, insert_manual_report, list_reports_paged,
};
use crate::services::evaluation::report::{Reporter, generate_pending};
use systemprompt::identifiers::MarketplaceId;

const PAGE_SIZE: i64 = 25;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReportsQuery {
    scope: Option<String>,
    page: Option<u32>,
}

fn parse_scope(raw: Option<&str>) -> Result<Option<ReportScope>, AdminError> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if raw == "global" {
        return Ok(Some(ReportScope::Global));
    }
    match raw.split_once(':') {
        Some(("marketplace", id)) if !id.is_empty() && id.len() <= 128 => {
            Ok(Some(ReportScope::Marketplace(id.to_owned())))
        },
        _ => Err(AdminError::BadRequest(
            "scope is `global` or `marketplace:<id>`".to_owned(),
        )),
    }
}

fn scope_param(scope: &ReportScope) -> String {
    match scope {
        ReportScope::Global => "global".to_owned(),
        ReportScope::Marketplace(id) => format!("marketplace:{id}"),
    }
}

#[derive(Serialize)]
struct ReportsContext {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    scope: String,
    reports: Vec<ReportListRow>,
    has_reports: bool,
    pagination: Pagination,
}

#[derive(Serialize)]
struct ReportListRow {
    view: ReportView,
    status: String,
    status_tone: &'static str,
    attempts: i32,
    last_error: Option<String>,
    theme_count: usize,
    recommendation_count: usize,
}

pub(crate) async fn list_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<ReportsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let scope = parse_scope(query.scope.as_deref())?;
    let page = i64::from(query.page.unwrap_or(1).max(1));
    let (rows, total) =
        list_reports_paged(&pool, scope.as_ref(), PAGE_SIZE, (page - 1) * PAGE_SIZE).await?;
    let shown = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let total_pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let link = |p: i64| {
        let scope = scope.as_ref().map(scope_param).unwrap_or_default();
        format!("{ANALYSIS_EVALUATION_URL}/report?scope={scope}&page={p}")
    };
    let context = ReportsContext {
        page: "analysis-evaluation-report",
        title: "Evaluation reports",
        breadcrumbs: vec![
            BreadcrumbView::link("Evaluation", ANALYSIS_EVALUATION_URL),
            BreadcrumbView::current("Reports"),
        ],
        scope: scope.as_ref().map(scope_param).unwrap_or_default(),
        has_reports: !rows.is_empty(),
        reports: rows
            .iter()
            .map(|r| {
                let view = ReportView::of(r);
                ReportListRow {
                    theme_count: view.themes.len(),
                    recommendation_count: view.recommendations.len(),
                    view,
                    status_tone: match r.status.as_str() {
                        "generated" => "ok",
                        "failed" => "err",
                        _ => "warn",
                    },
                    status: r.status.clone(),
                    attempts: r.attempts,
                    last_error: r.last_error.clone(),
                }
            })
            .collect(),
        pagination: Pagination {
            current_page: page.min(total_pages),
            total_pages,
            first_row: if shown == 0 {
                0
            } else {
                (page - 1) * PAGE_SIZE + 1
            },
            last_row: (page - 1) * PAGE_SIZE + shown,
            total_rows: total,
            noun: "reports",
            has_prev: page > 1,
            has_next: page < total_pages,
            prev_url: (page > 1).then(|| link(page - 1)),
            next_url: (page < total_pages).then(|| link(page + 1)),
        },
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-evaluation-reports",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}

#[derive(Serialize)]
struct ReportContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    report: ReportView,
    pending: Option<PendingView>,
    scope_href: String,
    inputs_json: String,
    reports_url: &'static str,
}

pub(crate) async fn detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let row = find_report(&pool, id.trim())
        .await?
        .ok_or_else(|| AdminError::NotFound("No evaluation report has that id.".to_owned()))?;
    let report = ReportView::of(&row);
    let context = ReportContext {
        page: "analysis-evaluation-report",
        title: format!("Report · {}", report.scope_label),
        breadcrumbs: vec![
            BreadcrumbView::link("Evaluation", ANALYSIS_EVALUATION_URL),
            BreadcrumbView::link("Reports", format!("{ANALYSIS_EVALUATION_URL}/report")),
            BreadcrumbView::current(report.generated_display.clone()),
        ],
        scope_href: match &row.scope {
            ReportScope::Global => ANALYSIS_EVALUATION_URL.to_owned(),
            ReportScope::Marketplace(m) => analysis_evaluation_url(&MarketplaceId::new(m)),
        },
        // Why: discard-ok: a JSON value always serialises
        inputs_json: serde_json::to_string_pretty(&row.inputs).unwrap_or_default(),
        pending: (row.status != "generated").then(|| PendingView::of(&row)),
        report,
        reports_url: "/admin/analysis/evaluation/report",
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-evaluation-report",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegenerateForm {
    scope: String,
}

// Why: the click queues the scope regardless of staleness, writes it inline
// as the job would, and lands back on the scope's page with the new report.
pub(crate) async fn regenerate(
    request: JudgeRequest,
    State(pool): State<Arc<PgPool>>,
    Form(form): Form<RegenerateForm>,
) -> AdminHtmlResult<Redirect> {
    request.require_console_write()?;
    let scope = parse_scope(Some(&form.scope))?
        .ok_or_else(|| AdminError::BadRequest("A scope is required.".to_owned()))?;
    let reporter: Reporter = request.reporter()?;
    let window_days = i32::try_from(reporter.config.report.window_days).unwrap_or(7);
    insert_manual_report(&pool, &scope, window_days, &request.user.user_id).await?;
    let leases = claim_report_leases(&pool, 1, Some(&scope)).await?;
    let pass = generate_pending(&pool, &reporter, &leases)
        .await
        .map_err(AdminError::internal)?;
    tracing::info!(requested_by = %request.user.user_id, scope = ?scope, generated = pass.generated, failed = pass.failed, "Evaluation report regenerated from the console");
    Ok(Redirect::to(&match &scope {
        ReportScope::Global => ANALYSIS_EVALUATION_URL.to_owned(),
        ReportScope::Marketplace(m) => analysis_evaluation_url(&MarketplaceId::new(m)),
    }))
}
