//! A restart changes nothing. Boot never writes a plane core enforces from
//! the database (access control, gateway policies): it reports drift and
//! leaves the projection as it is, even when it is empty, while the
//! install's own projections still seed once when empty. An explicit
//! overwrite apply is idempotent: running it again leaves both authz tables
//! byte-identical, so a `gateway_route/*` declared `default: open` stays open
//! on every apply after the first.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::governance::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::access_rules::declared::DeclaredSet;
use systemprompt_web_admin_core::repositories::access_rules::declared_load::build_declared_from_doc;
use systemprompt_web_admin_core::repositories::access_rules::rules_yaml_loader::parse_rules_doc;
use systemprompt_web_admin_core::repositories::access_rules::sync::{SyncMode, apply_sync};
use systemprompt_web_admin_core::repositories::sync::boot::{reconcile_plane, seeds_at_boot};
use systemprompt_web_admin_core::repositories::sync::plane::{
    Actor, ApplyOutcome, Export, PlaneDrift, SyncMode as PlaneSyncMode, SyncPlane,
};
use systemprompt_web_admin_core::repositories::sync::plane_ids;

use crate::fixtures::unique;
use crate::tempdb::TempDb;

const RULES: &str = "entities:\n  - entity: gateway_route/*\n    default: open\n    why: every signed-in user may reach the gateway\n    allow:\n      role: [user]\n";

async fn snapshot(pool: &PgPool) -> (Vec<String>, Vec<String>, Vec<String>) {
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
    let policies = sqlx::query_scalar::<_, String>(
        "SELECT row_to_json(p)::text FROM ai_gateway_policies p ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .expect("policies snapshot");
    (entities, rules, policies)
}

fn declared_for(routes: &[String]) -> DeclaredSet {
    let doc = parse_rules_doc(RULES).expect("rules doc");
    let registered = RegisteredEntities::new()
        .with_kind(EntityKind::GatewayRoute, routes.iter().map(String::as_str));
    build_declared_from_doc(&doc, &[], &registered).expect("declared set")
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

#[test]
fn boot_seeds_only_the_planes_this_install_owns() {
    assert!(!seeds_at_boot(plane_ids::ACCESS_CONTROL));
    assert!(!seeds_at_boot(plane_ids::GATEWAY_POLICIES));
    assert!(seeds_at_boot(plane_ids::GROUPS));
    assert!(seeds_at_boot(plane_ids::GATEWAY_ROUTES));
    assert!(seeds_at_boot(plane_ids::GOVERNANCE));
}

#[tokio::test]
async fn a_second_overwrite_apply_leaves_the_authz_tables_identical() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let routes = vec![unique("route-a"), unique("route-b")];
    let declared = declared_for(&routes);

    apply_sync(&db.pool, &declared, SyncMode::Overwrite)
        .await
        .expect("first apply");
    let first = snapshot(&db.pool).await;
    assert_eq!(
        open_routes(&db.pool, &routes).await,
        2,
        "rules.yaml declares the routes open"
    );

    apply_sync(&db.pool, &declared, SyncMode::Overwrite)
        .await
        .expect("second apply");
    assert_eq!(snapshot(&db.pool).await, first, "the second apply drifted");
    db.cleanup().await;
}

// A plane whose projection is empty while code declares three entries: the
// one state in which boot may seed. It records whether boot applied it.
struct EmptyPlane {
    id: &'static str,
    applied: AtomicBool,
}

impl EmptyPlane {
    const fn new(id: &'static str) -> Self {
        Self {
            id,
            applied: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl SyncPlane for EmptyPlane {
    fn id(&self) -> &'static str {
        self.id
    }
    fn label(&self) -> &'static str {
        "empty"
    }
    fn source_file(&self) -> &'static str {
        "empty.yaml"
    }
    fn projection(&self) -> &'static str {
        "empty"
    }
    async fn drift(&self, _pool: &PgPool) -> AdminResult<PlaneDrift> {
        Ok(PlaneDrift {
            declared_count: 3,
            in_db: 0,
            ..PlaneDrift::default()
        })
    }
    async fn apply(
        &self,
        _pool: &PgPool,
        _mode: PlaneSyncMode,
        _actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        self.applied.store(true, Ordering::SeqCst);
        Ok(ApplyOutcome {
            inserted: 3,
            ..ApplyOutcome::default()
        })
    }
    async fn export(&self, _pool: &PgPool) -> AdminResult<Option<Export>> {
        Ok(None)
    }
}

fn unconnected_pool() -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@127.0.0.1:1/unused")
        .expect("a lazy pool never connects")
}

#[tokio::test]
async fn boot_never_seeds_an_empty_core_enforced_plane() {
    let pool = unconnected_pool();
    for id in [plane_ids::ACCESS_CONTROL, plane_ids::GATEWAY_POLICIES] {
        let plane = EmptyPlane::new(id);
        let boot = reconcile_plane(&pool, &plane)
            .await
            .unwrap_or_else(|e| panic!("boot reconcile of {id}: {e}"));
        assert!(!boot.seeded, "{id} was seeded at boot");
        assert!(!plane.applied.load(Ordering::SeqCst), "{id} was written");
        assert_eq!(boot.drift_rows, 0);
    }
}

#[tokio::test]
async fn boot_still_seeds_an_empty_projection_the_install_owns() {
    let pool = unconnected_pool();
    let plane = EmptyPlane::new(plane_ids::GROUPS);
    let boot = reconcile_plane(&pool, &plane)
        .await
        .expect("boot reconcile");
    assert!(boot.seeded && plane.applied.load(Ordering::SeqCst));
    assert_eq!(boot.inserted, 3);
}
