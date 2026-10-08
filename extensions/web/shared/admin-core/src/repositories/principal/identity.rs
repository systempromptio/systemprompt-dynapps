//! The whole federated identity of one caller, in a single read.
//!
//! Assembled for `GET /api/public/bridge/whoami`, which answers the desktop
//! bridge's "who am I signed in as". The pieces live in four tables — the
//! core `users` and `federated_identities` rows and the web-owned
//! `user_groups` and `user_projects` views — and the bridge shows them on one
//! card, so they are fetched as one query rather than four round trips.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// Everything known about a caller's identity, except the AD group names the
/// directory asserted.
///
/// Those live on the `group_members.source_ad_group` column and are read by
/// `repositories::principal::group_membership::list_source_ad_groups`; the
/// group and project ids they resolved to come back here.
#[derive(Debug, Clone)]
pub struct IdentityEnvelope {
    pub user_id: UserId,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub email_verified: bool,
    pub roles: Vec<String>,
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub idp_issuer: Option<String>,
    pub external_sub: Option<String>,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

struct EnvelopeRow {
    id: String,
    username: String,
    email: String,
    display_name: Option<String>,
    status: String,
    email_verified: bool,
    roles: Vec<String>,
    group_ids: Vec<String>,
    project_ids: Vec<String>,
    created_at: DateTime<Utc>,
    idp_issuer: Option<String>,
    external_sub: Option<String>,
    linked_at: Option<DateTime<Utc>>,
    last_seen_at: Option<DateTime<Utc>>,
}

// Why: Postgres `LEFT JOIN LATERAL … LIMIT 1` — a plain LEFT JOIN would repeat
// the user once per IdP mapping; the bridge wants only the most recently seen
// one.
pub async fn find_identity_envelope(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<IdentityEnvelope>, sqlx::Error> {
    let row = sqlx::query_as!(
        EnvelopeRow,
        r#"
        SELECT
            u.id AS "id!",
            u.name AS "username!",
            u.email AS "email!",
            COALESCE(u.display_name, u.full_name) AS display_name,
            u.status AS "status!",
            u.email_verified AS "email_verified!",
            u.roles AS "roles!: Vec<String>",
            COALESCE(ARRAY(SELECT ug.group_id FROM user_groups ug
                           WHERE ug.user_id = u.id ORDER BY ug.group_id),
                     ARRAY[]::TEXT[]) AS "group_ids!: Vec<String>",
            COALESCE(ARRAY(SELECT pm.project_id FROM user_projects pm
                           WHERE pm.user_id = u.id ORDER BY pm.project_id),
                     ARRAY[]::TEXT[]) AS "project_ids!: Vec<String>",
            u.created_at AS "created_at!",
            fed.issuer AS idp_issuer,
            fed.external_sub,
            fed.created_at AS linked_at,
            fed.last_seen_at
        FROM users u
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
        user_id: UserId::new(r.id),
        username: r.username,
        email: r.email,
        display_name: r.display_name,
        status: r.status,
        email_verified: r.email_verified,
        roles: r.roles,
        group_ids: r.group_ids,
        project_ids: r.project_ids,
        created_at: r.created_at,
        idp_issuer: r.idp_issuer,
        external_sub: r.external_sub,
        linked_at: r.linked_at,
        last_seen_at: r.last_seen_at,
    }))
}
