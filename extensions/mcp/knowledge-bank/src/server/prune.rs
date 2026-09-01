//! `prune_knowledge`: the administrator's broom for the shared index.

use super::format::format_prune_report;
use super::tool::{require_admin, to_mcp_error};
use crate::maintenance::{DEFAULT_STALE_DAYS, PruneOptions, Pruner};
use crate::principal::Principal;
use crate::tools::{PRUNE_TOOL, PruneKnowledgeInput};
use rmcp::ErrorData as McpError;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

pub(super) struct PruneKnowledgeHandler<'a> {
    pub(super) pruner: &'a Pruner,
}

impl McpToolHandler for PruneKnowledgeHandler<'_> {
    type Input = PruneKnowledgeInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        PRUNE_TOOL
    }

    fn description(&self) -> &'static str {
        "Prune stale grants and orphan documents (administrators only)."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        require_admin(&principal, PRUNE_TOOL).map_err(|e| to_mcp_error(&e))?;
        let options = PruneOptions {
            stale_days: input.stale_days.unwrap_or(DEFAULT_STALE_DAYS).max(1),
            dry_run: input.dry_run,
        };
        tracing::info!(user_id = %principal.user_id, ?options, "knowledge prune requested");
        self.pruner
            .prune(options)
            .await
            .map(|report| format_prune_report(&report))
            .map_err(|e| to_mcp_error(&e))
    }
}
