//! Server-rendered bridge sign-in pages.
//!
//! The setup page and the device-link approval the desktop bridge opens in a
//! browser. The composition root layers these under its own user-context and
//! marketplace middleware; they are mounted apart from the admin router, so
//! they are not in its route table.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/setup", get(handlers::ssr::bridge_setup_page))
        .route("/device-link", get(handlers::ssr::device_link_page))
        .route(
            "/device-link/approve",
            post(handlers::ssr::device_link_approve),
        )
        .route("/device-link/deny", post(handlers::ssr::device_link_deny))
}
