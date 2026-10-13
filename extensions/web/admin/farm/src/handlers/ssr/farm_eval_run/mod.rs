//! `GET /admin/farm/evals/runs/{eval_run_id}` — one eval run: its suite,
//! agent and who launched it, the KPI strip, and an expandable row per case
//! listing each assertion's outcome, the judge's score and rationale, and the
//! farm run behind it. While the eval run is queued or running the live half
//! polls `/admin/api/farm/evals/runs/{eval_run_id}`.

mod view;

use crate::types::FarmAgentId;
use std::sync::Arc;
use systemprompt::identifiers::UserId;

use axum::extract::{Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time};
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

pub(crate) use view::{EvalRunLiveView, eval_run_live};

use crate::repositories::evals::{find_eval_run, list_eval_results};
use crate::repositories::rows::EvalRunRow;
use crate::types::EvalRunId;

#[derive(Debug, Serialize)]
struct HeaderView {
    eval_run_id: String,
    suite_id: String,
    farm_id: String,
    farm_url: String,
    agent_id: FarmAgentId,
    agent_url: String,
    user_id: UserId,
    user_url: String,
    created_local: String,
    created_relative: String,
    run_now_url: String,
}

#[derive(Debug, Serialize)]
struct EvalRunPage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    header: HeaderView,
    live: EvalRunLiveView,
}

pub(crate) fn parse_eval_run_id(raw: &str) -> Result<EvalRunId, AdminError> {
    if raw.is_empty() || raw.len() > 128 || raw.chars().any(char::is_control) {
        return Err(AdminError::NotFound("Eval run not found.".to_owned()));
    }
    Ok(EvalRunId::new(raw))
}

fn header(run: &EvalRunRow) -> HeaderView {
    let farm = run.farm_id.as_str();
    let agent = run.agent_id.as_str();
    HeaderView {
        eval_run_id: run.id.as_str().to_owned(),
        suite_id: run.suite_id.as_str().to_owned(),
        farm_id: farm.to_owned(),
        farm_url: format!("/admin/farm/{farm}"),
        agent_id: run.agent_id.clone(),
        agent_url: format!("/admin/farm/{farm}/agents/{agent}"),
        user_id: run.user_id.clone(),
        user_url: format!("/admin/users/{}", run.user_id.as_str()),
        created_local: local_time(run.created_at),
        created_relative: relative_time(run.created_at),
        run_now_url: format!("/admin/farm/evals/{}/run", run.suite_id.as_str()),
    }
}

pub(crate) async fn eval_run_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(eval_run_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let id = parse_eval_run_id(&eval_run_id)?;
    let run = find_eval_run(&pool, &id)
        .await?
        .ok_or_else(|| AdminError::NotFound("Eval run not found.".to_owned()))?;
    let mut faults = Faults::new();
    let results = faults.take("case results", list_eval_results(&pool, &id).await);
    let title = format!("Eval run · {}", run.suite_id.as_str());
    let ctx = EvalRunPage {
        page: "farm-eval-run",
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::link("Evals", "/admin/farm/evals"),
            BreadcrumbView::current(&title),
        ],
        title,
        header: header(&run),
        live: eval_run_live(&run, &results),
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-eval-run",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
