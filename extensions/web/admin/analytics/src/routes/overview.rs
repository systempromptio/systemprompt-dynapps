//! The overview page, the fragment it refreshes itself from, and the
//! attention counts the sidebar badges poll.
//!
//! The attention counts sit under `/governance` because the badge counts the
//! approval queue; the composition root merges them after the governance
//! pages.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new().route("/", get(handlers::ssr::overview_page))
}

pub fn api_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/api/overview", get(handlers::ssr::overview_fragment))
}

pub fn attention_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/governance/attention",
        get(handlers::ssr::attention_counts),
    )
}
