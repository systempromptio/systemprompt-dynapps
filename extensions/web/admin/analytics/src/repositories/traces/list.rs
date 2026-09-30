//! Trace list query: one aggregated summary row per session in the window.

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};

use super::list_row::TraceListRow;
use super::{TraceFilter, TraceSort, TraceSummary};
use systemprompt_web_admin_core::util::time_range::TimeRange;

#[derive(Debug, Clone, Copy)]
pub struct TracePage {
    pub sort: TraceSort,
    pub limit: i64,
    pub offset: i64,
}

// Why: sqlx `query_file_as!` checks one fixed statement at compile time, so
// each `(column, dir)` is bound as text and picked by a `CASE` in `ORDER BY`
// rather than interpolated into the SQL.
pub async fn list_traces(
    pool: &PgPool,
    filter: TraceFilter<'_>,
    range: TimeRange,
    page: TracePage,
) -> Result<(Vec<TraceSummary>, i64), sqlx::Error> {
    let TracePage {
        sort,
        limit,
        offset,
    } = page;
    let sort_col = sort.column.sql_key();
    let sort_dir = sort.dir.sql_key();

    let rows = sqlx::query_file_as!(
        TraceListRow,
        "src/repositories/traces/list.sql",
        range.from,
        range.to,
        filter.user_id,
        filter.agent_id,
        filter.agent_scope,
        filter.policy,
        filter.decision,
        filter.error_only,
        filter.deny_only,
        limit,
        offset,
        sort_col,
        sort_dir,
        filter.subject_ids,
        filter.include_lifecycle,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let summaries = rows.into_iter().map(TraceSummary::from).collect();
    Ok((summaries, total))
}
