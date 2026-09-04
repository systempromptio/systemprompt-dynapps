//! `/admin/analytics/users/{user_id}` — one person's analytics.
//!
//! The dashboard's `?user_id=` filter re-renders the site view scoped down;
//! this page is the person-shaped view instead: their trend, their model mix,
//! their daily records, and their session cost history, with links back to
//! the raw request log and the management page.
//!
//! Scoping: admin-only; an admin sees every project's people.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::analytics::site::SiteScope;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::project_scope::ProjectScope;
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

mod context;
mod data;
mod view;


const SESSION_ROW_LIMIT: i64 = 25;

#[derive(Debug, Deserialize)]
pub(crate) struct AnalyticsUserQuery {
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn analytics_user_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    Query(query): Query<AnalyticsUserQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let user_id = UserId::new(user_id_raw);

    let detail = crate::repositories::users::queries::find_user_detail(&pool, &user_id).await?;
    let Some(detail) = detail else {
        return Err(AdminError::NotFound("User not found.".to_owned()).into());
    };

    let range = resolve_range(&query);
    // Why: The page is already pinned to one user, so the query itself needs
    // no project predicate.
    let scope = SiteScope {
        project: ProjectScope::All,
        user_id: Some(user_id.clone()),
    };

    let fetched = data::load_user_data(&pool, &user_id, range, &scope, SESSION_ROW_LIMIT).await;
    let ctx = view::page_context(&view::PageInput {
        user_id: &user_id,
        detail: &detail,
        range,
        query: &query,
        fetched: &fetched,
    });

    Ok(super::render_typed_page(
        &engine,
        "analytics-user",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: a person's page defaults to a month — a week of one individual's
// traffic is usually too sparse to read as a trend.
fn resolve_range(query: &AnalyticsUserQuery) -> TimeRange {
    let user_picked = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    if user_picked {
        parse_time_range(&TimeRangeQuery {
            from: query.from.clone(),
            to: query.to.clone(),
            preset: query.preset.clone(),
        })
    } else {
        preset_to_range(TimeRangePreset::Days30)
    }
}
