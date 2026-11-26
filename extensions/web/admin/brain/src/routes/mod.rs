//! The `/admin/brain/*` pages. Paths are relative to the admin SSR mount,
//! which runs the user-context and non-admin gates before these handlers.

use axum::routing::{get, post};
use axum::{Extension, Router};

use crate::source::Source;
use crate::{access, actions, article, ingest, lab, overview, spaces, usage};

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    routes_with(Source::Live)
}

pub fn routes_with<S: Clone + Send + Sync + 'static>(source: Source) -> Router<S> {
    Router::new()
        .route("/brain", get(overview::overview_page))
        .route("/brain/spaces", get(spaces::spaces_page))
        .route("/brain/articles/{id}", get(article::article_page))
        .route("/brain/articles/{id}/reindex", post(actions::reindex))
        .route("/brain/articles/{id}/access", post(access::set_access))
        .route("/brain/search", get(lab::search_page))
        .route("/brain/ingest", get(ingest::ingest_page))
        .route("/brain/ingest/{job}/retry", post(actions::retry))
        .route("/brain/usage", get(usage::usage_page))
        .layer(Extension(source))
}
