//! The artifacts broker: a Drive page's capabilities, and its MCP tool
//! calls run as the person viewing the page.
//!
//! A live page never holds a credential and never reaches a server itself;
//! it asks the broker, and the broker runs the tool through this instance's
//! own MCP proxy with the viewer's own session, so the viewer's per-tool
//! access, the proxy's checks and a connector's per-user OAuth apply exactly
//! as they do for an agent's call (`proxy`). Before that, every call passes
//! one admission (`admit`): the viewer sees the document, the page's
//! manifest declares the server and the tool (`manifest`, written by the
//! agent through the `documents` server), the operator's policy allows them
//! (`policy`), and the viewer has allowed the server for this page
//! (`consent`). A tool that does not declare `readOnlyHint: true` also needs
//! a one-shot approval of that exact call (`approve`). A read may be served
//! from the viewer's cache (`cache`). The page can list and describe what it
//! may call (`tools`) and republish itself (`publish`).
//!
//! The routes take only the web login's browser session (`viewer`), never an
//! agent's or the bridge's bearer, and each tool call writes one audit row
//! to `artifact_tool_calls`. [`artifacts_router`] mounts them under
//! `/api/v1/artifacts/{document_id}/` outside the admin session; the web
//! facade merges it when the document store is on. [`LivePages`] is the part
//! the `documents` server shares to check and record a declaration.

mod admit;
mod answer;
mod approve;
mod cache;
mod call;
mod consent;
mod error;
mod live_pages;
pub mod manifest;
mod policy;
mod proxy;
mod publish;
mod tools;
mod viewer;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, post};
use sqlx::PgPool;
use systemprompt::manifest::services::ServicesView;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use crate::routes::drive_user::DriveDocuments;
pub use live_pages::{LivePages, LivePagesError};
pub use policy::{LivePagesPolicy, LivePagesPolicyError};

pub const CALL_PATH: &str = "/api/v1/artifacts/{document_id}/mcp/call";
const APPROVE_PATH: &str = "/api/v1/artifacts/{document_id}/mcp/approve";
const TOOLS_PATH: &str = "/api/v1/artifacts/{document_id}/mcp/tools";
const DESCRIBE_PATH: &str = "/api/v1/artifacts/{document_id}/mcp/describe";
const INVALIDATE_PATH: &str = "/api/v1/artifacts/{document_id}/mcp/invalidate";
const PERMISSIONS_PATH: &str = "/api/v1/artifacts/{document_id}/permissions";
const PERMISSION_PATH: &str = "/api/v1/artifacts/{document_id}/permissions/{scope}";
const PUBLISH_PATH: &str = "/api/v1/artifacts/{document_id}/artifact/publish";
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// What the broker is built from.
///
/// The document service the view check asks and a publish writes through,
/// the live services view the servers are looked up in, the boot values (the
/// token authority, the site's origins, the services root the live-pages
/// policy is read from and the API's internal URL) and the pool the
/// manifest, consents, approvals, cache and audit rows live in.
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
    live: LivePages,
}

impl std::fmt::Debug for ArtifactsBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactsBroker").finish_non_exhaustive()
    }
}

impl ArtifactsBroker {
    fn pool(&self) -> &PgPool {
        &self.inner.parts.pool
    }
}

pub fn artifacts_router(parts: ArtifactsBrokerParts) -> Result<Router, LivePagesError> {
    let live = LivePages::new(
        parts.boot.services_root(),
        parts.services.config.clone(),
        &parts.boot.config().api_internal_url,
    )?;
    let broker = ArtifactsBroker {
        inner: Arc::new(Broker { parts, live }),
    };
    Ok(Router::new()
        .route(CALL_PATH, post(call::call))
        .route(APPROVE_PATH, post(approve::approve))
        .route(TOOLS_PATH, post(tools::list))
        .route(DESCRIBE_PATH, post(tools::describe))
        .route(INVALIDATE_PATH, post(tools::invalidate))
        .route(PERMISSIONS_PATH, get(consent::read).post(consent::decide))
        .route(PERMISSION_PATH, delete(consent::revoke))
        .route(PUBLISH_PATH, post(publish::publish))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(broker))
}
