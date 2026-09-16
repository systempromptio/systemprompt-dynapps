//! Every credential one person's machines hold, in one list.
//!
//! Three tables answer "what can sign in as this user from a device": a
//! `bridge_sessions` row is a desktop bridge that has actually reported in, a
//! `user_device_certs` row is a client certificate enrolled to a machine, and a
//! `user_api_keys` row is a personal access token. They are separate tables
//! because they are revoked separately, but an admin auditing an account wants
//! one list with one revoke button, so the union happens here rather than in
//! three template blocks.
//!
//! `kind` is what the revoke action dispatches on, so it is part of the row
//! rather than something the view infers from which field is populated.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// One credential a device holds, whatever kind it is.
#[derive(Debug, Clone, Serialize)]
pub struct UserDeviceRow {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub detail: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    // Why: whether the row can be revoked from the page at all. A bridge
    // session is telemetry, not a credential — ending it is done by revoking
    // the token it authenticated with, so the action column stays empty.
    pub revocable: bool,
}

// Why: `UNION ALL` over three shapes rather than three round trips, so the
// combined list is ordered as one thing. Every branch projects the same
// columns; the ones a branch has no answer for are `NULL` rather than a
// fabricated value.
pub async fn list_user_devices(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserDeviceRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT k.id AS "id!", k.kind AS "kind!", k.label AS "label!",
                  k.detail, k.created_at, k.last_seen_at, k.revoked_at, k.revocable AS "revocable!"
             FROM (
               SELECT b.session_id AS id,
                      'bridge' AS kind,
                      COALESCE(b.hostname, b.session_id) AS label,
                      NULLIF(CONCAT_WS(' · ', b.os, b.bridge_version), '') AS detail,
                      b.started_at AS created_at,
                      GREATEST(b.last_heartbeat_at, b.last_activity_at) AS last_seen_at,
                      NULL::TIMESTAMPTZ AS revoked_at,
                      FALSE AS revocable
                 FROM bridge_sessions b
                WHERE b.user_id = $1
               UNION ALL
               SELECT c.id,
                      'cert',
                      COALESCE(c.label, c.fingerprint),
                      c.fingerprint,
                      c.enrolled_at,
                      NULL::TIMESTAMPTZ,
                      c.revoked_at,
                      TRUE
                 FROM user_device_certs c
                WHERE c.user_id = $1
               UNION ALL
               SELECT a.id,
                      'pat',
                      COALESCE(a.name, a.key_prefix),
                      a.key_prefix,
                      a.created_at,
                      a.last_used_at,
                      a.revoked_at,
                      TRUE
                 FROM user_api_keys a
                WHERE a.user_id = $1
             ) k
            ORDER BY (k.revoked_at IS NOT NULL), k.last_seen_at DESC NULLS LAST, k.created_at DESC NULLS LAST
            LIMIT 200"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| UserDeviceRow {
            id: row.id,
            kind: row.kind,
            label: row.label,
            detail: row.detail,
            created_at: row.created_at,
            last_seen_at: row.last_seen_at,
            revoked_at: row.revoked_at,
            revocable: row.revocable,
        })
        .collect())
}
