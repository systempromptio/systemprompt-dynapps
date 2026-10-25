//! The per-call authentication every bundled MCP server runs before a tool
//! executes: RBAC against core's registry, then an access-audit row for the
//! outcome.
//!
//! Returns the authenticated request context and the bearer the
//! caller presented, which a server forwards to the CLI or the gateway.

use rmcp::ErrorData as McpError;
use rmcp::service::{RequestContext, RoleServer};
use systemprompt::database::DbPool;
use systemprompt::governance::authz::SharedAuthzHook;
use systemprompt::identifiers::McpServerId;
use systemprompt::manifest::services::ServicesConfig;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::keys::SigningKeys;
use systemprompt::traits::ConfigWatch;

use crate::{record_mcp_access, record_mcp_access_rejected};

/// What a server authenticates its tool calls against: its pool, its id,
/// the authz hook, the services config and the signing keys it was built
/// with.
#[derive(Clone, Copy)]
pub struct ToolAuth<'a> {
    pub db_pool: &'a DbPool,
    pub service_id: &'a McpServerId,
    pub authz_hook: &'a SharedAuthzHook,
    pub services_config: &'a ServicesConfig,
    pub signing_keys: &'a SigningKeys,
}

/// What a server authorises every tool call against, held for its lifetime:
/// the authz hook, the live services config and the signing keys tokens are
/// validated with.
#[derive(Clone, Debug)]
pub struct ServerAccess {
    pub authz_hook: SharedAuthzHook,
    pub services: ConfigWatch<ServicesConfig>,
    pub signing_keys: SigningKeys,
}

impl std::fmt::Debug for ToolAuth<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolAuth")
            .field("service_id", self.service_id)
            .finish_non_exhaustive()
    }
}

pub async fn authenticate_tool_request(
    auth: ToolAuth<'_>,
    tool_name: &str,
    ctx: &RequestContext<RoleServer>,
) -> Result<(SysRequestContext, String), McpError> {
    let ToolAuth {
        db_pool,
        service_id,
        authz_hook,
        services_config,
        signing_keys,
    } = auth;
    let server_name = service_id.as_str();
    let rbac_result =
        enforce_rbac_from_registry(ctx, service_id, authz_hook, services_config, signing_keys)
            .await;

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
