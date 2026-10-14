//! `get_agent_run_stats`: the scope's rollup over `agent_runs`, total and per
//! origin kind, from the one statement in `stats.sql`.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::query::ScopeParams;
use super::rows::{
    ErrorClassCounts, OriginKindStats, RunFilter, RunScope, RunStats, RunStatsBucket, StatusCounts,
};

pub async fn get_agent_run_stats(
    pool: &PgPool,
    scope: &RunScope,
    filter: &RunFilter,
) -> Result<RunStats, sqlx::Error> {
    let params = ScopeParams::from(scope);
    let rows = sqlx::query_file!(
        "src/repositories/agent_run_facts/stats.sql",
        params.user_id.as_ref().map(UserId::as_str),
        params.project_ids.as_deref(),
        filter.origin_kind,
        filter.origin_id,
        filter.status,
        filter.since,
        filter.until,
        filter.farm_id,
    )
    .fetch_all(pool)
    .await?;

    let mut stats = RunStats::default();
    for row in rows {
        let bucket = RunStatsBucket {
            total: row.total,
            by_status: StatusCounts {
                queued: row.queued,
                preparing: row.preparing,
                running: row.running,
                succeeded: row.succeeded,
                failed: row.failed,
                cancelled: row.cancelled,
                timed_out: row.timed_out,
            },
            by_error_class: ErrorClassCounts {
                ok: row.err_ok,
                budget_exceeded: row.err_budget_exceeded,
                timeout: row.err_timeout,
                output_invalid: row.err_output_invalid,
                hook_denied: row.err_hook_denied,
                host_crashed: row.err_host_crashed,
                connector_unavailable: row.err_connector_unavailable,
            },
            cost_microdollars_reported: row.cost_microdollars_reported,
            cost_microdollars_metered: row.cost_microdollars_metered,
            p50_duration_ms: row.p50_duration_ms,
            p95_duration_ms: row.p95_duration_ms,
            tool_use_count: row.tool_use_count,
            hook_deny: row.hook_deny,
        };
        match (row.is_total, row.origin_kind) {
            (1, _) => stats.totals = bucket,
            (_, Some(origin_kind)) => stats.by_origin_kind.push(OriginKindStats {
                origin_kind,
                stats: bucket,
            }),
            (_, None) => {},
        }
    }
    Ok(stats)
}
