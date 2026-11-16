//! The handler shared by every tool that needs only the caller's Graph client.
//!
//! The parsed input goes to one async function and its reply comes back
//! through core's executor, like every other bundled server's tools.
//!
//! [`graph_tools!`] declares a domain's tools from one table, so each tool's
//! name, function, input, reply and description are written once and feed
//! both the tool list and the dispatch.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::{McpOutputSchema, McpToolHandler};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::reply;

use super::reply::{GraphItem, GraphList};
use super::{ToolKind, spec};
use crate::graph::{GraphClient, GraphError};

pub type GraphFuture<'g, O> = Pin<Box<dyn Future<Output = Result<O, GraphError>> + Send + 'g>>;

pub type GraphRun<I, O> = for<'g> fn(&'g GraphClient, I) -> GraphFuture<'g, O>;

/// A tool reply and the one-line summary core's executor records for it.
pub trait ToolReply: Serialize + JsonSchema + McpOutputSchema + Send {
    fn summary(&self) -> String;
}

impl ToolReply for GraphList {
    fn summary(&self) -> String {
        Self::summary(self)
    }
}

impl ToolReply for GraphItem {
    fn summary(&self) -> String {
        Self::summary(self)
    }
}

/// A tool served by one Graph function. `graph` is `None` only while the
/// tool list is built; a call always carries the caller's client.
pub struct GraphTool<'a, I, O> {
    pub name: &'static str,
    pub description: &'static str,
    pub graph: Option<&'a GraphClient>,
    pub run: GraphRun<I, O>,
}

impl<I, O> fmt::Debug for GraphTool<'_, I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphTool")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl<I, O> McpToolHandler for GraphTool<'_, I, O>
where
    I: DeserializeOwned + JsonSchema + Send,
    O: ToolReply,
{
    type Input = I;
    type Output = O;

    fn tool_name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn read_only(&self) -> bool {
        spec(self.name).is_some_and(|spec| spec.kind == ToolKind::Read)
    }

    async fn handle(
        &self,
        input: I,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(O, String), McpError> {
        let graph = self
            .graph
            .ok_or_else(|| reply::failure("the Graph client is not attached"))?;
        let output = (self.run)(graph, input).await?;
        let summary = output.summary();
        Ok((output, summary))
    }
}

// Why: core's `McpToolHandler` carries an associated input and output type,
// so a domain's tools cannot share one list of trait objects; the macro
// writes the definition list and the dispatch match from one table instead.
macro_rules! graph_tools {
    ($($name:literal => $run:path, $input:ty, $output:ty, $description:expr;)+) => {
        pub(super) fn definitions(
            server: &::systemprompt::identifiers::McpServerId,
        ) -> Vec<::rmcp::model::Tool> {
            use ::systemprompt::mcp::McpToolHandler as _;
            vec![$(
                $crate::tools::graph_tool::GraphTool::<$input, $output> {
                    name: $name,
                    description: $description,
                    graph: None,
                    run: |graph, input| Box::pin($run(graph, input)),
                }
                .tool_definition(server),
            )+]
        }

        pub(super) async fn dispatch(
            call: &$crate::tools::ToolCall<'_>,
        ) -> Option<Result<::rmcp::model::CallToolResult, ::rmcp::ErrorData>> {
            let graph = Some(call.graph);
            let result = match call.request.name.as_ref() {
                $($name => {
                    let tool = $crate::tools::graph_tool::GraphTool::<$input, $output> {
                        name: $name,
                        description: $description,
                        graph,
                        run: |graph, input| Box::pin($run(graph, input)),
                    };
                    Box::pin(call.executor.execute(&tool, call.request, call.context, call.client))
                        .await
                },)+
                _ => return None,
            };
            Some(result)
        }
    };
}

pub(crate) use graph_tools;
