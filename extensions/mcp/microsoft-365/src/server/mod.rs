//! Construction and the rmcp `ServerHandler` surface of the `microsoft-365`
//! server: authenticate the call, record the access, bind a Graph client to
//! the caller's own token, dispatch.

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
use systemprompt::traits::{DynToolCallIntentClaims, SecretLookup};
use systemprompt_mcp_shared::auth::{ServerAccess, ToolAuth, authenticate_tool_request};
use systemprompt_mcp_shared::record_mcp_access;

use crate::graph::{Broker, GraphClient, GraphClientParts, GraphEndpoint, GraphError, GraphLimits};
use crate::tools::{self, ToolCall};

/// What every call needs to reach Graph, held for the server's lifetime:
/// the shared HTTP client, the broker, the secrets the endpoint is read
/// from, and the bounds.
#[derive(Clone, Debug)]
pub struct GraphHost {
    pub http: reqwest::Client,
    pub broker: Broker,
    pub secrets: Arc<dyn SecretLookup>,
    pub limits: GraphLimits,
}

impl GraphHost {
    pub fn client_for(
        &self,
        bearer: String,
        purpose: Option<String>,
    ) -> Result<GraphClient, GraphError> {
        Ok(GraphClient::new(GraphClientParts {
            endpoint: GraphEndpoint::from_lookup(self.secrets.as_ref())?,
            http: self.http.clone(),
            tokens: Arc::new(self.broker.for_caller(bearer, purpose)),
            limits: self.limits,
        }))
    }
}

#[derive(Clone, Debug)]
pub struct Microsoft365Server {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    access: ServerAccess,
    graph: GraphHost,
}

impl Microsoft365Server {
    pub fn new(
        service_id: McpServerId,
        db_pool: DbPool,
        access: ServerAccess,
        artifact_ingest: Arc<ArtifactIngest>,
        graph: GraphHost,
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
            graph,
        }
    }
}

impl ServerHandler for Microsoft365Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(
                Implementation::new(
                    format!("Microsoft 365 ({})", self.service_id),
                    env!("CARGO_PKG_VERSION"),
                )
                .with_title("Microsoft 365")
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
        tracing::info!("microsoft-365 MCP server initialized");
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

        let (request_context, bearer) = authenticate_tool_request(
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

        let purpose = request_context.caller_purpose().map(str::to_owned);
        let graph = self.graph.client_for(bearer, purpose)?;
        let client = client_profile_from_peer(&ctx);
        tools::dispatch(&ToolCall {
            executor: &self.executor,
            request: &request,
            context: &request_context,
            client: &client,
            graph: &graph,
        })
        .await
        .map(Into::into)
    }
}
