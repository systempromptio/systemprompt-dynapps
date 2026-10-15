//! Queries over `clients` and `agent_farm_run_reports`.
//!
//! The per-client rollups, the daily outcome series the trend charts draw,
//! per-agent outcomes, recent runs with their completion report, and one
//! run's report, identities and governance decisions. Every rollup takes an
//! optional client and an optional launching user, so the admin pages (all
//! runs) and the dashboard tool (a non-admin caller's own runs) read the same
//! numbers.

mod catalog;
mod rows;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use crate::types::{ClientId, FarmAgentId, FarmId, FarmRunId};

pub use catalog::{ClientRow, find_client, list_clients, upsert_client};
pub use rows::{
    AgentOutcomeRow, ClientSummaryRow, DailyOutcomeRow, ReportRunRow, RunDecisionRow,
    RunIdentityRow, RunReportRow,
};

#[derive(Debug, Clone, Copy)]
pub struct ReportScope<'a> {
    pub client: Option<&'a ClientId>,
    pub user: Option<&'a UserId>,
    pub since: DateTime<Utc>,
}

pub async fn list_client_summaries(
    pool: &PgPool,
    scope: ReportScope<'_>,
) -> Result<Vec<ClientSummaryRow>, sqlx::Error> {
    sqlx::query_file_as!(
        ClientSummaryRow,
        "src/repositories/clients/list_client_summaries.sql",
        scope.client.map(ClientId::as_str),
        scope.user.map(UserId::as_str),
        scope.since,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_client_summary(
    pool: &PgPool,
    scope: ReportScope<'_>,
) -> Result<Option<ClientSummaryRow>, sqlx::Error> {
    Ok(list_client_summaries(pool, scope).await?.into_iter().next())
}

pub async fn list_daily_outcomes(
    pool: &PgPool,
    scope: ReportScope<'_>,
) -> Result<Vec<DailyOutcomeRow>, sqlx::Error> {
    sqlx::query_file_as!(
        DailyOutcomeRow,
        "src/repositories/clients/list_daily_outcomes.sql",
        scope.client.map(ClientId::as_str),
        scope.user.map(UserId::as_str),
        scope.since,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_agent_outcomes(
    pool: &PgPool,
    scope: ReportScope<'_>,
) -> Result<Vec<AgentOutcomeRow>, sqlx::Error> {
    sqlx::query_file_as!(
        AgentOutcomeRow,
        "src/repositories/clients/list_agent_outcomes.sql",
        scope.client.map(ClientId::as_str),
        scope.user.map(UserId::as_str),
        scope.since,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_report_runs(
    pool: &PgPool,
    client: Option<&ClientId>,
    user: Option<&UserId>,
    limit: i64,
) -> Result<Vec<ReportRunRow>, sqlx::Error> {
    sqlx::query_file_as!(
        ReportRunRow,
        "src/repositories/clients/list_report_runs.sql",
        client.map(ClientId::as_str),
        user.map(UserId::as_str),
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_run_report(
    pool: &PgPool,
    run_id: &FarmRunId,
) -> Result<Option<RunReportRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunReportRow,
        "src/repositories/clients/find_run_report.sql",
        run_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_run_identity(
    pool: &PgPool,
    run_id: &FarmRunId,
) -> Result<Option<RunIdentityRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunIdentityRow,
        "src/repositories/clients/find_run_identity.sql",
        run_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_run_governance(
    pool: &PgPool,
    session_ids: &[String],
    context: Option<&ContextId>,
    limit: i64,
) -> Result<Vec<RunDecisionRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunDecisionRow,
        "src/repositories/clients/list_run_governance.sql",
        session_ids,
        context.map(ContextId::as_str),
        limit,
    )
    .fetch_all(pool)
    .await
}
