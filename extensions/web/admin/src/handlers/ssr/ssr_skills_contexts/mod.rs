//! `/admin/contexts` — list of conversation contexts with per-user
//! drill-down. Filters by user, model, free text, and time range; supports a
//! flat "Contexts" view and a grouped "By user" summary view.

mod context;
mod data;
mod load;
mod view;

use crate::error::AdminError;
use std::sync::Arc;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories;
use crate::repositories::analytics::contexts_list;
use crate::repositories::analytics::contexts_list::{
    ContextSort, ContextSortColumn, ContextSortDir,
};
use crate::repositories::scope::{ScopeRequest, SubjectScope};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use context::{ContextsPageContext, FilterView, PageKpisView, PageStat, UserForFilterView};
use data::{
    build_contexts_json, build_user_summaries_json, group_contexts_by_user, microdollars_to_usd,
};
use load::{ContextsPageData, load_page_data};

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ContextsListQuery {
    pub user_id: Option<UserId>,
    pub model: Option<String>,
    pub q: Option<String>,
    pub since: Option<String>,
    pub view: Option<String>,
    pub limit: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    // Why: the shell's scope bar is the one window control on every AI-activity
    // page and it emits `?preset=`. This page's own parameter is `since`, so
    // the preset is read as a fallback rather than silently ignored.
    pub preset: Option<String>,
}

const BASE_URL: &str = "/admin/contexts";
const PAGE_SIZE: i64 = 50;

fn since_to_datetime(value: &str) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    let dur = match value {
        "24h" | "1d" => Duration::hours(24),
        "7d" => Duration::days(7),
        "30d" => Duration::days(30),
        "90d" => Duration::days(90),
        _ => return None,
    };
    Some(now - dur)
}

pub(super) struct ContextsPageInputs {
    user_id: Option<UserId>,
    model: Option<String>,
    q: Option<String>,
    since_label: Option<String>,
    view: String,
    pub(super) filter: contexts_list::ContextListFilter,
    pub(super) sort: ContextSort,
    pub(super) page: i64,
}

fn parse_inputs(params: &ContextsListQuery, scope: &SubjectScope) -> ContextsPageInputs {
    let trim_opt = |s: Option<String>| -> Option<String> {
        s.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
    };
    let user_id = params
        .user_id
        .clone()
        .filter(|u| !u.as_str().trim().is_empty());
    let model = trim_opt(params.model.clone());
    let q = trim_opt(params.q.clone());
    let since_label = trim_opt(params.since.clone()).or_else(|| trim_opt(params.preset.clone()));
    let since_dt = since_label.as_deref().and_then(since_to_datetime);
    let view = params
        .view
        .as_deref()
        .map(str::to_lowercase)
        .filter(|v| v == "users" || v == "contexts")
        .unwrap_or_else(|| "contexts".to_owned());
    let filter = contexts_list::ContextListFilter {
        user_id: user_id.clone(),
        model: model.clone(),
        free_text: q.clone(),
        since: since_dt,
        limit: params.limit.unwrap_or(0),
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
    };
    ContextsPageInputs {
        user_id,
        model,
        q,
        since_label,
        view,
        filter,
        sort: ContextSort {
            column: ContextSortColumn::parse_context_column(params.sort.as_deref()),
            dir: ContextSortDir::parse_context_dir(params.dir.as_deref()),
        },
        page: params.page.unwrap_or(0).max(0),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one call site; splitting the six inputs into a struct would only rename them"
)]
async fn build_page_json(
    pool: &PgPool,
    user_ctx: &UserContext,
    request: &ScopeRequest,
    inputs: &ContextsPageInputs,
    data: &ContextsPageData,
    params: &ContextsListQuery,
) -> ContextsPageContext {
    let contexts_by_user = group_contexts_by_user(&data.contexts);
    let contexts_json = build_contexts_json(&data.contexts);
    let user_summaries_json = build_user_summaries_json(&data.user_summaries, &contexts_by_user);
    let users_for_filter_json: Vec<UserForFilterView> = data
        .users_for_filter
        .iter()
        .map(|u| UserForFilterView {
            user_id: u.user_id.clone(),
            display_name: u.display_name.clone(),
        })
        .collect();
    let kpis = data.kpis;
    let selected_group = request.group.clone().unwrap_or_default();
    let selected_project = request.project.clone().unwrap_or_default();
    let total_tokens = kpis.total_input_tokens + kpis.total_output_tokens;
    let total_cost_usd = microdollars_to_usd(kpis.total_cost_microdollars);
    ContextsPageContext {
        page: "contexts",
        title: "Conversation Contexts",
        breadcrumbs: vec![
            crate::handlers::ssr::types::BreadcrumbView::link("AI activity", "/admin/analytics"),
            crate::handlers::ssr::types::BreadcrumbView::current("Contexts"),
        ],
        contexts: contexts_json,
        user_summaries: user_summaries_json,
        users_for_filter: users_for_filter_json,
        models: data.models.clone(),
        kpis: PageKpisView {
            total_contexts: kpis.total_contexts,
            active_users: kpis.active_users,
            total_requests: kpis.total_requests,
            total_messages: kpis.total_messages,
            total_tokens,
            total_cost_usd,
        },
        filter: FilterView {
            user_id: inputs
                .user_id
                .clone()
                .unwrap_or_else(|| UserId::new(String::new())),
            model: inputs.model.clone().unwrap_or_default(),
            q: inputs.q.clone().unwrap_or_default(),
            since: inputs.since_label.clone().unwrap_or_default(),
            view: inputs.view.clone(),
            group: selected_group,
            project: selected_project,
        },
        scope_filter: crate::handlers::ssr::list_view::scope_filter_view(
            pool,
            user_ctx,
            request,
            BASE_URL,
            vec![
                ("q".to_owned(), inputs.q.clone().unwrap_or_default()),
                ("model".to_owned(), inputs.model.clone().unwrap_or_default()),
                (
                    "since".to_owned(),
                    inputs.since_label.clone().unwrap_or_default(),
                ),
                ("view".to_owned(), inputs.view.clone()),
            ],
        )
        .await,
        view_tabs: view::view_tabs(params, &inputs.view),
        view_is_users: inputs.view == "users",
        view_is_contexts: inputs.view == "contexts",
        page_stats: page_stats(&kpis),
        pagination: view::build_pagination(
            params,
            PageWindow::new(
                inputs.page,
                PAGE_SIZE,
                data.total_contexts,
                i64::try_from(data.contexts.len()).unwrap_or(PAGE_SIZE),
                "contexts",
            ),
        ),
        sort_headers: view::build_sort_headers(params, inputs.sort.column, inputs.sort.dir),
        total_count: data.total_contexts,
        count_label: format!("{} contexts", data.total_contexts),
        has_contexts: !data.contexts.is_empty(),
        has_user_summaries: !data.user_summaries.is_empty(),
    }
}

pub(crate) async fn skills_contexts_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ContextsListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let request = ScopeRequest::from_query(
        &user_ctx,
        params.group.as_deref(),
        params.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let inputs = parse_inputs(&params, &scope);
    let data = load_page_data(&pool, &inputs, &scope).await;
    let payload = build_page_json(&pool, &user_ctx, &request, &inputs, &data, &params).await;
    Ok(super::render_typed_page(
        &engine,
        "skills-contexts",
        &payload,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn page_stats(kpis: &contexts_list::ContextListKpis) -> Vec<PageStat> {
    vec![
        PageStat {
            value: kpis.total_contexts,
            label: "Contexts",
        },
        PageStat {
            value: kpis.active_users,
            label: "Users",
        },
        PageStat {
            value: kpis.total_requests,
            label: "Requests",
        },
        PageStat {
            value: kpis.total_messages,
            label: "Messages",
        },
    ]
}
