//! Who is attached to an MCP server, and under what identity.
//!
//! The heartbeat half of liveness lives in `repositories::overview::liveness`,
//! which the dashboard also reads — one query answers "when did this server
//! last speak" for both surfaces. What this module adds is the part only the
//! MCP pages need: the proxy identities a live session is acting as, and the
//! per-session detail behind the count.
//!
//! Alive is derived from the heartbeat rather than stored, at twice the
//! interval below. Twice, rather than once, because a single missed beat is a
//! slow network and not an outage, and a console that cries dead on every
//! hiccup stops being read.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

// Why: Minutes an attached MCP client is expected to go between beats.
//
// A server counts as alive while it has spoken within twice this window;
// anything older is reported stale rather than dead, because its session rows
// are still open and it may yet speak again.
pub const HEARTBEAT_INTERVAL_MINUTES: i64 = 60;

// Why: True when `last_heartbeat` falls inside the alive window.
#[must_use]
pub fn is_alive(last_heartbeat: Option<chrono::DateTime<chrono::Utc>>) -> bool {
    let cutoff = chrono::Utc::now() - chrono::Duration::minutes(HEARTBEAT_INTERVAL_MINUTES * 2);
    last_heartbeat.is_some_and(|t| t >= cutoff)
}

#[derive(Debug, Clone)]
pub struct McpProxyIdentityCount {
    pub server_id: String,
    pub identities: i64,
    pub distinct_users: i64,
}

// Why: Unexpired proxy identities per server, with the people behind them.
//
// Keyed on the session the identity was minted for, so a server with open
// sessions and no identities is a server nothing has authenticated through —
// which is a different state from having no sessions at all.
pub async fn list_mcp_proxy_identity_counts(
    pool: &PgPool,
) -> Result<Vec<McpProxyIdentityCount>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
             s.mcp_server_id AS "server_id!",
             COUNT(p.session_id) FILTER (WHERE p.expires_at > NOW())::BIGINT AS "identities!",
             COUNT(DISTINCT s.user_id)::BIGINT AS "distinct_users!"
           FROM mcp_sessions s
           LEFT JOIN mcp_proxy_identities p ON p.session_id = s.session_id
          WHERE s.mcp_server_id IS NOT NULL
          GROUP BY s.mcp_server_id
          ORDER BY s.mcp_server_id"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpProxyIdentityCount {
            server_id: r.server_id,
            identities: r.identities,
            distinct_users: r.distinct_users,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct McpSessionRow {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub has_proxy_identity: bool,
    pub proxy_user_type: Option<String>,
}

// Why: The most recently active sessions attached to one server.
pub async fn list_mcp_sessions_for_server(
    pool: &PgPool,
    server_id: &str,
    limit: i64,
) -> Result<Vec<McpSessionRow>, sqlx::Error> {
    sqlx::query_as!(
        McpSessionRow,
        r#"SELECT
             s.session_id AS "session_id!: SessionId",
             s.user_id AS "user_id?: UserId",
             s.status AS "status!",
             s.created_at AS "created_at!",
             s.last_activity_at AS "last_activity_at!",
             s.expires_at AS "expires_at!",
             (p.session_id IS NOT NULL AND p.expires_at > NOW()) AS "has_proxy_identity!",
             p.user_type AS "proxy_user_type?"
           FROM mcp_sessions s
           LEFT JOIN mcp_proxy_identities p ON p.session_id = s.session_id
          WHERE s.mcp_server_id = $1
          ORDER BY s.last_activity_at DESC
          LIMIT $2"#,
        server_id,
        limit
    )
    .fetch_all(pool)
    .await
}
