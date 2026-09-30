//! Admin JSON API routes: the dashboard counters, open to any authenticated
//! caller, and the activity events feed, open to the console roles.
//!
//! The composition root merges each fragment into the tier that gates it.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub fn dashboard_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/dashboard", get(handlers::dashboard_handler))
}

pub fn events_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/events", get(handlers::list_events_handler))
}
