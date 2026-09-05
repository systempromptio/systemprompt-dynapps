//! `/admin/analytics?tab=cost` — the sendable month-end usage report.
//!
//! One group or project (or all of them), one calendar month: tokens, who used
//! them, which project they sat in, which models they reached. Nothing about
//! what serving them cost us — that is the internal report, and the two
//! share no query.
//!
//! Console-only, and scoped. A console role may pick with `?group=` and
//! `?project=`; the same [`ScopeRequest`] rule pins anyone else. The page
//! carries a print action, so "send it" is a PDF the operator produces from the
//! browser rather than a second rendering path that could drift from this one.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::scope_filter_view;
use crate::repositories::reports::customer;
use crate::repositories::scope::ScopeRequest;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::month_range::{MonthQuery, MonthRange, list_month_options, parse_month_range};

mod context;
pub(crate) mod csv;
mod view;

use context::ReportCustomerContext;

const BASE_URL: &str = "/admin/analytics?tab=cost";

#[derive(Debug, Deserialize)]
pub(crate) struct CustomerReportQuery {
    pub month: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
}

pub(crate) async fn report_customer_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<CustomerReportQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let month = parse_month_range(&MonthQuery {
        month: query.month.clone(),
    });
    let request =
        ScopeRequest::from_query(&user_ctx, query.group.as_deref(), query.project.as_deref());
    let scope = crate::repositories::scope::membership::get_subject_scope(&pool, &request).await?;

    let summary = customer::get_customer_month_summary(&pool, &scope, month.from, month.to).await?;
    let users = customer::list_customer_month_users(&pool, &scope, month.from, month.to).await?;
    let projects =
        customer::list_customer_month_projects(&pool, &scope, month.from, month.to).await?;
    let models = customer::list_customer_month_models(&pool, &scope, month.from, month.to).await?;

    let ctx = build_context(
        &BuildInput {
            user_ctx: &user_ctx,
            request: &request,
            month: &month,
            summary: &summary,
            users: &users,
            projects: &projects,
            models: &models,
        },
        &pool,
    )
    .await;

    Ok(super::render_typed_page(
        &engine,
        "report-customer",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

struct BuildInput<'a> {
    user_ctx: &'a UserContext,
    request: &'a ScopeRequest,
    month: &'a MonthRange,
    summary: &'a customer::CustomerMonthSummary,
    users: &'a [customer::CustomerUserUsage],
    projects: &'a [customer::CustomerProjectUsage],
    models: &'a [customer::CustomerModelUsage],
}

async fn build_context(input: &BuildInput<'_>, pool: &PgPool) -> ReportCustomerContext {
    let BuildInput {
        user_ctx,
        request,
        month,
        summary,
        users,
        projects,
        models,
    } = *input;

    let project_key = request.project.clone();
    let scope_name = scope_name(request);
    let next = month.next();

    ReportCustomerContext {
        page: "report-customer",
        title: format!("{scope_name} — {}", month.label),
        scope_name,
        project_key: project_key.clone(),
        month_key: month.key.clone(),
        month_label: month.label.clone(),
        month_complete: month.is_complete,
        months: list_month_options(month),
        prev_url: month_url(project_key.as_deref(), &month.previous().key),
        next_url: next
            .as_ref()
            .map(|m| month_url(project_key.as_deref(), &m.key)),
        has_next: next.is_some(),
        base_url: BASE_URL,
        generated_at: chrono::Utc::now().format("%d %B %Y").to_string(),
        scope_filter: scope_filter_view(
            pool,
            user_ctx,
            request,
            BASE_URL,
            vec![("month".to_owned(), month.key.clone())],
        )
        .await,
        summary: view::summary_view(summary),
        users: view::user_views(users),
        projects: view::project_views(projects),
        models: view::model_views(models),
    }
}

fn scope_name(request: &ScopeRequest) -> String {
    match (request.group.as_deref(), request.project.as_deref()) {
        (None, None) => "Astound Digital".to_owned(),
        (Some(group), None) => format!("Astound Digital · {group}"),
        (None, Some(project)) => format!("Astound Digital · {project}"),
        (Some(group), Some(project)) => format!("Astound Digital · {group} · {project}"),
    }
}

fn month_url(project: Option<&str>, key: &str) -> String {
    project.map_or_else(
        || format!("{BASE_URL}?month={key}"),
        |p| format!("{BASE_URL}?project={p}&month={key}"),
    )
}
