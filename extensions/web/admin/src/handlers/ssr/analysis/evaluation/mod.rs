//! `/admin/analysis/evaluation` — the judge as configured, and the runs it
//! has made.
//!
//! The criteria page renders the exact system prompt each criterion sends,
//! built by the same function the job uses, so what the operator reads is
//! what the model received. The runs table and its drilldown are the audit
//! trail: prompts, response, tokens, cost, and the gateway request behind it.

pub(crate) mod actions;
pub(crate) mod trigger;
pub(crate) mod views;

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::ai::AiService;
use systemprompt::config::ProfileBootstrap;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared::get_baked_services_path;
use crate::handlers::ssr::list_view::Pagination;
use crate::repositories::config::evaluation::{LoadedEvaluation, load_evaluation};
use crate::repositories::evaluation::runs::{self, RunsFilter};
use crate::repositories::evaluation::runs_summary::{RunsSummary, get_runs_summary};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use views::{CriterionView, RunDetailView, RunView};

const PAGE_SIZE: i64 = 50;
const SUMMARY_DAYS: i32 = 30;

#[derive(Serialize)]
struct CriteriaContext {
    page: &'static str,
    title: &'static str,
    configured: bool,
    enabled: bool,
    automatic: bool,
    inference_available: bool,
    can_trigger: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_plugin: Option<String>,
    quiet_minutes: u32,
    min_user_prompts: u32,
    lookback_hours: u32,
    config_hash_short: String,
    criteria: Vec<CriterionView>,
    summary: RunsSummary,
    judge_spend: String,
    summary_days: i32,
    intent_avg: String,
    quality_avg: String,
    speed_avg: String,
    cost_avg: String,
    runs_url: &'static str,
}

fn average(score: Option<f64>) -> String {
    score.map_or_else(|| "—".to_owned(), |s| format!("{s:.0}"))
}

pub(crate) fn load_config() -> Result<Option<LoadedEvaluation>, AdminError> {
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let path = get_baked_services_path()?;
    Ok(load_evaluation(&path, &services.providers, &services.ai)?)
}

fn owner_plugin() -> Result<Option<String>, AdminError> {
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    Ok(services
        .plugins
        .values()
        .find(|p| p.enabled && p.hooks.evaluation)
        .map(|p| p.id.to_string()))
}

pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(ai): Extension<Option<Arc<AiService>>>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let config = load_config()?;
    let owner = owner_plugin()?;
    let automatic = ProfileBootstrap::get()?.evaluation.automatic;
    let summary = get_runs_summary(&pool, SUMMARY_DAYS).await?;
    let enabled = config.as_ref().is_some_and(|c| c.enabled) && owner.is_some();
    let context = CriteriaContext {
        page: "analysis-evaluation",
        title: "Evaluation",
        configured: config.is_some(),
        enabled,
        automatic,
        inference_available: ai.is_some(),
        can_trigger: enabled && ai.is_some(),
        owner_plugin: owner,
        quiet_minutes: config.as_ref().map_or(0, |c| c.session_close.quiet_minutes),
        min_user_prompts: config
            .as_ref()
            .map_or(0, |c| c.session_close.min_user_prompts),
        lookback_hours: config
            .as_ref()
            .map_or(0, |c| c.session_close.lookback_hours),
        config_hash_short: config
            .as_ref()
            .map(|c| c.config_hash.chars().take(12).collect())
            .unwrap_or_default(),
        criteria: config
            .as_ref()
            .map(|c| c.criteria.iter().map(CriterionView::of).collect())
            .unwrap_or_default(),
        judge_spend: crate::handlers::ssr::format::format_cost(summary.cost_microdollars),
        intent_avg: average(summary.intent.score),
        quality_avg: average(summary.quality.score),
        speed_avg: average(summary.speed.score),
        cost_avg: average(summary.cost.score),
        summary,
        summary_days: SUMMARY_DAYS,
        runs_url: "/admin/analysis/evaluation/runs",
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluation",
        &context,
        &user,
        &marketplace,
    ))
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunsQuery {
    criterion: Option<String>,
    status: Option<String>,
    host: Option<String>,
    session: Option<String>,
    page: Option<u32>,
}

impl RunsQuery {
    fn url(&self, page: i64) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        for (key, value) in [
            ("criterion", self.criterion.clone()),
            ("status", self.status.clone()),
            ("host", self.host.clone()),
            ("session", self.session.clone()),
        ] {
            if let Some(value) = value.filter(|v| !v.is_empty()) {
                pairs.push((key, value));
            }
        }
        if page > 1 {
            pairs.push(("page", page.to_string()));
        }
        let query = pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={}", urlencoding::encode(&value)))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            "/admin/analysis/evaluation/runs".to_owned()
        } else {
            format!("/admin/analysis/evaluation/runs?{query}")
        }
    }
}

fn clean(value: Option<&String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

#[derive(Serialize)]
struct RunsContext {
    page: &'static str,
    title: &'static str,
    query: RunsQuery,
    runs: Vec<RunView>,
    has_runs: bool,
    pagination: Pagination,
    criteria: [&'static str; 4],
    statuses: [&'static str; 2],
    export: crate::export::ExportView,
}

pub(crate) async fn runs_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RunsQuery>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let page = i64::from(query.page.unwrap_or(1).max(1));
    let filter = RunsFilter {
        criterion: clean(query.criterion.as_ref()),
        status: clean(query.status.as_ref()),
        host: clean(query.host.as_ref()),
        session: clean(query.session.as_ref()),
        limit: PAGE_SIZE,
        offset: (page - 1) * PAGE_SIZE,
    };
    let (rows, total) = runs::list_runs_paged(&pool, &filter).await?;
    let shown = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let total_pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let pagination = Pagination {
        current_page: page.min(total_pages),
        total_pages,
        first_row: if shown == 0 {
            0
        } else {
            (page - 1) * PAGE_SIZE + 1
        },
        last_row: (page - 1) * PAGE_SIZE + shown,
        total_rows: total,
        noun: "runs",
        has_prev: page > 1,
        has_next: page < total_pages,
        prev_url: (page > 1).then(|| query.url(page - 1)),
        next_url: (page < total_pages).then(|| query.url(page + 1)),
    };
    let export = crate::export::ExportView::single(
        "evaluation-runs",
        &crate::export::view::query_string(&[
            ("criterion", query.criterion.as_deref()),
            ("status", query.status.as_deref()),
            ("host", query.host.as_deref()),
            ("session", query.session.as_deref()),
        ]),
    );
    let context = RunsContext {
        page: "analysis-evaluation-runs",
        title: "Judge runs",
        query,
        export,
        has_runs: !rows.is_empty(),
        runs: rows.iter().map(RunView::of).collect(),
        pagination,
        criteria: ["intent", "quality", "speed", "cost"],
        statuses: ["succeeded", "failed"],
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluation-runs",
        &context,
        &user,
        &marketplace,
    ))
}

#[derive(Serialize)]
struct RunContext {
    page: &'static str,
    title: String,
    detail: RunDetailView,
    runs_url: &'static str,
}

pub(crate) async fn run_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let detail = runs::find_run(&pool, id.trim())
        .await?
        .ok_or_else(|| AdminError::NotFound("No judge run has that id.".to_owned()))?;
    let detail = RunDetailView::of(&detail);
    let context = RunContext {
        page: "analysis-evaluation-run",
        title: format!(
            "Judge run · {} · {}",
            detail.run.criterion, detail.run.id_short
        ),
        detail,
        runs_url: "/admin/analysis/evaluation/runs",
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluation-run",
        &context,
        &user,
        &marketplace,
    ))
}
