//! The Drive pages, nested at `/drive` beside the admin plane: the listing,
//! one document's workspace and block editor, and the names the pages show.
//!
//! They sit outside the admin router so the non-admin path gate never
//! redirects them: every signed-in person reaches their own documents, and
//! each page authorizes its load against the document service.
//! The `/classes` routes read and set a folder's or document's class, as
//! the caller, beside the pages that show it. Every route runs its document
//! calls as the person (`drive_actor`).

use std::sync::Arc;

use axum::Router;
use axum::middleware::from_fn;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub use crate::handlers::ssr::{DriveDocuments, USER_BASE_URL};

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/", get(handlers::ssr::drive_page))
        .route("/people", get(handlers::ssr::drive_people))
        .route("/{document_id}", get(handlers::ssr::drive_workspace_page))
        .route(
            "/{document_id}/blocks",
            get(handlers::ssr::drive_document_page),
        )
        .route(
            "/classes",
            get(handlers::document_class::list_class_options),
        )
        .route(
            "/classes/{kind}",
            get(handlers::document_class_batch::list_classes_handler),
        )
        .route(
            "/classes/{kind}/{id}",
            get(handlers::document_class::get_class_handler)
                .put(handlers::document_class::set_class_handler),
        )
        .layer(from_fn(super::drive_actor::person_actor_middleware))
}
