//! `/hooks/track`, where Claude Code hooks post every session event.
//!
//! The composition root merges this fragment beside the governance webhooks
//! and layers the shared extensions (event hub, session service, managed
//! repository) over both.

use std::sync::Arc;

use axum::Router;
use axum::routing::post;
use sqlx::PgPool;

use crate::handlers;

pub fn track_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/hooks/track",
        post(handlers::hooks_track::handle_hook_track),
    )
}
