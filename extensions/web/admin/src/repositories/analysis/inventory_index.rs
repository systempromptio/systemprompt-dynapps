//! Marketplace → plugin → skill reachability and the people each marketplace
//! reaches, assembled from the services tree and the access-control resolver.
//!
//! Membership is authored in YAML (`services/marketplaces`,
//! `services/plugins`), so the index is rebuilt per request rather than
//! persisted. It describes the catalog as placed today; historical placement is
//! not a retained dimension.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::groups::crud::list_group_summaries;
use crate::repositories::marketplace::manifests::{
    MarketplaceConfigSummary, list_marketplace_configs,
};
use crate::repositories::marketplace::plugin_maps::{EntityPluginMap, build_entity_plugin_maps};
use crate::repositories::users::access_control::{
    MatrixSubject, SectionInput, group_subject, resolve_subject_matrices, role_subject,
};

const MARKETPLACE_ENTITY: &str = "marketplace";
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct InventoryIndex {
    pub marketplaces: Vec<MarketplaceConfigSummary>,
    pub plugins_by_skill: EntityPluginMap,
    pub marketplaces_by_plugin: HashMap<String, Vec<MarketplaceRef>>,
}

impl InventoryIndex {
    #[must_use]
    pub fn build(services_path: &Path) -> Self {
        let marketplaces = list_marketplace_configs(services_path).unwrap_or_else(|error| {
            tracing::warn!(error = %error, "marketplace manifests unavailable for analysis");
            Vec::new()
        });
        let (plugins_by_skill, _agents, _mcp) = build_entity_plugin_maps(services_path);
        let mut marketplaces_by_plugin: HashMap<String, Vec<MarketplaceRef>> = HashMap::new();
        for marketplace in &marketplaces {
            for plugin in &marketplace.plugins {
                marketplaces_by_plugin
                    .entry(plugin.clone())
                    .or_default()
                    .push(MarketplaceRef {
                        id: marketplace.id.as_str().to_owned(),
                        name: marketplace.name.clone(),
                    });
            }
        }
        Self {
            marketplaces,
            plugins_by_skill,
            marketplaces_by_plugin,
        }
    }
}

/// The distinct users each marketplace reaches, resolved per group and per
/// role and then materialised through memberships.
#[derive(Debug, Default)]
pub struct MarketplaceAudience {
    pub users_by_marketplace: HashMap<String, HashSet<UserId>>,
}

impl MarketplaceAudience {
    #[must_use]
    pub fn users_reaching_skill(&self, index: &InventoryIndex, skill: &str) -> usize {
        let mut users: HashSet<&UserId> = HashSet::new();
        for plugin in index.plugins_by_skill.get(skill).into_iter().flatten() {
            for marketplace in index
                .marketplaces_by_plugin
                .get(&plugin.0)
                .into_iter()
                .flatten()
            {
                if let Some(set) = self.users_by_marketplace.get(&marketplace.id) {
                    users.extend(set.iter());
                }
            }
        }
        users.len()
    }

    #[must_use]
    pub fn users_reaching_marketplace(&self, marketplace: &str) -> usize {
        self.users_by_marketplace
            .get(marketplace)
            .map_or(0, HashSet::len)
    }

    #[must_use]
    pub fn users_reaching_any_marketplace(&self) -> usize {
        self.users_by_marketplace
            .values()
            .flatten()
            .collect::<HashSet<_>>()
            .len()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MembershipRow {
    user_id: UserId,
    group_ids: Vec<String>,
    roles: Vec<String>,
}

// Why: audience goes through the same subject resolver the enforcement point
// uses, so a count here and a decision at the gateway cannot disagree.
pub async fn get_marketplace_audience(
    pool: &PgPool,
    manifests: &[MarketplaceConfigSummary],
    roles: &[String],
) -> Result<MarketplaceAudience, sqlx::Error> {
    let groups = list_group_summaries(pool).await?;
    let mut subjects: Vec<MatrixSubject> = Vec::with_capacity(groups.len() + roles.len());
    let mut subject_keys: Vec<(bool, String)> = Vec::with_capacity(subjects.capacity());
    for group in &groups {
        subjects.push(group_subject(group.id.as_str()));
        subject_keys.push((true, group.id.as_str().to_owned()));
    }
    for role in roles {
        subjects.push(role_subject(role));
        subject_keys.push((false, role.clone()));
    }
    let sections: Vec<SectionInput> = vec![(
        MARKETPLACE_ENTITY.to_owned(),
        "Marketplaces".to_owned(),
        manifests
            .iter()
            .map(|m| (m.id.as_str().to_owned(), m.name.clone(), None))
            .collect(),
    )];
    let resolved = resolve_subject_matrices(pool, &subjects, &sections).await?;
    let mut allowed_groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut allowed_roles: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for ((is_group, key), sections) in subject_keys.into_iter().zip(resolved) {
        for row in sections.into_iter().flat_map(|s| s.rows) {
            if row.effective != "allow" {
                continue;
            }
            let target = if is_group {
                &mut allowed_groups
            } else {
                &mut allowed_roles
            };
            target.entry(row.entity_id).or_default().insert(key.clone());
        }
    }
    let members = sqlx::query_as::<_, MembershipRow>(
        "SELECT u.id AS user_id, \
         coalesce(array_agg(g.group_id) FILTER (WHERE g.group_id IS NOT NULL), '{}') AS group_ids, \
         u.roles FROM users u LEFT JOIN user_groups g ON g.user_id = u.id \
         WHERE NOT ('anonymous' = ANY(u.roles)) GROUP BY u.id, u.roles",
    )
    .fetch_all(pool)
    .await?;
    let mut users_by_marketplace: HashMap<String, HashSet<UserId>> = HashMap::new();
    for manifest in manifests {
        let id = manifest.id.as_str();
        let groups = allowed_groups.get(id);
        let roles = allowed_roles.get(id);
        let reached = members
            .iter()
            .filter(|m| {
                groups.is_some_and(|g| m.group_ids.iter().any(|x| g.contains(x)))
                    || roles.is_some_and(|r| m.roles.iter().any(|x| r.contains(x)))
            })
            .map(|m| m.user_id.clone())
            .collect();
        users_by_marketplace.insert(id.to_owned(), reached);
    }
    Ok(MarketplaceAudience {
        users_by_marketplace,
    })
}
