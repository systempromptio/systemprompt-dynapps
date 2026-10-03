//! Stores an oversize CLI result whole through the artifact narrow waist.
//!
//! The executor already stores every tool result as the artifact of its
//! execution, but only what the handler returns — and what the handler
//! returns is what goes to the model. A result over the wire bound is
//! therefore ingested here first, as its own execution with no client
//! `tool_use_id`, so the current execution's artifact stays the short pointer
//! the model receives while the full body lives at `/admin/artifacts/{id}`,
//! scanned, content-addressed and linked to the same session and trace.

use chrono::Utc;
use rmcp::model::CallToolResult;
use serde_json::{Value as JsonValue, json};
use systemprompt::identifiers::{McpExecutionId, McpServerId, McpToolName};
use systemprompt::mcp::{ArtifactIngest, IngestRequest, MAX_PAYLOAD_BYTES};
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::mcp::ExecutionSource;

use crate::bounds::{StoredOverflow, row_count};
use crate::error::SystempromptToolError;

pub(super) struct OverflowStore<'a> {
    pub ingest: &'a ArtifactIngest,
    pub server_name: &'a McpServerId,
    pub ctx: &'a RequestContext,
    pub command: &'a str,
    pub exec_id: &'a McpExecutionId,
}

impl OverflowStore<'_> {
    pub(super) async fn store(
        &self,
        artifact: &CliArtifact,
        bytes: usize,
    ) -> Result<StoredOverflow, SystempromptToolError> {
        let mut wire = CallToolResult::success(Vec::new());
        wire.structured_content = Some(typed_body(artifact)?);
        let outcome = self
            .ingest
            .ingest(IngestRequest {
                result: wire,
                tool_name: McpToolName::new("systemprompt"),
                server_name: Some(self.server_name.clone()),
                ai_tool_call_id: None,
                mcp_execution_id: None,
                ctx: self.ctx.clone(),
                skill: None,
                source: ExecutionSource::InProcess,
                started_at: Some(Utc::now()),
                input: Some(json!({
                    "command": self.command,
                    "overflow_of": self.exec_id.as_str(),
                })),
            })
            .await?;
        Ok(StoredOverflow {
            artifact_id: outcome.artifact_id,
            bytes,
            rows: row_count(artifact),
            digest_only: bytes > MAX_PAYLOAD_BYTES,
        })
    }
}

// Why: core's ingest types a stored body by its `x-artifact-type`; without it a
// table lands as a generic tool result.
// JSON: variable-shape serialized CliArtifact
fn typed_body(artifact: &CliArtifact) -> Result<JsonValue, SystempromptToolError> {
    let mut value = serde_json::to_value(artifact)?;
    if let Some(map) = value.as_object_mut()
        && !map.contains_key("x-artifact-type")
    {
        map.insert(
            "x-artifact-type".to_owned(),
            JsonValue::String(artifact.artifact_type_str().to_owned()),
        );
    }
    Ok(value)
}
