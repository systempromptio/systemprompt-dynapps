//! Top containers by spend, under exclusive attribution.
//!
//! Exclusive is not a default that happened: it is what makes these two tables
//! readable side by side. Each request lands in exactly one group and one
//! project — the ones stamped on it when it was made — so the rows partition
//! the instance and the leaderboard cannot show a total larger than the KPI
//! strip above it.
//!
//! Traffic that matches no container is reported as its own row rather than
//! dropped — a rejected request with no user, a job or MCP actor, or a person
//! no primary container covers. Dropping it would make the leaderboard quietly
//! disagree with the cost KPI.

use sqlx::PgPool;

use systemprompt_web_admin_core::repositories::scope::membership::UNATTRIBUTED;
use systemprompt_web_admin_core::repositories::scope::{Attribution, ScopeKind};
use systemprompt_web_admin_core::util::time_range::TimeRange;

/// One container's slice of the window, or the unattributed remainder.
#[derive(Debug, Clone)]
pub struct ScopeCostRow {
    pub scope_id: String,
    pub label: String,
    pub requests: i64,
    pub cost_microdollars: i64,
}

impl ScopeCostRow {
    #[must_use]
    pub fn is_unattributed(&self) -> bool {
        self.scope_id == UNATTRIBUTED
    }
}

pub async fn list_top_scopes_by_cost(
    pool: &PgPool,
    kind: ScopeKind,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<ScopeCostRow>, sqlx::Error> {
    let exclusive = Attribution::Exclusive.is_exclusive();
    let rows = systemprompt_web_admin_core::scoped_query!(
        r#"SELECT COALESCE(rs.scope_id, $3) AS "scope_id!",
                  COALESCE(g.name, p.name, rs.scope_id, $3) AS "label!",
                  COUNT(*)::BIGINT AS "requests!",
                  COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
           FROM ai_requests r
           LEFT JOIN request_scope rs ON rs.request_id = r.id
           LEFT JOIN groups g ON g.id = rs.scope_id AND $1 = 'group'
           LEFT JOIN projects p ON p.id = rs.scope_id AND $1 = 'project'
           WHERE NOT r.synthetic
             AND r.created_at >= $4
             AND r.created_at < $5
           GROUP BY 1, 2
           ORDER BY 4 DESC, 3 DESC, 1
           LIMIT $6"#,
        kind.as_str(),
        exclusive,
        UNATTRIBUTED,
        range.from,
        range.to,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ScopeCostRow {
            scope_id: row.scope_id,
            label: row.label,
            requests: row.requests,
            cost_microdollars: row.cost_microdollars,
        })
        .collect())
}
