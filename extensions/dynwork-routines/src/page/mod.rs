//! `/admin/routines` — "My routines".
//!
//! The signed-in person's routines: which computer runs each, whether it is
//! on, how its last runs went, and pause, resume and delete. Creating and
//! editing a routine is `DynWork`'s form.
//!
//! Identity-scoped rather than admin-gated: every read and write names the
//! viewer's own user. The router runs over the write pool, so the page a
//! pause redirects to already shows it. Each action is a form post that
//! writes a new revision, sends the change hint to the person's desktops and
//! returns here with a flash.

mod view;

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::http::hint;
use crate::ids::RoutineId;
use crate::store::{self, StoreError, WriteOutcome};

pub const PAGE_URL: &str = "/admin/routines";
const RECENT_RUNS: i64 = 5;

pub fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/routines", get(routines_page))
        .route("/routines/{id}/pause", post(pause))
        .route("/routines/{id}/resume", post(resume))
        .route("/routines/{id}/delete", post(delete))
}

async fn routines_page(shell: Page, State(pool): State<Arc<PgPool>>) -> AdminHtmlResult<Response> {
    let user = &shell.user.user_id;
    let mut faults = Faults::new();
    let routines = faults.take("routines", store::list_live_routines(&pool, user).await);
    let runs = faults.take(
        "recent runs",
        store::list_recent_runs(&pool, user, RECENT_RUNS).await,
    );
    let ctx = view::page(routines, &runs);
    Ok(render_page_with_faults(
        &shell.engine,
        "routines",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

async fn pause(shell: Page, State(pool): State<Arc<PgPool>>, Path(id): Path<String>) -> Response {
    act(&pool, &shell.user.user_id, &id, Action::Enable(false)).await
}

async fn resume(shell: Page, State(pool): State<Arc<PgPool>>, Path(id): Path<String>) -> Response {
    act(&pool, &shell.user.user_id, &id, Action::Enable(true)).await
}

async fn delete(shell: Page, State(pool): State<Arc<PgPool>>, Path(id): Path<String>) -> Response {
    act(&pool, &shell.user.user_id, &id, Action::Delete).await
}

#[derive(Clone, Copy)]
enum Action {
    Enable(bool),
    Delete,
}

impl Action {
    const fn done(self) -> &'static str {
        match self {
            Self::Enable(true) => "Routine resumed",
            Self::Enable(false) => "Routine paused",
            Self::Delete => "Routine deleted",
        }
    }
}

async fn act(pool: &PgPool, user: &UserId, raw: &str, action: Action) -> Response {
    let Ok(id) = RoutineId::parse(raw) else {
        return redirect_with_flash(PAGE_URL, FlashTone::Err, "Routine not found").into_response();
    };
    let outcome = match action {
        Action::Enable(enabled) => store::set_enabled(pool, user, &id, enabled, Utc::now()).await,
        Action::Delete => store::delete_routine(pool, user, &id, None, Utc::now()).await,
    };
    let (tone, message) = outcome_flash(outcome, action);
    if tone == FlashTone::Ok {
        hint(pool, user, None).await;
    }
    redirect_with_flash(PAGE_URL, tone, message).into_response()
}

fn outcome_flash(
    outcome: Result<WriteOutcome, StoreError>,
    action: Action,
) -> (FlashTone, &'static str) {
    match outcome {
        Ok(WriteOutcome::Written(_)) => (FlashTone::Ok, action.done()),
        Ok(WriteOutcome::Conflict(_)) => (FlashTone::Info, "The routine was already deleted"),
        Ok(_) => (FlashTone::Err, "Routine not found"),
        Err(error) => {
            tracing::error!(%error, "dynwork routines page action failed");
            (
                FlashTone::Err,
                "The routine could not be changed. Try again.",
            )
        },
    }
}
