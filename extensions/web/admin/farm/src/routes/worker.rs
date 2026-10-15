//! The worker API, nested at `/api/public/agent-farm-worker`: what a farm
//! worker on a Fly Machine fetches its run from and reports it back to.
//!
//! The composition root mounts it without the session/PAT gate; every
//! handler authorises the run's own worker token instead. Output uploads
//! carry the edited workspace, so the body limit is raised to the bundle cap.

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

use crate::handlers::worker;
use crate::services::runner::fly::protocol::MAX_BUNDLE_BYTES;

const BODY_LIMIT: usize = 64 * 1024 * 1024;

pub fn agent_farm_worker_routes() -> Router {
    debug_assert!(MAX_BUNDLE_BYTES < BODY_LIMIT as u64);
    Router::new()
        .route("/runs/{run_id}/spec", get(worker::spec))
        .route("/runs/{run_id}/bundle", get(worker::bundle))
        .route("/runs/{run_id}/events", post(worker::events))
        .route("/runs/{run_id}/session", post(worker::session))
        .route("/runs/{run_id}/heartbeat", post(worker::heartbeat))
        .route("/runs/{run_id}/outputs", post(worker::outputs))
        .route("/runs/{run_id}/result", post(worker::result))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
}
