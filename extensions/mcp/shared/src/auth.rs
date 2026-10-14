//! The per-call authentication every bundled MCP server runs before a tool
//! executes: RBAC against core's registry, then an access-audit row for the
//! outcome.
//!
//! Returns the authenticated request context and the bearer the
//! caller presented, which a server forwards to the CLI or the gateway.

use rmcp::ErrorData as McpError;
use rmcp::service::{RequestContext, RoleServer};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::authz::SharedAuthzHook;

use crate::{record_mcp_access, record_mcp_access_rejected};

pub async fn authenticate_tool_request(
    db_pool: &DbPool,
    tool_name: &str,
    service_id: &McpServerId,
    ctx: &RequestContext<RoleServer>,
    authz_hook: &SharedAuthzHook,
) -> Result<(SysRequestContext, String), McpError> {
    let server_name = service_id.as_str();
    let rbac_result = enforce_rbac_from_registry(ctx, service_id, authz_hook).await;

    match rbac_result {
        Ok(result) => {
            match result
                .expect_authenticated("BUG: this server requires OAuth but auth was not enforced")
            {
                Ok(authenticated) => {
                    record_mcp_access(
                        db_pool,
                        authenticated.context.user_id(),
                        server_name,
                        tool_name,
                        "authenticated",
                    )
                    .await;
                    let token = authenticated
                        .auth_token()
                        .ok_or_else(|| {
                            McpError::internal_error(
                                "Authenticated request has no credential",
                                None,
                            )
                        })?
                        .as_str()
                        .to_owned();
                    Ok((authenticated.context.clone(), token))
                },
                Err(e) => {
                    record_mcp_access_rejected(db_pool, server_name, tool_name, e.message.as_ref())
                        .await;
                    Err(e)
                },
            }
        },
        Err(e) => {
            record_mcp_access_rejected(db_pool, server_name, tool_name, &format!("{e}")).await;
            Err(e)
        },
    }
}
