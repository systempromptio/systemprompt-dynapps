//! Each tool as a core [`McpToolHandler`], run through core's executor so
//! input parsing, output shaping and the tool-usage ledger are the same as
//! every other bundled server's.

use dynbrain::brain::Brain;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use schemars::JsonSchema;
use serde::Serialize;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::{ClientProfile, McpOutputSchema, McpToolExecutor, McpToolHandler};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::reply;

use super::{Json, ToolSpec, run, spec};

/// What a tool answers: the Brain's own JSON, as the operation returned it.
// JSON: the reply is the operation's view serialized, so its schema is the
// view's; the tool list declares only that it is an object.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct BrainReply(#[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub Json);

impl McpOutputSchema for BrainReply {
    fn artifact_type() -> &'static str {
        "brain_reply"
    }
}

/// A tool of the server. `brain` is `None` only while the tool list is built;
/// a call always carries the Brain.
pub struct BrainTool<'a> {
    pub spec: &'static ToolSpec,
    pub brain: Option<&'a dyn Brain>,
}

impl std::fmt::Debug for BrainTool<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrainTool")
            .field("name", &self.spec.name)
            .finish_non_exhaustive()
    }
}

impl McpToolHandler for BrainTool<'_> {
    type Input = Json;
    type Output = BrainReply;

    fn tool_name(&self) -> &'static str {
        self.spec.name
    }

    fn description(&self) -> &'static str {
        self.spec.description
    }

    fn input_schema(&self) -> Json {
        (self.spec.schema)()
    }

    fn read_only(&self) -> bool {
        self.spec.kind == super::ToolKind::Read
    }

    async fn handle(
        &self,
        input: Json,
        context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(BrainReply, String), McpError> {
        let brain = self
            .brain
            .ok_or_else(|| reply::failure("the Brain is not attached"))?;
        let output = run(brain, context, self.spec.name, input).await?;
        Ok((BrainReply(output), format!("{} done", self.spec.name)))
    }
}

/// One authenticated call: the request, its caller and the Brain it runs on.
#[derive(Debug)]
pub struct ToolCall<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub context: &'a RequestContext,
    pub client: &'a ClientProfile,
    pub brain: &'a dyn Brain,
}

pub async fn dispatch(call: &ToolCall<'_>) -> Result<CallToolResult, McpError> {
    let name = call.request.name.as_ref();
    let spec = spec(name).ok_or_else(|| reply::invalid(format!("Unknown tool: {name}")))?;
    let tool = BrainTool {
        spec,
        brain: Some(call.brain),
    };
    Box::pin(
        call.executor
            .execute(&tool, call.request, call.context, call.client),
    )
    .await
}
