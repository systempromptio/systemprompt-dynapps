//! Each tool as a core [`McpToolHandler`], run through core's executor so
//! input parsing, output shaping and the tool-usage ledger are the same as
//! every other bundled server's.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::{ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::reply;

use super::mail::{self, LIST_MAIL_DESCRIPTION, ListMailInput};
use super::reply::GraphList;
use crate::graph::GraphClient;

/// One authenticated call: the request, its caller and the caller's Graph
/// client.
#[derive(Debug)]
pub struct ToolCall<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub context: &'a RequestContext,
    pub client: &'a ClientProfile,
    pub graph: &'a GraphClient,
}

fn graph<'a>(graph: Option<&'a GraphClient>) -> Result<&'a GraphClient, McpError> {
    graph.ok_or_else(|| reply::failure("the Graph client is not attached"))
}

struct ListMail<'a> {
    graph: Option<&'a GraphClient>,
}

impl McpToolHandler for ListMail<'_> {
    type Input = ListMailInput;
    type Output = GraphList;

    fn tool_name(&self) -> &'static str {
        "d365_list_mail"
    }

    fn description(&self) -> &'static str {
        LIST_MAIL_DESCRIPTION
    }

    fn read_only(&self) -> bool {
        true
    }

    async fn handle(
        &self,
        input: ListMailInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(GraphList, String), McpError> {
        let page = mail::list_mail(graph(self.graph)?, &input).await?;
        let output = GraphList::from(page);
        let summary = output.summary();
        Ok((output, summary))
    }
}

pub(super) fn definitions(server: &McpServerId) -> Vec<Tool> {
    vec![ListMail { graph: None }.tool_definition(server)]
}

async fn run<H: McpToolHandler>(
    call: &ToolCall<'_>,
    handler: &H,
) -> Result<CallToolResult, McpError> {
    Box::pin(
        call.executor
            .execute(handler, call.request, call.context, call.client),
    )
    .await
}

pub async fn dispatch(call: &ToolCall<'_>) -> Result<CallToolResult, McpError> {
    let graph = Some(call.graph);
    match call.request.name.as_ref() {
        "d365_list_mail" => run(call, &ListMail { graph }).await,
        other => Err(reply::invalid(format!("Unknown tool: {other}"))),
    }
}
