//! `sync_knowledge`: start the caller's run in the background. Open to every
//! signed-in user. Files categories need nothing from the caller; Confluence
//! and Jira are read through the Atlassian account the caller connected on
//! `/admin/profile`, reached via this instance's own MCP gateway with the
//! caller's bearer. The connection is attempted *before* the run starts:
//! an unscoped sync that cannot reach Atlassian degrades to the files
//! categories and says why, a sync that names an Atlassian category
//! explicitly is refused with the same reason.

use super::format::{AtlassianReach, format_sync_started};
use super::tool::to_mcp_error;
use crate::error::KnowledgeBankError;
use crate::ingest::sources::AtlassianMcp;
use crate::ingest::{Ingestor, RunOptions};
use crate::principal::Principal;
use crate::tools::{SYNC_TOOL, SyncKnowledgeInput};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

pub(super) struct SyncKnowledgeHandler<'a> {
    pub(super) ingestor: &'a Arc<Ingestor>,
    pub(super) bearer: &'a str,
}

impl McpToolHandler for SyncKnowledgeHandler<'_> {
    type Input = SyncKnowledgeInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        SYNC_TOOL
    }

    fn description(&self) -> &'static str {
        "Index what your connected Atlassian account can read."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        start_sync(self.ingestor, self.bearer, ctx, &principal, input)
            .await
            .map_err(|e| to_mcp_error(&e))
    }
}

pub(super) async fn start_sync(
    ingestor: &Arc<Ingestor>,
    bearer: &str,
    ctx: &SysRequestContext,
    principal: &Principal,
    input: SyncKnowledgeInput,
) -> Result<(CliArtifact, String), KnowledgeBankError> {
    let config = ingestor.config();
    let mut sources = input.source_types.unwrap_or_default();
    for id in &sources {
        if !config.sources.contains_key(id) {
            return Err(KnowledgeBankError::UnknownSource(id.clone()));
        }
    }
    if ingestor.is_running_for(&principal.user_id) {
        return Err(KnowledgeBankError::Backend(
            "your knowledge sync is already in progress".to_owned(),
        ));
    }
    let (atlassian, reach) = if config.requires_atlassian(&sources) {
        match AtlassianMcp::connect(bearer, ctx, &config.atlassian).await {
            Ok(connected) => (Some(connected), AtlassianReach::Connected),
            Err(e) if sources.is_empty() && !config.files_source_ids().is_empty() => {
                let reason = e.to_string();
                tracing::info!(user_id = %principal.user_id, reason = %reason, "Atlassian unreachable; syncing files categories only");
                ingestor
                    .record_skipped(&principal.user_id, &config.atlassian_source_ids(), &reason)
                    .await;
                sources = config.files_source_ids();
                (None, AtlassianReach::Skipped(reason))
            },
            Err(e) => return Err(e),
        }
    } else {
        (None, AtlassianReach::NotNeeded)
    };
    let options = RunOptions {
        rebuild: input.rebuild,
        sources: sources.clone(),
    };
    let runner = Arc::clone(ingestor);
    let caller = principal.clone();
    tracing::info!(user_id = %principal.user_id, rebuild = input.rebuild, atlassian = ?reach, "knowledge sync requested");
    tokio::spawn(async move {
        match runner.run(&caller, atlassian, &options).await {
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
    Ok(format_sync_started(&sources, input.rebuild, &reach))
}
