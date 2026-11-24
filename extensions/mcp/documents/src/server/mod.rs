//! Construction and the rmcp `ServerHandler` surface of the `documents`
//! server.
//!
//! Core's tools, with a live page's `capabilities` taken off the calls that
//! declare them and recorded around core's write (`live_pages`).

mod live_pages;

use std::future::Future;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Icon, Implementation,
    InitializeRequestParams, InitializeResult, ListToolsResult, PaginatedRequestParams,
    ProtocolVersion, ServerCapabilities, ServerConfig,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::sync::Arc;
use systemprompt::ai::AiRequestRepository;
use systemprompt::database::DbPool;
use systemprompt::files::documents::agents::act_as;
use systemprompt::identifiers::{DocumentId, McpServerId};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    ArtifactIngest, McpToolExecutor, WEBSITE_URL, build_tool_list_result, client_profile_from_peer,
};
use systemprompt::models::documents::{AccessLevel, DocumentFormat};
use systemprompt::models::execution::context::RequestContext as CallContext;
use systemprompt::runtime::documents::actor::request_actor;
use systemprompt::runtime::documents::mcp::{DocumentsMcpTools, DocumentsToolCall, INSTRUCTIONS};
use systemprompt::traits::DynToolCallIntentClaims;
use systemprompt_mcp_shared::auth::{ServerAccess, ToolAuth, authenticate_tool_request};
use systemprompt_mcp_shared::record_mcp_access;
use systemprompt_web_admin_platform::artifacts::LivePages;
use systemprompt_web_admin_platform::artifacts::manifest::Capabilities;

#[derive(Clone, Debug)]
pub struct DocumentsServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    access: ServerAccess,
    tools: DocumentsMcpTools,
    live: LivePages,
}

impl DocumentsServer {
    pub fn new(
        db_pool: DbPool,
        access: ServerAccess,
        artifact_ingest: Arc<ArtifactIngest>,
        tools: DocumentsMcpTools,
        live: LivePages,
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
            live,
        }
    }
}

impl ServerHandler for DocumentsServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
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
            .with_instructions(format!(
                "{INSTRUCTIONS}\n\n{}",
                live_pages::INSTRUCTIONS_HINT
            ))
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
        std::future::ready(Ok(build_tool_list_result(live_pages::advertise(
            self.tools.list_tools(),
        ))))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let tool_name = request.name.to_string();
        let server_name = self.service_id.to_string();

        let (request_context, auth_token) = authenticate_tool_request(
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

        self.run(request, &request_context, &auth_token, &ctx)
            .await
            .map(Into::into)
    }
}

impl DocumentsServer {
    async fn run(
        &self,
        mut request: CallToolRequestParams,
        request_context: &CallContext,
        auth_token: &str,
        ctx: &RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let guide = live_pages::wants_guide(&mut request);
        if guide == Some(true) {
            return Ok(CallToolResult::success(vec![ContentBlock::text(
                live_pages::GUIDE,
            )]));
        }
        let declared = match live_pages::take(&mut request) {
            Ok(declared) => declared,
            Err(refused) => return Ok(live_pages::refused(&refused)),
        };
        if let Some(capabilities) = &declared {
            if let Err(refused) = self.live.check(capabilities, auth_token).await {
                return Ok(live_pages::refused(&refused));
            }
            if live_pages::only_declares(&request) {
                return Ok(self
                    .declare_only(&request, request_context, capabilities)
                    .await);
            }
        }
        let client = client_profile_from_peer(ctx);
        let hop = systemprompt_mcp_shared::auth::verified_hop_token(ctx);
        let mut result = self
            .tools
            .call(DocumentsToolCall {
                executor: &self.executor,
                request: &request,
                context: request_context,
                client: &client,
                hop: hop.as_ref(),
            })
            .await?;
        if guide.is_some() {
            result.content.push(ContentBlock::text(live_pages::GUIDE));
        }
        let Some(capabilities) = declared.filter(|_| result.is_error != Some(true)) else {
            return Ok(result);
        };
        let Some(document) = live_pages::named_document(&request, &result) else {
            return Ok(live_pages::noted(
                result,
                false,
                "capabilities not recorded: the result names no document",
            ));
        };
        let (recorded, note) = self
            .declare(&document, request_context, &capabilities)
            .await;
        Ok(live_pages::noted(result, recorded, &note))
    }

    async fn declare_only(
        &self,
        request: &CallToolRequestParams,
        context: &CallContext,
        capabilities: &Capabilities,
    ) -> CallToolResult {
        let Some(document) =
            live_pages::named_document(request, &CallToolResult::success(Vec::new()))
        else {
            return live_pages::refusal("documents_update needs document_id");
        };
        match self.declare(&document, context, capabilities).await {
            (true, note) => CallToolResult::success(vec![ContentBlock::text(note)]),
            (false, note) => live_pages::refusal(note),
        }
    }

    // Why: core's document service decides what the caller may do, so the
    // page's format and the caller's level are asked of it as the caller.
    async fn declare(
        &self,
        document: &DocumentId,
        context: &CallContext,
        capabilities: &Capabilities,
    ) -> (bool, String) {
        let Some(user) = context.user.as_ref() else {
            return (
                false,
                "capabilities not recorded: the call carried no user".to_owned(),
            );
        };
        let lookup = act_as(
            request_actor(context),
            self.tools.service().get(user, document, context.trace_id()),
        )
        .await;
        let view = match lookup {
            Ok(view) => view,
            Err(error) => return (false, format!("capabilities not recorded: {error}")),
        };
        if view.document.format != DocumentFormat::Html || view.my_access < AccessLevel::Edit {
            return (
                false,
                "capabilities not recorded: only an HTML page you can edit declares them"
                    .to_owned(),
            );
        }
        match self
            .live
            .record(
                self.db_pool.pool().as_ref(),
                document,
                capabilities,
                &user.id,
            )
            .await
        {
            Ok(()) if capabilities.is_empty() => (
                true,
                format!("capabilities cleared on {}", view.document.name),
            ),
            Ok(()) => (
                true,
                format!("capabilities recorded on {}", view.document.name),
            ),
            Err(error) => {
                tracing::error!(%error, "a live page's capabilities were not recorded");
                (
                    false,
                    "capabilities not recorded: the declaration could not be stored".to_owned(),
                )
            },
        }
    }
}
