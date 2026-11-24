//! `POST /api/v1/artifacts/{document_id}/mcp/approve`: the viewer's Allow
//! for one write a page asked for.
//!
//! The shell posts it when the viewer clicks «Permitir» on the call it is
//! showing, `{server, tool, input}` exactly as the page will call it. The
//! call must pass the same admission as the call itself (`admit`); the
//! answer is an approval id the next call presents once, within
//! `APPROVAL_TTL`, for that viewer, document, server, tool and input (its
//! hash, keys sorted). Spending it is the call's (`call`).

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::DocumentId;

use super::error::{ArtifactCallCode, ArtifactCallError};
use super::manifest::is_name;
use super::viewer::ArtifactViewer;
use super::{ArtifactsBroker, cache};
use crate::repositories::artifact_approvals::{self, ArtifactApproval};

const APPROVAL_TTL_SECONDS: i64 = 120;

// JSON: MCP-protocol boundary, the tool's arguments as the page will send them.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApproveBody {
    server: String,
    tool: String,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

pub(super) async fn approve(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = DocumentId::try_new(raw_document).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })?;
    let body: ApproveBody = serde_json::from_slice(&body)
        .ok()
        .filter(|body: &ApproveBody| is_name(&body.server) && is_name(&body.tool))
        .ok_or_else(|| {
            ArtifactCallError::new(
                ArtifactCallCode::BadRequest,
                "the body is `{server, tool, input}`",
            )
        })?;
    act_as(DocumentActor::Human, async {
        let page = broker.page(&viewer, &document).await?;
        broker
            .admit(&viewer, &page, &body.server, &body.tool)
            .await?;
        let input = super::call::arguments(body.input.as_ref())?;
        let id = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let expires_at = Utc::now() + Duration::seconds(APPROVAL_TTL_SECONDS);
        let approval = ArtifactApproval {
            id: &id,
            user: &viewer.user.id,
            document: &document,
            server: &body.server,
            tool: &body.tool,
            input_hash: &cache::input_hash(&input),
        };
        artifact_approvals::insert_artifact_approval(broker.pool(), &approval, expires_at)
            .await
            .map_err(|error| super::admit::internal(&error))?;
        Ok(Json(json!({ "approval": id, "expires_at": expires_at })).into_response())
    })
    .await
}
