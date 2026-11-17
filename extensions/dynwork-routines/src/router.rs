//! The desktop API, mounted at [`BASE_PATH`]. It sits under `/api/` so core's
//! session middleware does not treat it as a tracked site page.

use axum::Router;
use axum::routing::{get, put};

use crate::http;
use crate::state::RoutinesState;

pub const BASE_PATH: &str = "/api/dynwork/routines/v1";

pub fn routines_router(state: RoutinesState) -> Router {
    Router::new()
        .route("/changes", get(http::changes::list))
        .route(
            "/routines/{id}",
            put(http::routines::put).delete(http::routines::delete),
        )
        .route("/runs/{id}", put(http::runs::put))
        .route("/templates", get(http::templates::list))
        .route(
            "/templates/{id}/revisions/{revision}",
            get(http::templates::revision),
        )
        .with_state(state)
}
