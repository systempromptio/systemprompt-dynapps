//! `/admin/analysis/evaluation/{marketplace_id}` — the report for one
//! marketplace: its latest findings, every version's scores newest first,
//! the day-by-day trend, and the skills the judge rated worst and best.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::ai::AiService;
use systemprompt::identifiers::MarketplaceId;

use super::overview::trend_chart;
use super::report_view::ReportView;
use super::{PendingView, load_config};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis_urls::{
    ANALYSIS_EVALUATION_URL, analysis_skill_url, analysis_version_url,
};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use crate::handlers::ssr::types::{BreadcrumbView, SvgLineChartView};
use crate::repositories::evaluation::report_inputs::{
    ReportSkillFigure, ReportWindow, list_report_skill_figures,
};
use crate::repositories::evaluation::reports::{
    ReportScope, find_latest_report, find_pending_report,
};
use crate::repositories::evaluation::version_scores::{
    VersionScoreRow, list_score_trend, list_version_scores,
};

const TREND_DAYS: i32 = 30;
const SKILL_ROWS: i64 = 12;

#[derive(Serialize)]
struct VersionRowView {
    hash_short: String,
    href: String,
    is_current: bool,
    since: String,
    until: Option<String>,
    sessions: i64,
    scored_sessions: i64,
    scorecard: ScorecardView,
}

fn version_row(marketplace: &MarketplaceId, v: &VersionScoreRow, previous: Option<&VersionScoreRow>) -> VersionRowView {
    VersionRowView {
        hash_short: v.content_hash.chars().take(12).collect(),
        href: analysis_version_url(marketplace, Some(&v.content_hash)),
        is_current: v.is_current,
        since: v.first_seen_at.to_rfc3339(),
        until: v.effective_until.map(|t| t.to_rfc3339()),
        sessions: v.sessions,
        scored_sessions: v.scored_sessions,
        scorecard: ScorecardView::with_prior(
            Scores {
                intent: v.intent,
                quality: v.quality,
                speed: v.speed,
                cost: v.cost,
            },
            previous.map_or_else(Scores::default, |p| Scores {
                intent: p.intent,
                quality: p.quality,
                speed: p.speed,
                cost: p.cost,
            }),
            None,
            format!(
                "Mean judge scores over {} judged sessions served by this version; the delta is against the version before",
                v.scored_sessions
            ),
        ),
    }
}

#[derive(Serialize)]
struct SkillRowView {
    skill: String,
    href: String,
    scored_sessions: i64,
    invocations: i64,
    scorecard: ScorecardView,
}

fn skill_row(f: &ReportSkillFigure) -> SkillRowView {
    SkillRowView {
        href: analysis_skill_url(&f.skill),
        skill: f.skill.clone(),
        scored_sessions: f.scored_sessions,
        invocations: f.invocations,
        scorecard: ScorecardView::with_prior(
            Scores {
                intent: f.intent,
                quality: f.quality,
                speed: f.speed,
                cost: f.cost,
            },
            Scores {
                quality: f.quality_prior,
                ..Scores::default()
            },
            Some(analysis_skill_url(&f.skill)),
            format!("Mean over {} judged sessions; opens the skill", f.scored_sessions),
        ),
    }
}

#[derive(Serialize)]
struct MarketplaceContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    marketplace_id: MarketplaceId,
    marketplace_name: String,
    can_trigger: bool,
    catalog_href: String,
    versions_href: String,
    evaluation_href: &'static str,
    report: Option<ReportView>,
    pending: Option<PendingView>,
    versions: Vec<VersionRowView>,
    skills: Vec<SkillRowView>,
    trend: SvgLineChartView,
    trend_days: i32,
    window_days: u32,
}

pub(crate) async fn page(
    shell: Page,
    Extension(ai): Extension<Option<Arc<AiService>>>,
    State(pool): State<Arc<PgPool>>,
    Path(marketplace_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    if marketplace_id.len() > 128 || marketplace_id.chars().any(char::is_control) {
        return Err(AdminError::BadRequest("Invalid marketplace id".to_owned()).into());
    }
    let marketplace = MarketplaceId::new(&marketplace_id);
    let versions = list_version_scores(&pool, &marketplace).await?;
    if versions.is_empty() {
        return Err(AdminError::NotFound(format!(
            "No version of marketplace '{marketplace_id}' has been recorded"
        ))
        .into());
    }
    let config = load_config()?;
    let window_days = config.as_ref().map_or(7, |c| c.report.window_days);
    let scope = ReportScope::Marketplace(marketplace_id.clone());
    let report = find_latest_report(&pool, &scope).await?;
    let pending = find_pending_report(&pool, &scope).await?;
    let end = chrono::Utc::now();
    let skills = list_report_skill_figures(
        &pool,
        &scope,
        ReportWindow {
            start: end - chrono::Duration::days(i64::from(TREND_DAYS)),
            end,
        },
        SKILL_ROWS,
    )
    .await?;
    let trend = list_score_trend(&pool, Some(&marketplace), TREND_DAYS).await?;
    let context = MarketplaceContext {
        page: "analysis-evaluation-marketplace",
        title: format!("{marketplace_id} — evaluation"),
        breadcrumbs: vec![
            BreadcrumbView::link("Evaluation", ANALYSIS_EVALUATION_URL),
            BreadcrumbView::current(marketplace_id.clone()),
        ],
        marketplace_name: marketplace_id.clone(),
        can_trigger: config.as_ref().is_some_and(|c| c.enabled) && ai.is_some(),
        catalog_href: format!("/admin/marketplaces/{}", urlencoding::encode(&marketplace_id)),
        versions_href: analysis_version_url(&marketplace, None),
        evaluation_href: ANALYSIS_EVALUATION_URL,
        report: report.as_ref().map(ReportView::of),
        pending: pending.as_ref().map(PendingView::of),
        versions: versions
            .iter()
            .enumerate()
            .map(|(i, v)| version_row(&marketplace, v, versions.get(i + 1)))
            .collect(),
        skills: skills.iter().map(skill_row).collect(),
        trend: trend_chart(
            &trend,
            TREND_DAYS,
            "judged sessions that invoked one of this marketplace's skills".to_owned(),
        ),
        trend_days: TREND_DAYS,
        window_days,
        marketplace_id: marketplace,
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-evaluation-marketplace",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
