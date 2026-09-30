//! Admin extension for the Enterprise Demo template.
//!
//! Wires the admin dashboard, governance webhooks, bridge plane, and
//! supporting services onto a shared `PgPool`. Public surface is grouped by
//! concern:
//!
//! - [`admin_router`] — the SSR dashboard (auth-gated; admin-only and
//!   authenticated-read routes are layered together).
//! - [`hooks_webhook_router`] — the governance webhooks called by gateway / MCP
//!   / Claude Code (`/hooks/track`, `/hooks/govern`, `/govern/authz`).
//! - [`secrets_router`], [`share_manifest_router`] — per-plugin secret
//!   resolution and public manifest sharing.
//!
//! [`repositories`] owns every `sqlx` call; handlers/services never touch
//! the DB directly. Errors normalise on `error::MarketplaceError` via the
//! `MarketplaceError` re-export in [`systemprompt_web_shared`].

pub mod authz;
pub(crate) mod export;
pub(crate) mod handlers;
mod marketplace_context;
mod middleware;
pub mod repositories;
mod routes;
pub(crate) mod services;
pub mod sign_out;

use std::sync::Arc;

use axum::routing::post;
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;

pub use handlers::adfs_auth::{
    ADFS_ACS_PATH, ADFS_METADATA_PATH, AdfsConfig, AdfsDeps, AdfsError, AssertionClaims, FlowState,
    clear_state_cookie, group_matches_pattern, permissions_for_roles, read_state_cookie,
    state_cookie,
};
pub use handlers::dev_login::{
    DEV_LOGIN_PATH, dev_login_allowed, dev_login_enabled, dev_login_url,
};
pub use routes::{admin_ssr_router, bridge_auth_ssr_router};
pub use systemprompt_web_admin_catalog::routes::share::share_manifest_router;
use systemprompt_web_admin_core::state::StateError;
pub use systemprompt_web_admin_governance::routes::secrets::secrets_router;

pub mod test_support {
    pub use crate::handlers::ssr::analysis::{SkillRef, parse_skill_key};
    pub use crate::handlers::ssr::transcript_view::{
        ConversationView, EmptyReason, ParsedAssistant, SideCallRowView, SideCallsView, StepView,
        ThreadView, ToolChipView, ToolUseMarker, TranscriptMetaView, TranscriptOptions,
        TranscriptRequestIds, TurnView, build_conversation, meta_view, parse_assistant, preview,
        short_id, strip_system_reminders, tidy_lines, transcript_request_ids,
    };
}

pub fn hooks_webhook_router(
    db: &systemprompt::database::DbPool,
    pool: Arc<PgPool>,
    session_service: Arc<systemprompt::oauth::SessionCreationService>,
) -> Result<Router, StateError> {
    let managed = systemprompt::marketplace::managed::ManagedRepository::new(db)?;
    Ok(Router::new()
        .route(
            "/hooks/track",
            post(handlers::hooks_track::handle_hook_track),
        )
        .merge(systemprompt_web_admin_governance::routes::webhooks::govern_routes())
        .layer(Extension(
            systemprompt_web_admin_core::event_hub::EventHub::default(),
        ))
        .layer(Extension(session_service))
        .layer(Extension(managed))
        .with_state(pool))
}

pub fn admin_router(
    db: &systemprompt::database::DbPool,
    read_pool: Arc<PgPool>,
    write_pool: &Arc<PgPool>,
    owner: systemprompt::identifiers::UserId,
) -> Result<Router, StateError> {
    let admin_only = routes::build_admin_only_routes(db, &read_pool, write_pool, owner)?;
    let auth_reads = routes::build_auth_read_routes(&read_pool);
    let self_service = routes::build_self_service_routes(write_pool);

    Ok(admin_only
        .merge(auth_reads)
        .merge(self_service)
        .layer(axum_middleware::from_fn(
            systemprompt_web_admin_core::middleware::gates::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            middleware::user_context_middleware,
        )))
}
