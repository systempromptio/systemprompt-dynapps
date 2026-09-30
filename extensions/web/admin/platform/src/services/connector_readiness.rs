//! The live check a session-attested connector runs in place of an OAuth
//! probe.

use std::time::Instant;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::mcp::services::client::{
    McpConnectionResult, validate_connection_by_url, validate_connection_with_auth,
};
use systemprompt::models::auth::Permission;
use systemprompt::models::mcp::McpServerType;

use super::connector_accounts::{load_entitlement, scopes_granted};
use super::connector_oauth::Provider;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(crate) async fn probe_session_connection(
    pool: &PgPool,
    user: &UserId,
    provider: &Provider,
) -> AdminResult<VerificationReport> {
    let scopes = provider
        .session_scopes()
        .ok_or_else(|| AdminError::BadRequest("Not a session-attested connector".into()))?;
    let server = systemprompt::loader::ServicesBootstrap::get()
        .map_err(AdminError::internal)?
        .mcp_servers
        .get(provider.slug())
        .filter(|server| server.enabled)
        .ok_or_else(|| AdminError::Unavailable("Connector not configured".into()))?;
    let mut report = VerificationReport::for_provider(provider.slug());
    let started = Instant::now();
    let entitlement = load_entitlement(pool, user).await?;
    let granted = entitlement.active && scopes_granted(&entitlement.roles, scopes);
    let scope_names = scopes
        .iter()
        .map(Permission::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    let outcome = if granted {
        Ok(format!("Signed in with scope {scope_names}"))
    } else {
        Err(AdminError::Forbidden(format!(
            "Your session does not carry scope {scope_names}"
        )))
    };
    report.record("token", started, &outcome);
    if outcome.is_err() {
        return Ok(report.finish());
    }
    let started = Instant::now();
    let upstream =
        |error: systemprompt::mcp::McpDomainError| AdminError::Upstream(error.to_string());
    let probe: AdminResult<McpConnectionResult> =
        match (server.server_type, server.port, server.endpoint.as_deref()) {
            (McpServerType::Internal, Some(port), _) => {
                validate_connection_with_auth(provider.slug(), "127.0.0.1", port, true)
                    .await
                    .map_err(upstream)
            },
            (McpServerType::External, _, Some(endpoint)) => {
                validate_connection_by_url(provider.slug(), endpoint)
                    .await
                    .map_err(upstream)
            },
            _ => Err(AdminError::Unavailable(
                "Connector declares neither a port nor an endpoint".into(),
            )),
        };
    let reachable = match &probe {
        Ok(result) if result.success => Ok(format!(
            "Server answered in {} ms",
            result.connection_time_ms
        )),
        Ok(result) => Err(AdminError::Upstream(
            result
                .error_message
                .clone()
                .unwrap_or_else(|| "Server did not answer".to_owned()),
        )),
        Err(error) => Err(AdminError::Upstream(error.to_string())),
    };
    report.record("initialize", started, &reachable);
    if let (Ok(()), Ok(result)) = (reachable.map(|_| ()), probe) {
        let tools = result.tools_count.map_or_else(
            || "Tool list is behind the server's own authorization".to_owned(),
            |count| format!("{count} tools available"),
        );
        report.record("tools", started, &Ok(tools));
    }
    Ok(report.finish())
}
