//! The paged, sorted, filtered user roster behind `/admin/users`.
//!
//! One statement answers the whole page: the row set, every row's group and
//! project membership, its thirty-day gateway spend, and the total the
//! pagination footer needs, so a fifty-row page is one round trip rather than
//! fifty-one.
//!
//! Sorting and filtering are bound parameters, never interpolated text. The
//! `ORDER BY` is a fixed ladder of `CASE` arms — one pair per sortable column —
//! so the statement stays a static string the `sqlx` macros verify against the
//! live schema, and no caller can reach the query planner with a column name.
//!
//! "Unassigned" is a filter here, not a group: the `user_groups` view already
//! synthesises an `unassigned` row for an account no `group_members` row
//! covers, so the filter matches that derived id as well as a genuinely empty
//! membership.

mod stats;

pub use stats::{RosterStats, get_roster_stats};

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use crate::repositories::scope::SubjectScope;

// Why: how many rows a roster page carries unless the reader asks for another
// size.
pub const DEFAULT_PAGE_SIZE: i64 = 50;

// Why: the window every cost and request figure on the roster is measured over.
pub const WINDOW_DAYS: i32 = 30;

/// One roster row: who they are, what they belong to, what they spent.
#[derive(Debug, Clone, Serialize)]
pub struct RosterRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
    pub roles: Vec<String>,
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    // Why: `None` is "never seen", not "seen when they joined" — the wasted-seat
    // question the roster exists to answer needs those two apart.
    pub last_active: Option<DateTime<Utc>>,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

/// The chips above the roster. Each one is a `WHERE` arm, not a saved search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterFilter {
    None,
    Unassigned,
    NoRole,
    Inactive30d,
}

impl RosterFilter {
    #[must_use]
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.unwrap_or_default() {
            "unassigned" => Self::Unassigned,
            "no-role" => Self::NoRole,
            "inactive-30d" => Self::Inactive30d,
            _ => Self::None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Unassigned => "unassigned",
            Self::NoRole => "no-role",
            Self::Inactive30d => "inactive-30d",
        }
    }
}

/// Which column the roster is ordered by. Parsed from `?sort=`, so an unknown
/// value falls back to the default rather than erroring the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RosterSort {
    pub column: &'static str,
    pub descending: bool,
}

impl RosterSort {
    // Why: every column the header row offers, in table order.
    pub const COLUMNS: [&'static str; 9] = [
        "name", "email", "roles", "groups", "projects", "seen", "cost", "requests", "status",
    ];

    #[must_use]
    pub fn parse(column: Option<&str>, direction: Option<&str>) -> Self {
        let column = column
            .and_then(|c| Self::COLUMNS.into_iter().find(|known| *known == c))
            .unwrap_or("cost");
        Self {
            column,
            descending: direction.unwrap_or("desc") != "asc",
        }
    }
}

impl Default for RosterSort {
    fn default() -> Self {
        Self {
            column: "cost",
            descending: true,
        }
    }
}

/// What the roster asks for: which people, narrowed how, ordered how, which
/// page.
#[derive(Debug, Clone)]
pub struct RosterQuery {
    pub filter: RosterFilter,
    pub role: Option<String>,
    pub search: Option<String>,
    pub sort: RosterSort,
    pub limit: i64,
    pub offset: i64,
}

// Why: The window function carries the filtered total on every row, so the
// count and the page come from one scan of one predicate — there is no second
// statement to drift out of step with the first.
pub async fn list_users_paged(
    pool: &PgPool,
    scope: &SubjectScope,
    query: &RosterQuery,
) -> Result<(Vec<RosterRow>, i64), sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH base AS (
               SELECT u.id,
                      COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                      u.email, u.roles, (u.status = 'active') AS is_active, u.created_at
                 FROM users u
                WHERE NOT ('anonymous' = ANY(u.roles))
                  AND u.email NOT LIKE '%@anonymous.local'
                  AND ($1::TEXT[] IS NULL OR u.id = ANY($1))
           ), enriched AS (
               SELECT b.id, b.display_name, b.email, b.roles, b.is_active, b.created_at,
                      COALESCE(g.ids, ARRAY[]::TEXT[]) AS group_ids,
                      COALESCE(p.ids, ARRAY[]::TEXT[]) AS project_ids,
                      COALESCE(r.requests, 0)::BIGINT AS requests,
                      COALESCE(r.cost, 0)::BIGINT AS cost_microdollars,
                      COALESCE(r.tokens, 0)::BIGINT AS tokens,
                      GREATEST(r.last_request, a.last_seen) AS last_active
                 FROM base b
                 LEFT JOIN LATERAL (
                     SELECT array_agg(DISTINCT ug.group_id) AS ids
                       FROM user_groups ug WHERE ug.user_id = b.id) g ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT array_agg(DISTINCT pm.project_id) AS ids
                       FROM project_members pm WHERE pm.user_id = b.id) p ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT COUNT(*)::BIGINT AS requests,
                            COALESCE(SUM(ar.cost_microdollars), 0)::BIGINT AS cost,
                            COALESCE(SUM(COALESCE(ar.tokens_used,
                                COALESCE(ar.input_tokens, 0) + COALESCE(ar.output_tokens, 0))), 0)::BIGINT AS tokens,
                            MAX(ar.created_at) AS last_request
                       FROM ai_requests ar
                      WHERE ar.user_id = b.id
                        AND ar.created_at >= NOW() - make_interval(days => $2::INT)) r ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT MAX(ua.created_at) AS last_seen
                       FROM user_activity ua WHERE ua.user_id = b.id) a ON TRUE
           ), filtered AS (
               SELECT * FROM enriched e
                WHERE CASE $3::TEXT
                        WHEN 'unassigned' THEN
                            cardinality(e.group_ids) = 0 OR 'unassigned' = ANY(e.group_ids)
                        WHEN 'no-role' THEN
                            cardinality(array_remove(array_remove(e.roles, 'user'), 'anonymous')) = 0
                        WHEN 'inactive-30d' THEN
                            e.last_active IS NULL OR e.last_active < NOW() - INTERVAL '30 days'
                        ELSE TRUE
                      END
                  AND ($4::TEXT IS NULL OR $4 = ANY(e.roles))
                  AND ($5::TEXT IS NULL OR position(lower($5) in lower(
                          COALESCE(e.display_name, '') || ' ' || COALESCE(e.email, '') || ' ' || e.id)) > 0)
           )
           SELECT f.id AS "user_id!: UserId",
                  f.display_name,
                  f.email AS "email?: Email",
                  f.roles AS "roles!: Vec<String>",
                  f.group_ids AS "group_ids!: Vec<String>",
                  f.project_ids AS "project_ids!: Vec<String>",
                  f.is_active AS "is_active!",
                  f.created_at AS "created_at!",
                  f.last_active,
                  f.requests AS "requests!",
                  f.tokens AS "tokens!",
                  f.cost_microdollars AS "cost_microdollars!",
                  COUNT(*) OVER ()::BIGINT AS "total_rows!"
             FROM filtered f
            ORDER BY
              CASE WHEN $6::TEXT = 'name'     AND     $7::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END DESC,
              CASE WHEN $6::TEXT = 'name'     AND NOT $7::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END ASC,
              CASE WHEN $6::TEXT = 'email'    AND     $7::BOOLEAN THEN lower(COALESCE(f.email, '')) END DESC,
              CASE WHEN $6::TEXT = 'email'    AND NOT $7::BOOLEAN THEN lower(COALESCE(f.email, '')) END ASC,
              CASE WHEN $6::TEXT = 'roles'    AND     $7::BOOLEAN THEN cardinality(f.roles) END DESC,
              CASE WHEN $6::TEXT = 'roles'    AND NOT $7::BOOLEAN THEN cardinality(f.roles) END ASC,
              CASE WHEN $6::TEXT = 'groups'   AND     $7::BOOLEAN THEN cardinality(f.group_ids) END DESC,
              CASE WHEN $6::TEXT = 'groups'   AND NOT $7::BOOLEAN THEN cardinality(f.group_ids) END ASC,
              CASE WHEN $6::TEXT = 'projects' AND     $7::BOOLEAN THEN cardinality(f.project_ids) END DESC,
              CASE WHEN $6::TEXT = 'projects' AND NOT $7::BOOLEAN THEN cardinality(f.project_ids) END ASC,
              CASE WHEN $6::TEXT = 'seen'     AND     $7::BOOLEAN THEN f.last_active END DESC NULLS LAST,
              CASE WHEN $6::TEXT = 'seen'     AND NOT $7::BOOLEAN THEN f.last_active END ASC NULLS FIRST,
              CASE WHEN $6::TEXT = 'cost'     AND     $7::BOOLEAN THEN f.cost_microdollars END DESC,
              CASE WHEN $6::TEXT = 'cost'     AND NOT $7::BOOLEAN THEN f.cost_microdollars END ASC,
              CASE WHEN $6::TEXT = 'requests' AND     $7::BOOLEAN THEN f.requests END DESC,
              CASE WHEN $6::TEXT = 'requests' AND NOT $7::BOOLEAN THEN f.requests END ASC,
              CASE WHEN $6::TEXT = 'status'   AND     $7::BOOLEAN THEN f.is_active::INT END DESC,
              CASE WHEN $6::TEXT = 'status'   AND NOT $7::BOOLEAN THEN f.is_active::INT END ASC,
              f.id
            LIMIT $8 OFFSET $9"#,
        scope.as_sql(),
        WINDOW_DAYS,
        query.filter.as_str(),
        query.role.as_deref(),
        query.search.as_deref(),
        query.sort.column,
        query.sort.descending,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |row| row.total_rows);
    Ok((
        rows.into_iter()
            .map(|row| RosterRow {
                user_id: row.user_id,
                display_name: row.display_name,
                email: row.email,
                roles: row.roles,
                group_ids: row.group_ids,
                project_ids: row.project_ids,
                is_active: row.is_active,
                created_at: row.created_at,
                last_active: row.last_active,
                requests: row.requests,
                tokens: row.tokens,
                cost_microdollars: row.cost_microdollars,
            })
            .collect(),
        total,
    ))
}
