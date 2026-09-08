//! The knowledge-bank MCP server: struct construction and rmcp
//! `ServerHandler` surface (info, tool listing, call dispatch).
//!
//! Per-call logic lives in the submodules: `auth` (RBAC + auditing), `tool`
//! (handlers + dispatch), `format` (artifact rendering). Authentication and
//! auditing use the instance pool; the retriever, ingestor and pruner the
//! server is handed already point at the knowledge database. The caller's
//! bearer is passed down so `sync_knowledge` can read Atlassian through the
//! connected MCP as that caller.

mod add_document;
mod auth;
mod format;
mod prune;
mod sync;
#[doc(hidden)]
pub mod tool;

use crate::error::KnowledgeBankError;
use crate::ingest::Ingestor;
use crate::maintenance::Pruner;
use crate::retriever::KnowledgeRetriever;
use crate::tools::{self, SERVER_NAME};
use auth::authenticate_tool_request;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::future::Future;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    McpArtifactRepository, McpToolExecutor, WEBSITE_URL, build_tool_list_result,
    client_profile_from_peer,
};
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::record_mcp_access;
use tool::dispatch_tool;

/// The three backend handles the server serves, all on the knowledge database.
pub struct KnowledgeBackend {
    pub retriever: Arc<dyn KnowledgeRetriever>,
    pub ingestor: Arc<Ingestor>,
    pub pruner: Arc<Pruner>,
}

impl std::fmt::Debug for KnowledgeBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeBackend").finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct KnowledgeBankServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    authz_hook: SharedAuthzHook,
    retriever: Arc<dyn KnowledgeRetriever>,
    ingestor: Arc<Ingestor>,
    pruner: Arc<Pruner>,
}

impl std::fmt::Debug for KnowledgeBankServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeBankServer")
            .field("service_id", &self.service_id)
            .finish_non_exhaustive()
    }
}

impl KnowledgeBankServer {
    pub fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        authz_hook: SharedAuthzHook,
        backend: KnowledgeBackend,
    ) -> Result<Self, KnowledgeBankError> {
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .map_err(|e| KnowledgeBankError::Internal(e.to_string()))?,
        );
        let artifact_repo = Arc::new(
            McpArtifactRepository::new(&db_pool)
                .map_err(|e| KnowledgeBankError::Internal(e.to_string()))?,
        );
        let executor = McpToolExecutor::new(tool_usage_repo, artifact_repo, SERVER_NAME);

        Ok(Self {
            service_id,
            db_pool,
            executor,
            authz_hook,
            retriever: backend.retriever,
            ingestor: backend.ingestor,
            pruner: backend.pruner,
        })
    }
}

impl ServerHandler for KnowledgeBankServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_server_info(
                Implementation::new(
                    format!("Knowledge Bank ({})", self.service_id),
                    env!("CARGO_PKG_VERSION"),
                )
                .with_title("Astound Knowledge Bank")
                .with_website_url(WEBSITE_URL),
            )
            .with_instructions(
                "Project memory for Astound engagements: one shared index of what each user's \
                 Atlassian account can read. Search it with 'search_knowledge' — scope every \
                 query with source_types, read each hit's date and date_source before trusting \
                 it, and open the cited uri#lines when the answer drives scope, implementation, \
                 or a user-facing claim. 'list_knowledge_sources' shows what you may read and \
                 when you last synced; 'sync_knowledge' indexes as you through your connected \
                 Atlassian account (link it at /admin/profile#connected-accounts).",
            )
    }

    fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + MaybeSendFuture + '_ {
        tracing::info!("knowledge-bank MCP server initialized");
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
            &self.db_pool,
            &tool_name,
            self.service_id.as_str(),
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

        let client = client_profile_from_peer(&ctx);
        dispatch_tool(
            &tool::Dispatch {
                executor: &self.executor,
                request: &request,
                request_context: &request_context,
                client: &client,
                bearer: &bearer,
                retriever: self.retriever.as_ref(),
                ingestor: &self.ingestor,
                pruner: &self.pruner,
            },
            &tool_name,
        )
        .await
        .map(Into::into)
    }
}
