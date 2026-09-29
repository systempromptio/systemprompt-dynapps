//! Server-rendered governance and gateway pages.
//!
//! They are the decisions, the approvals queue, the secrets trail, the quota
//! ceilings, the request audit record, the gateway routes and the gateway
//! policy editor. The policy editor writes
//! `ai_gateway_policies`, which core re-reads per request, so its saves are
//! live edits rather than file edits.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/governance", get(handlers::ssr::governance_page))
        .route(
            "/governance/decisions/{decision_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route("/governance/approvals", get(handlers::ssr::approvals_page))
        .route(
            "/governance/secrets",
            get(handlers::ssr::secrets_audit_page),
        )
        .route(
            "/governance/quotas",
            get(handlers::ssr::governance_quotas_page),
        )
}

pub fn request_detail_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/requests/{request_id}",
        get(handlers::ssr::governance_audit_detail_page),
    )
}

pub fn gateway_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/gateway", get(handlers::ssr::gateway_page))
        .route(
            "/gateway/routes/{route_id}",
            get(handlers::ssr::gateway_route_page),
        )
        .route(
            "/gateway/policies",
            get(handlers::ssr::gateway_policies_page).post(handlers::ssr::save_gateway_policy),
        )
        .route(
            "/gateway/policies/delete",
            post(handlers::ssr::delete_gateway_policy),
        )
}

pub fn api_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/api/chain/{id}", get(handlers::ssr::chain_envelope))
        .route("/api/search/resolve", get(handlers::ssr::search_resolve))
}
