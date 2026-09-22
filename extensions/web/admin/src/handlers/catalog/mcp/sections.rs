//! The reads behind everything under the header on `/admin/mcp/{id}`.
//!
//! Split out of the handler because six loosely related lists and their
//! pagination is most of what the page does, and a handler that also owns the
//! request flow stops being readable at that size. Every windowed read takes
//! the page's range; the connections list is "now", not a window.

use std::sync::Arc;
use systemprompt::identifiers::McpServerId;

use sqlx::PgPool;
use systemprompt_security::authz::{AccessControlRepository, EntityKind};

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories;
use crate::repositories::mcp::runtime;
use crate::types::ENTITY_MCP_SERVER;
use crate::util::time_range::TimeRange;

use super::rows::BASE_URL;
use super::{detail, view};

const EXECUTIONS_PAGE_SIZE: i64 = 50;
const TOOL_LIMIT: i64 = 50;
const CALLER_LIMIT: i64 = 25;
const CONNECTION_LIMIT: i64 = 50;

// Why: the page links carry the window, or turning a page would silently
// reset the log to the default 24 hours.
fn build_pagination(
    page_index: i64,
    window: PageWindow,
    base: &str,
    window_query: &str,
) -> Pagination {
    let (first_row, last_row) = window.bounds();
    let suffix = if window_query.is_empty() {
        String::new()
    } else {
        format!("&{window_query}")
    };
    let prev_url = (page_index > 0).then(|| format!("{base}?page={}{suffix}", page_index - 1));
    let next_url = (page_index + 1 < window.total_pages)
        .then(|| format!("{base}?page={}{suffix}", page_index + 1));
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

// Why: everything below the header on the detail page, in one read. It is a
// struct rather than a tuple because seven loosely related lists returned
// positionally is how the wrong one ends up rendered in the wrong section.
pub(super) struct DetailSections {
    pub tools: Vec<view::McpToolRow>,
    pub callers: Vec<view::McpCallerRowView>,
    pub executions: Vec<view::McpExecutionRowView>,
    pub executions_total: i64,
    pub pagination: Pagination,
    pub connections: Vec<view::McpConnectionRowView>,
    pub grants: Vec<view::McpGrantRow>,
    pub default_included: bool,
}

// Why: every read is best-effort for the same reason the fleet reads are — a
// section that will not load renders its own empty state rather than taking
// the page down with it.
pub(super) async fn detail_sections(
    pool: &Arc<PgPool>,
    mcp_id: &McpServerId,
    range: TimeRange,
    page_index: i64,
    window_query: &str,
) -> DetailSections {
    let (executions, executions_total) = runtime::list_mcp_executions_paged(
        pool,
        Some(mcp_id.as_str()),
        range.from,
        range.to,
        EXECUTIONS_PAGE_SIZE,
        page_index * EXECUTIONS_PAGE_SIZE,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "mcp: execution log read failed"))
    .unwrap_or_default();
    let shown = i64::try_from(executions.len()).unwrap_or(0);

    let tools = runtime::list_mcp_tool_stats(
        pool,
        Some(mcp_id.as_str()),
        range.from,
        range.to,
        TOOL_LIMIT,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "mcp: tool stats read failed"))
    .unwrap_or_default();
    let callers =
        runtime::list_mcp_callers(pool, mcp_id.as_str(), range.from, range.to, CALLER_LIMIT)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "mcp: caller read failed"))
            .unwrap_or_default();
    let connections =
        runtime::list_mcp_connections_for_server(pool, mcp_id.as_str(), CONNECTION_LIMIT)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "mcp: connection read failed"))
            .unwrap_or_default();
    let rules = repositories::users::access_control::list_rules_for_entity(
        pool,
        ENTITY_MCP_SERVER,
        mcp_id.as_str(),
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "mcp: grant read failed"))
    .unwrap_or_default();
    let default_included = AccessControlRepository::from_pool(Arc::clone(pool))
        .get_entity(EntityKind::McpServer, mcp_id.as_str())
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: entity access read failed"))
        .unwrap_or_default()
        .is_some_and(|e| e.default_included);

    DetailSections {
        pagination: build_pagination(
            page_index,
            PageWindow::new(
                page_index,
                EXECUTIONS_PAGE_SIZE,
                executions_total,
                shown,
                "calls",
            ),
            &format!("{BASE_URL}/{mcp_id}"),
            window_query,
        ),
        tools: detail::tool_rows(tools),
        callers: detail::caller_rows(callers),
        executions: detail::execution_rows(executions),
        executions_total,
        connections: detail::connection_rows(connections),
        grants: detail::grant_rows(rules),
        default_included,
    }
}
