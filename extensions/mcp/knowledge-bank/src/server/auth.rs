//! Per-call authentication: RBAC against the registry, with every outcome
//! audited — an authenticated call as `authenticated`, a refused one as a
//! rejection attributed to the tool it asked for. The caller's bearer is
//! returned beside the context so `sync_knowledge` can reach the Atlassian
//! MCP through this instance as that caller.

use rmcp::ErrorData as McpError;
use rmcp::service::{RequestContext, RoleServer};
use systemprompt::database::DbPool;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

pub(super) async fn authenticate_tool_request(
    db_pool: &DbPool,
    tool_name: &str,
    service_id: &str,
    ctx: &RequestContext<RoleServer>,
    authz_hook: &SharedAuthzHook,
) -> Result<(SysRequestContext, String), McpError> {
    let server_name = service_id;
    let rbac_result = enforce_rbac_from_registry(ctx, service_id, authz_hook).await;

    match rbac_result {
        Ok(result) => {
            match result.expect_authenticated(
                "BUG: knowledge-bank requires OAuth but auth was not enforced",
            ) {
                Ok(authenticated) => {
                    record_mcp_access(
                        db_pool,
                        authenticated.context.user_id(),
                        server_name,
                        tool_name,
                        "authenticated",
                    )
                    .await;
                    let token = authenticated.token().to_owned();
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
