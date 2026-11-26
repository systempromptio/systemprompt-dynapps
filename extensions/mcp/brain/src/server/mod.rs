//! Construction and the rmcp `ServerHandler` surface of the `brain` server:
//! authenticate the call, record the access, hand the authenticated person
//! to the tool.

use std::future::Future;
use std::sync::Arc;

use dynbrain::brain::Brain;
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
use systemprompt::traits::DynToolCallIntentClaims;
use systemprompt_mcp_shared::auth::{ServerAccess, ToolAuth, authenticate_tool_request};
use systemprompt_mcp_shared::record_mcp_access;

use crate::tools::{self, ToolCall};

#[derive(Clone, Debug)]
pub struct BrainServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    access: ServerAccess,
    brain: Arc<dyn Brain>,
}

impl BrainServer {
    pub fn new(
        service_id: McpServerId,
        db_pool: DbPool,
        access: ServerAccess,
        artifact_ingest: Arc<ArtifactIngest>,
        brain: Arc<dyn Brain>,
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
            access,
            brain,
        }
    }
}

impl ServerHandler for BrainServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(
                Implementation::new(
                    format!("Brain ({})", self.service_id),
                    env!("CARGO_PKG_VERSION"),
                )
                .with_title("Brain")
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
        tracing::info!("brain MCP server initialized");
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

        let (request_context, _bearer) = authenticate_tool_request(
            ToolAuth {
                db_pool: &self.db_pool,
                service_id: &self.service_id,
                authz_hook: &self.access.authz_hook,
                services_config: &self.access.services.get(),
            },
            &tool_name,
            &ctx,
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

        let client = client_profile_from_peer(&ctx);
        tools::dispatch(&ToolCall {
            executor: &self.executor,
            request: &request,
            context: &request_context,
            client: &client,
            brain: self.brain.as_ref(),
        })
        .await
        .map(Into::into)
    }
}
