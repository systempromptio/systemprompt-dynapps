//! A second boot changes nothing: the gateway-route catalog plane and the
//! access-control plane each own their columns of `access_control_entities`,
//! so running them again, in either order, leaves both authz tables
//! byte-identical. The bug this pins flipped every `gateway_route/*` declared
//! `default: open` to closed on the restart after the first, because the
//! catalog upsert rewrote `default_included` on conflict.

use std::collections::BTreeMap;

use sqlx::PgPool;
use systemprompt_security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_admin_core::repositories::access_rules::declared::DeclaredSet;
use systemprompt_web_admin_core::repositories::access_rules::declared_load::build_declared_from_doc;
use systemprompt_web_admin_core::repositories::access_rules::rules_yaml_loader::parse_rules_doc;
use systemprompt_web_admin_core::repositories::access_rules::sync::{SyncMode, apply_sync};
use systemprompt_web_admin_core::repositories::sync::route_catalog::reconcile_route_catalog;

use crate::fixtures::unique;
use crate::tempdb::TempDb;

const RULES: &str = "entities:\n  - entity: gateway_route/*\n    default: open\n    why: every signed-in user may reach the gateway\n    allow:\n      role: [user]\n";
const SOURCE: &str = "services:test/ai/gateway.yaml";

async fn snapshot(pool: &PgPool) -> (Vec<String>, Vec<String>) {
    let entities = sqlx::query_scalar::<_, String>(
        "SELECT row_to_json(e)::text FROM access_control_entities e
         ORDER BY entity_type, entity_id",
    )
    .fetch_all(pool)
    .await
    .expect("entities snapshot");
    let rules = sqlx::query_scalar::<_, String>(
        "SELECT row_to_json(r)::text FROM access_control_rules r
         ORDER BY entity_type, entity_id, rule_type, rule_value",
    )
    .fetch_all(pool)
    .await
    .expect("rules snapshot");
    (entities, rules)
}

fn declared_for(routes: &[String]) -> DeclaredSet {
    let doc = parse_rules_doc(RULES).expect("rules doc");
    let registered = RegisteredEntities::new()
        .with_kind(EntityKind::GatewayRoute, routes.iter().map(String::as_str));
    build_declared_from_doc(&doc, &[], &registered).expect("declared set")
}

fn route_defaults(declared: &DeclaredSet) -> BTreeMap<String, bool> {
    declared
        .entities
        .values()
        .filter(|e| e.entity_type == "gateway_route")
        .map(|e| (e.entity_id.clone(), e.default_included))
        .collect()
}

async fn open_routes(pool: &PgPool, routes: &[String]) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM access_control_entities
         WHERE entity_type = 'gateway_route' AND entity_id = ANY($1) AND default_included",
    )
    .bind(routes)
    .fetch_one(pool)
    .await
    .expect("open count")
}

#[tokio::test]
async fn second_boot_in_either_order_leaves_the_authz_tables_identical() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let routes = vec![unique("route-a"), unique("route-b")];
    let declared = declared_for(&routes);
    let defaults = route_defaults(&declared);

    reconcile_route_catalog(&db.pool, &routes, &defaults, SOURCE)
        .await
        .expect("first catalog pass");
    apply_sync(&db.pool, &declared, SyncMode::Overwrite)
        .await
        .expect("first access pass");
    let first = snapshot(&db.pool).await;
    assert_eq!(
        open_routes(&db.pool, &routes).await,
        2,
        "rules.yaml declares the routes open"
    );

    apply_sync(&db.pool, &declared, SyncMode::Overwrite)
        .await
        .expect("second access pass");
    reconcile_route_catalog(&db.pool, &routes, &defaults, SOURCE)
        .await
        .expect("second catalog pass");
    assert_eq!(
        snapshot(&db.pool).await,
        first,
        "access then catalog drifted"
    );

    reconcile_route_catalog(&db.pool, &routes, &BTreeMap::new(), SOURCE)
        .await
        .expect("catalog pass without declared defaults");
    assert_eq!(
        snapshot(&db.pool).await,
        first,
        "the catalog rewrote an access column it does not own"
    );
    db.cleanup().await;
}
