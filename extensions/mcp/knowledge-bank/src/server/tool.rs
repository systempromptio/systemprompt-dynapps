//! Retrieval tool handlers and dispatch. Handlers hand the caller's
//! [`Principal`] to the backend; `knowledge_index_stats` carries an admin
//! gate of its own, because the shared index's totals are administrative
//! however the server's ACL is widened for readers. `sync_knowledge`,
//! `add_knowledge_document` and `prune_knowledge` live in the sibling `sync`,
//! `add_document` and `prune` modules. Errors that carry driver or upstream
//! text (database, HTTP, serialization, internal) are reported to the caller
//! as a generic backend failure and logged in full here (FR-ACC-5).

use super::add_document::AddDocumentHandler;
use super::format::{format_hits, format_index_stats, format_sources};
use super::prune::PruneKnowledgeHandler;
use super::sync::SyncKnowledgeHandler;
use crate::error::KnowledgeBankError;
use crate::ingest::Ingestor;
use crate::maintenance::Pruner;
use crate::principal::Principal;
use crate::retriever::{KnowledgeRetriever, SearchFilter};
use crate::tools::{
    ADD_DOCUMENT_TOOL, INDEX_STATS_TOOL, IndexStatsInput, LIST_SOURCES_TOOL,
    ListKnowledgeSourcesInput, PRUNE_TOOL, SEARCH_TOOL, SYNC_TOOL, SearchKnowledgeInput,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::{ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

pub const ADMIN_ROLE: &str = "admin";

pub const BACKEND_FAILURE_MESSAGE: &str = "The knowledge bank could not complete the request; the failure has been logged for the operator.";

pub(super) fn to_mcp_error(e: &KnowledgeBankError) -> McpError {
    match e {
        KnowledgeBankError::Forbidden(_)
        | KnowledgeBankError::UnknownSource(_)
        | KnowledgeBankError::InvalidInput(_) => McpError::invalid_params(e.to_string(), None),
        KnowledgeBankError::NotConfigured(_)
        | KnowledgeBankError::Config(_)
        | KnowledgeBankError::AtlassianUnavailable(_)
        | KnowledgeBankError::Backend(_) => McpError::internal_error(e.to_string(), None),
        KnowledgeBankError::Database(_)
        | KnowledgeBankError::Http(_)
        | KnowledgeBankError::Serialization(_)
        | KnowledgeBankError::Internal(_) => {
            tracing::error!(error = %e, "knowledge bank backend failure");
            McpError::internal_error(BACKEND_FAILURE_MESSAGE, None)
        },
    }
}

pub(super) fn require_admin(principal: &Principal, tool: &str) -> Result<(), KnowledgeBankError> {
    if principal.roles.iter().any(|r| r == ADMIN_ROLE) {
        Ok(())
    } else {
        Err(KnowledgeBankError::Forbidden(format!(
            "{tool} is restricted to administrators"
        )))
    }
}

pub(super) struct SearchKnowledgeHandler<'a> {
    pub(super) retriever: &'a dyn KnowledgeRetriever,
}

impl McpToolHandler for SearchKnowledgeHandler<'_> {
    type Input = SearchKnowledgeInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        SEARCH_TOOL
    }

    fn description(&self) -> &'static str {
        "Search the project knowledge bank."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        let filter = SearchFilter {
            source_types: input.source_types.unwrap_or_default(),
            top_k: input.top_k,
        };
        self.retriever
            .search(&principal, &input.query, &filter)
            .await
            .map(|hits| format_hits(&hits, &input.query))
            .map_err(|e| to_mcp_error(&e))
    }
}

pub(super) struct ListSourcesHandler<'a> {
    pub(super) retriever: &'a dyn KnowledgeRetriever,
}

impl McpToolHandler for ListSourcesHandler<'_> {
    type Input = ListKnowledgeSourcesInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        LIST_SOURCES_TOOL
    }

    fn description(&self) -> &'static str {
        "List searchable knowledge sources."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        self.retriever
            .list_sources(&principal)
            .await
            .map(|sources| format_sources(&sources))
            .map_err(|e| to_mcp_error(&e))
    }
}

pub(super) struct IndexStatsHandler<'a> {
    pub(super) retriever: &'a dyn KnowledgeRetriever,
}

impl McpToolHandler for IndexStatsHandler<'_> {
    type Input = IndexStatsInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        INDEX_STATS_TOOL
    }

    fn description(&self) -> &'static str {
        "Report shared-index statistics (administrators only)."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        require_admin(&principal, INDEX_STATS_TOOL).map_err(|e| to_mcp_error(&e))?;
        self.retriever
            .index_stats()
            .await
            .map(|stats| format_index_stats(&stats))
            .map_err(|e| to_mcp_error(&e))
    }
}

#[doc(hidden)]
#[expect(
    missing_debug_implementations,
    reason = "borrows `dyn KnowledgeRetriever`, which has no Debug bound"
)]
pub struct Dispatch<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub request_context: &'a SysRequestContext,
    pub client: &'a ClientProfile,
    pub bearer: &'a str,
    pub retriever: &'a dyn KnowledgeRetriever,
    pub ingestor: &'a Arc<Ingestor>,
    pub pruner: &'a Pruner,
}

// Why: Exposed (behind `#[doc(hidden)]`) so the external test workspace can
// assert the unknown-tool arm without an rmcp `Peer`, which only exists once
// a transport is serving. Not part of the public API.
#[doc(hidden)]
pub async fn dispatch_tool(
    ctx: &Dispatch<'_>,
    tool_name: &str,
) -> Result<CallToolResult, McpError> {
    match tool_name {
        SEARCH_TOOL => {
            let handler = SearchKnowledgeHandler {
                retriever: ctx.retriever,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        LIST_SOURCES_TOOL => {
            let handler = ListSourcesHandler {
                retriever: ctx.retriever,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        INDEX_STATS_TOOL => {
            let handler = IndexStatsHandler {
                retriever: ctx.retriever,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        SYNC_TOOL => {
            let handler = SyncKnowledgeHandler {
                ingestor: ctx.ingestor,
                bearer: ctx.bearer,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        ADD_DOCUMENT_TOOL => {
            let handler = AddDocumentHandler {
                ingestor: ctx.ingestor,
            };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        PRUNE_TOOL => {
            let handler = PruneKnowledgeHandler { pruner: ctx.pruner };
            ctx.executor
                .execute(&handler, ctx.request, ctx.request_context, ctx.client)
                .await
        },
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown tool: '{tool_name}'. Available tools: '{SEARCH_TOOL}', \
                 '{LIST_SOURCES_TOOL}', '{INDEX_STATS_TOOL}', '{SYNC_TOOL}', \
                 '{ADD_DOCUMENT_TOOL}', '{PRUNE_TOOL}'."
            ),
            None,
        )),
    }
}
