//! `GET /admin/api/farm/activity` — the farms overview's KPI strip and
//! running board as a fragment, rendered from the same partial and the same
//! view the full page embeds. The overview's live list polls it and swaps
//! each `data-live-region` in place.
//!
//! A non-console caller is refused rather than redirected: a fetch that
//! followed the redirect would swap a login page into the board.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::UserContext;

use crate::handlers::ssr::farm_overview::activity::{ACTIVITY_PARTIAL, load_activity};

pub(crate) async fn activity_fragment(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }
    let mut faults = Faults::new();
    let view = load_activity(&pool, &mut faults).await;
    let html = engine.render(ACTIVITY_PARTIAL, &view)?;
    Ok((StatusCode::OK, [(CACHE_CONTROL, "no-store")], Html(html)).into_response())
}
