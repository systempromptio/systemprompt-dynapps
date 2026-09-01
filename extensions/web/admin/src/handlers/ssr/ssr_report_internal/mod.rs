//! `/admin/reports/internal` — the month-end provider cost report.
//!
//! One calendar month of what inference cost us at the providers, by
//! provider and by model, with the twelve-month trend. This is the
//! operator's report and nobody else's — the sibling `ssr_report_customer`
//! is the sendable half, and it shares no cost figure with this one.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::reports::internal;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::month_range::{MonthQuery, list_month_options, parse_month_range};

mod context;
pub(crate) mod csv;
mod view;

use context::ReportInternalContext;

const BASE_URL: &str = "/admin/reports/internal";
const TREND_MONTHS: i32 = 12;

pub(crate) async fn report_internal_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<MonthQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let month = parse_month_range(&query);

    // Why: a failed query must not render as zero cost. This page states a
    // financial result, and an empty one reads as a true statement about the
    // business rather than as a database blip.
    let providers = internal::list_provider_month_costs(&pool, month.from, month.to).await?;
    let models = internal::list_model_month_costs(&pool, month.from, month.to).await?;
    let series = internal::list_platform_month_series(&pool, TREND_MONTHS).await?;

    let provider_views = view::supplier_views(&providers);
    let model_views = view::supplier_views(&models);
    let next = month.next();

    let ctx = ReportInternalContext {
        page: "report-internal",
        title: "Provider cost",
        subtitle: "What inference cost us at the providers for one calendar month.",
        month_key: month.key.clone(),
        month_label: month.label.clone(),
        month_complete: month.is_complete,
        months: list_month_options(&month),
        prev_url: month_url(&month.previous().key),
        next_url: next.as_ref().map(|m| month_url(&m.key)),
        has_next: next.is_some(),
        base_url: BASE_URL,
        project_key: None,
        generated_at: chrono::Utc::now().format("%d %B %Y").to_string(),
        totals: view::totals(&providers),
        providers: provider_views,
        models: model_views,
        trend: view::trend_chart(&series),
    };

    Ok(super::render_typed_page(
        &engine,
        "report-internal",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn month_url(key: &str) -> String {
    format!("{BASE_URL}?month={key}")
}
