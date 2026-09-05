//! Gateway usage rolled up per people container — a group or a project.
//!
//! Groups and projects differ in exactly one thing here: which relation names
//! the people in them. That difference is a bound parameter rather than an
//! interpolated table name, so every statement below is one static string the
//! `sqlx` macros verify against the live schema. The membership CTE selects
//! from both relations and discards the branch the caller did not ask for.
//!
//! Group membership reads the `user_groups` view, so the derived `unassigned`
//! bucket rolls up exactly like a real group. Project membership is read
//! `DISTINCT`, because a person the directory *and* an operator both placed in
//! a project holds two rows and would otherwise be counted twice.
//!
//! `tokens` is the provider's own count when it reported one; the four
//! component columns are the fallback for rows written before it existed.

pub mod breakdown;

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

// Why: The window every page and endpoint defaults to.
pub const DEFAULT_WINDOW_DAYS: i32 = 30;

// Why: How many rows a leaderboard carries. Ten is what the screens draw.
pub const LEADERBOARD_LIMIT: i64 = 10;

/// Which people container a usage query is asked of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Group,
    Project,
}

impl Scope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Project => "project",
        }
    }

    // Why: the overlap question is symmetric — a group's projects and a
    // project's groups are the same join read from either side.
    pub const fn linked(self) -> Self {
        match self {
            Self::Group => Self::Project,
            Self::Project => Self::Group,
        }
    }
}

/// One container's thirty-day totals, keyed back to the container.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScopeUsageRow {
    pub scope_id: String,
    pub active_members: i64,
    pub requests: i64,
    pub tokens: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_microdollars: i64,
}

/// What one member of a container spent inside it.
#[derive(Debug, Clone)]
pub struct MemberUsageRow {
    pub user_id: UserId,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub last_active: Option<DateTime<Utc>>,
}

/// One day's request and cost total, gap-filled so a chart's spine is the
/// calendar rather than the days that happened to carry traffic.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DailyRequests {
    pub day: NaiveDate,
    pub requests: i64,
    pub cost_microdollars: i64,
}

pub async fn get_scope_usage(
    pool: &PgPool,
    scope: Scope,
    id: &str,
    window_days: i32,
) -> Result<ScopeUsageRow, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           )
           SELECT COUNT(r.id)::BIGINT AS "requests!",
                  COUNT(DISTINCT r.user_id)::BIGINT AS "active_members!",
                  COALESCE(SUM(COALESCE(r.tokens_used, COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0) + COALESCE(r.cache_read_tokens, 0) + COALESCE(r.cache_creation_tokens, 0))), 0)::BIGINT AS "tokens!",
                  COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "tokens_in!",
                  COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "tokens_out!",
                  COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
           FROM membership m
           LEFT JOIN ai_requests r
             ON r.user_id = m.user_id
            AND r.created_at >= NOW() - make_interval(days => $3)
           WHERE m.scope_id = $2"#,
        scope.as_str(),
        id,
        window_days
    )
    .fetch_one(pool)
    .await?;
    Ok(ScopeUsageRow {
        scope_id: id.to_owned(),
        active_members: row.active_members,
        requests: row.requests,
        tokens: row.tokens,
        tokens_in: row.tokens_in,
        tokens_out: row.tokens_out,
        cost_microdollars: row.cost_microdollars,
    })
}

pub async fn list_member_usage(
    pool: &PgPool,
    scope: Scope,
    id: &str,
    window_days: i32,
) -> Result<HashMap<String, MemberUsageRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           )
           SELECT m.user_id AS "user_id!: UserId",
                  COUNT(r.id)::BIGINT AS "requests!",
                  COALESCE(SUM(COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0)), 0)::BIGINT AS "tokens!",
                  COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!",
                  MAX(r.created_at) AS "last_active?"
           FROM membership m
           LEFT JOIN ai_requests r
             ON r.user_id = m.user_id
            AND r.created_at >= NOW() - make_interval(days => $3)
           WHERE m.scope_id = $2
           GROUP BY m.user_id"#,
        scope.as_str(),
        id,
        window_days
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.user_id.as_str().to_owned(),
                MemberUsageRow {
                    user_id: row.user_id,
                    requests: row.requests,
                    tokens: row.tokens,
                    cost_microdollars: row.cost_microdollars,
                    last_active: row.last_active,
                },
            )
        })
        .collect())
}

pub async fn list_daily_requests(
    pool: &PgPool,
    scope: Scope,
    id: &str,
    window_days: i32,
) -> Result<Vec<DailyRequests>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           ), days AS (
               SELECT generate_series(
                   (NOW() - make_interval(days => $3 - 1))::date,
                   NOW()::date,
                   INTERVAL '1 day'
               )::date AS day
           ), counts AS (
               SELECT r.created_at::date AS day,
                      COUNT(*)::BIGINT AS requests,
                      COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost_microdollars
               FROM membership m
               JOIN ai_requests r ON r.user_id = m.user_id
               WHERE m.scope_id = $2
                 AND r.created_at >= NOW() - make_interval(days => $3)
               GROUP BY 1
           )
           SELECT days.day AS "day!",
                  COALESCE(counts.requests, 0)::BIGINT AS "requests!",
                  COALESCE(counts.cost_microdollars, 0)::BIGINT AS "cost_microdollars!"
           FROM days LEFT JOIN counts ON counts.day = days.day
           ORDER BY days.day"#,
        scope.as_str(),
        id,
        window_days
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| DailyRequests {
            day: row.day,
            requests: row.requests,
            cost_microdollars: row.cost_microdollars,
        })
        .collect())
}
