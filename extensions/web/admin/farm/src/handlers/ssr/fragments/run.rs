//! `GET /admin/api/farm/runs/{run_id}?after=<seq>` — the live block of the
//! run page as an HTML fragment.
//!
//! It is the same `farm/run-live` partial rendered from the same
//! [`RunData`] the page uses, carrying only the events after `after` (at most
//! `FRAGMENT_EVENT_CAP`) and stamping `data-run-status`, `data-last-seq`
//! and `data-live-active` on its root so the script knows what it has and
//! whether to keep polling. A non-console caller gets a bare 403, not a
//! login redirect, because the caller is a script swapping markup.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::page::Page;

use crate::handlers::ssr::farm_run_detail::context::live_view;
use crate::handlers::ssr::farm_run_detail::{load_run_data, not_found};
use crate::handlers::ssr::run_events_view::events_after;
use crate::types::FarmRunId;

const LIVE_PARTIAL: &str = "farm/run-live";

#[derive(Debug, Deserialize)]
pub(crate) struct RunFragmentQuery {
    #[serde(default)]
    after: Option<i64>,
}

pub(crate) async fn run_fragment(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(run_id): Path<String>,
    Query(query): Query<RunFragmentQuery>,
) -> AdminResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }
    let mut faults = Faults::new();
    let Some(data) = load_run_data(&pool, &FarmRunId::new(run_id), &mut faults).await? else {
        return Err(not_found());
    };
    let after = query.after.unwrap_or(0).max(0);
    let fresh = events_after(&data.events, after);
    let mut live = live_view(&data, fresh);
    live.last_seq = live.events.last().map_or(after, |e| e.seq);
    let html = shell.engine.render(LIVE_PARTIAL, &live)?;
    Ok((StatusCode::OK, [(CACHE_CONTROL, "no-store")], Html(html)).into_response())
}
