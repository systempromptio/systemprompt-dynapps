//! Which MCP servers are alive, and who is attached to them.
//!
//! Liveness is derived, not stored: a server is alive when one of its sessions
//! has spoken within twice the heartbeat interval. Twice, rather than once,
//! because a single missed beat is a slow network and not an outage, and a
//! console that cries dead on every hiccup stops being read.

use sqlx::PgPool;

/// Minutes an attached MCP client is expected to go between beats.
///
/// A server counts as alive while a session of its has spoken within twice
/// this window; anything older is reported stale rather than dead, because the
/// session row is still open and may yet speak again.
pub const HEARTBEAT_INTERVAL_MINUTES: i64 = 60;

#[derive(Debug, Clone)]
pub struct McpServerLiveness {
    pub server_id: String,
    pub sessions_open: i64,
    pub sessions_alive: i64,
    pub distinct_users: i64,
    pub last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    pub proxy_identities: i64,
}

impl McpServerLiveness {
    /// True while at least one session has beaten inside the alive window.
    #[must_use]
    pub const fn is_alive(&self) -> bool {
        self.sessions_alive > 0
    }
}

/// Every server the session table has ever seen, with its current liveness.
///
/// Servers with no session row at all are absent here by construction — the
/// catalog is what knows those exist, so the caller left-joins this onto it
/// and renders the misses as "never connected".
pub async fn list_mcp_server_liveness(
    pool: &PgPool,
) -> Result<Vec<McpServerLiveness>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
             s.mcp_server_id AS "server_id!",
             COUNT(*) FILTER (WHERE s.status = 'active')::BIGINT AS "sessions_open!",
             COUNT(*) FILTER (
               WHERE s.status = 'active'
                 AND s.last_activity_at >= NOW() - make_interval(mins => $1::int)
             )::BIGINT AS "sessions_alive!",
             COUNT(DISTINCT s.user_id)::BIGINT AS "distinct_users!",
             MAX(s.last_activity_at) AS "last_activity_at?",
             COUNT(p.session_id) FILTER (WHERE p.expires_at > NOW())::BIGINT AS "proxy_identities!"
           FROM mcp_sessions s
           LEFT JOIN mcp_proxy_identities p ON p.session_id = s.session_id
          WHERE s.mcp_server_id IS NOT NULL
          GROUP BY s.mcp_server_id
          ORDER BY s.mcp_server_id"#,
        i32::try_from(HEARTBEAT_INTERVAL_MINUTES * 2).unwrap_or(120)
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpServerLiveness {
            server_id: r.server_id,
            sessions_open: r.sessions_open,
            sessions_alive: r.sessions_alive,
            distinct_users: r.distinct_users,
            last_activity_at: r.last_activity_at,
            proxy_identities: r.proxy_identities,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct McpSessionRow {
    pub session_id: String,
    pub user_id: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub has_proxy_identity: bool,
    pub proxy_user_type: Option<String>,
}

/// The most recently active sessions attached to one server.
pub async fn list_mcp_sessions_for_server(
    pool: &PgPool,
    server_id: &str,
    limit: i64,
) -> Result<Vec<McpSessionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
             s.session_id AS "session_id!",
             s.user_id AS "user_id?",
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
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpSessionRow {
            session_id: r.session_id,
            user_id: r.user_id,
            status: r.status,
            created_at: r.created_at,
            last_activity_at: r.last_activity_at,
            expires_at: r.expires_at,
            has_proxy_identity: r.has_proxy_identity,
            proxy_user_type: r.proxy_user_type,
        })
        .collect())
}
