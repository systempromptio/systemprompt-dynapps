//! The `agent-farm` MCP server: construction and the rmcp `ServerHandler`
//! surface. Per-call work (viewer resolution, dispatch) is in `tool`; who
//! may see a run is decided once, in `viewer`.

mod tool;
mod viewer;

use std::future::Future;
use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Icon, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    ServerConfig,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::ai::AiRequestRepository;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    ArtifactIngest, McpToolExecutor, WEBSITE_URL, build_tool_list_result, client_profile_from_peer,
};
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt::traits::DynToolCallIntentClaims;
use systemprompt_mcp_shared::auth::authenticate_tool_request;
use systemprompt_mcp_shared::record_mcp_access;

use crate::env::Env;
use crate::tools;

pub use tool::{Dispatch, ToolCtx, dispatch_tool};
pub use viewer::{RunOrigin, Viewer, authorize_run, viewer_for};

#[derive(Clone, Debug)]
pub struct AgentFarmServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    authz_hook: SharedAuthzHook,
    env: Arc<Env>,
}

impl AgentFarmServer {
    pub fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        authz_hook: SharedAuthzHook,
        artifact_ingest: Arc<ArtifactIngest>,
        env: Arc<Env>,
    ) -> Self {
        let tool_usage_repo = Arc::new(ToolUsageRepository::new(&db_pool));
        let intent_claims: DynToolCallIntentClaims = Arc::new(AiRequestRepository::new(&db_pool));
        let executor = McpToolExecutor::new(
            tool_usage_repo,
            intent_claims,
            artifact_ingest,
            service_id.clone(),
        );
        Self {
            service_id,
            db_pool,
            executor,
            authz_hook,
            env,
        }
    }
}

impl ServerHandler for AgentFarmServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(
                Implementation::new(
                    format!("Agent Farm ({})", self.service_id),
                    env!("CARGO_PKG_VERSION"),
                )
                .with_title("Agent Farm")
                .with_icons(vec![
                    Icon::new(format!("{WEBSITE_URL}/files/images/favicon-32x32.png"))
                        .with_mime_type("image/png")
                        .with_sizes(vec!["32x32".to_owned()]),
                ])
                .with_website_url(WEBSITE_URL),
            )
            .with_instructions(tools::INSTRUCTIONS.to_owned())
    }

    fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + MaybeSendFuture + '_ {
        tracing::info!("agent-farm MCP server initialized");
        std::future::ready(Ok(self.get_info()))
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(build_tool_list_result(tools::list_tools())))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let tool_name = request.name.to_string();
        let server_name = self.service_id.to_string();

        let (request_context, _auth_token) = authenticate_tool_request(
            &self.db_pool,
            &tool_name,
            &self.service_id,
            &ctx,
            &self.authz_hook,
        )
        .await?;

        record_mcp_access(
            &self.db_pool,
            request_context.user_id(),
            &server_name,
            &tool_name,
            "used",
        )
        .await;

        let viewer = viewer_for(&self.env.pool, request_context.user_id()).await?;
        let client = client_profile_from_peer(&ctx);
        let tool_ctx = ToolCtx {
            env: Some(&self.env),
            viewer,
        };
        dispatch_tool(
            &Dispatch {
                executor: &self.executor,
                request: &request,
                request_context: &request_context,
                client: &client,
                tool_ctx: &tool_ctx,
            },
            &tool_name,
        )
        .await
        .map(Into::into)
    }
}
