//! Authenticated MCP entry points. Human decisions have no MCP tool.
//!
//! Composition root for the standalone binary: the tool-usage and artifact
//! repositories are built once here and injected into the executor.
mod execute;
mod tools;

use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    McpArtifactRepository, McpToolExecutor, build_tool_list_result, client_profile_from_peer,
};
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::record_mcp_access;

pub use tools::ToolInput;

#[derive(Debug, Clone)]
pub struct WorkflowServer {
    db: DbPool,
    pool: Arc<PgPool>,
    knowledge: Arc<PgPool>,
    service_id: McpServerId,
    hook: SharedAuthzHook,
    executor: McpToolExecutor,
}

impl WorkflowServer {
    pub fn new(
        db: DbPool,
        pool: Arc<PgPool>,
        knowledge: Arc<PgPool>,
        hook: SharedAuthzHook,
    ) -> Result<Self> {
        let executor = McpToolExecutor::new(
            Arc::new(ToolUsageRepository::new(&db)?),
            Arc::new(McpArtifactRepository::new(&db)?),
            "requirements-workflow",
        );
        Ok(Self {
            db,
            pool,
            knowledge,
            service_id: McpServerId::new("requirements-workflow"),
            hook,
            executor,
        })
    }
}

impl ServerHandler for WorkflowServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "Astound Requirements Workflow",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Use the four requirements skills. Sync Jira and Confluence before starting. Each artifact must pass validation before the next phase. Human review occurs in the browser; no tool can approve. Always use the returned revision. Evidence and review receipts are immutable history.",
            )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = std::result::Result<ListToolsResult, McpError>> + MaybeSendFuture + '_
    {
        std::future::ready(tools::catalogue().map(build_tool_list_result))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, McpError> {
        let name = tools::lookup(request.name.as_ref())?;
        let result = enforce_rbac_from_registry(&ctx, self.service_id.as_str(), &self.hook).await?;
        let auth = result.expect_authenticated("requirements-workflow requires authentication")?;
        record_mcp_access(
            &self.db,
            auth.context.user_id(),
            "requirements-workflow",
            name,
            "authenticated",
        )
        .await;
        let handler = tools::Handler {
            server: self,
            name,
            bearer: auth.token(),
        };
        self.executor
            .execute(
                &handler,
                &request,
                &auth.context,
                &client_profile_from_peer(&ctx),
            )
            .await
            .map(Into::into)
    }
}
