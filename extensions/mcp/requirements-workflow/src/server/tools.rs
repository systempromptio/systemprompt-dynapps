//! The tool catalogue and the per-call handler the executor drives.
use super::WorkflowServer;
use rmcp::ErrorData as McpError;
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysContext;

const TOOLS: [&str; 10] = [
    "refresh_requirements_evidence",
    "start_requirements_workflow",
    "get_requirements_workflow",
    "submit_context",
    "submit_specification",
    "set_requirements_destination",
    "validate_requirements_phase",
    "request_requirements_review",
    "publish_requirements",
    "verify_requirements_publication",
];

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolInput {
    pub workflow_id: Option<String>,
    pub expected_revision: Option<i64>,
    pub name: Option<String>,
    pub project: Option<String>,
    pub delivery_scope: Option<String>,
    pub phase: Option<String>,
    pub gate: Option<String>,
    pub files: Option<BTreeMap<String, String>>,
    pub sources: Option<Vec<crate::model::SourceSelection>>,
    pub destination: Option<crate::model::Destination>,
}

pub(super) fn lookup(name: &str) -> Result<&'static str, McpError> {
    TOOLS
        .into_iter()
        .find(|n| *n == name)
        .ok_or_else(|| McpError::invalid_params("Unknown workflow tool", None))
}

pub(super) fn catalogue() -> Result<Vec<Tool>, McpError> {
    let schema = serde_json::to_value(schemars::schema_for!(ToolInput))
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let schema = Arc::new(schema.as_object().cloned().unwrap_or_default());
    Ok(TOOLS
        .into_iter()
        .map(|name| {
            let mut tool = Tool::default();
            tool.name = name.into();
            tool.description = Some(
                format!(
                    "{}; strict server-side phase and approval gates. Read workflow status for current revision.",
                    name.replace('_', " ")
                )
                .into(),
            );
            tool.input_schema = Arc::clone(&schema);
            tool
        })
        .collect())
}

pub(super) struct Handler<'a> {
    pub(super) server: &'a WorkflowServer,
    pub(super) name: &'static str,
    pub(super) bearer: &'a str,
}

impl McpToolHandler for Handler<'_> {
    type Input = ToolInput;
    type Output = CliArtifact;
    fn tool_name(&self) -> &'static str {
        self.name
    }
    fn description(&self) -> &'static str {
        "Advance an authenticated, versioned requirements workflow"
    }
    async fn handle(
        &self,
        input: ToolInput,
        ctx: &SysContext,
        _id: &McpExecutionId,
    ) -> Result<(CliArtifact, String), McpError> {
        let result = self
            .execute(input, ctx)
            .await
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let body = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok((
            CliArtifact::text(TextArtifact::new(&body).with_title("Requirements workflow")),
            self.name.to_owned(),
        ))
    }
}
