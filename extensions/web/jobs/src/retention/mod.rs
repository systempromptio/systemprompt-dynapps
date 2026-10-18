//! The data-lifecycle jobs: what the instance measures, archives and checks
//! about its own database, on a daily, weekly and monthly cadence.
//!
//! Deletion itself is core's `database_cleanup` (windows from
//! `profile.retention`) and `expire_raw_evidence` (90-day raw evidence).
//! These jobs are the other half of a retention policy: the daily job
//! records size and age per managed table into `retention_runs` so growth is
//! a series rather than a guess; the weekly job archives the raw request
//! tables for the previous ISO week to `exports/weekly/` in core's file
//! storage; the monthly
//! job archives the rollups that are kept indefinitely, runs the health
//! check and vacuums the managed tables. Every archive is one gzipped JSON
//! Lines file per table, streamed from `row_to_json`, so it is complete and
//! typed. A table with no `window_days` is kept indefinitely or bounded by
//! something other than age; the health check judges only dated windows.
//!
//! The SQL here is dynamic by nature — export `SELECT`s, `VACUUM`, catalog
//! lookups over a static table registry — so this module is the one
//! allow-listed path in `scripts/check-sqlx.sh`; no statement takes user input.

mod archive;
mod daily_report;
mod export_monthly;
mod export_weekly;
mod health;
mod ledger;

pub use daily_report::RetentionDailyReportJob;
pub use export_monthly::{MonthlyParams, RetentionExportMonthlyJob};
pub use export_weekly::{RetentionExportWeeklyJob, WeeklyParams};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ManagedTable {
    pub name: &'static str,
    pub time_column: &'static str,
    pub window_days: Option<i32>,
}

const fn managed(
    name: &'static str,
    time_column: &'static str,
    window_days: Option<i32>,
) -> ManagedTable {
    ManagedTable {
        name,
        time_column,
        window_days,
    }
}

pub(crate) const RAW_TABLES: &[ManagedTable] = &[
    managed("ai_requests", "created_at", Some(90)),
    managed("ai_request_messages", "created_at", Some(30)),
    managed("ai_request_payloads", "created_at", Some(7)),
    managed("ai_request_tool_calls", "created_at", Some(90)),
    managed("ai_request_scopes", "resolved_at", Some(90)),
    managed("ai_safety_findings", "created_at", Some(90)),
    managed("governance_decisions", "created_at", Some(180)),
    managed("mcp_tool_executions", "started_at", Some(180)),
    managed("plugin_usage_events", "created_at", Some(90)),
    managed("user_sessions", "started_at", Some(180)),
    managed("logs", "timestamp", Some(14)),
    managed("analytics_events", "timestamp", Some(30)),
    managed("engagement_events", "created_at", Some(90)),
];

pub(crate) const ROLLUP_TABLES: &[ManagedTable] = &[
    managed("admin_usage_daily_rollups", "date", None),
    managed("plugin_usage_daily", "date", None),
    managed("conversation_facts", "last_at", None),
    managed("conversation_skill_facts", "first_invoked_at", None),
    managed("usage_anomalies", "detected_at", None),
    managed("content_performance_metrics", "updated_at", None),
    managed("retention_runs", "run_at", None),
];

pub(crate) const TRANSIENT_TABLES: &[ManagedTable] = &[
    managed("event_outbox", "created_at", None),
    managed("ai_quota_buckets", "window_start", None),
    managed("user_rate_limit_buckets", "window_start", None),
];

pub(crate) fn all_managed() -> impl Iterator<Item = &'static ManagedTable> {
    RAW_TABLES
        .iter()
        .chain(ROLLUP_TABLES.iter())
        .chain(TRANSIENT_TABLES.iter())
}
