//! Export routes: one surface for every table the console renders — the
//! file in any format and column selection, and the preview the dialog
//! counts from (`export::registry` says which tables exist) — and the
//! document shape, one conversation with every message body, tool call,
//! decision, finding and hook event, alone or as JSON Lines for a set.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::export;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/export/{dataset}", get(export::handler::export_file))
        .route(
            "/export/{dataset}/preview",
            get(export::handler::export_preview),
        )
        // Why: static segments win over `{dataset}`, so the document routes
        // sit beside the table route without a dataset named `conversation`.
        .route(
            "/export/conversation/{context_id}",
            get(export::document::handler::export_conversation),
        )
        .route(
            "/export/conversations",
            get(export::document::handler::export_conversations),
        )
}
