//! `GET /admin/api/farm/runs?…` — the runs explorer's KPI strip and table as
//! an HTML fragment, read with the page's own query and rendered from the
//! page's own view model through the `farm/runs-live` partial.
//!
//! A non-console caller is refused with 403 rather than redirected: the
//! caller is the live-list script, and a fetch that followed a login redirect
//! would swap a login page into the table.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::handlers::ssr::farm_runs::query::RunsQuery;
use crate::handlers::ssr::farm_runs::{live_view, read_runs, time_range};

const LIVE_PARTIAL: &str = "farm/runs-live";

pub(crate) async fn runs_fragment(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RunsQuery>,
) -> AdminResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }
    let read = read_runs(&pool, &query, time_range(&query)).await?;
    let html = shell
        .engine
        .render(LIVE_PARTIAL, &live_view(&query, &read))?;
    Ok((StatusCode::OK, [(CACHE_CONTROL, "no-store")], Html(html)).into_response())
}
