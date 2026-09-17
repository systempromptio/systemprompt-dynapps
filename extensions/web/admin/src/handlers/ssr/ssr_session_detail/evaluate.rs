//! `POST /admin/sessions/{session_id}/evaluate` — score one session now.
//!
//! Queues the session's criteria (all four, or the one named), claims just
//! those rows and drains them inline, so the redirect lands on a page that
//! already shows the verdict. A failed judge call is on the run row, not on
//! the response.

use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum::response::Redirect;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::evaluation::{Trigger, drain};
use crate::handlers::ssr::analysis::evaluation::trigger::JudgeRequest;
use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::repositories::config::evaluation::CriterionName;
use crate::repositories::evaluation::queue;

const CONVERSATIONS_PAGE: &str = "/admin/analysis/conversations";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvaluateForm {
    #[serde(default)]
    criterion: Option<String>,
    #[serde(default)]
    back: Option<String>,
}

// Why: the form says where it came from; only the session's own page and
// the Conversations list are honoured, so the redirect can never leave the
// console.
fn back_url(session_id: &SessionId, back: Option<&str>) -> String {
    match back {
        Some(url)
            if url == CONVERSATIONS_PAGE || url.starts_with(&format!("{CONVERSATIONS_PAGE}?")) =>
        {
            url.to_owned()
        },
        _ => session_detail_url(session_id),
    }
}

pub(crate) async fn evaluate_session(
    request: JudgeRequest,
    State(pool): State<Arc<PgPool>>,
    Path(session_id): Path<String>,
    Form(form): Form<EvaluateForm>,
) -> AdminHtmlResult<Redirect> {
    request.require_console_write()?;
    let session_id = SessionId::new(session_id.trim());
    if session_id.as_str().is_empty() {
        return Err(AdminError::BadRequest("A session id is required.".to_owned()).into());
    }
    let criteria: Vec<CriterionName> = match form.criterion.as_deref().map(str::trim) {
        None | Some("") => CriterionName::ALL.to_vec(),
        Some(name) => vec![CriterionName::parse(name).ok_or_else(|| {
            AdminError::BadRequest(format!("'{name}' is not an evaluation criterion."))
        })?],
    };
    let judge = request.judge()?;
    match queue::insert_session_now(&pool, &session_id, &criteria).await {
        Ok(_) => {},
        Err(sqlx::Error::RowNotFound) => {
            return Err(AdminError::NotFound(
                "That session has no summary row to evaluate yet.".to_owned(),
            )
            .into());
        },
        Err(e) => return Err(AdminError::Database(e).into()),
    }
    let leases =
        queue::claim_session_leases(&pool, &session_id, judge.config.worker.lease_seconds).await?;
    let trigger = Trigger::Manual {
        requested_by: request.user.user_id.clone(),
    };
    let report = drain(&pool, &judge, &leases, &trigger).await?;
    tracing::info!(
        session_id = %session_id,
        requested_by = %request.user.user_id,
        leased = report.leased,
        succeeded = report.succeeded,
        failed = report.failed,
        "Session evaluated from the console"
    );
    Ok(Redirect::to(&back_url(&session_id, form.back.as_deref())))
}
