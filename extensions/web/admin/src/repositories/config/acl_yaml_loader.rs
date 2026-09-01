//! Bootstrap loader: `services/access-control/*.yaml` → DB.
//!
//! YAML is the declarative source of truth committed to the source repo. On
//! every server startup the governance bootstrap calls [`load_from_yaml`],
//! which upserts entity and rule rows into the runtime database. There is no
//! write-back: dashboard edits live only in the DB of the instance that
//! received them, so deployments sharing the same YAML baseline can drift
//! independently without trampling each other.
//!
//! Three files are read (all optional; missing-file = no-op), plus one
//! projection out of the services config:
//!
//! - `services/access-control/roles.yaml` — role-scoped allow/deny rules.
//!   Parsed into core's [`AccessControlConfig`] and projected by core
//!   ingestion, which owns `entity_match` glob expansion, `default_included`,
//!   and `entity_id` self-materialisation for the kinds the caller does not
//!   enforce through [`RegisteredEntities`] — a `gateway_route` id outside the
//!   profile's routes is rejected, never materialised.
//! - `services/access-control/adfs.yaml` — entities confined to members of
//!   named AD groups at `rule_type = 'adfs_group'`. Owned by
//!   [`super::adfs_yaml_loader`].
//! - `services/slack/*.yaml` — each app's `authz.allowed_roles`, projected onto
//!   its `slack_workspace` entity. Core writes the rows; nothing called it, so
//!   an app's `allowed_roles` documented an intention it never enforced.

use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::loader::ConfigLoader;
use systemprompt_security::authz::{
    AccessControlConfig, AccessControlIngestionService, IngestOptions, RegisteredEntities,
};
use systemprompt_web_shared::error::MarketplaceError;

use super::acl_yaml_types::LoadReport;

const ROLES_FILE: &str = "access-control/roles.yaml";

// Why: `registered` is the caller's because only the caller knows where its
// truth lives — for `gateway_route` that is the live profile, reconciled by
// the governance bootstrap immediately before this runs. An empty
// `RegisteredEntities` enforces nothing.
pub async fn load_from_yaml(
    pool: &PgPool,
    services_path: &Path,
    registered: &RegisteredEntities,
) -> Result<LoadReport, MarketplaceError> {
    let mut report = LoadReport::default();

    load_roles_file(pool, services_path, registered, &mut report).await?;

    // Why: after the roles pass — the gate forces default_included=false on
    // its entities and must own the final word over any role-file default.
    let adfs_groups =
        super::adfs_yaml_loader::load_adfs_groups_from_yaml(pool, services_path).await?;

    // Why: same reasoning as the AD-group gate above — these force
    // default_included=false on their entities and must run after roles.
    let link_gates =
        super::linked_yaml_loader::load_link_gates_from_yaml(pool, services_path).await?;

    let slack_workspaces = load_slack_apps(pool).await?;

    tracing::info!(
        rules = report.rules_upserted,
        adfs_group_grants = adfs_groups.grants_projected,
        link_gate_grants = link_gates.grants_projected,
        slack_workspaces,
        "bootstrap_yaml_loaded"
    );
    Ok(report)
}

async fn read_yaml<T: for<'de> Deserialize<'de> + Default>(
    services_path: &Path,
    rel: &str,
) -> Result<Option<T>, MarketplaceError> {
    let path = services_path.join(rel);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(T::default())),
        Ok(s) => serde_yaml::from_str::<T>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(rel, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn load_roles_file(
    pool: &PgPool,
    services_path: &Path,
    registered: &RegisteredEntities,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let Some(cfg) = read_yaml::<AccessControlConfig>(services_path, ROLES_FILE).await? else {
        return Ok(());
    };

    let svc = AccessControlIngestionService::from_pool(Arc::new(pool.clone()));
    let ingested = svc
        .ingest_config(
            &cfg,
            IngestOptions {
                override_existing: true,
                delete_orphans: false,
            },
            registered,
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    report.rules_upserted = ingested.inserted + ingested.updated;
    Ok(())
}

// Why: an unreadable services config is not this loader's failure to report —
// the server does not start without one, and treating it as fatal here would
// turn every unrelated config error into "access control failed to load".
async fn load_slack_apps(pool: &PgPool) -> Result<usize, MarketplaceError> {
    let Ok(services) = ConfigLoader::load() else {
        return Ok(0);
    };
    if services.slack_apps.is_empty() {
        return Ok(0);
    }

    let svc = AccessControlIngestionService::from_pool(Arc::new(pool.clone()));
    let ingested = svc
        .ingest_slack_apps(
            &services.slack_apps,
            IngestOptions {
                override_existing: true,
                delete_orphans: false,
            },
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    Ok(ingested.inserted + ingested.updated)
}
