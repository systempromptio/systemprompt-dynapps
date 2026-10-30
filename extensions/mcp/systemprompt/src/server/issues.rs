//! Core's issue report tools, served by the `systemprompt` server beside its
//! own tools.
//!
//! The tools are built once at startup from the application context. An
//! instance with `files.issues.enabled` off gets no issue tools at all rather
//! than four tools that refuse every call.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::{ClientProfile, McpToolExecutor};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::system::AppContext;
use systemprompt::system::issues::mcp::{IssuesMcpTools, IssuesToolCall};

/// The issue tools, when this instance takes issue reports.
#[derive(Clone, Debug, Default)]
pub struct IssueTools(Option<IssuesMcpTools>);

impl IssueTools {
    pub fn from_context(ctx: &AppContext, service_id: McpServerId) -> Self {
        match IssuesMcpTools::from_context(ctx, service_id) {
            Ok(tools) => Self(Some(tools)),
            Err(error) => {
                tracing::info!(%error, "issue report tools omitted");
                Self(None)
            },
        }
    }

    pub(super) fn list(&self) -> Vec<Tool> {
        self.0
            .as_ref()
            .map_or_else(Vec::new, IssuesMcpTools::list_tools)
    }

    pub(super) async fn call(
        &self,
        executor: &McpToolExecutor,
        request: &CallToolRequestParams,
        context: &RequestContext,
        client: &ClientProfile,
    ) -> Option<Result<CallToolResult, McpError>> {
        let tools = self.0.as_ref()?;
        if !IssuesMcpTools::tool_names().contains(&request.name.as_ref()) {
            return None;
        }
        Some(
            tools
                .call(IssuesToolCall {
                    executor,
                    request,
                    context,
                    client,
                })
                .await,
        )
    }
}
