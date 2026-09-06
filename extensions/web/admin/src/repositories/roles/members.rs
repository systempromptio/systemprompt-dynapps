//! Who holds each role, and how much they used the platform holding it.
//!
//! One row per (role, user) pair, so a person with three roles appears three
//! times. The manual flag is the same distinction `users::roles` draws: a
//! grant with a `user_manual_roles` row was made by hand and can be revoked
//! here, and one without comes from the directory and cannot.

use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

/// One holder of one role.
#[derive(Debug, Clone)]
pub struct RoleMemberRow {
    pub role: String,
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
    pub is_active: bool,
    pub is_manual: bool,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
}

// Why: the caller passes the role vocabulary rather than the query hardcoding
// it, because `users.roles` is a free-text array an older release may have
// written into and a stale value must not invent a role card.
pub async fn list_role_members(
    pool: &PgPool,
    roles: &[String],
    limit: i64,
) -> Result<Vec<RoleMemberRow>, sqlx::Error> {
    sqlx::query_as!(
        RoleMemberRow,
        r#"SELECT
                r AS "role!",
                u.id AS "user_id!: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email AS "email?: Email",
                (u.status = 'active') AS "is_active!",
                (m.user_id IS NOT NULL) AS "is_manual!",
                COALESCE(a.requests, 0)::BIGINT AS "requests_30d!",
                COALESCE(a.cost, 0)::BIGINT AS "cost_30d_microdollars!"
           FROM users u
           CROSS JOIN LATERAL UNNEST(u.roles) AS r
           LEFT JOIN user_manual_roles m ON m.user_id = u.id AND m.role = r
           LEFT JOIN LATERAL (
                SELECT COUNT(*) AS requests,
                       SUM(ar.cost_microdollars) AS cost
                FROM ai_requests ar
                WHERE ar.user_id = u.id
                  AND ar.created_at >= NOW() - INTERVAL '30 days'
           ) a ON TRUE
           WHERE r = ANY($1::TEXT[])
           ORDER BY r, LOWER(COALESCE(u.email, u.id))
           LIMIT $2"#,
        roles,
        limit,
    )
    .fetch_all(pool)
    .await
}
