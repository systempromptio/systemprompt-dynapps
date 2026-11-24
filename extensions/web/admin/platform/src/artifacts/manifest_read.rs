//! `GET /api/v1/artifacts/{document_id}/manifest`: whether a document is a
//! live page, for a client deciding to open the viewer shell for it.
//!
//! Answers `{declared, capabilities, contract, live_url}`: the manifest and
//! its contract version, and the shell's absolute URL, or `declared: false`
//! with the rest `null` for a document with none. The reader must see the
//! document (`view`, a 404 otherwise, as in Drive). Two credentials are
//! taken: the web login's browser session from this site's pages (as the
//! broker takes it, `viewer`), or the person's `ui` bearer, the session
//! `DynWork` desktop holds for its person (a `ui_session` token, well formed
//! as core shapes it). Nothing here runs a tool or changes a consent, so the
//! `ui` session that may not reach core's MCP proxy may read this.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::identifiers::{DocumentId, TraceId, UserId};
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience};
use systemprompt::oauth::validate_jwt_token;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use super::ArtifactsBroker;
use super::admit::internal;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::live_pages::LivePages;
use super::viewer::ArtifactViewer;
use crate::routes::drive_user::USER_BASE_URL;

const BEARER: &str = "Bearer ";

pub(super) async fn read(
    State(broker): State<ArtifactsBroker>,
    Path(raw_document): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ArtifactCallError> {
    let boot = &broker.inner.parts.boot;
    let reader = reader(boot, &headers)?;
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
        }))
        .into_response())
    })
    .await
}

fn reader(
    boot: &ExtensionBoot,
    headers: &HeaderMap,
) -> Result<AuthenticatedUser, ArtifactCallError> {
    let Some(bearer) = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix(BEARER))
    else {
        return ArtifactViewer::from_request(boot, headers).map(|viewer| viewer.user);
    };
    let refused = || {
        ArtifactCallError::new(
            ArtifactCallCode::Reauth,
            "a bearer reads a page's manifest only as the person's `ui` session",
        )
    };
    let claims = validate_jwt_token(
        boot.token_authority().as_ref(),
        bearer.trim(),
        &[JwtAudience::Ui],
    )
    .map_err(|error| {
        tracing::debug!(%error, "a manifest read bearer did not validate");
        refused()
    })?;
    if !claims.custom.ui_session || !claims.ui_session_well_formed() {
        return Err(refused());
    }
    Ok(AuthenticatedUser::new_with_roles(
        UserId::new(claims.registered.sub),
        claims.custom.username,
        claims.custom.email,
        claims.custom.scope,
        claims.custom.roles,
    ))
}
