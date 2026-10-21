//! The member-key share of the request log's totals.
//!
//! `ai_requests.billing_source = 'user_key'` marks a request a member paid for
//! with their own subscription key. Its cost is a list-price estimate, so the
//! KPI strip subtracts it from platform spend and shows it on its own tile.
//! The agent and tool filters are answered from the governance and tool-call
//! ledgers, which this query does not read; under either filter the split is
//! not computed and the strip shows the unsplit total.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::RequestFilter;
use systemprompt_web_admin_core::util::time_range::TimeRange;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BillingSplit {
    pub user_key_cost_microdollars: i64,
    pub user_key_requests: i64,
}

pub async fn find_request_billing_split(
    pool: &PgPool,
    filter: &RequestFilter,
    range: TimeRange,
) -> Result<Option<BillingSplit>, sqlx::Error> {
    if filter.agent_id.is_some() || filter.tool.is_some() {
        return Ok(None);
    }
    let search_pattern = filter
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));
    let row = sqlx::query_file!(
        "src/repositories/analytics/requests/billing.sql",
        range.from,
        range.to,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.model.as_deref(),
        filter.provider.as_deref(),
        filter.status.as_deref(),
        search_pattern,
        filter.scope.as_sql(),
        filter.group.as_deref(),
        filter.project.as_deref(),
    )
    .fetch_one(pool)
    .await?;
    Ok(Some(BillingSplit {
        user_key_cost_microdollars: row.user_key_cost_microdollars,
        user_key_requests: row.user_key_requests,
    }))
}
