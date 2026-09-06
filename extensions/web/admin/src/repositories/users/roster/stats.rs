//! The roster's headline numbers — the KPI row, and every filter chip's count.
//!
//! One statement rather than one per chip: the four populations are `FILTER`
//! clauses over the same scan, so the tiles and the table they filter can never
//! disagree about how many people there are.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::scope::SubjectScope;

use super::WINDOW_DAYS;

/// The five numbers the KPI row states, each one also a filter chip's count.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct RosterStats {
    pub total: i64,
    pub active: i64,
    pub unassigned: i64,
    pub no_role: i64,
    pub inactive_30d: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
    // Why: the previous window, so the two spend KPIs can carry a delta rather
    // than a bare number the reader has nothing to compare against.
    pub prior_cost_microdollars: i64,
    pub prior_requests: i64,
}

pub async fn get_roster_stats(
    pool: &PgPool,
    scope: &SubjectScope,
) -> Result<RosterStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH base AS (
               SELECT u.id, u.roles, (u.status = 'active') AS is_active
                 FROM users u
                WHERE NOT ('anonymous' = ANY(u.roles))
                  AND u.email NOT LIKE '%@anonymous.local'
                  AND ($1::TEXT[] IS NULL OR u.id = ANY($1))
           ), marked AS (
               SELECT b.id, b.is_active,
                      cardinality(array_remove(array_remove(b.roles, 'user'), 'anonymous')) = 0 AS no_role,
                      NOT EXISTS (SELECT 1 FROM group_members gm WHERE gm.user_id = b.id) AS unassigned,
                      (SELECT MAX(ar.created_at) FROM ai_requests ar WHERE ar.user_id = b.id) AS last_request,
                      (SELECT MAX(ua.created_at) FROM user_activity ua WHERE ua.user_id = b.id) AS last_seen
                 FROM base b
           ), spend AS (
               SELECT
                 COALESCE(SUM(ar.cost_microdollars) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT)), 0)::BIGINT AS cost,
                 COUNT(*) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT))::BIGINT AS requests,
                 COALESCE(SUM(ar.cost_microdollars) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT * 2)
                       AND ar.created_at <  NOW() - make_interval(days => $2::INT)), 0)::BIGINT AS prior_cost,
                 COUNT(*) FILTER (
                     WHERE ar.created_at >= NOW() - make_interval(days => $2::INT * 2)
                       AND ar.created_at <  NOW() - make_interval(days => $2::INT))::BIGINT AS prior_requests
               FROM ai_requests ar
               JOIN base b ON b.id = ar.user_id
           )
           SELECT COUNT(*)::BIGINT AS "total!",
                  COUNT(*) FILTER (WHERE m.is_active)::BIGINT AS "active!",
                  COUNT(*) FILTER (WHERE m.unassigned)::BIGINT AS "unassigned!",
                  COUNT(*) FILTER (WHERE m.no_role)::BIGINT AS "no_role!",
                  COUNT(*) FILTER (
                      WHERE GREATEST(m.last_request, m.last_seen) IS NULL
                         OR GREATEST(m.last_request, m.last_seen) < NOW() - INTERVAL '30 days'
                  )::BIGINT AS "inactive_30d!",
                  (SELECT cost FROM spend) AS "cost!",
                  (SELECT requests FROM spend) AS "requests!",
                  (SELECT prior_cost FROM spend) AS "prior_cost!",
                  (SELECT prior_requests FROM spend) AS "prior_requests!"
             FROM marked m"#,
        scope.as_sql(),
        WINDOW_DAYS,
    )
    .fetch_one(pool)
    .await?;

    Ok(RosterStats {
        total: row.total,
        active: row.active,
        unassigned: row.unassigned,
        no_role: row.no_role,
        inactive_30d: row.inactive_30d,
        requests: row.requests,
        cost_microdollars: row.cost,
        prior_cost_microdollars: row.prior_cost,
        prior_requests: row.prior_requests,
    })
}
