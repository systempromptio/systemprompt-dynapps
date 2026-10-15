//! The public run API, nested at `/api/public/agent-farm`.
//!
//! Callers are the dashboard's own scripts and remote clients holding a
//! personal access token; the handlers read the caller from the request's
//! user context.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::handlers::public;
use crate::state::FarmState;

pub fn agent_farm_public_routes() -> Router<Arc<FarmState>> {
    Router::new()
        .route("/agents", get(public::list_agents))
        .route("/runs", post(public::launch_run).get(public::list_runs))
        .route("/runs/{run_id}", get(public::get_run))
        .route("/runs/{run_id}/report", get(public::get_run_report))
        .route("/runs/{run_id}/events", get(public::list_run_events))
        .route("/runs/{run_id}/cancel", post(public::cancel_run))
        .route("/evals/{suite_id}/run", post(public::run_eval_suite))
        .route("/evals/runs/{eval_run_id}", get(public::get_eval_run))
        .route("/evals/history", get(public::eval_history))
        .route(
            "/evals/suites/{suite_id}/runs",
            get(public::list_suite_runs),
        )
}
