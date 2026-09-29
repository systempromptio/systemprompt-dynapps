//! Server-rendered governance pages: the decisions, the approvals queue, the
//! secrets trail and the quota ceilings, merged from the governance crate,
//! plus the attention counts the sidebar badge polls.
//!
//! Split from `ssr.rs` on size alone; the sidebar group and the handlers are
//! unchanged.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;
use systemprompt_web_admin_governance::routes::ssr_governance as governance_routes;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    governance_routes::routes().route(
        "/governance/attention",
        get(handlers::ssr::attention_counts),
    )
}
