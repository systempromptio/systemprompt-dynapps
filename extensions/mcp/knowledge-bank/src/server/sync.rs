//! `sync_knowledge`: start the caller's run in the background. Open to every
//! signed-in user. Files categories need nothing from the caller; Confluence
//! and Jira are read with the Atlassian credentials in the request headers,
//! which are required only when such a category is in the run — naming one
//! explicitly without them is refused, leaving them out of an unscoped sync
//! is fine and the files categories are synced alone.

use super::format::format_sync_started;
use super::tool::to_mcp_error;
use crate::downstream::AtlassianCredentials;
use crate::error::KnowledgeBankError;
use crate::ingest::{Ingestor, RunOptions};
use crate::principal::Principal;
use crate::tools::{SYNC_TOOL, SyncKnowledgeInput};
use axum::http::HeaderMap;
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

pub(super) struct SyncKnowledgeHandler<'a> {
    pub(super) ingestor: &'a Arc<Ingestor>,
    pub(super) headers: Option<&'a HeaderMap>,
}

impl McpToolHandler for SyncKnowledgeHandler<'_> {
    type Input = SyncKnowledgeInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        SYNC_TOOL
    }

    fn description(&self) -> &'static str {
        "Index what your Atlassian account can read."
    }

    #[expect(
        clippy::unused_async_trait_impl,
        reason = "the trait method is async; starting a run spawns and returns immediately"
    )]
    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        start_sync(self.ingestor, self.headers, &principal, input).map_err(|e| to_mcp_error(&e))
    }
}

pub(super) fn start_sync(
    ingestor: &Arc<Ingestor>,
    headers: Option<&HeaderMap>,
    principal: &Principal,
    input: SyncKnowledgeInput,
) -> Result<(CliArtifact, String), KnowledgeBankError> {
    let empty = HeaderMap::new();
    let config = ingestor.config();
    let mut sources = input.source_types.unwrap_or_default();
    for id in &sources {
        if !config.sources.contains_key(id) {
            return Err(KnowledgeBankError::UnknownSource(id.clone()));
        }
    }
    let credentials = if config.requires_atlassian(&sources) {
        match AtlassianCredentials::from_headers(headers.unwrap_or(&empty), &config.atlassian) {
            Ok(credentials) => Some(credentials),
            Err(e) if sources.is_empty() && !config.files_source_ids().is_empty() => {
                tracing::info!(user_id = %principal.user_id, reason = %e, "no Atlassian credentials; syncing files categories only");
                sources = config.files_source_ids();
                None
            },
            Err(e) => return Err(e),
        }
    } else {
        None
    };
    let with_atlassian = credentials.is_some();
    if ingestor.is_running_for(&principal.user_id) {
        return Err(KnowledgeBankError::Backend(
            "your knowledge sync is already in progress".to_owned(),
        ));
    }
    let options = RunOptions {
        rebuild: input.rebuild,
        sources: sources.clone(),
    };
    let runner = Arc::clone(ingestor);
    let caller = principal.clone();
    tracing::info!(user_id = %principal.user_id, rebuild = input.rebuild, "knowledge sync requested");
    tokio::spawn(async move {
        match runner.run(&caller, credentials, &options).await {
            Ok(report) => tracing::info!(
                user_id = %caller.user_id,
                duration_ms = report.duration_ms,
                errors = report.has_errors(),
                "requested knowledge sync finished"
            ),
            Err(e) => {
                tracing::error!(user_id = %caller.user_id, error = %e, "requested knowledge sync failed");
            },
        }
    });
    Ok(format_sync_started(&sources, input.rebuild, with_atlassian))
}
