//! `/admin` — the console landing page.
//!
//! One health-and-money screen: what the instance spent, how much of it
//! failed, which models and clients carried it, how the tokens, tool calls
//! and latency moved over the window, what is waiting on a human, and which
//! containers the money went to. Every tile is a link to the page that owns
//! the detail, carrying the window the reader is already looking at, so the
//! overview never becomes a place to read numbers off rather than a place to
//! start from.
//!
//! A non-console viewer is redirected to their profile rather than refused:
//! every page this shell offers them would refuse them anyway, so a 403 here
//! would be a dead end where a redirect is a door.

mod charts;
mod data;
mod fragment;
mod kpi_tiles;
mod kpis_view;
mod panels_view;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

pub(crate) use fragment::overview_fragment;
use view::OverviewRange;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct OverviewQuery {
    pub preset: Option<String>,
}

pub(crate) async fn overview_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<OverviewQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Ok(Redirect::to("/admin/profile").into_response());
    }

    let range = OverviewRange::from_query(query.preset.as_deref());
    let loaded = data::load_overview(&pool, range).await;
    let page = view::overview_page(range, &loaded);

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &engine, "overview", &page, &user_ctx, &mkt_ctx,
        ),
    )
}
