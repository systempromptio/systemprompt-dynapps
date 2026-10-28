//! The user-facing Drive pages, nested at `/drive` beside the admin plane.
//!
//! They sit outside the admin router so the non-admin path gate never
//! redirects them: every signed-in person reaches their own documents, and
//! the document page authorizes each load against the document service.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub use crate::handlers::ssr::{DriveDocuments, USER_BASE_URL};

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/", get(handlers::ssr::user_drive_page))
        .route(
            "/{document_id}",
            get(handlers::ssr::user_drive_document_page),
        )
}
