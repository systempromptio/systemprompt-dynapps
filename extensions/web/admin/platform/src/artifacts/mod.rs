//! The artifacts broker: a Drive page's MCP tool call, run as the person
//! viewing the page.
//!
//! A live page never holds a credential and never reaches a server itself;
//! it asks the broker, and the broker runs the tool through this instance's
//! own MCP proxy with the viewer's own session, so the viewer's per-tool
//! access, the proxy's checks and a connector's per-user OAuth apply exactly
//! as they do for an agent's call (`proxy`). The route takes only the web
//! login's browser session (`viewer`), checks the viewer's `view` on the
//! document and the call's admission (`call`, [`ArtifactsBroker::admit`]),
//! and writes one audit row per call to `artifact_tool_calls`.
//!
//! [`artifacts_router`] mounts `POST /api/v1/artifacts/{document_id}/mcp/call`
//! outside the admin session; the web facade merges it when the document
//! store is on.

mod answer;
mod call;
mod error;
mod proxy;
mod viewer;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use sqlx::PgPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::manifest::services::ServicesView;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use crate::routes::drive_user::DriveDocuments;
use error::{ArtifactCallCode, ArtifactCallError};
use proxy::LoopbackProxy;

pub const CALL_PATH: &str = "/api/v1/artifacts/{document_id}/mcp/call";
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// What the broker is built from.
///
/// The document service the view check asks, the live services view the
/// servers are looked up in, the boot values (the token authority, the site's
/// origins and the API's internal URL) and the pool the audit rows go to.
pub struct ArtifactsBrokerParts {
    pub documents: DriveDocuments,
    pub services: ServicesView,
    pub boot: ExtensionBoot,
    pub pool: Arc<PgPool>,
}

impl std::fmt::Debug for ArtifactsBrokerParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactsBrokerParts")
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub(crate) struct ArtifactsBroker {
    inner: Arc<Broker>,
}

struct Broker {
    parts: ArtifactsBrokerParts,
    proxy: LoopbackProxy,
}

impl std::fmt::Debug for ArtifactsBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactsBroker").finish_non_exhaustive()
    }
}

pub fn artifacts_router(parts: ArtifactsBrokerParts) -> Result<Router, reqwest::Error> {
    let proxy = LoopbackProxy::new(&parts.boot.config().api_internal_url)?;
    let broker = ArtifactsBroker {
        inner: Arc::new(Broker { parts, proxy }),
    };
    Ok(Router::new()
        .route(CALL_PATH, post(call::call))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(broker))
}

impl ArtifactsBroker {
    // Why: every admission check lives here (today the server; next the
    // page's declared capabilities and the viewer's consent), so no call
    // reaches the proxy around one of them.
    fn admit(&self, server: &McpServerId) -> Result<(), ArtifactCallError> {
        let services = self.inner.parts.services.config();
        match services.mcp_servers.get(server.as_str()) {
            None => Err(ArtifactCallError::new(
                ArtifactCallCode::BadRequest,
                "this instance has no such MCP server",
            )),
            Some(deployment) if !deployment.enabled => Err(ArtifactCallError::new(
                ArtifactCallCode::ServerUnavailable,
                "this MCP server is switched off",
            )),
            Some(_) => Ok(()),
        }
    }
}
