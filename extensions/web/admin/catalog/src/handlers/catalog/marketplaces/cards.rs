//! One card per marketplace: the manifest, who reaches it, and where it came
//! from.

use std::collections::HashMap;

use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_shared::GroupId;

use super::skills_of;
use super::view::{AudienceMatrixView, MarketplaceCardView, marketplace_url};
use systemprompt_web_admin_core::catalog_index::manifests::MarketplaceConfigSummary;
use systemprompt_web_admin_core::repositories::sync::marketplace_versions_db::list_current_marketplace_versions;
use systemprompt_web_admin_core::repositories::sync::provenance::{OwnedKind, owned_by};
use systemprompt_web_admin_core::ssr::sync_plane::HashView;

pub(super) async fn current_hashes(pool: &PgPool) -> HashMap<String, String> {
    // Why: discard-ok: a missing versions table leaves every card with no hash
    list_current_marketplace_versions(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: versions unreadable"))
        .unwrap_or_default()
        .into_iter()
        .map(|v| (v.marketplace_id.as_str().to_owned(), v.content_hash))
        .collect()
}

pub(super) struct CardInputs<'a> {
    pub manifests: &'a [MarketplaceConfigSummary],
    pub audience: &'a AudienceMatrixView,
    pub grants: &'a HashMap<MarketplaceId, Vec<GroupId>>,
    pub plugin_catalog: &'a [systemprompt_web_admin_core::types::PluginDetail],
    pub versions: &'a HashMap<String, String>,
    pub sources: &'a systemprompt_web_admin_core::repositories::sync::sources::SourcesView,
}

pub(super) fn card_views(inputs: &CardInputs<'_>) -> Vec<MarketplaceCardView> {
    inputs
        .manifests
        .iter()
        .map(|m| {
            let provenance = owned_by(inputs.sources, OwnedKind::Marketplace, m.id.as_str());
            let assigned_groups = inputs.grants.get(&m.id).cloned().unwrap_or_default();
            let allowed_subjects = inputs
                .audience
                .rows
                .iter()
                .filter(|row| {
                    row.cells
                        .iter()
                        .any(|c| c.marketplace_id == m.id && c.is_allow)
                })
                .count();
            MarketplaceCardView {
                id: m.id.clone(),
                name: m.name.clone(),
                description: m.description.clone(),
                version: m.version.clone(),
                enabled: m.enabled,
                visibility: m.visibility.clone(),
                detail_url: marketplace_url(m.id.as_str()),
                roles: m.access.roles.clone(),
                groups: m.access.groups.clone(),
                projects: m.access.projects.clone(),
                plugin_count: m.plugins.len(),
                skill_count: skills_of(inputs.plugin_catalog, &m.plugins).len(),
                mcp_count: m.mcp_servers.len(),
                default_included: m.access.default_included,
                assigned_group_count: assigned_groups.len(),
                assigned_groups,
                allowed_subjects,
                source: provenance.source,
                source_tone: provenance.tone,
                content_hash: inputs
                    .versions
                    .get(m.id.as_str())
                    .and_then(|h| HashView::of(h)),
            }
        })
        .collect()
}
