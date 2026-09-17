//! `/admin/analysis/evaluation` — the zoomed-out view: the latest report for
//! the whole instance, every marketplace's current version against its
//! history, the day-by-day trend, and the engine's configuration folded
//! away at the bottom.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::ai::AiService;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::MarketplaceId;

use super::report_view::ReportView;
use super::views::CriterionView;
use super::{PendingView, load_config, owner_plugin};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis_urls::{analysis_evaluation_url, analysis_version_url};
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::scorecard::{ScorecardView, Scores, score_trend_chart};
use crate::handlers::ssr::types::SvgLineChartView;
use crate::repositories::analysis::retained::RetainedWindow;
use crate::repositories::evaluation::reports::{
    ReportScope, find_latest_report, find_pending_report,
};
use crate::repositories::evaluation::runs_summary::{RunsSummary, get_runs_summary};
use crate::repositories::evaluation::version_scores::{
    MarketplaceScoreRollup, list_marketplace_score_rollups, list_score_trend,
};

const SUMMARY_DAYS: i32 = 30;

#[derive(Serialize)]
pub(super) struct MarketplaceRowView {
    pub marketplace_id: MarketplaceId,
    pub href: String,
    pub versions_href: String,
    pub catalog_href: String,
    pub version_short: Option<String>,
    pub versions: i64,
    pub scorecard: ScorecardView,
    pub scored_sessions: i64,
    pub history_scored_sessions: i64,
}

pub(super) fn marketplace_row(r: &MarketplaceScoreRollup) -> MarketplaceRowView {
    MarketplaceRowView {
        href: analysis_evaluation_url(&r.marketplace_id),
        versions_href: analysis_version_url(&r.marketplace_id, r.current_hash.as_deref()),
        catalog_href: format!(
            "/admin/marketplaces/{}",
            urlencoding::encode(r.marketplace_id.as_str())
        ),
        version_short: r.current_hash.as_deref().map(|h| h.chars().take(12).collect()),
        versions: r.versions,
        scorecard: ScorecardView::with_prior(
            Scores {
                intent: r.current.intent,
                quality: r.current.quality,
                speed: r.current.speed,
                cost: r.current.cost,
            },
            Scores {
                intent: r.history.intent,
                quality: r.history.quality,
                speed: r.history.speed,
                cost: r.history.cost,
            },
            Some(analysis_evaluation_url(&r.marketplace_id)),
            format!(
                "Current version over {} judged sessions; the delta is against every earlier version ({} judged sessions)",
                r.current.scored_sessions, r.history.scored_sessions
            ),
        ),
        scored_sessions: r.current.scored_sessions,
        history_scored_sessions: r.history.scored_sessions,
        marketplace_id: r.marketplace_id.clone(),
    }
}

pub(super) fn trend_chart(
    points: &[crate::repositories::evaluation::version_scores::ScoreTrendPoint],
    days: i32,
    subtitle: String,
) -> SvgLineChartView {
    let end = chrono::Utc::now();
    let window = RetainedWindow {
        start: end - chrono::Duration::days(i64::from(days)),
        end,
    };
    score_trend_chart(
        "Judge scores by day",
        subtitle,
        window,
        &[
            ("Quality", points.iter().map(|p| (p.day, p.quality)).collect()),
            ("Intent", points.iter().map(|p| (p.day, p.intent)).collect()),
            ("Speed", points.iter().map(|p| (p.day, p.speed)).collect()),
            ("Cost", points.iter().map(|p| (p.day, p.cost)).collect()),
        ],
    )
}

#[derive(Serialize)]
struct OverviewContext {
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
    report_model: String,
    report_instructions: String,
    report_window_days: u32,
    summary: RunsSummary,
    judge_spend: String,
    summary_days: i32,
    scorecard: ScorecardView,
    report: Option<ReportView>,
    pending: Option<PendingView>,
    marketplaces: Vec<MarketplaceRowView>,
    trend: SvgLineChartView,
    runs_url: &'static str,
    reports_url: &'static str,
}

pub(crate) async fn page(
    shell: Page,
    Extension(ai): Extension<Option<Arc<AiService>>>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let config = load_config()?;
    let owner = owner_plugin()?;
    let automatic = ProfileBootstrap::get()?.evaluation.automatic;
    let summary = get_runs_summary(&pool, SUMMARY_DAYS).await?;
    let enabled = config.as_ref().is_some_and(|c| c.enabled) && owner.is_some();
    let report = find_latest_report(&pool, &ReportScope::Global).await?;
    let pending = find_pending_report(&pool, &ReportScope::Global).await?;
    let rollups = list_marketplace_score_rollups(&pool).await?;
    let trend = list_score_trend(&pool, None, SUMMARY_DAYS).await?;
    let context = OverviewContext {
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
        report_model: config
            .as_ref()
            .map(|c| format!("{}/{}", c.report.provider, c.report.model))
            .unwrap_or_default(),
        report_instructions: config
            .as_ref()
            .map(|c| c.report.instructions_markdown.clone())
            .unwrap_or_default(),
        report_window_days: config.as_ref().map_or(0, |c| c.report.window_days),
        judge_spend: format_cost(summary.cost_microdollars),
        scorecard: ScorecardView::new(
            Scores {
                intent: summary.intent.score,
                quality: summary.quality.score,
                speed: summary.speed.score,
                cost: summary.cost.score,
            },
            None,
            format!(
                "Mean judge scores over the last {SUMMARY_DAYS} days ({} sessions)",
                summary.quality.sessions
            ),
        ),
        summary,
        summary_days: SUMMARY_DAYS,
        report: report.as_ref().map(ReportView::of),
        pending: pending.as_ref().map(PendingView::of),
        marketplaces: rollups.iter().map(marketplace_row).collect(),
        trend: trend_chart(
            &trend,
            SUMMARY_DAYS,
            "every judged session on the instance; days without one carry the previous mean"
                .to_owned(),
        ),
        runs_url: "/admin/analysis/evaluation/runs",
        reports_url: "/admin/analysis/evaluation/report",
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-evaluation",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
