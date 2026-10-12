//! HTML fragments the farm pages poll while a run is live.
//!
//! A non-console caller gets a bare 403 rather than a login redirect,
//! because the caller is a script swapping markup into a page, not a
//! browser navigating.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers::ssr;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/api/farm/activity", get(ssr::activity_fragment))
        .route("/api/farm/runs", get(ssr::runs_fragment))
        .route("/api/farm/runs/{run_id}", get(ssr::run_fragment))
        .route(
            "/api/farm/evals/runs/{eval_run_id}",
            get(ssr::eval_run_fragment),
        )
}
