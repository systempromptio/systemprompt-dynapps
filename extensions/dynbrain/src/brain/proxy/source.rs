//! The source reached through core's MCP client.
//!
//! [`McpSource`] calls a tool of the declared server with core's
//! [`McpClient`] and the person's request context, so core resolves the
//! person's token for the source the way it does for every external server
//! (`external_auth`); this crate signs nothing. A failure becomes a
//! [`SourceFault`]: a 401 or 403 of the source, or of the token accessor, is
//! `Refused`; a protocol error `NotFound` or `Invalid` with the source's
//! message; anything else `Unavailable`.

use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use rmcp::ServiceError;
use rmcp::model::ErrorCode;
use rmcp::service::ClientInitializeError;
use rmcp::transport::streamable_http_client::StreamableHttpError;
// JSON: tool arguments and answers are the JSON the MCP protocol carries.
use serde_json::Value;
use systemprompt::mcp::services::client::{McpClient, McpTransportError};
use systemprompt::mcp::{McpDomainError, McpServerConfig};
use systemprompt::models::CallToolResult;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::traits::{ExtensionSecret, SecretLookup, SecretStore};

use super::Source;
use crate::brain::fault::SourceFault;

const MAX_CAUSES: usize = 16;

/// The source: core's MCP client and the declared server it calls.
#[derive(Debug, Clone)]
pub struct McpSource {
    client: McpClient,
    server: McpServerConfig,
}

impl McpSource {
    pub const fn new(client: McpClient, server: McpServerConfig) -> Self {
        Self { client, server }
    }
}

#[async_trait]
impl Source for McpSource {
    async fn call(
        &self,
        context: &RequestContext,
        tool: &str,
        arguments: Value,
    ) -> Result<Value, SourceFault> {
        let result = self
            .client
            .call_tool(&self.server, tool.to_owned(), Some(arguments), context)
            .await
            .map_err(classify)?;
        decode(result)
    }
}

/// The instance's secrets as core's MCP client looks them up.
#[derive(Debug, Clone)]
pub struct StoreSecrets(pub Arc<dyn SecretStore>);

impl SecretLookup for StoreSecrets {
    fn lookup(&self, name: &str) -> Option<ExtensionSecret> {
        self.0.secret(name)
    }
}

// JSON: the structured answer, or the JSON its text carries.
pub fn decode(result: CallToolResult) -> Result<Value, SourceFault> {
    let text = result
        .content
        .iter()
        .find_map(|block| block.as_text())
        .map(|content| content.text.clone());
    if result.is_error == Some(true) {
        return Err(SourceFault::Invalid(
            text.unwrap_or_else(|| "the source refused the call".to_owned()),
        ));
    }
    if let Some(value) = result.structured_content {
        return Ok(value);
    }
    let Some(text) = text else {
        return Ok(Value::Null);
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(value) => Ok(value),
        Err(error) => {
            tracing::debug!(%error, "dynbrain: the source answered plain text");
            Ok(Value::String(text))
        },
    }
}

pub fn classify(error: McpDomainError) -> SourceFault {
    let refused = causes(&error).into_iter().find_map(refusal);
    if let Some(reason) = refused {
        tracing::warn!(%error, "dynbrain: the source refused the person");
        return SourceFault::Refused(reason);
    }
    if let Some(fault) = causes(&error).into_iter().find_map(protocol_fault) {
        return fault;
    }
    match error {
        McpDomainError::AuthRequired(_) => {
            SourceFault::Refused("the call carries no signed-in person".to_owned())
        },
        McpDomainError::ExternalAuthUnavailable { message, .. } => {
            tracing::warn!(reason = %message, "dynbrain: no source token for the person");
            SourceFault::Refused(message)
        },
        McpDomainError::ExternalAccountNotConnected { .. } => {
            SourceFault::Refused("the person has no account at the source".to_owned())
        },
        other => {
            tracing::warn!(error = %other, "dynbrain: the source could not be reached");
            SourceFault::Unavailable(Box::new(other))
        },
    }
}

fn causes(error: &McpDomainError) -> Vec<&(dyn Error + 'static)> {
    let mut found: Vec<&(dyn Error + 'static)> = Vec::new();
    let mut cause: Option<&(dyn Error + 'static)> = Some(error);
    while let Some(current) = cause {
        found.push(current);
        if found.len() >= MAX_CAUSES {
            break;
        }
        cause = next(current);
    }
    found
}

// Why: rmcp keeps the transport error of a failed handshake and the client
// error of the HTTP layer in fields that are not `source()`, and core's error
// hands its rmcp causes over still boxed, so the chain is followed through
// them by hand.
fn next<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a (dyn Error + 'static)> {
    if let Some(boxed) = error.downcast_ref::<Box<ClientInitializeError>>() {
        return Some(boxed.as_ref());
    }
    if let Some(boxed) = error.downcast_ref::<Box<ServiceError>>() {
        return Some(boxed.as_ref());
    }
    if let Some(ClientInitializeError::TransportError { error, .. }) =
        error.downcast_ref::<ClientInitializeError>()
    {
        return Some(error);
    }
    if let Some(StreamableHttpError::Client(inner)) =
        error.downcast_ref::<StreamableHttpError<McpTransportError>>()
    {
        return Some(inner);
    }
    if let Some(ServiceError::TransportSend(inner)) = error.downcast_ref::<ServiceError>() {
        return Some(inner);
    }
    error.source()
}

fn refusal(error: &(dyn Error + 'static)) -> Option<String> {
    match error.downcast_ref::<McpTransportError>()? {
        McpTransportError::AuthorizationRequired { reason, .. } => Some(reason.clone()),
        McpTransportError::Http(http) => {
            matches!(http.status().map(|status| status.as_u16()), Some(401 | 403))
                .then(|| "the source refused the person's token".to_owned())
        },
        _ => None,
    }
}

fn protocol_fault(error: &(dyn Error + 'static)) -> Option<SourceFault> {
    let service = error
        .downcast_ref::<ServiceError>()
        .or_else(|| error.downcast_ref::<Box<ServiceError>>().map(AsRef::as_ref))?;
    let ServiceError::McpError(data) = service else {
        return None;
    };
    let message = data.message.clone().into_owned();
    if data.code == ErrorCode::RESOURCE_NOT_FOUND {
        return Some(SourceFault::NotFound(message));
    }
    Some(SourceFault::Invalid(message))
}
