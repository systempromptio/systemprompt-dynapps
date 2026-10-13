//! Server-rendered farm pages and the form actions they post to. Every page
//! is console-gated in its handler; the actions answer with a flash redirect.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers::ssr;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/farm", get(ssr::farm_overview_page))
        .route("/farm/runs", get(ssr::runs_page))
        .route("/farm/runs/{run_id}", get(ssr::run_detail_page))
        .route("/farm/runs/{run_id}/cancel", post(ssr::cancel_run_action))
        .route("/farm/evals", get(ssr::evals_page))
        .route("/farm/evals/runs/{eval_run_id}", get(ssr::eval_run_page))
        .route("/farm/evals/{suite_id}/run", post(ssr::run_now_action))
        .route("/farm/people", get(ssr::people_page))
        .route("/farm/{farm_id}", get(ssr::farm_detail_page))
        .route(
            "/farm/{farm_id}/agents/{agent_id}",
            get(ssr::farm_agent_page),
        )
        .route(
            "/farm/{farm_id}/agents/{agent_id}/runs",
            post(ssr::launch_run_action),
        )
}
