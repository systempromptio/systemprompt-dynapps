//! Assembles the declared set from disk and the catalog.
//!
//! The I/O half of [`super::declared`]: reads `rules.yaml`, lists the live
//! gateway routes, and hands both to the pure projection. Marketplace ids
//! arrive as a parameter — the boot path takes them from the process-wide
//! services cache, a test from a fixture tree — so the same function serves
//! both without the cache leaking into tests.

use std::path::Path;

use sqlx::PgPool;
use systemprompt::loader::ConfigLoader;
use systemprompt_security::authz::RegisteredEntities;
use systemprompt_web_shared::error::MarketplaceError;

use super::declared::{DeclaredInputs, DeclaredSet, build_declared_set};
use super::rules::list_entity_defaults;
use crate::repositories::config::rules_yaml_loader::read_rules_doc;

pub async fn load_declared_set(
    pool: &PgPool,
    services_path: &Path,
    marketplace_ids: &[String],
    registered: &RegisteredEntities,
) -> Result<DeclaredSet, MarketplaceError> {
    let Some(doc) = read_rules_doc(services_path).await? else {
        return Ok(DeclaredSet::default());
    };
    let gateway_routes: Vec<String> = list_entity_defaults(pool)
        .await?
        .into_iter()
        .filter(|e| e.entity_type == "gateway_route")
        .map(|e| e.entity_id)
        .collect();
    build_declared_set(
        &doc,
        &DeclaredInputs {
            gateway_routes: &gateway_routes,
            marketplace_ids,
            registered,
        },
    )
    .map_err(|e| MarketplaceError::config_file("access-control/rules.yaml", e))
}

// Why: the marketplace ids the active services tree defines, or none when the
// tree is unreadable — the server does not start without one, so an error
// here is someone else's to report, not this loader's.
#[must_use]
pub fn marketplace_ids_from_services() -> Vec<String> {
    match ConfigLoader::load() {
        Ok(services) => services
            .marketplaces
            .keys()
            .map(|id| id.as_str().to_owned())
            .collect(),
        Err(e) => {
            tracing::error!(error = %e, "access-control: services config unreadable");
            Vec::new()
        },
    }
}
