//! Construction and the rmcp `ServerHandler` surface of the `documents`
//! server.

use std::future::Future;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Icon, Implementation, InitializeRequestParams,
    InitializeResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse, ServerCapabilities,
    ServerConfig,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::sync::Arc;
use systemprompt::ai::AiRequestRepository;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    ArtifactIngest, McpToolExecutor, WEBSITE_URL, build_tool_list_result, client_profile_from_peer,
};
use systemprompt::runtime::documents::mcp::{DocumentsMcpTools, DocumentsToolCall, INSTRUCTIONS};
use systemprompt::traits::DynToolCallIntentClaims;
use systemprompt_mcp_shared::auth::{ServerAccess, ToolAuth, authenticate_tool_request};
use systemprompt_mcp_shared::record_mcp_access;

#[derive(Clone, Debug)]
pub struct DocumentsServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    access: ServerAccess,
    tools: DocumentsMcpTools,
}

impl DocumentsServer {
    pub fn new(
        db_pool: DbPool,
        access: ServerAccess,
        artifact_ingest: Arc<ArtifactIngest>,
        tools: DocumentsMcpTools,
    ) -> Self {
        let service_id = tools.server_name().clone();
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
            tools,
        }
    }
}

impl ServerHandler for DocumentsServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_protocol_version(ProtocolVersion::V_2026_07_28)
        .with_server_info(
            Implementation::new(
                format!("Documents ({})", self.service_id),
                env!("CARGO_PKG_VERSION"),
            )
            .with_title("Documents")
            .with_icons(vec![
                Icon::new(format!("{WEBSITE_URL}/files/images/favicon-32x32.png"))
                    .with_mime_type("image/png")
                    .with_sizes(vec!["32x32".to_owned()]),
            ])
            .with_website_url(WEBSITE_URL),
        )
        .with_instructions(INSTRUCTIONS.to_owned())
    }

    fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + MaybeSendFuture + '_ {
        tracing::info!("documents MCP server initialized");
        std::future::ready(Ok(self.get_info()))
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(build_tool_list_result(self.tools.list_tools())))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let tool_name = request.name.to_string();
        let server_name = self.service_id.to_string();

        let (request_context, _auth_token) = authenticate_tool_request(
            ToolAuth {
                db_pool: &self.db_pool,
                service_id: &self.service_id,
                authz_hook: &self.access.authz_hook,
                services_config: &self.access.services.get(),
                verifier: self.access.verifier(),
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
        self.tools
            .call(DocumentsToolCall {
                executor: &self.executor,
                request: &request,
                context: &request_context,
                client: &client,
            })
            .await
            .map(Into::into)
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(self.tools.list_resources()))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        self.tools.read_resource(&request).map(Into::into)
    }
}
