//! `/admin/reports/customer` — the sendable month-end usage report.
//!
//! One project (or every project), one calendar month: tokens, who used
//! them, which project they sat in, which models they reached. Nothing about
//! what serving them cost us — that is the internal report, and the two
//! share no query.
//!
//! Admin-only, and scoped. An admin may pick a project with `?project=`; the
//! same [`ProjectScope`] rule pins anyone else to their own. The page carries
//! a print action, so "send it" is a PDF the operator produces from the
//! browser rather than a second rendering path that could drift from this one.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::project_filter_view;
use crate::repositories::reports::customer;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::month_range::{MonthQuery, MonthRange, list_month_options, parse_month_range};
use crate::util::project_scope::ProjectScope;

mod context;
pub(crate) mod csv;
mod view;

use context::ReportCustomerContext;

const BASE_URL: &str = "/admin/reports/customer";

#[derive(Debug, Deserialize)]
pub(crate) struct CustomerReportQuery {
    pub month: Option<String>,
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
    let scope = ProjectScope::resolve(&user_ctx, query.project.as_deref());

    let summary = customer::get_customer_month_summary(&pool, &scope, month.from, month.to).await?;
    let users = customer::list_customer_month_users(&pool, &scope, month.from, month.to).await?;
    let projects =
        customer::list_customer_month_projects(&pool, &scope, month.from, month.to).await?;
    let models = customer::list_customer_month_models(&pool, &scope, month.from, month.to).await?;

    let ctx = build_context(&BuildInput {
        user_ctx: &user_ctx,
        scope,
        month: &month,
        summary: &summary,
        users: &users,
        projects: &projects,
        models: &models,
    });

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
    scope: ProjectScope,
    month: &'a MonthRange,
    summary: &'a customer::CustomerMonthSummary,
    users: &'a [customer::CustomerUserUsage],
    projects: &'a [customer::CustomerProjectUsage],
    models: &'a [customer::CustomerModelUsage],
}

fn build_context(input: &BuildInput<'_>) -> ReportCustomerContext {
    let BuildInput {
        user_ctx,
        scope,
        month,
        summary,
        users,
        projects,
        models,
    } = *input;

    let scope_name = scope_name(scope);
    let project_key = scope.selected().map(|p| p.as_str().to_owned());
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
        project_filter: project_filter_view(
            user_ctx,
            scope,
            BASE_URL,
            vec![("month".to_owned(), month.key.clone())],
        ),
        summary: view::summary_view(summary),
        users: view::user_views(users),
        projects: view::project_views(projects),
        models: view::model_views(models),
    }
}

fn scope_name(scope: ProjectScope) -> String {
    match scope {
        ProjectScope::All => "Astound Digital".to_owned(),
        ProjectScope::Only(project) => format!("Astound Digital · {}", project.label()),
        ProjectScope::Nothing => "No project".to_owned(),
    }
}

fn month_url(project: Option<&str>, key: &str) -> String {
    project.map_or_else(
        || format!("{BASE_URL}?month={key}"),
        |p| format!("{BASE_URL}?project={p}&month={key}"),
    )
}
