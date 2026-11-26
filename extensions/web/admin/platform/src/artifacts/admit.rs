//! The one admission every page request passes, in the order the page's
//! runtime documents its codes.
//!
//! [`ArtifactsBroker::page`] is the viewer's `view` on the document, asked
//! of core's document service as the viewer (a document they cannot see is a
//! 404 exactly as in Drive, and their cached results on it are dropped), and
//! the page's manifest. [`ArtifactsBroker::admit`] then takes a server and a
//! tool through: the server exists here (`bad_request`) and is on
//! (`server_unavailable`); the manifest declares the pair
//! (`not_in_manifest`); the operator's policy allows it
//! (`blocked_by_policy`); and the viewer allowed the server for this page
//! (`approval_required` until asked, `not_in_manifest` once declined).

use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::service::DocumentView;
use systemprompt::identifiers::{DocumentId, TraceId};

use super::ArtifactsBroker;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::live_pages::{LivePages, ServerStanding};
use super::manifest::Capabilities;
use super::viewer::ArtifactViewer;
use crate::repositories::artifact_call_cache::{self, CachedCallScope};
use crate::repositories::artifact_consents::{self, ArtifactConsent};

pub(super) struct Page {
    pub(super) view: DocumentView,
    pub(super) manifest: Option<Capabilities>,
    pub(super) contract_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConsentState {
    Prompt,
    Granted,
    Denied,
}

pub(super) fn mcp_scope(server: &str) -> String {
    format!("mcp:{server}")
}

pub(super) fn consent_of(consents: &[ArtifactConsent], scope: &str) -> ConsentState {
    match consents.iter().find(|consent| consent.scope == scope) {
        None => ConsentState::Prompt,
        Some(consent) if consent.granted => ConsentState::Granted,
        Some(_) => ConsentState::Denied,
    }
}

pub(super) fn internal(error: &sqlx::Error) -> ArtifactCallError {
    tracing::error!(%error, "an artifacts broker query failed");
    ArtifactCallError::new(
        ArtifactCallCode::Internal,
        "the page's state could not be read",
    )
}

impl ArtifactsBroker {
    pub(super) async fn page(
        &self,
        viewer: &ArtifactViewer,
        document: &DocumentId,
    ) -> Result<Page, ArtifactCallError> {
        let lookup = self
            .inner
            .parts
            .documents
            .0
            .get(&viewer.user, document, &TraceId::generate())
            .await;
        let view = match lookup {
            Ok(view) => view,
            Err(
                DocumentsError::Forbidden { .. }
                | DocumentsError::NotFound { .. }
                | DocumentsError::Trashed { .. },
            ) => {
                self.forget(viewer, document, None).await;
                return Err(ArtifactCallError::new(
                    ArtifactCallCode::NotFound,
                    "document not found",
                ));
            },
            Err(error) => {
                tracing::error!(%error, "the document service failed an artifacts broker view check");
                return Err(ArtifactCallError::new(
                    ArtifactCallCode::Internal,
                    "the document could not be checked",
                ));
            },
        };
        let declared = LivePages::manifest(self.pool(), document)
            .await
            .map_err(|error| internal(&error))?;
        let (manifest, contract_version) = declared.map_or((None, None), |(manifest, version)| {
            (Some(manifest), Some(version))
        });
        Ok(Page {
            view,
            manifest,
            contract_version,
        })
    }

    pub(super) async fn consents(
        &self,
        viewer: &ArtifactViewer,
        document: &DocumentId,
    ) -> Result<Vec<ArtifactConsent>, ArtifactCallError> {
        artifact_consents::list_artifact_consents(self.pool(), &viewer.user.id, document)
            .await
            .map_err(|error| internal(&error))
    }

    pub(super) async fn admit(
        &self,
        viewer: &ArtifactViewer,
        page: &Page,
        server: &str,
        tool: &str,
    ) -> Result<(), ArtifactCallError> {
        let refuse = |code, message| Err(ArtifactCallError::new(code, message));
        let standing = self.inner.live.standing(server);
        match standing {
            ServerStanding::Unknown => {
                return refuse(
                    ArtifactCallCode::BadRequest,
                    "this instance has no such MCP server",
                );
            },
            ServerStanding::Off => {
                return refuse(
                    ArtifactCallCode::ServerUnavailable,
                    "this MCP server is switched off",
                );
            },
            ServerStanding::Closed | ServerStanding::Open => {},
        }
        if !page
            .manifest
            .as_ref()
            .is_some_and(|manifest| manifest.declares(server, tool))
        {
            return refuse(
                ArtifactCallCode::NotInManifest,
                "the page does not declare this tool",
            );
        }
        if standing == ServerStanding::Closed || !self.inner.live.allows(server, tool) {
            return refuse(
                ArtifactCallCode::BlockedByPolicy,
                "this instance's policy keeps pages off this tool",
            );
        }
        let consents = self.consents(viewer, &page.view.document.id).await?;
        match consent_of(&consents, &mcp_scope(server)) {
            ConsentState::Granted => Ok(()),
            ConsentState::Prompt => refuse(
                ArtifactCallCode::ApprovalRequired,
                "the viewer has not allowed this server for the page yet",
            ),
            ConsentState::Denied => refuse(
                ArtifactCallCode::NotInManifest,
                "the viewer turned this server off for the page",
            ),
        }
    }

    pub(super) async fn forget(
        &self,
        viewer: &ArtifactViewer,
        document: &DocumentId,
        server: Option<&str>,
    ) {
        let scope = CachedCallScope {
            user: &viewer.user.id,
            document,
            server,
            tool: None,
            input_hash: None,
        };
        if let Err(error) =
            artifact_call_cache::delete_artifact_cached_calls(self.pool(), &scope).await
        {
            tracing::warn!(%error, "a viewer's cached page results were not dropped");
        }
    }
}
