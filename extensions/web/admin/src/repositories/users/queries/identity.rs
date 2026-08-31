//! The whole federated identity of one caller, in a single read.
//!
//! Assembled for `GET /api/public/bridge/whoami`, which answers the desktop
//! bridge's "who am I signed in as". The pieces live in five tables — the core
//! `users` and `federated_identities` rows, the web-owned `user_profile_ext`,
//! `organization_members`, and `organizations` — and the bridge shows them on
//! one card, so they are fetched as one query rather than five round trips.

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
    pub department: String,
    pub created_at: DateTime<Utc>,
    pub idp_issuer: Option<String>,
    pub external_sub: Option<String>,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub organization_id: Option<String>,
    pub organization_slug: Option<String>,
    pub organization_name: Option<String>,
    pub org_role: Option<String>,
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
            COALESCE(upe.department, 'Default') AS "department!",
            u.created_at AS "created_at!",
            fed.issuer AS idp_issuer,
            fed.external_sub AS external_sub,
            fed.created_at AS linked_at,
            fed.last_seen_at AS last_seen_at,
            org.id AS organization_id,
            org.slug AS organization_slug,
            org.name AS organization_name,
            m.org_role AS org_role
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        LEFT JOIN LATERAL (
            SELECT f.issuer, f.external_sub, f.created_at, f.last_seen_at
            FROM federated_identities f
            WHERE f.user_id = u.id
            ORDER BY f.last_seen_at DESC
            LIMIT 1
        ) fed ON TRUE
        LEFT JOIN organization_members m ON m.user_id = u.id
        LEFT JOIN organizations org ON org.id = m.org_id
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
        department: r.department,
        created_at: r.created_at,
        idp_issuer: r.idp_issuer,
        external_sub: r.external_sub,
        linked_at: r.linked_at,
        last_seen_at: r.last_seen_at,
        organization_id: r.organization_id,
        organization_slug: r.organization_slug,
        organization_name: r.organization_name,
        org_role: r.org_role,
    }))
}
