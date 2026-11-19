//! The handler for tools that move files. Besides the caller's Graph client
//! they get the shared file layer and the call's identity (request context,
//! execution id, hop token), so a `FileRef` resolves only as the caller and
//! a download is kept as one of the caller's own documents.
//!
//! [`file_tools!`] declares them from one table, like `graph_tools!`, and
//! writes `file_definitions` and `file_dispatch` so a domain can hold both.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use systemprompt::identifiers::{JwtToken, McpExecutionId};
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::files::{FileCall, McpFiles};
use systemprompt_mcp_shared::reply;

use super::graph_tool::ToolReply;
use super::{ToolKind, spec};
use crate::graph::GraphClient;

pub(super) type FileFuture<'g, O> = Pin<Box<dyn Future<Output = Result<O, McpError>> + Send + 'g>>;

pub(super) type FileRun<I, O> =
    for<'g> fn(&'g GraphClient, &'g McpFiles, FileCall<'g>, I) -> FileFuture<'g, O>;

#[derive(Clone, Copy)]
pub(super) struct FileHost<'a> {
    pub(super) graph: &'a GraphClient,
    pub(super) files: &'a McpFiles,
    pub(super) hop: Option<&'a JwtToken>,
}

impl fmt::Debug for FileHost<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileHost")
            .field("graph", self.graph)
            .field("hop", &self.hop.is_some())
            .finish_non_exhaustive()
    }
}

pub(super) struct FileTool<'a, I, O> {
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) host: Option<FileHost<'a>>,
    pub(super) run: FileRun<I, O>,
}

impl<I, O> fmt::Debug for FileTool<'_, I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileTool")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl<I, O> McpToolHandler for FileTool<'_, I, O>
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
        context: &RequestContext,
        execution: &McpExecutionId,
    ) -> Result<(O, String), McpError> {
        let host = self
            .host
            .ok_or_else(|| reply::failure("the Graph client is not attached"))?;
        let call = FileCall {
            ctx: context,
            tool: self.name,
            exec_id: execution,
            hop: host.hop,
        };
        let output = (self.run)(host.graph, host.files, call, input).await?;
        let summary = output.summary();
        Ok((output, summary))
    }
}

// Why: core's `McpToolHandler` carries an associated input and output type,
// so the file tools cannot share one list of trait objects either; the macro
// writes their definition list and dispatch match from one table.
macro_rules! file_tools {
    ($($name:literal => $run:path, $input:ty, $output:ty, $description:expr;)+) => {
        pub(super) fn file_definitions(
            server: &::systemprompt::identifiers::McpServerId,
        ) -> Vec<::rmcp::model::Tool> {
            use ::systemprompt::mcp::McpToolHandler as _;
            vec![$(
                $crate::tools::file_tool::FileTool::<$input, $output> {
                    name: $name,
                    description: $description,
                    host: None,
                    run: |graph, files, call, input| Box::pin($run(graph, files, call, input)),
                }
                .tool_definition(server),
            )+]
        }

        pub(super) async fn file_dispatch(
            call: &$crate::tools::ToolCall<'_>,
        ) -> Option<Result<::rmcp::model::CallToolResult, ::rmcp::ErrorData>> {
            let host = Some($crate::tools::file_tool::FileHost {
                graph: call.graph,
                files: call.files,
                hop: call.hop,
            });
            let result = match call.request.name.as_ref() {
                $($name => {
                    let tool = $crate::tools::file_tool::FileTool::<$input, $output> {
                        name: $name,
                        description: $description,
                        host,
                        run: |graph, files, call, input| Box::pin($run(graph, files, call, input)),
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

pub(crate) use file_tools;
