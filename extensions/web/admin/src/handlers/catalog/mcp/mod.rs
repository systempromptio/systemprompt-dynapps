//! `/admin/mcp` and `/admin/mcp/{id}` — the MCP servers, what they are serving,
//! and who is on them.
//!
//! These are the only admin pages that read the MCP runtime tables, so they are
//! the only place an operator can see that a declared server has never
//! connected, or that traffic is arriving under a name nothing declares. Both
//! states are silent everywhere else.

mod detail;
mod rows;
mod view;

pub use rows::status_of;

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt_security::authz::{AccessControlRepository, EntityKind};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::repositories::mcp::runtime;
use crate::templates::AdminTemplateEngine;
use crate::types::{ENTITY_MCP_SERVER, MarketplaceContext, UserContext};

use super::super::ssr::ssr_helpers::render_typed_page;
use super::sorting::{direction, matches, preserved_search, sort_headers};
use super::view::assignment_counts_by_type;
use rows::{BASE_URL, Runtime, RowInputs, WINDOW_HOURS};
use view::{McpDetailData, McpPageData};

const EXECUTIONS_PAGE_SIZE: i64 = 50;
const TOOL_LIMIT: i64 = 50;
const SESSION_LIMIT: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct McpListQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    // Why: filtering is a server round trip, not a class toggle — the count in
    // the toolbar, the empty state and the URL then cannot disagree with the
    // rows on screen.
    pub q: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct McpDetailQuery {
    pub page: Option<i64>,
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

// Why: both reads are best-effort. A runtime table that will not answer must
// not take the page down with it — the declaration alone is still worth
// rendering, and the status column says "never connected" rather than lying.
async fn load_runtime(pool: &PgPool) -> Runtime {
    let liveness = runtime::list_mcp_server_liveness(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: liveness read failed"))
        .unwrap_or_default();
    let activity = runtime::list_mcp_server_activity(pool, WINDOW_HOURS)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: activity read failed"))
        .unwrap_or_default();
    Runtime::new(liveness, activity)
}

pub(crate) async fn mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<McpListQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = super::data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_MCP_SERVER).await;
    let rt = load_runtime(&pool).await;

    let mut ids: Vec<String> = catalog
        .mcp
        .iter()
        .map(|s| s.id.as_str().to_owned())
        .collect();
    for name in rt.names() {
        if !ids.contains(&name) {
            ids.push(name);
        }
    }

    let mut servers: Vec<view::McpServerRow> = ids
        .iter()
        .map(|id| {
            let server = catalog.mcp.iter().find(|s| s.id.as_str() == id);
            rows::build_row(&RowInputs {
                id,
                server,
                runtime: &rt,
                plugin_count: catalog.plugins_by_mcp.get(id).map_or(0, Vec::len),
                assignment_count: counts.get(id).copied().unwrap_or(0),
            })
        })
        .collect();

    let search = query.q.unwrap_or_default();
    let kpis = rows::kpis(&servers);
    servers.retain(|s| matches(&[&s.id, &s.description, &s.server_type], &search));
    let sort_key = query.sort.unwrap_or_else(|| "calls".to_owned());
    let sort_dir = direction(query.dir.as_deref());
    rows::sort_rows(&mut servers, &sort_key, sort_dir);

    let unconfigured_count = servers.iter().filter(|s| !s.configured).count();
    let page = McpPageData {
        page: "mcp",
        title: "MCP servers",
        subtitle: "Every tool server this instance declares, what it is serving right now, and who may reach it.",
        breadcrumbs: vec![BreadcrumbView::current("MCP servers")],
        window_label: "last 24 hours",
        heartbeat_label: format!(
            "alive = a session spoke within {} minutes",
            runtime::HEARTBEAT_INTERVAL_MINUTES * 2
        ),
        // Why: the tiles count the fleet, not the filtered view. A filter that
        // moved "alive now" would make the number mean two different things
        // depending on what was typed in the box above it.
        kpis,
        sort_headers: sort_headers(
            BASE_URL,
            &rows::columns(),
            &sort_key,
            sort_dir,
            &preserved_search(&search),
        ),
        servers_count: servers.len(),
        unconfigured_count,
        servers,
        access_control_url: "/admin/access-control?entity_type=mcp_server",
        sort_key,
        sort_dir: sort_dir.to_owned(),
        search,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-mcp",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn build_pagination(page_index: i64, window: PageWindow, base: &str) -> Pagination {
    let (first_row, last_row) = window.bounds();
    let prev_url = (page_index > 0).then(|| format!("{base}?page={}", page_index - 1));
    let next_url =
        (page_index + 1 < window.total_pages).then(|| format!("{base}?page={}", page_index + 1));
    Pagination {
        current_page: page_index + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

pub(crate) async fn mcp_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(mcp_id): Path<String>,
    Query(query): Query<McpDetailQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = super::data::load_catalog(&path, &user_ctx.roles);
    let server = catalog.mcp.iter().find(|s| s.id.as_str() == mcp_id);
    let rt = load_runtime(&pool).await;
    let known_at_runtime =
        rt.liveness.contains_key(&mcp_id) || rt.activity.contains_key(&mcp_id);

    // Why: a server the catalog does not declare but the runtime has served is
    // a real page, because the list links to it. Only a name neither half knows
    // is a 404.
    if server.is_none() && !known_at_runtime {
        return Err(AdminError::NotFound("No such MCP server.".to_owned()).into());
    }

    let counts = assignment_counts_by_type(&pool, ENTITY_MCP_SERVER).await;
    let row = rows::build_row(&RowInputs {
        id: &mcp_id,
        server,
        runtime: &rt,
        plugin_count: catalog.plugins_by_mcp.get(&mcp_id).map_or(0, Vec::len),
        assignment_count: counts.get(&mcp_id).copied().unwrap_or(0),
    });

    let page_index = query.page.unwrap_or(0).max(0);
    let (executions, total) = runtime::list_mcp_executions_paged(
        &pool,
        &mcp_id,
        EXECUTIONS_PAGE_SIZE,
        page_index * EXECUTIONS_PAGE_SIZE,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "mcp: execution log read failed"))
    .unwrap_or_default();
    let shown = i64::try_from(executions.len()).unwrap_or(0);

    let tools = runtime::list_mcp_tool_stats(&pool, &mcp_id, WINDOW_HOURS, TOOL_LIMIT)
        .await
        .unwrap_or_default();
    let sessions = runtime::list_mcp_sessions_for_server(&pool, &mcp_id, SESSION_LIMIT)
        .await
        .unwrap_or_default();
    let rules = repositories::users::access_control::list_rules_for_entity(
        &pool,
        ENTITY_MCP_SERVER,
        &mcp_id,
    )
    .await
    .unwrap_or_default();
    let default_included = AccessControlRepository::from_pool(Arc::clone(&pool))
        .get_entity(EntityKind::McpServer, &mcp_id)
        .await
        .unwrap_or_default()
        .is_some_and(|e| e.default_included);

    let tools = detail::tool_rows(tools);
    let executions = detail::execution_rows(executions);
    let sessions = detail::session_rows(sessions);
    let grants = detail::grant_rows(rules);
    let detail_base = format!("{BASE_URL}/{mcp_id}");

    let page = McpDetailData {
        page: "mcp",
        title: mcp_id.clone(),
        subtitle: row.description.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("MCP servers", BASE_URL),
            BreadcrumbView::current(mcp_id.clone()),
        ],
        configured: row.configured,
        enabled: row.enabled,
        status_label: row.status_label,
        status_tone: row.status_tone,
        window_label: "last 24 hours",
        kpis: rows::kpis(std::slice::from_ref(&row)),
        tools_count: tools.len(),
        tools,
        executions_count: total,
        pagination: build_pagination(
            page_index,
            PageWindow::new(page_index, EXECUTIONS_PAGE_SIZE, total, shown, "calls"),
            &detail_base,
        ),
        executions,
        sessions_count: sessions.len(),
        sessions,
        grants_count: grants.len(),
        grants,
        default_included,
        config_facts: detail::config_facts(server),
        oauth_scopes: server.map(|s| s.oauth_scopes.clone()).unwrap_or_default(),
        included_by_count: catalog.plugins_by_mcp.get(&mcp_id).map_or(0, Vec::len),
        included_by: catalog.plugins_by_mcp.get(&mcp_id).cloned().unwrap_or_default(),
        matrix_url: row.matrix_url.clone(),
        access_control_url: "/admin/access-control",
        id: mcp_id,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-mcp-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
