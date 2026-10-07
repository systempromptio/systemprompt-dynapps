//! The admin plane's composition root.
//!
//! Every router the admin plane mounts is assembled here from the route
//! fragments of the admin domain crates (catalog, governance, identity,
//! platform, analytics), behind the session middleware and the role gates:
//!
//! - [`admin_router`] — the JSON API (auth-gated; admin-only and
//!   authenticated-read routes are layered together).
//! - [`admin_ssr_router`], [`bridge_auth_ssr_router`] — the server-rendered
//!   dashboard and the desktop-bridge sign-in flow.
//! - [`hooks_webhook_router`] — the governance webhooks called by gateway / MCP
//!   / Claude Code (`/hooks/track`, `/hooks/govern`, `/govern/authz`).
//! - [`secrets_router`], [`share_manifest_router`] — per-plugin secret
//!   resolution and public manifest sharing.
//!
//! Every `sqlx` call, handler and route table lives in the domain crate that
//! owns the data. Composing all five is what places this module in the entry
//! tier: no domain crate may depend on another, so the one place that merges
//! them sits above every one of them.

mod api;
mod login;
mod marketplace_context;
mod middleware;
mod pat;
mod redirects;
mod ssr;

use std::sync::Arc;

use axum::{Extension, Router, middleware as axum_middleware};
use systemprompt::prelude::PgPool;

pub use ssr::{admin_ssr_router, bridge_auth_ssr_router};
pub use systemprompt_web_admin_catalog::routes::share::share_manifest_router;
pub use systemprompt_web_admin_governance::routes::secrets::secrets_router;

pub fn hooks_webhook_router(
    db: &systemprompt::database::DbPool,
    pool: Arc<PgPool>,
    session_service: Arc<systemprompt::oauth::SessionCreationService>,
) -> Router {
    let managed = systemprompt::marketplace::managed::ManagedRepository::new(db);
    systemprompt_web_admin_analytics::routes::hooks_track::track_routes()
        .merge(systemprompt_web_admin_governance::routes::webhooks::govern_routes())
        .layer(Extension(
            systemprompt_web_admin_core::event_hub::EventHub::default(),
        ))
        .layer(Extension(session_service))
        .layer(Extension(managed))
        .with_state(pool)
}

pub fn admin_router(
    read_pool: Arc<PgPool>,
    write_pool: &Arc<PgPool>,
    managed: Arc<systemprompt_web_admin_core::state::ManagedState>,
) -> Router {
    let admin_only = api::build_admin_only_routes(&read_pool, write_pool, managed);
    let auth_reads = api::build_auth_read_routes(&read_pool);
    let self_service = api::build_self_service_routes(write_pool);

    admin_only
        .merge(auth_reads)
        .merge(self_service)
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            middleware::user_context_middleware,
        ))
}
