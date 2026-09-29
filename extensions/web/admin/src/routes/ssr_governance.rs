//! Server-rendered governance pages: the decisions, the approvals queue, the
//! secrets trail and the quota ceilings.
//!
//! Split from `ssr.rs` on size alone; the sidebar group and the handlers are
//! unchanged.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/governance", get(handlers::ssr::governance_page))
        .route(
            "/governance/decisions/{decision_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route("/governance/approvals", get(handlers::ssr::approvals_page))
        .route(
            "/governance/attention",
            get(handlers::ssr::attention_counts),
        )
        .route(
            "/governance/secrets",
            get(handlers::ssr::secrets_audit_page),
        )
        .route(
            "/governance/quotas",
            get(handlers::ssr::governance_quotas_page),
        )
}
