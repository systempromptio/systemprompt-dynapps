//! The fake toolset the broker suite's MCP server hosts: four tools that
//! answer with who called them and how many times each has run. Core's
//! toolset host authenticates and authorises each call before it reaches
//! `call`, so a result here is proof the call passed core's gate as the
//! identity it names, and the run count is proof of whether it ran at all.
//!
//! `echo` and `secret` declare `readOnlyHint: true`, `write` declares
//! nothing (so a page's call to it is a write), and `off` is a read the
//! suite's live-pages policy leaves out.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Tool, ToolAnnotations};
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::{McpToolset, ToolsetCall};
use systemprompt::system::HostContext;
use systemprompt::system::mcp_host::ToolsetBuildResult;

pub(super) const TOOLSET: &str = "artifacts-fake";
pub(super) const ECHO: &str = "artifacts_fake_echo";
pub(super) const SECRET: &str = "artifacts_fake_secret";
pub(super) const WRITE: &str = "artifacts_fake_write";
pub(super) const OFF: &str = "artifacts_fake_off";

const TOOLS: [&str; 4] = [ECHO, SECRET, WRITE, OFF];
static RUNS: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];

pub(super) fn runs(tool: &str) -> u64 {
    TOOLS
        .iter()
        .position(|name| *name == tool)
        .map_or(0, |index| RUNS[index].load(Ordering::SeqCst))
}

#[derive(Debug)]
struct FakeTools;

fn schema() -> Arc<serde_json::Map<String, serde_json::Value>> {
    let serde_json::Value::Object(schema) = serde_json::json!({
        "type": "object",
        "properties": { "text": { "type": "string" } },
    }) else {
        unreachable!("a JSON object literal")
    };
    Arc::new(schema)
}

#[async_trait]
impl McpToolset for FakeTools {
    fn id(&self) -> &str {
        TOOLSET
    }

    fn tools(&self) -> Vec<Tool> {
        TOOLS
            .into_iter()
            .map(|name| {
                let tool = Tool::new(name, "fake", schema());
                if name == WRITE {
                    tool
                } else {
                    tool.annotate(ToolAnnotations::new().read_only(true))
                }
            })
            .collect()
    }

    async fn call(&self, call: ToolsetCall<'_>) -> Result<CallToolResult, McpError> {
        let name = call.request.name.as_ref();
        let runs = TOOLS
            .iter()
            .position(|tool| *tool == name)
            .map_or(0, |index| RUNS[index].fetch_add(1, Ordering::SeqCst) + 1);
        let text = call
            .request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("text"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(CallToolResult::structured(serde_json::json!({
            "tool": name,
            "caller": call.context.user_id().as_str(),
            "text": text,
            "runs": runs,
        })))
    }
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "core's `ToolsetFactory` fixes the factory's signature"
)]
fn build(_ctx: &HostContext, _server: &McpServerId) -> ToolsetBuildResult {
    Ok(Some(Box::new(FakeTools)))
}

systemprompt::system::submit_toolset!(TOOLSET, build);
