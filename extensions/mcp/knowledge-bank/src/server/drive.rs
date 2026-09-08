//! Read-only Drive inventory exposed through the governed MCP executor.

use super::tool::to_mcp_error;
use crate::drive::store;
use crate::ingest::Ingestor;
use crate::tools::DriveInventoryInput;
use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext;

pub(super) struct DriveInventoryHandler<'a> {
    pub ingestor: &'a Ingestor,
}

impl McpToolHandler for DriveInventoryHandler<'_> {
    type Input = DriveInventoryInput;
    type Output = CliArtifact;
    fn tool_name(&self) -> &'static str {
        crate::tools::DRIVE_INVENTORY_TOOL
    }
    fn description(&self) -> &'static str {
        "List Drive files, project IDs and extraction outcomes"
    }
    async fn handle(
        &self,
        input: Self::Input,
        _ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(CliArtifact, String), ErrorData> {
        if !(1..=500).contains(&input.limit) {
            return Err(ErrorData::invalid_params("limit must be 1–500", None));
        }
        if !self
            .ingestor
            .config()
            .google_drive
            .sources
            .contains_key(&input.source_type)
        {
            return Err(ErrorData::invalid_params("Unknown Drive source", None));
        }
        let rows = store::inventory(
            self.ingestor.pool(),
            &input.source_type,
            &input.after,
            input.limit,
        )
        .await
        .map_err(|e| to_mcp_error(&e))?;
        let body = serde_json::to_string_pretty(&rows)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok((
            CliArtifact::text(TextArtifact::new(body)),
            format!("{} files; use the last file_id as after", rows.len()),
        ))
    }
}
