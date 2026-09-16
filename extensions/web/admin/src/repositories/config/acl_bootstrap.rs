//! What boot does with `rules.yaml`: seed an empty database, otherwise look
//! and report.
//!
//! The database is what is enforced and what the console edits. The file is
//! what the database is expected to contain. Boot therefore writes exactly
//! once — when there is nothing yet to disagree with — and every later start
//! computes the drift and says so in the log, leaving the decision to
//! reconcile to an administrator on `/admin/sync`. Upserting on
//! every start was how console edits vanished overnight and how rules for
//! deleted entities lived on for months.

use std::path::Path;

use sqlx::PgPool;
use systemprompt_security::authz::RegisteredEntities;
use systemprompt_web_shared::error::MarketplaceError;

use crate::repositories::access_control::declared_load::{
    load_declared_set, marketplace_ids_from_services,
};
use crate::repositories::access_control::drift::{DriftCounts, compute_drift};
use crate::repositories::access_control::rules::{
    count_band_rules, list_band_rules, list_entity_defaults,
};
use crate::repositories::access_control::sync::{SyncMode, apply_sync};
use crate::repositories::sync::access_control::{PLANE_ID, applied_from};
use crate::repositories::sync::state::{Applied, BOOT_ACTOR, record_applied, record_declared};

#[derive(Debug, Default, Clone, Copy)]
pub struct BootstrapReport {
    pub declared: usize,
    pub seeded: bool,
    pub inserted: usize,
    pub drift: DriftCounts,
}

pub async fn run(
    pool: &PgPool,
    services_path: &Path,
    registered: &RegisteredEntities,
) -> Result<BootstrapReport, MarketplaceError> {
    let marketplace_ids = marketplace_ids_from_services();
    let declared = load_declared_set(pool, services_path, &marketplace_ids, registered).await?;
    let mut report = BootstrapReport {
        declared: declared.rule_count(),
        ..BootstrapReport::default()
    };

    let declared_hash = declared.declared_hash();
    if count_band_rules(pool).await? == 0 {
        let outcome = apply_sync(pool, &declared, SyncMode::Overwrite).await?;
        record_applied(
            pool,
            &Applied {
                plane: PLANE_ID,
                declared_hash: &declared_hash,
                mode: None,
                actor: BOOT_ACTOR,
                from: applied_from(),
            },
        )
        .await?;
        report.seeded = true;
        report.inserted = outcome.inserted;
        tracing::info!(
            declared = report.declared,
            inserted = outcome.inserted,
            entities = outcome.entities_inserted,
            "access_control_seeded: empty table populated from rules.yaml"
        );
        return Ok(report);
    }

    record_declared(pool, PLANE_ID, &declared_hash).await?;
    let rules = list_band_rules(pool).await?;
    let entities = list_entity_defaults(pool).await?;
    let drift = compute_drift(&declared, &rules, &entities);
    report.drift = drift.counts();
    if !declared.awaiting.is_empty() {
        tracing::info!(
            awaiting = declared.awaiting.len(),
            "access_control_awaiting_bundles: rules.yaml declares entities whose bundle is not active"
        );
    }
    if drift.is_clean() {
        tracing::info!(
            declared = report.declared,
            "access_control_in_sync: rules.yaml and the database agree; nothing written"
        );
    } else {
        tracing::warn!(
            missing_in_db = report.drift.missing_in_db,
            only_in_db = report.drift.only_in_db,
            changed = report.drift.changed,
            default_changed = report.drift.default_changed,
            "access_control_drift: rules.yaml and the database differ; nothing written — \
             review and synchronise on /admin/sync"
        );
    }
    Ok(report)
}
