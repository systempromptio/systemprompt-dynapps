//! `GET /admin/api/overview?preset=` — the landing page's body as a fragment.
//!
//! The live layer on `/admin` asks here for fresh figures and morphs them in
//! place; the fragment is the very same `overview/live-body` partial the full
//! page embeds, rendered from the very same context, so a number can never
//! read differently after a refresh than it did on first load.
//!
//! A non-console viewer is refused rather than redirected: a fetch that
//! followed the page's redirect would land on profile HTML and morph it into
//! the overview.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::UserContext;

use super::view::OverviewRange;
use super::{OverviewQuery, data, view};

const LIVE_BODY_PARTIAL: &str = "overview/live-body";

pub(crate) async fn overview_fragment(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<OverviewQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }

    let range = OverviewRange::from_query(query.preset.as_deref());
    let loaded = data::load_overview(&pool, range).await;
    let page = view::overview_page(range, &loaded);
    let html = engine.render(LIVE_BODY_PARTIAL, &page)?;

    Ok((StatusCode::OK, [(CACHE_CONTROL, "no-store")], Html(html)).into_response())
}
