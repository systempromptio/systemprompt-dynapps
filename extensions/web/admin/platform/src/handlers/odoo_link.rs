//! `POST|DELETE /api/public/admin/user/odoo-link` — the signed-in person links
//! or unlinks their own Odoo account. The key is accepted on link and never
//! returned; Odoo verifies the pair before anything is stored.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::types::UserContext;

use crate::services::machine_credentials::{
    OdooLinkRequest, link_odoo_account, unlink_odoo_account,
};

#[derive(Debug, Serialize)]
struct Linked {
    odoo_uid: i64,
}

pub(crate) async fn link(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<OdooLinkRequest>,
) -> AdminResult<Response> {
    let odoo_uid = link_odoo_account(&pool, &user_ctx.user_id, body, &user_ctx.user_id).await?;
    Ok(Json(Linked { odoo_uid }).into_response())
}

pub(crate) async fn unlink(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    unlink_odoo_account(&pool, &user_ctx.user_id, &user_ctx.user_id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
