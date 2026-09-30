//! Export routes: one surface for every table the console renders.
//!
//! They serve the file in any format and column selection, and the preview the
//! dialog counts from (`export::registry` says which tables exist) — and the
//! document shape, one conversation with every message body, tool call,
//! decision, finding and hook event, alone or as JSON Lines for a set.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::export;
use systemprompt_web_admin_core::export::handler as table_export;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/export/{dataset}", get(table_export::export_file))
        .route(
            "/export/{dataset}/preview",
            get(table_export::export_preview),
        )
        // Why: axum routes a static segment ahead of `{dataset}`, so no
        // dataset may be named `transcripts` or it is shadowed.
        .route(
            "/export/transcripts",
            get(export::document::handler::export_conversations),
        )
        .route(
            "/export/transcripts/preview",
            get(export::document::handler::export_conversations_preview),
        )
        .route(
            "/export/transcripts/{context_id}",
            get(export::document::handler::export_conversation),
        )
}
