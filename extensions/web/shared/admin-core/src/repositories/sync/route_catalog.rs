//! The gateway-route catalog plane: which `gateway_route` rows exist in
//! `access_control_entities`, and nothing else.
//!
//! The catalog owns row existence; the access-control plane
//! (`access_rules::sync`) owns every entitlement column on the same row. So a
//! conflicting insert here is a no-op: re-registering a route never rewrites
//! `default_included` or `source`, and a restart cannot flip a route that
//! `rules.yaml` declared open back to closed. A route seen for the first time
//! takes the default `rules.yaml` declares for it, closed when it declares
//! none. Routes the profile no longer dispatches are deleted by set
//! difference, which cascades their rules away.

use std::collections::BTreeMap;

use sqlx::PgPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteCatalogReport {
    pub inserted: u64,
    pub pruned: u64,
}

pub async fn reconcile_route_catalog(
    pool: &PgPool,
    route_ids: &[String],
    declared_open: &BTreeMap<String, bool>,
    source: &str,
) -> Result<RouteCatalogReport, sqlx::Error> {
    let opens: Vec<bool> = route_ids
        .iter()
        .map(|id| declared_open.get(id).copied().unwrap_or(false))
        .collect();
    let mut tx = pool.begin().await?;
    let inserted = sqlx::query!(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         SELECT 'gateway_route', t.id, t.open, $3
         FROM UNNEST($1::text[], $2::bool[]) AS t(id, open)
         ON CONFLICT (entity_type, entity_id) DO NOTHING",
        route_ids,
        &opens,
        source,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    let pruned = sqlx::query!(
        "DELETE FROM access_control_entities
         WHERE entity_type = 'gateway_route' AND entity_id <> ALL($1::text[])",
        route_ids,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tx.commit().await?;
    Ok(RouteCatalogReport { inserted, pruned })
}
