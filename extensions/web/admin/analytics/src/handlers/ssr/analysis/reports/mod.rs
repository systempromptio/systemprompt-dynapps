//! `/admin/analysis/reports` — on-demand AI reports over the deterministic
//! record. The list is the history of every request with its status,
//! headline and assessment and the form that queues a new one; a report page
//! shows the model's headline, assessment, grounded themes with evidence
//! links into Conversations, Skills and Tools, its recommendations, and the
//! digest it was written from. A request writes the row and starts the one
//! model call on a background task (`services::analysis_report`); the page
//! shows a loading widget that polls `/status` until the verdict is there.
//! The digest is computed when the request is made, so a report answers the
//! record as it stood at that moment.

pub(crate) mod banner;
mod form;
mod report_context;
mod view;

use std::sync::Arc;
use systemprompt::traits::BackgroundTasks;

use axum::Json;
use axum::extract::{Extension, Form, Path, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::ai::AiService;

use crate::handlers::ssr::analysis::help::{HelpView, item};
use crate::repositories::analysis::reports::{AnalysisReportRow, find_report, list_reports};
use crate::services::analysis_report::generate_report;
use systemprompt_web_admin_core::catalog_index::inventory_index::InventoryIndex;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::ssr::list_view::SelectOptionView;
use systemprompt_web_admin_core::ssr::page::Page;

use form::{ReportRequestForm, queue, regenerate_form, resolve};
use report_context::report_context;
use view::{ReportsPageContext, report_row};

pub(crate) const REPORTS_URL: &str = "/admin/analysis/reports";
const HISTORY_LIMIT: i64 = 100;

fn reports_help() -> HelpView {
    HelpView::new(
        "AI reports — what they are and how they are grounded",
        "A report is the model's reading of the deterministic record for one scope and window. It sees figures only — never a transcript — and every theme cites the ids it rests on, which become the links on the page.",
    )
    .section(
        "The report",
        vec![
            item("sparkle", "Headline · assessment", "One sentence and one verdict: healthy, watch or degraded."),
            item("layers", "Themes", "Findings grounded in the digest, most important first, each with a severity and evidence chips that open the skill, conversation, model, tool or person cited."),
            item("check", "Recommendations", "Concrete actions on this console, ordered by priority."),
            item("report", "Digest", "The exact figures the model was given — totals, top skills, models and people, best and worst judged conversations, cost outliers, denied tools, intent and client mix — kept with the report for audit."),
        ],
    )
    .section(
        "Generating",
        vec![
            item("calendar", "Scope and window", "Whole instance, one marketplace, one skill, or the filters of the page you came from (Report on this view). The window is anchored to now."),
            item("clock", "On demand only", "A request queues a row; the analysis_report job generates it when run — `systemprompt infra jobs run analysis_report`. Nothing is scheduled, so no report costs anything unless someone asked for it."),
            item("coins", "Cost", "One structured call to the configured judge model per report, audited under the job's identity and shown on the row."),
        ],
    )
}

fn require_console(shell: &Page) -> Result<(), AdminError> {
    if shell.user.is_console {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Console access required".into()))
    }
}

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let rows = list_reports(&pool, HISTORY_LIMIT).await?;
    let index = InventoryIndex::build(&systemprompt_web_admin_core::request::get_services_path()?);
    let context = ReportsPageContext {
        page: "analysis-reports",
        title: "Reports",
        has_rows: !rows.is_empty(),
        rows: rows.iter().map(report_row).collect(),
        marketplace_options: index
            .marketplaces
            .iter()
            .map(|m| SelectOptionView {
                value: m.id.as_str().to_owned(),
                label: m.name.clone(),
                selected: false,
            })
            .collect(),
        help: reports_help(),
    };
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &shell.engine,
            "analysis-reports",
            &context,
            &shell.user,
            &shell.marketplace,
        ),
    )
}

async fn load_report(pool: &PgPool, id: &str) -> Result<AnalysisReportRow, AdminError> {
    let id = id.trim();
    if id.is_empty() || id.len() > 64 {
        return Err(AdminError::BadRequest(
            "A report id is required.".to_owned(),
        ));
    }
    find_report(pool, id)
        .await?
        .ok_or_else(|| AdminError::NotFound("No report matches that id.".to_owned()))
}

pub(crate) async fn report_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let row = load_report(&pool, &id).await?;
    let context = report_context(&row);
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &shell.engine,
            "analysis-report",
            &context,
            &shell.user,
            &shell.marketplace,
        ),
    )
}

pub(crate) struct ReportRunner {
    ai: Option<Arc<AiService>>,
    tasks: BackgroundTasks,
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ReportRunner {
    type Rejection = axum::extract::rejection::ExtensionRejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Extension(ai) = Extension::from_request_parts(parts, state).await?;
        let Extension(tasks) = Extension::from_request_parts(parts, state).await?;
        Ok(Self { ai, tasks })
    }
}

async fn start(
    tasks: &BackgroundTasks,
    pool: &Arc<PgPool>,
    ai: Option<Arc<AiService>>,
    shell: &Page,
    form: &ReportRequestForm,
) -> AdminResult<String> {
    let resolved = resolve(pool, &shell.user, form).await?;
    let queued = queue(pool, &shell.user, resolved).await?;
    let row = load_report(pool, &queued.id).await?;
    tasks.spawn(
        "analysis_report",
        generate_report(
            Arc::clone(pool),
            ai,
            shell.user.user_id.clone(),
            row,
            queued.lease_token,
        ),
    );
    tracing::info!(report_id = %queued.id, requested_by = %shell.user.user_id, "analysis report generation started from the console");
    Ok(queued.id)
}

fn report_redirect(id: &str) -> Redirect {
    Redirect::to(&format!("{REPORTS_URL}/{}", urlencoding::encode(id)))
}

pub(crate) async fn create(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    ReportRunner { ai, tasks }: ReportRunner,
    headers: HeaderMap,
    Form(form): Form<ReportRequestForm>,
) -> AdminHtmlResult<Redirect> {
    require_console(&shell)?;
    systemprompt_web_admin_core::request::require_write_origin(&headers)?;
    let id = start(&tasks, &pool, ai, &shell, &form).await?;
    Ok(report_redirect(&id))
}

pub(crate) async fn regenerate(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    ReportRunner { ai, tasks }: ReportRunner,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    require_console(&shell)?;
    systemprompt_web_admin_core::request::require_write_origin(&headers)?;
    let row = load_report(&pool, &id).await?;
    let new_id = start(&tasks, &pool, ai, &shell, &regenerate_form(&row)).await?;
    Ok(report_redirect(&new_id))
}

#[derive(Debug, Serialize)]
pub(crate) struct ReportStatus {
    status: String,
    error: Option<String>,
    elapsed_ms: i64,
}

pub(crate) async fn status(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ReportStatus>> {
    require_console(&shell)?;
    let row = load_report(&pool, &id).await?;
    Ok(Json(ReportStatus {
        status: row.status.clone(),
        error: row.last_error.clone(),
        elapsed_ms: (chrono::Utc::now() - row.created_at).num_milliseconds(),
    }))
}
