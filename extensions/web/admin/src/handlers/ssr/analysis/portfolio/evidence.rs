//! The evidence a portfolio row is assembled from: the YAML-derived
//! marketplace/plugin index, the resolved audience, the managed resources
//! that bind retained snapshots to inventory entries, the version each
//! marketplace is serving, and the window's live per-skill figures.

use super::plan::Plan;
use super::rows::{InventoryRow, InventoryRowInputs, live_for, scores_for};
use crate::error::AdminResult;
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use crate::repositories::analysis::inventory_index::{
    InventoryIndex, MarketplaceAudience, get_marketplace_audience,
};
use crate::repositories::analysis::marketplace_versions::{SkillLiveRow, list_skill_live_figures};
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use crate::repositories::analysis::retained::RetainedWindow;
use crate::repositories::evaluation::skill_scores::{
    SkillScoreFigures, WindowScoreMeans, get_window_score_means, list_skill_score_figures,
};
use crate::repositories::sync::marketplace_versions_db::{
    MarketplaceVersionRow, list_current_marketplace_versions,
};
use crate::routes::managed_state::ManagedState;
use crate::types::Role;
use sqlx::PgPool;
use systemprompt::marketplace::managed::ResourceSummary;

pub(super) struct Evidence<'a> {
    pub(super) index: &'a InventoryIndex,
    pub(super) audience: MarketplaceAudience,
    pub(super) managed: Vec<ResourceSummary>,
    pub(super) versions: Vec<MarketplaceVersionRow>,
    pub(super) live: Vec<SkillLiveRow>,
    pub(super) scores: Vec<SkillScoreFigures>,
    pub(super) score_means: WindowScoreMeans,
}

impl<'a> Evidence<'a> {
    pub(super) async fn gather(
        managed: &ManagedState,
        pool: &PgPool,
        index: &'a InventoryIndex,
        window: RetainedWindow,
    ) -> AdminResult<Self> {
        let roles: Vec<String> = Role::ALL.iter().map(|r| r.as_str().to_owned()).collect();
        Ok(Self {
            audience: get_marketplace_audience(pool, &index.marketplaces, &roles).await?,
            managed: list_all_resources(managed).await?,
            versions: list_current_marketplace_versions(pool).await?,
            live: list_skill_live_figures(pool, &managed.owner, window).await?,
            scores: list_skill_score_figures(pool, window).await?,
            score_means: get_window_score_means(pool, window).await?,
            index,
        })
    }
}

impl Evidence<'_> {
    // Why: the headline scorecard counts each judged session once, however
    // many skills it invoked, with the window before for direction.
    pub(super) fn score_kpis(&self) -> (i64, ScorecardView) {
        let m = self.score_means;
        (
            m.scored_sessions,
            ScorecardView::with_prior(
                Scores {
                    intent: m.intent,
                    quality: m.quality,
                    speed: m.speed,
                    cost: m.cost,
                },
                Scores {
                    quality: m.quality_prior,
                    ..Scores::default()
                },
                None,
                format!(
                    "Mean judge scores over the {} judged sessions that invoked a skill in this window",
                    m.scored_sessions
                ),
            ),
        )
    }

    // Why: the grouped view counts people entitled to any marketplace; a
    // narrowed view counts only the one selected.
    pub(super) fn audience_of(&self, plan: &Plan, selected: &str) -> usize {
        if plan.all_marketplaces {
            self.audience.users_reaching_any_marketplace()
        } else {
            self.audience.users_reaching_marketplace(selected)
        }
    }
}

async fn list_all_resources(managed: &ManagedState) -> AdminResult<Vec<ResourceSummary>> {
    let mut all = Vec::new();
    let mut offset = 0;
    loop {
        let page = managed
            .repository
            .list_resources(&managed.owner, offset)
            .await?;
        all.extend(page.items);
        if !page.has_more {
            return Ok(all);
        }
        offset += 50;
    }
}

// Why: a skill carried by several plugins is one inventory entry but several
// audiences; each plugin gets its own row so the marketplace grouping holds.
pub(super) fn expand_rows(
    item: PortfolioEntry,
    evidence: &Evidence<'_>,
    window: (&str, &str),
) -> AdminResult<Vec<InventoryRow>> {
    let key = item.entry.resource_key.clone();
    let plugins: Vec<(String, String)> = evidence
        .index
        .plugins_by_skill
        .get(&key)
        .map(|list| list.iter().map(|p| (p.0.clone(), p.1.clone())).collect())
        .unwrap_or_default();
    let users_with_access = evidence.audience.users_reaching_skill(evidence.index, &key);
    if plugins.is_empty() {
        return Ok(vec![InventoryRow::build(InventoryRowInputs {
            item,
            plugin: None,
            marketplaces: Vec::new(),
            users_with_access,
            managed: &evidence.managed,
            live: None,
            scores: None,
            window,
        })?]);
    }
    plugins
        .iter()
        .map(|(plugin_id, plugin_name)| {
            InventoryRow::build(InventoryRowInputs {
                item: PortfolioEntry {
                    entry: item.entry.clone(),
                    snapshot: item.snapshot.clone(),
                    installation: item.installation,
                },
                plugin: Some((plugin_id, plugin_name)),
                marketplaces: evidence
                    .index
                    .marketplaces_by_plugin
                    .get(plugin_id)
                    .cloned()
                    .unwrap_or_default(),
                users_with_access,
                managed: &evidence.managed,
                live: live_for(&evidence.live, Some(plugin_id), &key),
                scores: scores_for(&evidence.scores, Some(plugin_id), &key),
                window,
            })
        })
        .collect()
}
