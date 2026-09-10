//! Each marketplace's own `access` block, projected into
//! `access_control_rules`.
//!
//! A private workspace authors its entitlement beside itself rather than in
//! `roles.yaml`, and `default_included: false` closes the entity. The two
//! halves have to land together: writing the entity without its rules leaves
//! the workspace shut to the very group it exists for, and nothing in the
//! bootstrap tally distinguishes that from a clean start.
//!
//! The fixtures here are the shipped `services/marketplaces/` tree, not a
//! synthetic one. The defect this covers was a group and project band that
//! parsed, validated and then never reached a row, which a hand-written
//! fixture asserting its own shape would have reproduced perfectly and proved
//! nothing about what the instance actually ships.

use std::path::PathBuf;

use systemprompt::loader::ConfigLoader;
use systemprompt_web_admin::repositories::config::acl_yaml_loader::ingest_marketplace_map;

use crate::tempdb::TempDb;

// The repo's own services tree, found by climbing to the workspace root rather
// than by a fixed depth: a moved test crate would otherwise walk an empty tree
// and pass.
fn services_config_path() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("services/config/config.yaml");
        if candidate.is_file() {
            return candidate;
        }
        assert!(
            dir.pop(),
            "no services/config/config.yaml above the test crate"
        );
    }
}

#[tokio::test]
async fn every_marketplace_access_band_reaches_a_rule_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let services = ConfigLoader::load_from_path(&services_config_path())
        .expect("the shipped services tree loads");
    assert!(
        !services.marketplaces.is_empty(),
        "the services tree declares no marketplaces, so this test would assert nothing"
    );

    // What the YAML declares, counted from the configs themselves so adding a
    // marketplace does not silently weaken the assertion.
    let declared: usize = services
        .marketplaces
        .values()
        .map(|cfg| {
            cfg.access.roles.len()
                + cfg
                    .access
                    .rules
                    .iter()
                    .map(|rule| rule.values.len())
                    .sum::<usize>()
        })
        .sum();

    let written = ingest_marketplace_map(&db.pool, &services.marketplaces)
        .await
        .expect("marketplace access ingests");

    assert_eq!(
        written, declared,
        "every declared band must reach a row: the configs declare {declared} and ingestion \
         reported {written}"
    );

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT entity_id, rule_type, rule_value FROM access_control_rules
         WHERE entity_type = 'marketplace' ORDER BY 1, 2, 3",
    )
    .fetch_all(&*db.pool)
    .await
    .expect("read the ingested marketplace rules");

    assert_eq!(
        rows.len(),
        declared,
        "one row per declared value; got {rows:?}"
    );

    // The group and project bands are the ones that were missing. Asserting on
    // the shape rather than on named ids keeps this from breaking every time a
    // workspace is renamed, while still failing if the non-role bands stop
    // being written at all.
    let non_role = rows.iter().filter(|(_, kind, _)| kind != "role").count();
    let declared_non_role: usize = services
        .marketplaces
        .values()
        .flat_map(|cfg| cfg.access.rules.iter())
        .map(|rule| rule.values.len())
        .sum();
    assert_eq!(
        non_role, declared_non_role,
        "the group and project bands are declared {declared_non_role} times and reached \
         {non_role} rows: {rows:?}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_closed_marketplace_carries_the_grants_that_open_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let services = ConfigLoader::load_from_path(&services_config_path())
        .expect("the shipped services tree loads");

    ingest_marketplace_map(&db.pool, &services.marketplaces)
        .await
        .expect("marketplace access ingests");

    for (id, cfg) in &services.marketplaces {
        if cfg.access.default_included || !cfg.access.declares_rules() {
            continue;
        }
        let granted: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_control_rules
             WHERE entity_type = 'marketplace' AND entity_id = $1 AND access = 'allow'",
        )
        .bind(id.as_str())
        .fetch_one(&*db.pool)
        .await
        .expect("count the grants on a closed marketplace");

        assert!(
            granted > 0,
            "marketplace '{}' is closed by default and declares grants, but none reached a \
             row — the workspace is shut to everyone, including the group it exists for",
            id.as_str()
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn admin_marketplace_is_closed_and_grants_every_admin_role() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let services = ConfigLoader::load_from_path(&services_config_path()).expect("services");
    let (id, admin) = services
        .marketplaces
        .iter()
        .find(|(id, _)| id.as_str() == "astound-super-admin")
        .expect("admin marketplace");
    assert!(!admin.access.default_included);
    // Why: the control plane, plus the UK sales bundle it carries for
    // super_admin. roles.yaml grants that plugin and its skills to super_admin,
    // but a plugin only reaches a user through a marketplace — without this the
    // grants were inert and a super_admin outside the `uk` group got nothing.
    assert_eq!(
        admin.plugins.include,
        vec!["astound-super-admin", "astound-sales-uk"]
    );
    let general = services
        .marketplaces
        .iter()
        .find(|(id, _)| id.as_str() == "astound-commons")
        .expect("general")
        .1;
    assert!(
        !general
            .plugins
            .include
            .iter()
            .any(|id| id == "astound-super-admin")
    );
    ingest_marketplace_map(&db.pool, &services.marketplaces)
        .await
        .expect("ingest");
    let grants: Vec<(String, String)> = sqlx::query_as(
        "SELECT rule_type, rule_value FROM access_control_rules WHERE entity_type='marketplace' AND entity_id=$1 AND access='allow' ORDER BY rule_value"
    ).bind(id.as_str()).fetch_all(&*db.pool).await.expect("grants");
    assert_eq!(
        grants,
        vec![
            ("role".into(), "admin".into()),
            ("role".into(), "platform_admin".into()),
            ("role".into(), "super_admin".into())
        ]
    );
    db.cleanup().await;
}

#[tokio::test]
async fn admin_access_migration_retires_only_the_previous_bootstrap_exclusions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    for (kind, id, justification) in [
        (
            "mcp_server",
            "systemprompt",
            "configured access for members of a group or project",
        ),
        (
            "plugin",
            "astound-admin",
            "configured access for members of a group or project",
        ),
        (
            "skill",
            "admin_activity_report",
            "configured access for members of a group or project",
        ),
        (
            "skill",
            "admin_user_report",
            "configured access for members of a group or project",
        ),
        ("skill", "systemprompt_cli", "operator restriction"),
        (
            "mcp_server",
            "knowledge-bank",
            "configured access for members of a group or project",
        ),
    ] {
        sqlx::query("INSERT INTO access_control_entities (entity_type,entity_id,default_included,source) VALUES ($1,$2,false,'test') ON CONFLICT DO NOTHING")
            .bind(kind).bind(id).execute(&*db.pool).await.expect("entity");
        sqlx::query("INSERT INTO access_control_rules (entity_type,entity_id,rule_type,rule_value,access,justification) VALUES ($1,$2,'group','india-devs','deny',$3)")
            .bind(kind).bind(id).bind(justification).execute(&*db.pool).await.expect("old rule");
    }
    let migration = include_str!(
        "../../../../extensions/web/schema/migrations/047_admin_marketplace_access.sql"
    );
    for _ in 0..2 {
        sqlx::raw_sql(migration)
            .execute(&*db.pool)
            .await
            .expect("migration");
    }
    let remaining: Vec<String> = sqlx::query_scalar("SELECT entity_id FROM access_control_rules WHERE rule_type='group' AND rule_value='india-devs' AND access='deny' ORDER BY entity_id")
        .fetch_all(&*db.pool).await.expect("remaining rules");
    assert_eq!(remaining, vec!["knowledge-bank", "systemprompt_cli"]);
    db.cleanup().await;
}
