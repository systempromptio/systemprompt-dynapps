//! `POST /api/v1/artifacts/{document_id}/mcp/call`: one tool call a Drive
//! page asks for, run as the person viewing it.
//!
//! First the request itself: the viewer (`viewer`), a well-formed document
//! id and a body `{server, tool, input?, approval?, cache?}`. A request that
//! fails there is answered without an audit row, because it names nothing to
//! record. Then, audited: the page and the call's admission (`admit`), the
//! viewer's cache for a read (`cache`), and one session with core's MCP
//! proxy as the viewer: the server's tool list says whether the tool is a
//! read (`readOnlyHint: true`) or a write, and a write runs only on an
//! approval of exactly this call, which the call spends. The row holds the
//! outcome (`ok`, `cached` or the error key) and the duration, never the
//! input or the result.

use std::time::Instant;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
// JSON: MCP-protocol boundary, a tool's arguments and result are schema-less.
use serde_json::Value;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::{DocumentId, McpServerId, McpToolName};

use super::ArtifactsBroker;
use super::cache::{self, CacheOption, Freshness};
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::manifest::is_name;
use super::viewer::ArtifactViewer;
use crate::repositories::artifact_approvals::{self, ArtifactApproval};
use crate::repositories::artifact_call_cache::{self, CachedCallKey, CachedCallScope};
use crate::repositories::artifact_tool_calls::{self, NewToolCall};

const OK: &str = "ok";
const CACHED: &str = "cached";

// JSON: MCP-protocol boundary, the tool's arguments as the page sends them.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallBody {
    server: String,
    tool: String,
    #[serde(default)]
    input: Option<Value>,
    #[serde(default)]
    approval: Option<String>,
    #[serde(default)]
    cache: Option<CacheOption>,
}

struct Target {
    document: DocumentId,
    server: McpServerId,
    tool: McpToolName,
    input: Option<Value>,
    approval: Option<String>,
    cache: Option<CacheOption>,
}

// JSON: MCP-protocol boundary, the result and whether the cache served it.
struct Served {
    result: Value,
    cached: bool,
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
    let outcome = match &answer {
        Ok(served) if served.cached => CACHED,
        Ok(served) => result_outcome(&served.result),
        Err(error) => error.code().as_str(),
    };
    let recorded = artifact_tool_calls::record_artifact_tool_call(
        broker.pool(),
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
    answer.map(|served| Json(served.result).into_response())
}

fn target(raw_document: String, body: &[u8]) -> Result<Target, ArtifactCallError> {
    let document = DocumentId::try_new(raw_document).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })?;
    let body: CallBody = serde_json::from_slice(body).map_err(|error| {
        tracing::debug!(%error, "artifact tool call body did not parse");
        ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "the body is `{server, tool, input, approval, cache}`",
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
        approval: body.approval,
        cache: body.cache,
    })
}

// JSON: MCP-protocol boundary, the call's arguments.
pub(super) fn arguments(input: Option<&Value>) -> Result<Value, ArtifactCallError> {
    match input {
        None | Some(Value::Null) => Ok(Value::Object(serde_json::Map::new())),
        Some(input @ Value::Object(_)) => Ok(input.clone()),
        Some(_) => Err(ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "`input` is a JSON object",
        )),
    }
}

async fn run(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    target: &Target,
) -> Result<Served, ArtifactCallError> {
    let page = broker.page(viewer, &target.document).await?;
    let arguments = arguments(target.input.as_ref())?;
    broker
        .admit(viewer, &page, target.server.as_str(), target.tool.as_str())
        .await?;
    let input_hash = cache::input_hash(&arguments);
    let key = CachedCallKey {
        user: &viewer.user.id,
        document: &target.document,
        server: target.server.as_str(),
        tool: target.tool.as_str(),
        input_hash: &input_hash,
    };
    let freshness = Freshness::of(target.cache.as_ref());
    if freshness.serve_within_ms > 0
        && let Some(hit) = artifact_call_cache::find_artifact_cached_call(broker.pool(), &key)
            .await
            .map_err(|error| super::admit::internal(&error))?
            .filter(|hit| freshness.serves(hit.stored_at))
    {
        return Ok(Served {
            result: cache::served(hit.result, hit.stored_at),
            cached: true,
        });
    }
    let proxy = broker.inner.live.proxy();
    let mut session = proxy.open(&target.server, viewer.credential()).await?;
    let ran = in_session(broker, &mut session, target, &key, arguments).await;
    session.close().await;
    let (result, read) = match ran {
        Ok(ran) => ran,
        Err(error) => {
            forget_on_denial(broker, &key, &error).await;
            return Err(error);
        },
    };
    let result = cache::stripped(result);
    if read
        && cache::storable(&result)
        && let Some(expires_at) = freshness.expiry()
        && let Err(error) =
            artifact_call_cache::set_artifact_cached_call(broker.pool(), &key, &result, expires_at)
                .await
    {
        tracing::warn!(%error, "a page's read ran but was not cached");
    }
    Ok(Served {
        result,
        cached: false,
    })
}

// JSON: MCP-protocol boundary, the tool listing and the call's result.
async fn in_session(
    broker: &ArtifactsBroker,
    session: &mut super::proxy::Session<'_>,
    target: &Target,
    key: &CachedCallKey<'_>,
    arguments: Value,
) -> Result<(Value, bool), ArtifactCallError> {
    let tools = session.tools().await?;
    let Some(listed) = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some(target.tool.as_str()))
    else {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::BlockedByPolicy,
            "the server lists no such tool for this viewer",
        ));
    };
    let read = listed
        .pointer("/annotations/readOnlyHint")
        .and_then(Value::as_bool)
        == Some(true);
    if !read {
        spend_approval(broker, target, key).await?;
    }
    session
        .call(&target.tool, arguments)
        .await
        .map(|result| (result, read))
}

async fn spend_approval(
    broker: &ArtifactsBroker,
    target: &Target,
    key: &CachedCallKey<'_>,
) -> Result<(), ArtifactCallError> {
    let needed = || {
        ArtifactCallError::new(
            ArtifactCallCode::ApprovalRequired,
            "this tool changes things; the viewer approves each call",
        )
    };
    let id = target.approval.as_deref().ok_or_else(needed)?;
    let approval = ArtifactApproval {
        id,
        user: key.user,
        document: key.document,
        server: key.server,
        tool: key.tool,
        input_hash: key.input_hash,
    };
    let spent = artifact_approvals::delete_artifact_approval(broker.pool(), &approval)
        .await
        .map_err(|error| super::admit::internal(&error))?;
    if spent { Ok(()) } else { Err(needed()) }
}

// Why: Anthropic's artifact runtime drops a cached entry when a call on it
// is denied, so revoked access never keeps answering from the cache.
async fn forget_on_denial(
    broker: &ArtifactsBroker,
    key: &CachedCallKey<'_>,
    error: &ArtifactCallError,
) {
    let denied = matches!(
        error.code(),
        ArtifactCallCode::Reauth
            | ArtifactCallCode::BlockedByPolicy
            | ArtifactCallCode::ServerNotConnected
    );
    if !denied {
        return;
    }
    let scope = CachedCallScope {
        user: key.user,
        document: key.document,
        server: Some(key.server),
        tool: Some(key.tool),
        input_hash: Some(key.input_hash),
    };
    if let Err(error) =
        artifact_call_cache::delete_artifact_cached_calls(broker.pool(), &scope).await
    {
        tracing::warn!(%error, "a denied call's cached result was not dropped");
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
