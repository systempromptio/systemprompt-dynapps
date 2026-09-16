//! Skill drill-down over retained reporting data: organisation totals, one row
//! per managed resource, and the conversations behind a selected resource.
//! Nothing here is computed live; every number is read from the snapshot
//! pipeline's retained tables through `repositories::analysis::retained`.
use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::list_view::Pagination;
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::retained::{
    self, PAGE_SIZE, RetainedResourceRef, RetainedSkillRef, RetainedTotals,
};
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
pub(crate) mod campaign_completion;
pub(crate) mod campaigns;
pub(crate) mod candidates;
pub(crate) mod experiments;
pub(crate) mod experiments_view;
pub(crate) mod impact;
pub(crate) mod impact_cohorts;
pub(crate) mod lifecycle;
pub(crate) mod portfolio;
mod query;
pub(crate) mod time;
pub(crate) mod versions;
pub(crate) mod views;
pub use query::{AnalysisQuery, Drilldown, drilldown_url};
pub(crate) use views::dollars;
pub use views::{RetainedConversationView, RetainedSkillView};

fn paginate(
    query: &AnalysisQuery,
    target: Option<&Drilldown>,
    total: i64,
    shown: usize,
) -> Pagination {
    let page = i64::from(query.page());
    let size = i64::from(PAGE_SIZE);
    let total_pages = ((total + size - 1) / size).max(1);
    let offset = (page - 1) * size;
    let shown = i64::try_from(shown).unwrap_or(i64::MAX);
    let link = |page: i64| query.url(u32::try_from(page).unwrap_or(u32::MAX), target);
    Pagination {
        current_page: page.min(total_pages),
        total_pages,
        first_row: if shown == 0 { 0 } else { offset + 1 },
        last_row: offset + shown,
        total_rows: total,
        noun: if target.is_some() {
            "conversations"
        } else {
            "skills"
        },
        has_prev: page > 1,
        has_next: page < total_pages,
        prev_url: (page > 1).then(|| link(page - 1)),
        next_url: (page < total_pages).then(|| link(page + 1)),
    }
}

#[derive(Serialize)]
struct AnalysisPage {
    page: &'static str,
    title: &'static str,
    totals: RetainedTotals,
    cost: String,
    filter: AnalysisQuery,
    all_href: String,
    skills: Vec<RetainedSkillView>,
    selected: Option<RetainedResourceRef>,
    selected_skill: Option<RetainedSkillRef>,
    conversations: Vec<RetainedConversationView>,
    detail: bool,
    pagination: Pagination,
}

async fn load(
    managed: &ManagedState,
    pool: &PgPool,
    mut query: AnalysisQuery,
) -> AdminResult<AnalysisPage> {
    let window = query.window(Utc::now())?;
    query.start = Some(window.start);
    query.end = Some(window.end);
    let resource = query.resource()?;
    query.resource = resource.as_ref().map(|r| r.as_str().to_owned());
    let skill = query.skill()?;
    let page = query.page();
    let owner = &managed.owner;
    let totals = retained::get_retained_totals(pool, owner, window).await?;
    let mut skills = Vec::new();
    let mut conversations = Vec::new();
    let mut selected = None;
    let target;
    let total;
    if let Some(resource) = resource.as_ref() {
        selected = retained::find_retained_resource(pool, owner, resource).await?;
        if selected.is_none() {
            return Err(AdminError::NotFound("Unknown managed resource".into()));
        }
        let (rows, count) =
            retained::list_retained_conversations(pool, owner, resource, window, page).await?;
        total = count;
        conversations.extend(rows.into_iter().map(RetainedConversationView::new));
        target = Some(Drilldown::Resource(resource.as_str().to_owned()));
    } else if let Some(skill) = skill.as_ref() {
        let (rows, count) =
            retained::list_retained_skill_conversations(pool, owner, skill, window, page).await?;
        total = count;
        conversations.extend(rows.into_iter().map(RetainedConversationView::new));
        target = Some(Drilldown::Skill(skill.clone()));
    } else {
        let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
        let (rows, count) = retained::list_retained_skills(pool, owner, window, page).await?;
        total = count;
        for row in rows {
            let href = query.url(
                1,
                Some(&Drilldown::Skill(RetainedSkillRef {
                    plugin_id: row.plugin_id.clone(),
                    skill: row.skill.clone(),
                })),
            );
            skills.push(RetainedSkillView::new(row, href, &index));
        }
        target = None;
    }
    let shown = skills.len().max(conversations.len());
    let pagination = paginate(&query, target.as_ref(), total, shown);
    Ok(AnalysisPage {
        page: "analysis-skills",
        title: "Conversations",
        cost: dollars(totals.cost),
        totals,
        all_href: query.url(1, None),
        filter: query,
        skills,
        selected,
        selected_skill: skill,
        conversations,
        detail: target.is_some(),
        pagination,
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn skills_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalysisQuery>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let context = load(&managed, &pool, query).await?;
    Ok(super::render_typed_page(
        &engine,
        "analysis-skills",
        &context,
        &user,
        &marketplace,
    ))
}
