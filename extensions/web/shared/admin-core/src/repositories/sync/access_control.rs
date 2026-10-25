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

use sqlx::PgPool;
use systemprompt::loader::services_root::ServicesRootBootstrap;

use super::state::AppliedFrom;
use crate::error::{AdminError, AdminResult};
use crate::repositories::access_rules::declared::DeclaredSet;
use crate::repositories::access_rules::declared_load::{
    load_declared_set, marketplace_ids_from_services,
};
use crate::repositories::access_rules::drift::{DriftReport, compute_drift};
use crate::repositories::access_rules::rules::{list_band_rules, list_entity_defaults};
use crate::repositories::gateway_catalog::registered_routes_from_services;

pub const PLANE_ID: &str = super::plane_ids::ACCESS_CONTROL;
pub const REVIEW_URL: &str = "/admin/sync?tab=access";

pub async fn declared_now() -> AdminResult<DeclaredSet> {
    let registered = registered_routes_from_services()?;
    let marketplace_ids = marketplace_ids_from_services()
        .map_err(|source| AdminError::invalid("services tree could not be composed", source))?;
    let services_path = ServicesRootBootstrap::require()?.path;
    load_declared_set(&services_path, &marketplace_ids, &registered)
        .await
        .map_err(|source| AdminError::invalid("rules.yaml could not be loaded", source))
}

pub async fn drift_now(pool: &PgPool, declared: &DeclaredSet) -> AdminResult<DriftReport> {
    let rules = list_band_rules(pool).await?;
    let entities = list_entity_defaults(pool).await?;
    Ok(compute_drift(declared, &rules, &entities))
}

#[must_use]
pub fn applied_from() -> AppliedFrom {
    // Why: discard-ok: an unreadable profile or cache records no hashes
    let sources = super::sources::build_sources().ok();
    AppliedFrom {
        base_tree_hash: sources.as_ref().and_then(|s| s.base.tree_hash.clone()),
        composed_hash: sources.and_then(|s| s.composed_hash),
    }
}
