//! The access-control declaration as the console reads it.
//!
//! `services/access-control/rules.yaml` is resolved against the composed
//! services tree, and its drift against `access_control_rules` and
//! `access_control_entities` is computed entity by entity.
//!
//! Core owns the plane and is its only writer ([`super::core_planes`]); this
//! module only reads, for the Access review tab and the entity panels, which
//! explain each difference in the console's terms (band, subject, who wrote
//! the row). The database-only rows a kit's own `access:` block wrote (source
//! `bundle:<name>`) surface as *bundle* rows. An unreadable file is the
//! review's headline, not a 500.

use std::path::Path;

use sqlx::PgPool;
use systemprompt::manifest::ServicesConfig;

use super::state::AppliedFrom;
use crate::error::{AdminError, AdminResult};
use crate::extension_boot::ExtensionBoot;
use crate::repositories::access_rules::declared::DeclaredSet;
use crate::repositories::access_rules::declared_load::{
    load_declared_set, marketplace_ids_from_services,
};
use crate::repositories::access_rules::drift::{DriftReport, compute_drift};
use crate::repositories::access_rules::rules::{list_band_rules, list_entity_defaults};
use crate::repositories::gateway_catalog::registered_routes_from_services;

pub const PLANE_ID: &str = super::plane_ids::ACCESS_CONTROL;
pub const REVIEW_URL: &str = "/admin/sync?tab=access";

pub async fn declared_now(
    services: &ServicesConfig,
    services_path: &Path,
) -> AdminResult<DeclaredSet> {
    let registered = registered_routes_from_services(services);
    let marketplace_ids = marketplace_ids_from_services(services);
    load_declared_set(services_path, &marketplace_ids, &registered)
        .await
        .map_err(|source| AdminError::invalid("rules.yaml could not be loaded", source))
}

pub async fn drift_now(pool: &PgPool, declared: &DeclaredSet) -> AdminResult<DriftReport> {
    let rules = list_band_rules(pool).await?;
    let entities = list_entity_defaults(pool).await?;
    Ok(compute_drift(declared, &rules, &entities))
}

#[must_use]
pub fn applied_from(boot: &ExtensionBoot) -> AppliedFrom {
    let (base_tree_hash, composed_hash) =
        super::sources::tree_hashes(boot.profile(), boot.active_services_root());
    AppliedFrom {
        base_tree_hash,
        composed_hash,
    }
}
