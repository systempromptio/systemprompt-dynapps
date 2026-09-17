//! `/admin/analysis/skills/{plugin:skill}` — one skill's performance: the
//! judge's scores inherited from every session that invoked it, with the
//! window before for direction, the day-by-day quality line, and the
//! conversations behind the figures with each session's own scorecard.
//!
//! Conversations come from retained reporting data (`repositories::
//! analysis::retained`); scores come from the attribution views (schema 43).
//! `?resource=` narrows the conversations to the revision-verified evidence
//! of one managed resource, the stricter cohort the Versions page counts on.

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt::identifiers::PluginId;

use super::query::{AnalysisQuery, parse_skill_key};
use super::views::{RetainedConversationView, dollars};
use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::analysis_urls::{ANALYSIS_SKILLS_URL, analysis_skill_url};
use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::scorecard::{ScorecardView, Scores, score_trend_chart};
use crate::handlers::ssr::types::{BreadcrumbView, SvgLineChartView};
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::retained::{
    self, PAGE_SIZE, RetainedConversationRow, RetainedResourceRef, RetainedSkillRef, RetainedWindow,
};
use crate::repositories::evaluation::skill_scores::{
    list_session_scorecards, list_skill_score_days, list_skill_score_figures,
};
use crate::routes::managed_state::ManagedState;

#[derive(Serialize)]
struct SkillPage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    skill_key: String,
    skill_name: String,
    plugin_id: PluginId,
    plugin_name: String,
    marketplace_count: usize,
    marketplaces_display: String,
    catalog_href: String,
    skills_href: String,
    filter: AnalysisQuery,
    window_days: i64,
    sessions: i64,
    scored_sessions: i64,
    coverage: String,
    scorecard: ScorecardView,
    trend: SvgLineChartView,
    selected: Option<RetainedResourceRef>,
    all_href: String,
    conversations: Vec<SkillConversationView>,
    conversation_count: i64,
    cost: String,
    pagination: Pagination,
}

#[derive(Serialize)]
struct SkillConversationView {
    #[serde(flatten)]
    conversation: RetainedConversationView,
    scorecard: ScorecardView,
}

fn paginate(query: &AnalysisQuery, key: &str, total: i64, shown: usize) -> Pagination {
    let page = i64::from(query.page());
    let size = i64::from(PAGE_SIZE);
    let total_pages = ((total + size - 1) / size).max(1);
    let offset = (page - 1) * size;
    let shown = i64::try_from(shown).unwrap_or(i64::MAX);
    let link = |page: i64| query.url(u32::try_from(page).unwrap_or(u32::MAX), key);
    Pagination {
        current_page: page.min(total_pages),
        total_pages,
        first_row: if shown == 0 { 0 } else { offset + 1 },
        last_row: offset + shown,
        total_rows: total,
        noun: "conversations",
        has_prev: page > 1,
        has_next: page < total_pages,
        prev_url: (page > 1).then(|| link(page - 1)),
        next_url: (page < total_pages).then(|| link(page + 1)),
    }
}

// Why: the fact carries the hook's ids; the inventory index resolves the
// plugin's display name and the marketplaces that include it.
fn resolve_names(
    index: &InventoryIndex,
    skill: &RetainedSkillRef,
) -> (String, String, Vec<String>) {
    let skill_name = skill
        .skill
        .split_once(':')
        .map_or_else(|| skill.skill.clone(), |(_, s)| s.to_owned());
    let plugin_name = index
        .plugins_by_skill
        .get(&skill_name.replace('-', "_"))
        .and_then(|list| list.iter().find(|p| p.0 == skill.plugin_id.as_str()))
        .map_or_else(|| skill.plugin_id.as_str().to_owned(), |p| p.1.clone());
    let marketplaces = index
        .marketplaces_by_plugin
        .get(skill.plugin_id.as_str())
        .map(|list| list.iter().map(|m| m.name.clone()).collect())
        .unwrap_or_default();
    (skill_name, plugin_name, marketplaces)
}

struct JudgedConversations {
    rows: Vec<SkillConversationView>,
    cost: String,
}

async fn judged_conversations(
    pool: &PgPool,
    rows: Vec<RetainedConversationRow>,
) -> AdminResult<JudgedConversations> {
    let session_ids: Vec<String> = rows
        .iter()
        .map(|r| r.session_id.as_str().to_owned())
        .collect();
    let cards: HashMap<String, _> = list_session_scorecards(pool, &session_ids)
        .await?
        .into_iter()
        .map(|c| (c.session_id.as_str().to_owned(), c))
        .collect();
    let cost = dollars(rows.iter().map(|r| r.cost).sum());
    let rows = rows
        .into_iter()
        .map(|row| {
            let card = cards.get(row.session_id.as_str());
            let scores = card.map_or_else(Scores::default, |c| {
                Scores::from_points(c.intent, c.quality, c.speed, c.cost)
            });
            let conversation = RetainedConversationView::new(row);
            let href = format!("{}#evaluation", conversation.href);
            SkillConversationView {
                scorecard: ScorecardView::new(
                    scores,
                    Some(href),
                    if scores.any() {
                        "Judge scores for this session — opens the rationale"
                    } else {
                        "Not judged yet"
                    },
                ),
                conversation,
            }
        })
        .collect();
    Ok(JudgedConversations { rows, cost })
}

struct SkillJudgement {
    sessions: i64,
    scored_sessions: i64,
    scorecard: ScorecardView,
    trend: SvgLineChartView,
}

impl SkillJudgement {
    fn coverage(&self) -> String {
        if self.sessions == 0 {
            "—".to_owned()
        } else {
            format!("{}%", self.scored_sessions * 100 / self.sessions)
        }
    }
}

async fn skill_judgement(
    pool: &PgPool,
    skill: &RetainedSkillRef,
    window: RetainedWindow,
) -> AdminResult<SkillJudgement> {
    let figures = list_skill_score_figures(pool, window)
        .await?
        .into_iter()
        .find(|f| f.plugin_id == skill.plugin_id && f.skill == skill.skill);
    let (sessions, scored_sessions) = figures
        .as_ref()
        .map_or((0, 0), |f| (f.sessions, f.scored_sessions));
    let scores = figures.as_ref().map_or_else(Scores::default, |f| Scores {
        intent: f.intent,
        quality: f.quality,
        speed: f.speed,
        cost: f.cost,
    });
    let prior = Scores {
        quality: figures.as_ref().and_then(|f| f.quality_prior),
        ..Scores::default()
    };
    let window_days = (window.end - window.start).num_days().max(1);
    let days = list_skill_score_days(pool, &skill.plugin_id, &skill.skill, window).await?;
    let trend = score_trend_chart(
        "Quality over the window",
        format!("{scored_sessions} judged sessions; days without one carry the previous mean"),
        window,
        &[
            ("Quality", days.iter().map(|d| (d.day, d.quality)).collect()),
            ("Intent", days.iter().map(|d| (d.day, d.intent)).collect()),
        ],
    );
    Ok(SkillJudgement {
        sessions,
        scored_sessions,
        scorecard: ScorecardView::with_prior(
            scores,
            prior,
            None,
            format!(
                "Mean judge scores over the {scored_sessions} judged sessions that invoked this skill; the delta is against the {window_days} days before"
            ),
        ),
        trend,
    })
}

async fn load(
    managed: &ManagedState,
    pool: &PgPool,
    key: &str,
    mut query: AnalysisQuery,
) -> AdminResult<SkillPage> {
    let skill = parse_skill_key(key)?;
    let window = query.window(Utc::now())?;
    query.start = window.start.into();
    query.end = window.end.into();
    let resource = query.resource()?;
    query.resource = resource.as_ref().map(|r| r.as_str().to_owned());
    let page = query.page();
    let owner = &managed.owner;
    let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
    let (skill_name, plugin_name, marketplaces) = resolve_names(&index, &skill);

    let mut selected = None;
    let (rows, total) = if let Some(resource) = resource.as_ref() {
        selected = retained::find_retained_resource(pool, owner, resource).await?;
        if selected.is_none() {
            return Err(AdminError::NotFound("Unknown managed resource".into()));
        }
        retained::list_retained_conversations(pool, owner, resource, window, page).await?
    } else {
        retained::list_retained_skill_conversations(pool, owner, &skill, window, page).await?
    };
    let conversations = judged_conversations(pool, rows).await?;
    let judged = skill_judgement(pool, &skill, window).await?;
    Ok(SkillPage {
        page: "analysis-skill",
        title: skill_name.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Skills", ANALYSIS_SKILLS_URL),
            BreadcrumbView::current(skill_name.clone()),
        ],
        skill_key: skill.skill.clone(),
        catalog_href: format!(
            "/admin/skills/{}",
            urlencoding::encode(&skill_name.replace('-', "_"))
        ),
        skills_href: ANALYSIS_SKILLS_URL.to_owned(),
        skill_name,
        plugin_id: skill.plugin_id.clone(),
        plugin_name,
        marketplace_count: marketplaces.len(),
        marketplaces_display: marketplaces.join(", "),
        window_days: (window.end - window.start).num_days().max(1),
        sessions: judged.sessions,
        scored_sessions: judged.scored_sessions,
        coverage: judged.coverage(),
        scorecard: judged.scorecard,
        trend: judged.trend,
        selected,
        all_href: analysis_skill_url(&skill.skill),
        conversation_count: total,
        cost: conversations.cost,
        pagination: paginate(&query, &skill.skill, total, conversations.rows.len()),
        conversations: conversations.rows,
        filter: query,
    })
}

pub(crate) async fn skill_page(
    shell: Page,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Path(key): Path<String>,
    Query(query): Query<AnalysisQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let context = load(&managed, &pool, &key, query).await?;
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-skill",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
