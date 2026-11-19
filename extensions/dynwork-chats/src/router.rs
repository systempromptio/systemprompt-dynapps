//! The chats routes, relative to [`EXTENSION_BASE_PATH`].
//!
//! The base path is the chats API itself, not the relay's `/api/dynwork`:
//! core nests each extension router at its own base path, and two extensions
//! cannot share one. It sits under `/api/` for the relay's reason: core's
//! session middleware treats every other path as a tracked site page.

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};

use crate::error::ChatsError;
use crate::http;
use crate::state::ChatsState;

pub const EXTENSION_BASE_PATH: &str = "/api/dynwork/chats/v1";
pub const MAX_BODY_BYTES: usize = 64 * 1024;

pub fn chats_router(state: ChatsState) -> Router {
    Router::new()
        .route("/models", get(http::models::list))
        .route("/changes", get(http::changes::list))
        .route(
            "/threads/{id}",
            put(http::threads::put).delete(http::threads::delete),
        )
        .route("/threads/{id}/messages", get(http::messages::list))
        .route("/threads/{id}/prompt", post(http::prompt::prompt))
        .route("/threads/{id}/interrupt", post(http::prompt::interrupt))
        .route("/threads/{id}/log", get(http::log::log))
        .fallback(|| async { ChatsError::NotFound })
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}
