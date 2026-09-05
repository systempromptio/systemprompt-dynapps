//! Bootstrap loader: `services/access-control/adfs.yaml` → DB.
//!
//! Projects the AD-group gate into `access_control_rules`: one
//! `rule_type = 'adfs_group'` allow-row per (entity, group) listed, with the
//! catalog row forced to `default_included = false`. Runs after the roles
//! pass so it owns the final word on those entities' defaults: an entity in
//! this file is reachable through a listed group or not at all, whatever a
//! role grant used to say.
//!
//! Reconciliation is scoped to this loader's own rows (`rule_type =
//! 'adfs_group'`), so dashboard edits at other rule types survive.

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, UpsertRuleParams};
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::adfs_group::adfs_group_rule_type;

use super::adfs_yaml_types::{AdfsGroupGrant, AdfsGroupsDoc, AdfsGroupsLoadReport};

const ADFS_FILE: &str = "access-control/adfs.yaml";

const SOURCE_LABEL: &str = "adfs.yaml";

pub async fn load_adfs_groups_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<AdfsGroupsLoadReport, MarketplaceError> {
    let mut report = AdfsGroupsLoadReport::default();
    let Some(doc) = read_doc(services_path).await? else {
        return Ok(report);
    };

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    report.grants_projected = project_grants(pool, &repo, &doc.grants).await?;

    tracing::info!(
        grants = report.grants_projected,
        "bootstrap_adfs_group_gate_loaded"
    );
    Ok(report)
}

async fn read_doc(services_path: &Path) -> Result<Option<AdfsGroupsDoc>, MarketplaceError> {
    let path = services_path.join(ADFS_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(AdfsGroupsDoc::default())),
        Ok(s) => serde_yaml::from_str::<AdfsGroupsDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(ADFS_FILE, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn project_grants(
    pool: &PgPool,
    repo: &AccessControlRepository,
    grants: &[AdfsGroupGrant],
) -> Result<usize, MarketplaceError> {
    let rule_type = adfs_group_rule_type();
    let mut kept_types: Vec<String> = Vec::new();
    let mut kept_ids: Vec<String> = Vec::new();
    let mut kept_groups: Vec<String> = Vec::new();
    let mut projected = 0;

    for grant in grants {
        let entry = format!("{ADFS_FILE}: {}", grant.entity_id);
        let kind = EntityKind::from_str(&grant.entity_type)
            .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
        if grant.groups.is_empty() {
            return Err(MarketplaceError::config_file(
                entry,
                "a grant must name at least one group",
            ));
        }

        // Why: `default_included = false` is load-bearing here — a user outside
        // every listed group matches no `adfs_group` rule and falls through to
        // the entity default, which must therefore be closed.
        repo.upsert_entity(kind, &grant.entity_id, false, SOURCE_LABEL)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        for group in &grant.groups {
            repo.upsert_rule(UpsertRuleParams {
                entity_type: kind,
                entity_id: &grant.entity_id,
                rule_type: rule_type.clone(),
                rule_value: group,
                access: Access::Allow,
                justification: Some("granted to members of an AD group"),
            })
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
            kept_types.push(grant.entity_type.clone());
            kept_ids.push(grant.entity_id.clone());
            kept_groups.push(group.clone());
            projected += 1;
        }
    }

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE rule_type = $1
           AND (entity_type, entity_id, rule_value) NOT IN (
                 SELECT t, i, g FROM UNNEST($2::TEXT[], $3::TEXT[], $4::TEXT[]) AS kept(t, i, g)
               )",
        rule_type.as_str(),
        &kept_types,
        &kept_ids,
        &kept_groups,
    )
    .execute(pool)
    .await?;

    Ok(projected)
}
