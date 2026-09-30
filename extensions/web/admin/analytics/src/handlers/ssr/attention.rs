//! `GET /admin/governance/attention` — the counts behind the sidebar's to-do
//! badges, as JSON.
//!
//! The shell draws the access-control drift badge from a value it already
//! holds in memory; the approval queue lives in the database and changes
//! while a page is open. `services/attention.js` asks here once a minute and
//! updates the badges in place, so an operator on any page sees a call
//! arrive in the queue without reloading. Console only: a participant's
//! sidebar carries no governance group and gets a 403.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::overview::queues::count_pending_approvals;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::sync::attention::access_attention;
use systemprompt_web_admin_core::types::UserContext;

#[derive(Debug, Serialize)]
pub(crate) struct AttentionView {
    approvals: i64,
    access: usize,
}

pub(crate) async fn attention_counts(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Json<AttentionView>> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    Ok(Json(AttentionView {
        approvals: count_pending_approvals(&pool).await?,
        access: access_attention(),
    }))
}
