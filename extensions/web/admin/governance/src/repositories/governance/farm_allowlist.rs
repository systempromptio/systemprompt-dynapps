//! The declared tool list of the farm run a governance hook token belongs to.
//!
//! A run's hook token is minted for the run's gateway session, so the
//! session in its claims names exactly one `agent_runs` row. The list read
//! is `request.allowed_tools`, the snapshot stored verbatim at launch, so a
//! kit re-sync mid-run cannot widen what the run may call. `None` means the
//! session is no farm run's and no allowlist applies.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use systemprompt_web_shared::error::MarketplaceError;

pub async fn find_farm_run_allowlist(
    pool: &PgPool,
    gateway_session_id: &SessionId,
) -> Result<Option<Vec<String>>, MarketplaceError> {
    let row = sqlx::query_scalar!(
        r#"SELECT ARRAY(
                SELECT jsonb_array_elements_text(COALESCE(r.request->'allowed_tools', '[]'::jsonb))
            ) AS "tools!"
           FROM agent_runs r
           JOIN agent_farm_runs f ON f.agent_run_id = r.id
           WHERE r.gateway_session_id = $1
           ORDER BY r.created_at DESC
           LIMIT 1"#,
        gateway_session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
