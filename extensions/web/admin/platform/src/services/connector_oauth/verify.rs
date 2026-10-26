//! Read-only provider identity and MCP protocol verification before connection
//! use.

use super::Grant;
use super::response::{body, tool_error};
use super::transport::client;
use serde_json::{Value, json};
use std::time::Instant;
use systemprompt::models::net::validate_outbound_url;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

async fn rpc(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    http: &reqwest::Client,
    grant: &Grant,
    session: &mut Option<String>,
    payload: Value,
) -> AdminResult<Value> {
    let mut request = http
        .post(validate_outbound_url(
            &grant.provider.endpoint(&services.config()),
        )?)
        .header(
            "Authorization",
            format!("{} {}", grant.authorization_scheme, grant.access_token),
        )
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-03-26")
        .json(&payload);
    if let Some(id) = session.as_ref() {
        request = request.header("Mcp-Session-Id", id);
    }
    let response = request
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(target: "connector_oauth", error = %error.without_url(), "MCP verification unavailable");
            AdminError::Upstream("MCP verification unavailable".into())
        })?;
    tracing::info!(target: "connector_oauth", user_id = %grant.user, provider = grant.provider.slug(),
        stage = payload.get("method").and_then(|value| value.as_str()).unwrap_or("request"),
        status = response.status().as_u16(), "MCP verification responded");
    if let Some(id) = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
    {
        *session = Some(id.into());
    }
    if payload.get("id").is_none() {
        return if response.status().is_success() {
            Ok(Value::Null)
        } else {
            Err(AdminError::Upstream(
                "MCP initialization notification rejected".into(),
            ))
        };
    }
    let value = body(response).await?;
    if value.get("error").is_some()
        || value.pointer("/result/isError").and_then(Value::as_bool) == Some(true)
    {
        return Err(tool_error(grant, &payload, &value));
    }
    value
        .get("result")
        .cloned()
        .ok_or_else(|| AdminError::Upstream("MCP response has no result".into()))
}

async fn identity(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    http: &reqwest::Client,
    grant: &mut Grant,
) -> AdminResult<()> {
    grant.resource_name = grant.provider.endpoint(&services.config());
    super::userinfo::generic_identity(services, http, grant).await
}

pub async fn verify(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
) -> AdminResult<()> {
    verify_with_client(services, grant, &client()?).await
}

pub async fn verify_with_client(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    http: &reqwest::Client,
) -> AdminResult<()> {
    let mut report = VerificationReport::for_provider(grant.provider.slug());
    verify_reporting(services, grant, http, &mut report).await
}

pub async fn verify_reporting(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    http: &reqwest::Client,
    report: &mut VerificationReport,
) -> AdminResult<()> {
    let mut probe_session = None;
    let result = handshake(services, grant, http, &mut probe_session, report).await;
    if let Some(session) = probe_session {
        let endpoint = validate_outbound_url(&grant.provider.endpoint(&services.config()))?;
        let _cleanup_result = http
            .delete(endpoint)
            .header("Mcp-Session-Id", session)
            .header(
                "Authorization",
                format!("{} {}", grant.authorization_scheme, grant.access_token),
            )
            .send()
            .await;
    }
    result
}

async fn handshake(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    http: &reqwest::Client,
    session: &mut Option<String>,
    report: &mut VerificationReport,
) -> AdminResult<()> {
    let started = Instant::now();
    let outcome = async {
        rpc(
            services,
            http,
            grant,
            session,
            json!({"jsonrpc":"2.0", "id":1, "method":"initialize",
            "params":{"protocolVersion":"2025-03-26", "capabilities":{},
                "clientInfo":{"name":"Dynapps connection verification", "version":"1.0"}}}),
        )
        .await?;
        rpc(
            services,
            http,
            grant,
            session,
            json!({"jsonrpc":"2.0", "method":"notifications/initialized"}),
        )
        .await?;
        Ok(format!(
            "MCP session open at {}",
            host(&grant.provider.endpoint(&services.config()))
        ))
    }
    .await;
    report.record("initialize", started, &outcome);
    outcome?;

    let started = Instant::now();
    let outcome = async {
        let tools = rpc(
            services,
            http,
            grant,
            session,
            json!({"jsonrpc":"2.0", "id":2, "method":"tools/list"}),
        )
        .await?;
        let count = tools
            .get("tools")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        if count == 0 {
            return Err(AdminError::Upstream(
                "MCP server has no accessible tools".into(),
            ));
        }
        Ok(format!("{count} tools available"))
    }
    .await;
    report.record("tools", started, &outcome);
    outcome?;

    let started = Instant::now();
    let outcome = async {
        identity(services, http, grant).await?;
        Ok(
            match (
                grant.account_name.is_empty(),
                grant.resource_name.is_empty(),
            ) {
                (false, false) => {
                    format!("{} · {}", grant.account_name, host(&grant.resource_name))
                },
                (false, true) => grant.account_name.clone(),
                _ => "Identity confirmed".to_owned(),
            },
        )
    }
    .await;
    report.record("identity", started, &outcome);
    outcome.map(|_| ())
}

fn host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}
