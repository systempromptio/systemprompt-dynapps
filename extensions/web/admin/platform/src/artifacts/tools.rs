//! What a page may call, as its runtime's `listTools`, `describeTool` and
//! `invalidate` ask.
//!
//! `POST .../mcp/tools {server?}`: the manifest as far as the viewer
//! reaches. A server the viewer allowed is asked for its tools as the
//! viewer, and lists the declared ones it has for them, with their
//! annotations (`authStatus: connected`); one not asked about yet lists from
//! the manifest alone (`authStatus: unknown`, no descriptions); one whose
//! connector the viewer must connect again lists no tools (`authStatus:
//! needs_reauth`); one the viewer declined, that is off, outside the policy
//! or not connected for the viewer is left out. Listing never asks anything.
//! `POST .../mcp/describe {server, tool}`: one declared tool of an allowed
//! server, with its schemas. `POST .../mcp/invalidate {server?, tool?,
//! input?}` drops the viewer's cached results on the page, narrowed by each
//! argument given.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
// JSON: MCP-protocol boundary, tool listings are schema-less.
use serde_json::{Value, json};
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::{DocumentId, McpServerId};

use super::admit::{ConsentState, consent_of, internal, mcp_scope};
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::live_pages::ServerStanding;
use super::manifest::ServerDeclaration;
use super::viewer::ArtifactViewer;
use super::{ArtifactsBroker, cache};
use crate::repositories::artifact_call_cache::{self, CachedCallScope};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListBody {
    #[serde(default)]
    server: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DescribeBody {
    server: String,
    tool: String,
}

// JSON: MCP-protocol boundary, the call's input as the page sends it.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct InvalidateBody {
    #[serde(default)]
    server: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    input: Option<Value>,
}

fn document_id(raw: String) -> Result<DocumentId, ArtifactCallError> {
    DocumentId::try_new(raw).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })
}

fn body<T: serde::de::DeserializeOwned>(
    raw: &[u8],
    shape: &'static str,
) -> Result<T, ArtifactCallError> {
    serde_json::from_slice(raw).map_err(|error| {
        tracing::debug!(%error, "a page tools body did not parse");
        ArtifactCallError::new(ArtifactCallCode::BadRequest, shape)
    })
}

pub(super) async fn list(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    raw: Bytes,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = document_id(raw_document)?;
    let wanted: ListBody = body(&raw, "the body is `{server?}`")?;
    act_as(DocumentActor::Human, async {
        let page = broker.page(&viewer, &document).await?;
        let consents = broker.consents(&viewer, &document).await?;
        let declared = page
            .manifest
            .as_ref()
            .map_or(&[][..], |manifest| manifest.servers());
        let mut servers = Vec::new();
        for server in declared {
            if wanted
                .server
                .as_ref()
                .is_some_and(|wanted| *wanted != server.server)
                || broker.inner.live.standing(&server.server) != ServerStanding::Open
            {
                continue;
            }
            let listed = match consent_of(&consents, &mcp_scope(&server.server)) {
                ConsentState::Denied => None,
                ConsentState::Prompt => Some(unasked(server)),
                ConsentState::Granted => connected(&broker, &viewer, server).await,
            };
            servers.extend(listed);
        }
        Ok(Json(json!({ "servers": servers })).into_response())
    })
    .await
}

// JSON: the runtime's `ServerInfo` for a server not asked about yet.
fn unasked(server: &ServerDeclaration) -> Value {
    let tools: Vec<Value> = server
        .tools
        .iter()
        .map(|tool| json!({ "name": tool, "description": "" }))
        .collect();
    json!({ "server": server.server, "authStatus": "unknown", "tools": tools })
}

// JSON: the runtime's `ServerInfo` from the server's own listing.
async fn connected(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    server: &ServerDeclaration,
) -> Option<Value> {
    let listed = match listing(broker, viewer, &server.server).await {
        Ok(listed) => listed,
        Err(error) if error.code() == ArtifactCallCode::ServerNotConnected => return None,
        Err(error) if error.code() == ArtifactCallCode::NeedsReauth => {
            return Some(
                json!({ "server": server.server, "authStatus": "needs_reauth", "tools": [] }),
            );
        },
        Err(error) => {
            tracing::debug!(
                error_key = error.code().as_str(),
                "a page's server could not be listed"
            );
            return Some(json!({ "server": server.server, "authStatus": "unknown", "tools": [] }));
        },
    };
    let tools: Vec<Value> = server
        .tools
        .iter()
        .filter_map(|name| {
            listed
                .iter()
                .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
        })
        .map(|tool| {
            let mut info = json!({
                "name": tool.get("name"),
                "description": tool.get("description").and_then(Value::as_str).unwrap_or(""),
            });
            if let Some(annotations) = tool.get("annotations") {
                info["annotations"] = annotations.clone();
            }
            info
        })
        .collect();
    Some(json!({ "server": server.server, "authStatus": "connected", "tools": tools }))
}

// JSON: MCP-protocol boundary, the server's tools as the viewer sees them.
async fn listing(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    server: &str,
) -> Result<Vec<Value>, ArtifactCallError> {
    let mut session = broker
        .inner
        .live
        .proxy()
        .open(&McpServerId::new(server), viewer.credential())
        .await?;
    let tools = session.tools().await;
    session.close().await;
    tools
}

pub(super) async fn describe(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    raw: Bytes,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = document_id(raw_document)?;
    let wanted: DescribeBody = body(&raw, "the body is `{server, tool}`")?;
    act_as(DocumentActor::Human, async {
        let page = broker.page(&viewer, &document).await?;
        broker
            .admit(&viewer, &page, &wanted.server, &wanted.tool)
            .await
            .map_err(|error| {
                if error.code() == ArtifactCallCode::ApprovalRequired {
                    ArtifactCallError::new(
                        ArtifactCallCode::NotInManifest,
                        "the viewer has not allowed this server yet",
                    )
                } else {
                    error
                }
            })?;
        let listed = listing(&broker, &viewer, &wanted.server).await?;
        let tool = listed
            .into_iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some(wanted.tool.as_str()))
            .ok_or_else(|| {
                ArtifactCallError::new(
                    ArtifactCallCode::BadRequest,
                    "the server has no schema for this tool",
                )
            })?;
        Ok(Json(tool).into_response())
    })
    .await
}

pub(super) async fn invalidate(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    raw: Bytes,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = document_id(raw_document)?;
    let wanted: InvalidateBody = body(&raw, "the body is `{server?, tool?, input?}`")?;
    if wanted.input.is_some() && (wanted.server.is_none() || wanted.tool.is_none()) {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "`input` needs `server` and `tool`",
        ));
    }
    act_as(DocumentActor::Human, async {
        broker.page(&viewer, &document).await?;
        let input_hash = wanted.input.as_ref().map(cache::input_hash);
        let scope = CachedCallScope {
            user: &viewer.user.id,
            document: &document,
            server: wanted.server.as_deref(),
            tool: wanted.tool.as_deref(),
            input_hash: input_hash.as_deref(),
        };
        artifact_call_cache::delete_artifact_cached_calls(broker.pool(), &scope)
            .await
            .map_err(|error| internal(&error))?;
        Ok(StatusCode::NO_CONTENT.into_response())
    })
    .await
}
