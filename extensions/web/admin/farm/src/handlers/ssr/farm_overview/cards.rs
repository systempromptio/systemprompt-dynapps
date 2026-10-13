//! One card per declared farm: where its agents come from, which commit the
//! last sync recorded, whether that commit is in step with the pinned
//! bundle, and who owns it.

use serde::Serialize;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time};

use crate::handlers::ssr::shared::BadgeView;
use crate::handlers::ssr::sync_card::{
    BundleFacts, FarmSyncFacts, bundle_for, short_sha, sync_badge, sync_state,
};
use crate::repositories::rows::{AgentRow, FarmRow};

#[derive(Debug, Serialize)]
pub(crate) struct FarmCardView {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) url: String,
    pub(crate) marketplace_id: MarketplaceId,
    pub(crate) marketplace_url: String,
    pub(crate) source_repo: String,
    pub(crate) source_ref: String,
    pub(crate) sha: String,
    pub(crate) sha_short: String,
    pub(crate) synced_local: String,
    pub(crate) synced_relative: String,
    pub(crate) sync_badge: BadgeView,
    pub(crate) sync_error: String,
    pub(crate) owner: String,
    pub(crate) owner_url: String,
    pub(crate) agent_count: usize,
}

pub(crate) fn sync_facts(farm: &FarmRow) -> FarmSyncFacts<'_> {
    FarmSyncFacts {
        synced_commit_sha: farm.synced_commit_sha.as_deref(),
        synced_at: farm.synced_at,
        synced_by: farm.synced_by.as_deref(),
        sync_error: farm.sync_error.as_deref(),
        source_repo: &farm.source_repo,
        source_ref: &farm.source_ref,
    }
}

pub(crate) fn farm_card(
    farm: &FarmRow,
    agents: &[AgentRow],
    bundles: &[BundleFacts],
) -> FarmCardView {
    let id = farm.id.as_str();
    let marketplace_id = &farm.marketplace_id;
    let state = sync_state(&sync_facts(farm), bundle_for(bundles, marketplace_id));
    let sha = farm.synced_commit_sha.clone().unwrap_or_default();
    let owner = farm.owner_user_id.clone().unwrap_or_default();
    FarmCardView {
        id: id.to_owned(),
        name: farm.name.clone(),
        description: farm.description.clone().unwrap_or_default(),
        url: format!("/admin/farm/{id}"),
        marketplace_id: marketplace_id.clone(),
        marketplace_url: format!("/admin/marketplaces/{marketplace_id}"),
        source_repo: farm.source_repo.clone(),
        source_ref: farm.source_ref.clone(),
        sha_short: short_sha(&sha),
        sha,
        synced_local: farm.synced_at.map(local_time).unwrap_or_default(),
        synced_relative: farm.synced_at.map(relative_time).unwrap_or_default(),
        sync_badge: sync_badge(state),
        sync_error: farm.sync_error.clone().unwrap_or_default(),
        owner_url: if owner.is_empty() {
            String::new()
        } else {
            format!("/admin/users/{owner}")
        },
        owner,
        agent_count: agents.iter().filter(|a| a.farm_id == farm.id).count(),
    }
}
