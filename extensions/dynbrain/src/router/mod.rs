//! The Brain routes, relative to [`EXTENSION_BASE_PATH`].
//!
//! The base path is the Brain's own API, not `/api/dynwork`: core nests each
//! extension router at its own base path, and two extensions cannot share one.
//! It sits under `/api/` for the relay's reason: core's session middleware
//! treats every other path as a tracked site page.
//!
//! The routes know nothing about where the documentation lives: each one
//! calls the [`crate::brain::Brain`] in the state and returns what it answers.
//!
//! Two routes take no bearer token, because their URL is the permission:
//! `POST uploads/{id}` (the one-time upload URL) and `GET assets/{id}/raw`
//! (the signed download URL). Both check the signature before anything else.

pub mod assets;
pub mod error;
pub mod extract;
pub mod read;
pub mod state;
pub mod write;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};

use error::HttpFault;
use state::BrainState;

pub const EXTENSION_BASE_PATH: &str = "/api/dynbrain/v1";
pub const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

pub fn brain_router(state: BrainState) -> Router {
    Router::new()
        .route("/spaces", get(read::spaces))
        .route("/spaces/{id}/articles", get(read::articles))
        .route("/spaces/{id}/outline", get(read::outline))
        .route("/spaces/{id}/export", get(read::export))
        .route("/spaces/{id}/schema", get(read::schema))
        .route(
            "/spaces/{id}/grants",
            get(read::space_grants).put(write::set_space_grants),
        )
        .route("/articles", post(write::create))
        .route("/articles/tree", post(write::create_tree))
        .route("/articles/{id}", get(read::article).patch(write::update))
        .route("/articles/{id}/metadata", get(read::metadata))
        .route("/articles/{id}/move", post(write::move_article))
        .route("/articles/{id}/trash", post(write::trash))
        .route(
            "/articles/{id}/grants",
            get(read::article_grants).put(write::set_article_grants),
        )
        .route("/articles/{id}/inherit", put(write::set_inherit))
        .route(
            "/articles/{id}/assets",
            get(assets::list).post(assets::attach),
        )
        .route("/articles/{id}/assets/upload-url", post(assets::upload_url))
        .route("/uploads/{id}", post(assets::receive))
        .route("/assets/{id}", get(assets::content))
        .route("/assets/{id}/link", get(assets::link))
        .route("/assets/{id}/raw", get(assets::raw))
        .route("/search", post(read::search))
        .route("/answer", post(read::answer))
        .fallback(|| async { HttpFault::NotFound })
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}
