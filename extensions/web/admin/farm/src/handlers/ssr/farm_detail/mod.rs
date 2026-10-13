//! `/admin/farm/{farm_id}` — one farm: its sync card, then a tab strip over
//! its agents, recent runs, eval suites and eval runs, and the full sync
//! record. Each tab is a `?tab=` link, so the server reads only what the
//! active tab shows beyond the counts on the strip.

mod views;

use std::sync::Arc;
use systemprompt::identifiers::MarketplaceId;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

use super::farm_overview::cards::sync_facts;
use super::shared::{RunRowView, run_row_views};
use super::sync_card::{SyncCardView, bundle_for, load_bundles, sync_card_view};
use crate::repositories::agents::list_farm_agents;
use crate::repositories::evals::{list_eval_suites, list_recent_eval_runs};
use crate::repositories::farms::find_farm;
use crate::repositories::runs::list_recent_runs;
use crate::types::FarmId;
use views::{AgentView, EvalRunView, FarmSuiteTabView, agent_view, eval_run_view, suite_view};

const RECENT_RUNS: i64 = 50;
const RECENT_EVAL_RUNS: i64 = 50;
const TABS: [(&str, &str); 4] = [
    ("agents", "Agents"),
    ("runs", "Runs"),
    ("evals", "Evals"),
    ("sync", "Sync"),
];

#[derive(Debug, Deserialize)]
pub(crate) struct FarmDetailQuery {
    pub(crate) tab: Option<String>,
}

#[derive(Debug, Serialize)]
struct FarmDetailContext {
    page: &'static str,
    title: String,
    subtitle: String,
    breadcrumbs: Vec<BreadcrumbView>,
    farm_id: String,
    marketplace_id: MarketplaceId,
    marketplace_url: String,
    owner: String,
    owner_url: String,
    sync: SyncCardView,
    tabs: Vec<TabLinkView>,
    tab: &'static str,
    agents: Vec<AgentView>,
    runs: Vec<RunRowView>,
    suites: Vec<FarmSuiteTabView>,
    suite_count: usize,
    eval_runs: Vec<EvalRunView>,
    eval_run_count: usize,
}

fn active_tab(query: &FarmDetailQuery) -> &'static str {
    TABS.iter()
        .map(|(slug, _)| *slug)
        .find(|slug| query.tab.as_deref() == Some(*slug))
        .unwrap_or("agents")
}

fn tab_links(farm_id: &str, active: &str, agents: usize, suites: usize) -> Vec<TabLinkView> {
    TABS.iter()
        .map(|&(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("/admin/farm/{farm_id}?tab={slug}"),
            is_active: slug == active,
            count: match slug {
                "agents" => i64::try_from(agents).ok(),
                "evals" => i64::try_from(suites).ok(),
                _ => None,
            },
        })
        .collect()
}

async fn tab_rows(
    pool: &PgPool,
    farm_id: &FarmId,
    tab: &str,
    faults: &mut Faults,
) -> (Vec<RunRowView>, Vec<EvalRunView>) {
    match tab {
        "runs" => {
            let rows = list_recent_runs(pool, Some(farm_id), None, RECENT_RUNS).await;
            (run_row_views(&faults.take("recent runs", rows)), Vec::new())
        },
        "evals" => {
            let rows = faults.take(
                "eval runs",
                list_recent_eval_runs(pool, Some(farm_id), None, RECENT_EVAL_RUNS).await,
            );
            (Vec::new(), rows.iter().map(eval_run_view).collect())
        },
        _ => (Vec::new(), Vec::new()),
    }
}

pub(crate) async fn farm_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(farm_id): Path<String>,
    Query(query): Query<FarmDetailQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let farm_id = FarmId::new(farm_id);
    let farm = find_farm(&pool, &farm_id)
        .await
        .map_err(AdminError::from)?
        .ok_or_else(|| AdminError::NotFound(format!("No farm {}", farm_id.as_str())))?;
    let tab = active_tab(&query);
    let mut faults = Faults::new();

    let (agents, suites) = tokio::join!(
        list_farm_agents(&pool, &farm_id),
        list_eval_suites(&pool, &farm_id)
    );
    let agents = faults.take("farm agents", agents);
    let suites = faults.take("eval suites", suites);
    let (runs, eval_runs) = tab_rows(&pool, &farm_id, tab, &mut faults).await;
    let bundles = faults.take("pinned bundles", load_bundles());
    let sync = sync_card_view(
        &sync_facts(&farm),
        bundle_for(&bundles, &farm.marketplace_id),
    );

    let id = farm.id.as_str().to_owned();
    let owner = farm.owner_user_id.clone().unwrap_or_default();
    let ctx = FarmDetailContext {
        page: "farm-detail",
        title: farm.name.clone(),
        subtitle: farm.description.clone().unwrap_or_else(|| {
            "One farm: its source commit, the agents it ships, and their recent runs and evals."
                .to_owned()
        }),
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::current(farm.name.clone()),
        ],
        marketplace_url: format!("/admin/marketplaces/{}", farm.marketplace_id.as_str()),
        marketplace_id: farm.marketplace_id.clone(),
        owner_url: if owner.is_empty() {
            String::new()
        } else {
            format!("/admin/users/{owner}")
        },
        owner,
        tabs: tab_links(&id, tab, agents.len(), suites.len()),
        farm_id: id,
        sync,
        tab,
        agents: agents
            .iter()
            .map(|a| agent_view(a, farm.synced_commit_sha.as_deref()))
            .collect(),
        runs,
        suites: suites.iter().map(suite_view).collect(),
        suite_count: suites.len(),
        eval_run_count: eval_runs.len(),
        eval_runs,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-detail",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
