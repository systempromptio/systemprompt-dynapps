//! Atlassian reads through the local authenticated MCP gateway, never shared
//! credentials.

use super::{failure, invalid};
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation, Tool};
use rmcp::service::RunningService;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use rmcp::{RoleClient, ServiceExt};
// JSON: Atlassian MCP responses contain heterogeneous external tool payloads.
use serde_json::Value;
use systemprompt::config::ProfileBootstrap;
use systemprompt::mcp::services::client::HttpClientWithContext;
use systemprompt::models::execution::context::RequestContext;

pub struct Atlassian {
    client: RunningService<RoleClient, ClientInfo>,
    tools: Vec<Tool>,
}

impl Atlassian {
    pub(super) async fn connect(
        token: &str,
        context: &RequestContext,
    ) -> Result<Self, rmcp::ErrorData> {
        let profile =
            ProfileBootstrap::get().map_err(|_error| failure("Active profile unavailable"))?;
        let url = format!(
            "http://127.0.0.1:{}/api/v1/mcp/atlassian/mcp",
            profile.server.port
        );
        Self::connect_gateway(&url, token, context).await
    }

    pub async fn connect_gateway(
        url: &str,
        token: &str,
        context: &RequestContext,
    ) -> Result<Self, rmcp::ErrorData> {
        let config = StreamableHttpClientTransportConfig::with_uri(url.to_owned())
            .auth_header(token.to_owned());
        let transport = StreamableHttpClientTransport::with_client(
            HttpClientWithContext::new(context.clone()),
            config,
        );
        let info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("astound-admin-reports", "1.0.0"),
        );
        let client = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            info.serve(transport),
        )
        .await
        .map_err(|_error| failure("Atlassian connection timed out"))?
        .map_err(|_error| {
            failure("Atlassian connection failed; check your linked account and connector access")
        })?;
        let tools =
            tokio::time::timeout(std::time::Duration::from_secs(30), client.list_all_tools())
                .await
                .map_err(|_error| failure("Atlassian tool discovery timed out"))?
                .map_err(|_error| failure("Atlassian tools unavailable"))?;
        Ok(Self { client, tools })
    }

    pub async fn read(&self, names: &[&str], args: Value) -> Result<Value, rmcp::ErrorData> {
        if let Some(name) = names
            .iter()
            .find(|name| self.tools.iter().any(|t| t.name == **name))
        {
            return self.call(name, args).await;
        }
        Err(failure(format!(
            "Atlassian does not expose {}; enable the complete tool catalog (?tools=all) and check account permissions",
            names.join(" or ")
        )))
    }

    async fn call(&self, name: &str, args: Value) -> Result<Value, rmcp::ErrorData> {
        let mut params = CallToolRequestParams::new(name.to_owned());
        params.arguments = args.as_object().cloned();
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.client.call_tool(params),
        )
        .await
        .map_err(|_error| failure(format!("Atlassian {name} timed out")))?
        .map_err(|_error| {
            failure(format!(
                "Atlassian {name} failed; check permissions and connector status"
            ))
        })?;
        if response.is_error == Some(true) {
            return Err(failure(format!("Atlassian rejected {name}")));
        }
        if let Some(data) = response.structured_content {
            return Ok(data);
        }
        for content in response.content {
            if let Some(text) = content.as_text()
                && let Ok(data) = serde_json::from_str(&text.text)
            {
                return Ok(data);
            }
        }
        Err(failure(format!(
            "Atlassian {name} returned no structured JSON"
        )))
    }
}

pub(super) fn identifier(value: &str) -> Result<(), rmcp::ErrorData> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(invalid("Invalid Atlassian identifier"));
    }
    Ok(())
}

pub(super) fn rows(value: &Value, keys: &[&str]) -> Result<Vec<Value>, rmcp::ErrorData> {
    if let Some(rows) = value.as_array() {
        return Ok(rows.clone());
    }
    for key in keys {
        if let Some(rows) = value.get(*key).and_then(Value::as_array) {
            return Ok(rows.clone());
        }
    }
    Err(failure("Atlassian response has no supported record list"))
}

impl std::fmt::Debug for Atlassian {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Atlassian")
            .field("tools", &self.tools.len())
            .finish_non_exhaustive()
    }
}
