//! `GET /api/v1/artifacts/{document_id}/manifest`: whether a document is a
//! live page, for a client deciding to draw it as one.
//!
//! Answers `{declared, capabilities, contract, live_url, servers}`: the
//! manifest and its contract version, the shell's absolute URL, and each
//! declared server as `{id, name}`, the name the viewer is shown for it
//! (`LivePages::names`); or `declared: false` with the rest `null` and no
//! servers for a document with none. The reader must see the document
//! (`view`, a 404 otherwise, as in Drive), and is taken as every broker route
//! takes the person (`viewer`): the web session from this site's pages or
//! the person's `ui` session as the bearer.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::{DocumentId, TraceId};

use super::ArtifactsBroker;
use super::admit::internal;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::live_pages::LivePages;
use super::viewer::ArtifactViewer;
use crate::routes::drive_user::USER_BASE_URL;

pub(super) async fn read(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ArtifactCallError> {
    let boot = &broker.inner.parts.boot;
    let reader = ArtifactViewer::from_request(boot, &headers)?.user;
    let document = DocumentId::try_new(raw_document).map_err(|_invalid| {
        ArtifactCallError::new(ArtifactCallCode::NotFound, "document not found")
    })?;
    act_as(DocumentActor::Human, async {
        let lookup = broker
            .inner
            .parts
            .documents
            .0
            .get(&reader, &document, &TraceId::generate())
            .await;
        match lookup {
            Ok(_) => {},
            Err(
                DocumentsError::Forbidden { .. }
                | DocumentsError::NotFound { .. }
                | DocumentsError::Trashed { .. },
            ) => {
                return Err(ArtifactCallError::new(
                    ArtifactCallCode::NotFound,
                    "document not found",
                ));
            },
            Err(error) => {
                tracing::error!(%error, "the document service failed a manifest read");
                return Err(ArtifactCallError::new(
                    ArtifactCallCode::Internal,
                    "the document could not be checked",
                ));
            },
        }
        let declared = LivePages::manifest(broker.pool(), &document)
            .await
            .map_err(|error| internal(&error))?;
        let Some((capabilities, contract)) = declared else {
            return Ok(Json(json!({
                "declared": false,
                "capabilities": null,
                "contract": null,
                "live_url": null,
                "servers": [],
            }))
            .into_response());
        };
        let site = boot.config().api_external_url.trim_end_matches('/');
        let live_url = format!(
            "{site}{USER_BASE_URL}/{}/live",
            urlencoding::encode(document.as_str())
        );
        Ok(Json(json!({
            "declared": true,
            "capabilities": capabilities,
            "contract": contract,
            "live_url": live_url,
            "servers": broker.inner.live.names(Some(&capabilities)),
        }))
        .into_response())
    })
    .await
}
