//! `POST /api/v1/artifacts/{document_id}/mcp/call`: one tool call a Drive
//! page asks for, run as the person viewing it.
//!
//! First the request itself: the viewer (`viewer`), a well-formed document
//! id and a body naming a server and a tool. A request that fails there is
//! answered without an audit row, because it names nothing to record. Then,
//! audited: the viewer's `view` on the document (asked of core's document
//! service as the viewer, so a document they cannot see is a 404 exactly as
//! in Drive), the input, the call's admission and the call through core's
//! MCP proxy. The row holds the outcome and the duration, never the input or
//! the result.

use std::time::Instant;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
// JSON: MCP-protocol boundary, a tool's arguments and result are schema-less.
use serde_json::Value;
use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::{DocumentId, McpServerId, McpToolName, TraceId};

use super::ArtifactsBroker;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::proxy::ProxiedCall;
use super::viewer::ArtifactViewer;
use crate::repositories::artifact_tool_calls::{self, NewToolCall};

const OK: &str = "ok";
const MAX_NAME_BYTES: usize = 128;

// JSON: MCP-protocol boundary, the tool's arguments as the page sends them.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallBody {
    server: String,
    tool: String,
    #[serde(default)]
    input: Option<Value>,
}

struct Target {
    document: DocumentId,
    server: McpServerId,
    tool: McpToolName,
    input: Option<Value>,
}

pub(super) async fn call(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ArtifactCallError> {
    let started = Instant::now();
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let target = target(raw_document, &body)?;
    let answer = act_as(DocumentActor::Human, run(&broker, &viewer, &target)).await;
    let outcome = answer
        .as_ref()
        .map_or_else(|error| error.code().as_str(), result_outcome);
    let recorded = artifact_tool_calls::record(
        &broker.inner.parts.pool,
        &NewToolCall {
            user: &viewer.user.id,
            document: &target.document,
            server: &target.server,
            tool: &target.tool,
            outcome,
            duration_ms: i32::try_from(started.elapsed().as_millis()).unwrap_or(i32::MAX),
        },
    )
    .await;
    if let Err(error) = recorded {
        tracing::error!(%error, outcome, "artifact tool call ran but its audit row was not written");
    }
    answer.map(|result| Json(result).into_response())
}

fn target(raw_document: String, body: &[u8]) -> Result<Target, ArtifactCallError> {
    let document = DocumentId::try_new(raw_document).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })?;
    let body: CallBody = serde_json::from_slice(body).map_err(|error| {
        tracing::debug!(%error, "artifact tool call body did not parse");
        ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "the body is `{server, tool, input}`",
        )
    })?;
    if !is_name(&body.server) || !is_name(&body.tool) {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "`server` and `tool` are MCP names",
        ));
    }
    Ok(Target {
        document,
        server: McpServerId::new(body.server),
        tool: McpToolName::new(body.tool),
        input: body.input,
    })
}

// Why: MCP's tool-name alphabet also covers every server key this instance
// declares, and it bounds what an audit row can hold.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_BYTES
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

// JSON: MCP-protocol boundary, the tool result as core shapes it.
async fn run(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    target: &Target,
) -> Result<Value, ArtifactCallError> {
    ensure_viewable(broker, viewer, &target.document).await?;
    let arguments = match &target.input {
        None => Value::Object(serde_json::Map::new()),
        Some(input @ Value::Object(_)) => input.clone(),
        Some(_) => {
            return Err(ArtifactCallError::new(
                ArtifactCallCode::BadRequest,
                "`input` is a JSON object",
            ));
        },
    };
    broker.admit(&target.server)?;
    broker
        .inner
        .proxy
        .call_tool(ProxiedCall {
            server: &target.server,
            tool: &target.tool,
            arguments,
            bearer: viewer.credential(),
        })
        .await
}

async fn ensure_viewable(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    document: &DocumentId,
) -> Result<(), ArtifactCallError> {
    let lookup = broker
        .inner
        .parts
        .documents
        .0
        .get(&viewer.user, document, &TraceId::generate())
        .await;
    match lookup {
        Ok(_) => Ok(()),
        Err(
            DocumentsError::Forbidden { .. }
            | DocumentsError::NotFound { .. }
            | DocumentsError::Trashed { .. },
        ) => Err(ArtifactCallError::new(
            ArtifactCallCode::NotFound,
            "document not found",
        )),
        Err(error) => {
            tracing::error!(%error, "the document service failed an artifact tool call's view check");
            Err(ArtifactCallError::new(
                ArtifactCallCode::Internal,
                "the document could not be checked",
            ))
        },
    }
}

// JSON: MCP-protocol boundary, `isError` on the tool result.
fn result_outcome(result: &Value) -> &'static str {
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        ArtifactCallCode::ToolError.as_str()
    } else {
        OK
    }
}
