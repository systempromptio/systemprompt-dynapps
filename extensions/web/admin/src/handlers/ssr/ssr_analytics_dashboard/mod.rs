//! `/admin/analytics` — every question about what the AI actually did.
//!
//! Six URL-driven tabs over one scope and one window. **Overview** puts the
//! whole instance on one screen: KPIs, request and cost trends, the model mix,
//! the top-user leaderboard and the latency split.
//! **Models** is per-model gateway behaviour including the unrouted bucket.
//! **Skills** is adoption and the conversations it reached. **Tools** is MCP
//! execution health. **Sessions** is client-reported session cost and rating.
//! **Cost** is the supplier bill, with a customer view that carries no
//! supplier figure at all.
//!
//! Scope and window live in the query string and are shared with the rest of
//! the AI-activity group through `js/services/scope.js`: `group`, `project`,
//! `user_id`, `preset`. Attribution is a separate claim about the same rows —
//! `Exclusive` by default, so container totals partition the instance; `?attr=
//! member` opts into the overlapping "who uses what" reading, and the page
//! says on screen that it overlaps.
//!
//! Console roles only. Every row drills into `/admin/requests` carrying the
//! dimension it names alongside the scope and window already on screen.

use std::sync::Arc;
use systemprompt_web_shared::{GroupId, ProjectId};

use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::repositories::analytics::site::cost::ContainerAxis;
use crate::repositories::analytics::site::leaderboards::LeaderboardSort;
use crate::repositories::analytics::site::resolve_site_scope;
use crate::repositories::analytics::site::series::SeriesBucket;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::scope::{Attribution, Scope, ScopeRequest};
use systemprompt_web_admin_core::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

mod context;
mod context_overview;
mod context_tabs;
mod data;
mod data_tabs;
mod filters;
mod page;
mod tab_cost;
mod tab_models;
pub(crate) use tab_models::{ms, qualifier, short_name};
mod tab_sessions;
mod tab_skills;
mod tab_tools;
mod urls;
mod urls_controls;
mod view;
mod view_models;
mod view_spend;
mod view_tables;

use context::DashboardTab;
use page::{PageInput, page_context};
use systemprompt_web_admin_core::ssr::list_view::DEFAULT_PAGE_SIZE;
use systemprompt_web_admin_core::ssr::page::Page;

const BASE_URL: &str = "/admin/analytics";
const TRENDS_DEFAULT_PRESET: TimeRangePreset = TimeRangePreset::Days7;
#[derive(Debug, Deserialize)]
pub(crate) struct AnalyticsDashboardQuery {
    pub tab: Option<String>,
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub bucket: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<systemprompt::identifiers::UserId>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub slo_ms: Option<i32>,
    pub attr: Option<String>,
    pub audience: Option<String>,
    pub axis: Option<String>,
}

impl AnalyticsDashboardQuery {
    pub(crate) fn scope(&self) -> Scope {
        if let Some(user_id) = self.user_id.clone().filter(|u| !u.as_str().is_empty()) {
            return Scope::User(user_id);
        }
        if let Some(project) = self.project.clone().filter(|p| !p.is_empty()) {
            return Scope::Project(ProjectId::new(project));
        }
        self.group
            .clone()
            .filter(|g| !g.is_empty())
            .map_or(Scope::All, |group| Scope::Group(GroupId::new(group)))
    }

    pub(crate) fn attribution(&self) -> Attribution {
        if self.attr.as_deref() == Some("member") {
            Attribution::Member
        } else {
            Attribution::Exclusive
        }
    }

    pub(crate) fn is_internal_audience(&self) -> bool {
        self.audience.as_deref() != Some("customer")
    }

    pub(crate) fn container_axis(&self) -> ContainerAxis {
        if self.axis.as_deref() == Some("project") {
            ContainerAxis::Project
        } else {
            ContainerAxis::Group
        }
    }
}

pub(crate) async fn analytics_dashboard_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = DashboardTab::from_query(query.tab.as_deref());
    let bucket = SeriesBucket::from_bucket_param(query.bucket.as_deref());
    let sort = LeaderboardSort::from_sort_param(query.sort.as_deref());
    let range = resolve_range(&query);
    let page = query.page.unwrap_or(0).max(0);

    let request = ScopeRequest::from_query(
        &shell.user,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = resolve_site_scope(&pool, &query.scope(), query.attribution()).await?;
    let slo_ms = crate::repositories::analytics::site::latency::resolve_slo_ms(query.slo_ms);

    let fetched = data::load_dashboard_data(
        &pool,
        data::DashboardQueryPlan {
            tab,
            scope: &scope,
            range,
            bucket,
            sort,
            page_size: DEFAULT_PAGE_SIZE,
            offset: page * DEFAULT_PAGE_SIZE,
            slo_ms,
            axis: query.container_axis(),
        },
    )
    .await;
    let filters = filters::build_filters(&pool, &shell.user, &query, &request, bucket).await;
    let ctx = page_context(PageInput {
        query: &query,
        tab,
        range,
        bucket,
        page,
        filters,
        fetched: &fetched,
        slo_ms,
    });

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &shell.engine,
            "analytics-dashboard",
            &ctx,
            &shell.user,
            &shell.marketplace,
        ),
    )
}

pub(super) fn resolve_range(query: &AnalyticsDashboardQuery) -> TimeRange {
    let user_picked = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    if user_picked {
        parse_time_range(&TimeRangeQuery {
            from: query.from.clone(),
            to: query.to.clone(),
            preset: query.preset.clone(),
        })
    } else {
        preset_to_range(TRENDS_DEFAULT_PRESET)
    }
}
