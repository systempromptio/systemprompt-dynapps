//! `/admin/artifacts` — every tool result the platform ingested, as a
//! first-class entity.
//!
//! An artifact is one typed tool result, stored once through the ingestion
//! narrow waist whichever client produced it and joined by the client's
//! `tool_use_id` to the model's intent, the execution that ran, and the
//! decision that governed it. The list shows structured results by default —
//! the ones an MCP server declared as data — and `?all=1` widens it to every
//! result, including builtin file reads and shell output.
//!
//! Bound to the same `?preset=&from=&to=&user_id=&page=` contract as the
//! other AI-activity list pages plus `?group=&project=` scope, and its own
//! `?tool=&source=&skill=&session=&errors=1&all=1` facets.

mod context;
mod detail;
mod preview;
mod rows;
mod view;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::artifacts::{
    ArtifactFilter, ArtifactStats, get_artifact_stats, list_artifacts,
};
use crate::repositories::governance::filter_options::{FilterOptions, get_filter_options};
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use crate::util::time_range::{TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range};

pub(crate) use detail::artifact_detail_page;
pub(crate) use preview::artifact_preview;

const BASE_URL: &str = "/admin/artifacts";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct ArtifactListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<UserId>,
    pub session: Option<SessionId>,
    pub tool: Option<String>,
    pub source: Option<String>,
    pub skill: Option<String>,
    pub errors: Option<String>,
    pub all: Option<String>,
    pub page: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
}

pub(crate) async fn artifacts_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<ArtifactListQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let page = query.page.unwrap_or(0).max(0);
    let request = ScopeRequest::from_query(
        &shell.user,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let subjects =
        crate::repositories::scope::membership::get_subject_scope(&pool, &request).await?;

    let filter = build_filter(&query, range, subjects.as_sql());
    let loaded = load_artifacts(&pool, &filter, range, page).await;
    let preset = preset_str(&query, range);
    let scope_filter = view::scope_filter(
        &pool,
        &shell.user,
        &view::ArtifactScopeFilterArgs {
            request: &request,
            query: &query,
            range,
            preset: &preset,
        },
    )
    .await;
    let ctx = build_context(
        &query,
        &filter,
        &loaded,
        PageFrame {
            range,
            preset: &preset,
            scope_filter,
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "artifacts",
        &ctx,
        &shell.user,
        &shell.marketplace,
    ))
}

// Why: the query string is the only source of every filter column, so the
// mapping lives in one place rather than inline in the page assembly.
fn build_filter(
    query: &ArtifactListQuery,
    range: TimeRange,
    subject_ids: Option<&[String]>,
) -> ArtifactFilter {
    ArtifactFilter {
        since: Some(range.from),
        until: Some(range.to),
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        subject_ids: subject_ids.map(<[String]>::to_vec),
        session_id: query.session.clone().filter(|s| !s.as_str().is_empty()),
        tool_name: non_empty(query.tool.as_deref()),
        source: non_empty(query.source.as_deref()),
        skill: non_empty(query.skill.as_deref()),
        structured_only: query.all.as_deref() != Some("1"),
        errors_only: query.errors.as_deref() == Some("1"),
    }
}

struct LoadedArtifacts {
    items: Vec<crate::repositories::analytics::artifacts::ArtifactRow>,
    total: i64,
    stats: ArtifactStats,
    options: FilterOptions,
    page: i64,
}

// Why: a failed secondary read degrades to an empty strip or option list;
// the list itself is what the reader came for and is logged the same way.
async fn load_artifacts(
    pool: &PgPool,
    filter: &ArtifactFilter,
    range: TimeRange,
    page: i64,
) -> LoadedArtifacts {
    let (list_res, stats_res, options_res) = tokio::join!(
        list_artifacts(pool, filter, PAGE_SIZE, page * PAGE_SIZE),
        get_artifact_stats(pool, filter),
        get_filter_options(pool, range),
    );
    let (items, total) = list_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_artifacts failed");
        (Vec::new(), 0)
    });
    LoadedArtifacts {
        items,
        total,
        stats: stats_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_artifact_stats failed");
            ArtifactStats::default()
        }),
        options: options_res
            .inspect_err(|e| tracing::warn!(error = %e, "get_filter_options failed"))
            .unwrap_or_default(),
        page,
    }
}

// Why: the window and the scope form travel together; both re-render from
// the same resolved range.
struct PageFrame<'a> {
    range: TimeRange,
    preset: &'a str,
    scope_filter: crate::handlers::ssr::list_view::ScopeFilterView,
}

fn build_context(
    query: &ArtifactListQuery,
    filter: &ArtifactFilter,
    loaded: &LoadedArtifacts,
    frame: PageFrame<'_>,
) -> context::ArtifactsPageContext {
    let PageFrame {
        range,
        preset,
        scope_filter,
    } = frame;
    let artifact_rows: Vec<_> = loaded.items.iter().map(rows::artifact_row).collect();
    let has_artifacts = !artifact_rows.is_empty();
    let shown = i64::try_from(artifact_rows.len()).unwrap_or(PAGE_SIZE);
    let window = PageWindow::new(loaded.page, PAGE_SIZE, loaded.total, shown, "artifacts");
    let total = loaded.total;

    context::ArtifactsPageContext {
        page: "artifacts",
        title: "Artifacts",
        breadcrumbs: vec![
            BreadcrumbView::link("AI activity", "/admin/analytics"),
            BreadcrumbView::current("Artifacts"),
        ],
        time_range: view::time_range_context(range, preset),
        filter_ribbon: context::ArtifactFilterRibbon {
            base_url: BASE_URL,
            preserved: view::build_preserved(query, range, preset),
            options: view::annotate_options(
                &loaded.options.users,
                filter.user_id.as_ref().map(UserId::as_str),
            ),
            chips: view::build_chips(query),
        },
        scope_filter,
        stats: context::stats_view(&loaded.stats),
        artifacts: artifact_rows,
        has_artifacts,
        total_count: total,
        count_label: format!("{total} artifacts"),
        pagination: view::build_pagination(query, window),
        structured_only: filter.structured_only,
        all_toggle_url: view::all_toggle_url(query, !filter.structured_only),
        errors_only: filter.errors_only,
        errors_toggle_url: view::errors_toggle_url(query, filter.errors_only),
        export: crate::export::ExportView::single(
            "artifacts",
            &crate::export::view::query_string(&[
                ("preset", Some(preset)),
                ("from", Some(&range.from.to_rfc3339())),
                ("to", Some(&range.to.to_rfc3339())),
                ("user_id", query.user_id.as_ref().map(UserId::as_str)),
                ("session", query.session.as_ref().map(SessionId::as_str)),
                ("tool", query.tool.as_deref()),
                ("source", query.source.as_deref()),
                ("skill", query.skill.as_deref()),
                ("errors", query.errors.as_deref()),
                ("all", query.all.as_deref()),
                ("group", query.group.as_deref()),
                ("project", query.project.as_deref()),
            ]),
        ),
    }
}

// Why: the export reads the rows the list shows — same scope and filter —
// through the page's own query type.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: ArtifactListQuery,
    range: TimeRange,
    limit: i64,
) -> crate::error::AdminResult<(
    Vec<crate::repositories::analytics::artifacts::ArtifactRow>,
    i64,
)> {
    let request =
        ScopeRequest::from_query(user_ctx, query.group.as_deref(), query.project.as_deref());
    let subjects =
        crate::repositories::scope::membership::get_subject_scope(pool, &request).await?;
    let filter = build_filter(&query, range, subjects.as_sql());
    Ok(list_artifacts(pool, &filter, limit, 0).await?)
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.filter(|s| !s.is_empty()).map(str::to_owned)
}

fn preset_str(query: &ArtifactListQuery, range: TimeRange) -> String {
    if let Some(p) = query.preset.as_deref()
        && !p.is_empty()
    {
        return p.to_owned();
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_owned();
    }
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_owned()
}
