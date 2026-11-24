//! The fake toolset the broker suite's MCP server hosts: two tools that
//! answer with who called them. Core's toolset host authenticates and
//! authorises each call before it reaches `call`, so a result here is proof
//! the call passed core's gate as the identity it names.

use std::sync::Arc;

use async_trait::async_trait;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Tool};
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::{McpToolset, ToolsetCall};
use systemprompt::system::HostContext;
use systemprompt::system::mcp_host::ToolsetBuildResult;

pub(super) const TOOLSET: &str = "artifacts-fake";
pub(super) const ECHO: &str = "artifacts_fake_echo";
pub(super) const SECRET: &str = "artifacts_fake_secret";

#[derive(Debug)]
struct FakeTools;

#[async_trait]
impl McpToolset for FakeTools {
    fn id(&self) -> &str {
        TOOLSET
    }

    fn tools(&self) -> Vec<Tool> {
        [ECHO, SECRET]
            .into_iter()
            .map(|name| Tool::new(name, "fake", Arc::new(serde_json::Map::new())))
            .collect()
    }

    async fn call(&self, call: ToolsetCall<'_>) -> Result<CallToolResult, McpError> {
        let text = call
            .request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("text"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(CallToolResult::structured(serde_json::json!({
            "tool": call.request.name.as_ref(),
            "caller": call.context.user_id().as_str(),
            "text": text,
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
