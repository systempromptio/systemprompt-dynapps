//! A restart changes nothing. Boot runs the install's own planes alone
//! (groups, gateway routes), which still seed once when empty; the planes
//! core owns are core's. Core's access-control apply is idempotent: running it
//! again leaves both authz tables byte-identical, so a `gateway_route/*`
//! declared `default: open` stays open on every apply after the first.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::governance::authz::AccessControlPlane;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::sync::boot::{BOOT_ORDER, reconcile_plane};
use systemprompt_web_admin_core::repositories::sync::core_planes::CorePlanes;
use systemprompt_web_admin_core::repositories::sync::plane::{
    Actor, ApplyOutcome, Export, PlaneDrift, SyncMode as PlaneSyncMode, SyncPlane,
};
use systemprompt_web_admin_core::repositories::sync::plane_ids;

use crate::fixtures::unique;
use crate::tempdb::TempDb;

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

fn rules_for(routes: &[String]) -> String {
    let mut rules = String::from("entities:\n");
    for route in routes {
        rules.push_str(&format!(
            "  - entity: gateway_route/{route}\n    default: open\n    why: every signed-in user may reach the gateway\n    allow:\n      role: [user]\n"
        ));
    }
    rules
}

fn access_planes(db: &TempDb, root: &std::path::Path) -> CorePlanes {
    let pool: systemprompt::database::DbPool =
        std::sync::Arc::new(systemprompt::database::Database::from_pools(
            std::sync::Arc::clone(&db.pool),
            Some(std::sync::Arc::clone(&db.pool)),
        ));
    let planes = systemprompt::extension::prelude::ConfigPlaneSet::new(
        vec![std::sync::Arc::new(AccessControlPlane)],
        std::collections::BTreeMap::new(),
        root.to_path_buf(),
    )
    .expect("access-control plane set");
    CorePlanes::over(
        planes,
        systemprompt::scheduler::repository::ConfigReconcileRepository::new(&pool),
    )
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
fn boot_runs_only_the_planes_this_install_owns() {
    assert_eq!(BOOT_ORDER, [plane_ids::GROUPS, plane_ids::GATEWAY_ROUTES]);
}

#[tokio::test]
async fn a_second_core_apply_leaves_the_authz_tables_identical() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let routes = vec![unique("route-a"), unique("route-b")];
    let dir = tempfile::tempdir().expect("temp services dir");
    std::fs::create_dir_all(dir.path().join("access-control")).expect("rules dir");
    std::fs::write(
        dir.path().join("access-control/rules.yaml"),
        rules_for(&routes),
    )
    .expect("write rules.yaml");
    let planes = access_planes(&db, dir.path());
    let actor = UserId::new("restart-tester");

    planes
        .apply("access_control", &actor, "first apply")
        .await
        .expect("first apply");
    let first = snapshot(&db.pool).await;
    assert_eq!(
        open_routes(&db.pool, &routes).await,
        2,
        "rules.yaml declares the routes open"
    );

    planes
        .apply("access_control", &actor, "second apply")
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
async fn boot_still_seeds_an_empty_projection_the_install_owns() {
    let pool = unconnected_pool();
    let plane = EmptyPlane::new(plane_ids::GROUPS);
    let boot = reconcile_plane(&pool, &plane)
        .await
        .expect("boot reconcile");
    assert!(boot.seeded && plane.applied.load(Ordering::SeqCst));
    assert_eq!(boot.inserted, 3);
}
