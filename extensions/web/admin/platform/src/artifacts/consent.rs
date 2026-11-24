//! `/api/v1/artifacts/{document_id}/permissions`: what a page declares and
//! what its viewer decided, read and changed only by that viewer's web
//! session.
//!
//! `GET` answers `{capabilities, contract_version, revision, writer,
//! states}`: the manifest (`null` when none), the version the viewer loaded
//! (a publish's base), whether the viewer may edit, and a state per name in
//! the shape of the runtime's `permissions.state()`: `mcp:<server>` for each
//! declared server (`granted`, `denied` or `prompt`), `mcp` for all of them
//! together, and `artifact` and `user` when declared (they need no standing
//! grant). `POST {scope, decision}` records `granted` or `denied` for
//! `mcp:<server>` or for every declared server with `mcp`; `DELETE
//! .../permissions/{scope}` forgets it, so the next call asks again. Denying
//! or forgetting drops the viewer's cached results from those servers.

use std::collections::BTreeMap;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::DocumentId;
use systemprompt::models::documents::AccessLevel;

use super::ArtifactsBroker;
use super::admit::{ConsentState, Page, consent_of, internal, mcp_scope};
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::viewer::ArtifactViewer;
use crate::repositories::artifact_consents::{self, ArtifactConsent};

const ALL_SERVERS: &str = "mcp";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecideBody {
    scope: String,
    decision: Decision,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Decision {
    Granted,
    Denied,
}

pub(super) async fn read(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = document_id(raw_document)?;
    act_as(DocumentActor::Human, answer(&broker, &viewer, &document)).await
}

pub(super) async fn decide(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = document_id(raw_document)?;
    let body: DecideBody = serde_json::from_slice(&body).map_err(|error| {
        tracing::debug!(%error, "a page permission body did not parse");
        ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "the body is `{scope, decision}`",
        )
    })?;
    act_as(DocumentActor::Human, async {
        let page = broker.page(&viewer, &document).await?;
        for server in servers_in(&page, &body.scope)? {
            let consent = ArtifactConsent {
                scope: mcp_scope(&server),
                granted: matches!(body.decision, Decision::Granted),
            };
            artifact_consents::set_artifact_consent(
                broker.pool(),
                &viewer.user.id,
                &document,
                &consent,
            )
            .await
            .map_err(|error| internal(&error))?;
            if !consent.granted {
                broker.forget(&viewer, &document, Some(&server)).await;
            }
        }
        answer(&broker, &viewer, &document).await
    })
    .await
}

pub(super) async fn revoke(
    State(broker): State<ArtifactsBroker>,
    Path((raw_document, scope)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ArtifactCallError> {
    let viewer = ArtifactViewer::from_request(&broker.inner.parts.boot, &headers)?;
    let document = document_id(raw_document)?;
    act_as(DocumentActor::Human, async {
        let page = broker.page(&viewer, &document).await?;
        for server in servers_in(&page, &scope)? {
            artifact_consents::delete_artifact_consent(
                broker.pool(),
                &viewer.user.id,
                &document,
                &mcp_scope(&server),
            )
            .await
            .map_err(|error| internal(&error))?;
            broker.forget(&viewer, &document, Some(&server)).await;
        }
        answer(&broker, &viewer, &document).await
    })
    .await
}

fn document_id(raw: String) -> Result<DocumentId, ArtifactCallError> {
    DocumentId::try_new(raw).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })
}

fn servers_in(page: &Page, scope: &str) -> Result<Vec<String>, ArtifactCallError> {
    let declared: Vec<String> = page
        .manifest
        .as_ref()
        .map(|manifest| {
            manifest
                .servers()
                .iter()
                .map(|server| server.server.clone())
                .collect()
        })
        .unwrap_or_default();
    let chosen: Vec<String> = match scope.strip_prefix("mcp:") {
        _ if scope == ALL_SERVERS => declared,
        Some(server) => declared
            .into_iter()
            .filter(|declared| declared == server)
            .collect(),
        None => Vec::new(),
    };
    if chosen.is_empty() {
        return Err(ArtifactCallError::new(
            ArtifactCallCode::BadRequest,
            "a viewer decides `mcp` or `mcp:<server>` for a server the page declares",
        ));
    }
    Ok(chosen)
}

async fn answer(
    broker: &ArtifactsBroker,
    viewer: &ArtifactViewer,
    document: &DocumentId,
) -> Result<Response, ArtifactCallError> {
    let page = broker.page(viewer, document).await?;
    let consents = broker.consents(viewer, document).await?;
    Ok(Json(json!({
        "capabilities": page.manifest,
        "contract_version": page.contract_version,
        "revision": page.view.document.head_revision_id,
        "writer": page.view.my_access >= AccessLevel::Edit,
        "states": states(&page, &consents),
    }))
    .into_response())
}

fn states(page: &Page, consents: &[ArtifactConsent]) -> BTreeMap<String, &'static str> {
    let Some(manifest) = &page.manifest else {
        return BTreeMap::new();
    };
    let mut states = BTreeMap::new();
    let decided: Vec<ConsentState> = manifest
        .servers()
        .iter()
        .map(|declared| {
            let scope = mcp_scope(&declared.server);
            let consent = consent_of(consents, &scope);
            states.insert(scope, name(consent));
            consent
        })
        .collect();
    if !decided.is_empty() {
        let all = if decided
            .iter()
            .all(|consent| *consent == ConsentState::Granted)
        {
            ConsentState::Granted
        } else if decided.contains(&ConsentState::Prompt) {
            ConsentState::Prompt
        } else {
            ConsentState::Denied
        };
        states.insert(ALL_SERVERS.to_owned(), name(all));
    }
    if manifest.publishes() {
        states.insert("artifact".to_owned(), "granted");
    }
    if manifest.user.is_some() {
        states.insert("user".to_owned(), "granted");
    }
    states
}

const fn name(consent: ConsentState) -> &'static str {
    match consent {
        ConsentState::Prompt => "prompt",
        ConsentState::Granted => "granted",
        ConsentState::Denied => "denied",
    }
}
