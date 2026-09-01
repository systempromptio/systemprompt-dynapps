//! The whole federated identity of one caller, in a single read.
//!
//! Assembled for `GET /api/public/bridge/whoami`, which answers the desktop
//! bridge's "who am I signed in as". The pieces live in three tables — the
//! core `users` and `federated_identities` rows and the web-owned
//! `user_profile_ext` — and the bridge shows them on one card, so they are
//! fetched as one query rather than three round trips.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// Everything known about a caller's identity, except their AD groups.
///
/// Group memberships live in the web-owned `adfs_group_memberships` side
/// table and are read by [`super::super::adfs_groups::list_groups`].
#[derive(Debug, Clone)]
pub struct IdentityEnvelope {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub email_verified: bool,
    pub roles: Vec<String>,
    pub project: Option<String>,
    pub created_at: DateTime<Utc>,
    pub idp_issuer: Option<String>,
    pub external_sub: Option<String>,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

// Why: A user may hold several federated mappings (one per IdP). The bridge
// asks which identity this session came from, and the closest honest answer
// from a row alone is the most recently used one — hence ORDER BY
// last_seen_at DESC and a LATERAL join rather than a plain LEFT JOIN, which
// would multiply the result row per mapping.
pub async fn find_identity_envelope(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<IdentityEnvelope>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            u.id AS "user_id!: UserId",
            u.name AS "username!",
            u.email AS "email!",
            COALESCE(u.display_name, u.full_name) AS display_name,
            u.status AS "status!",
            u.email_verified AS "email_verified!",
            u.roles AS "roles!: Vec<String>",
            upe.project AS "project?",
            u.created_at AS "created_at!",
            fed.issuer AS idp_issuer,
            fed.external_sub AS external_sub,
            fed.created_at AS linked_at,
            fed.last_seen_at AS last_seen_at
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        LEFT JOIN LATERAL (
            SELECT f.issuer, f.external_sub, f.created_at, f.last_seen_at
            FROM federated_identities f
            WHERE f.user_id = u.id
            ORDER BY f.last_seen_at DESC
            LIMIT 1
        ) fed ON TRUE
        WHERE u.id = $1
        "#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| IdentityEnvelope {
        user_id: r.user_id,
        username: r.username,
        email: r.email,
        display_name: r.display_name,
        status: r.status,
        email_verified: r.email_verified,
        roles: r.roles,
        project: r.project,
        created_at: r.created_at,
        idp_issuer: r.idp_issuer,
        external_sub: r.external_sub,
        linked_at: r.linked_at,
        last_seen_at: r.last_seen_at,
    }))
}
