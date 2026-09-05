//! `repositories::config::member_grants_yaml_loader` — access-control
//! groups.yaml -> the `rule_type = 'group'` gate rows and closed entity
//! defaults.
//!
//! The loader is shared with `projects.yaml`, which differs only in the
//! dimension it writes at, so exercising one file exercises both paths.

use systemprompt_web_admin::authz::group::{group_dimension, group_rule_type};
use systemprompt_web_admin::repositories::config::member_grants_yaml_loader::load_one;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

const GROUPS_FILE: &str = "access-control/groups.yaml";

fn grants_yaml(entity_type: &str, entity_id: &str, members: &[&str]) -> String {
    let members = members
        .iter()
        .map(|g| format!("      - {g}\n"))
        .collect::<String>();
    format!(
        "grants:\n  - entity_type: {entity_type}\n    entity_id: {entity_id}\n    \
         members:\n{members}"
    )
}

async fn load(pool: &sqlx::PgPool, dir: &std::path::Path) -> Result<usize, impl std::fmt::Debug> {
    load_one(pool, dir, GROUPS_FILE, group_rule_type()).await
}

async fn group_rules(pool: &sqlx::PgPool) -> Vec<(String, String)> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT entity_id, rule_value FROM access_control_rules
         WHERE rule_type = $1 ORDER BY entity_id, rule_value",
    )
    .bind(group_rule_type().as_str())
    .fetch_all(pool)
    .await
    .expect("read projected rules")
}

#[test]
fn the_group_dimension_sits_below_role() {
    let dim = group_dimension();
    assert_eq!(dim.rule_type.as_str(), "group");
    assert_eq!(dim.label, "Group");
    assert!(
        dim.precedence > 100 && dim.precedence < 200,
        "must out-rank the role band and defer to the user band, got {}",
        dim.precedence
    );
}

#[tokio::test]
async fn load_is_a_no_op_when_the_file_is_absent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");

    let report = load(&db.pool, dir.path())
        .await
        .expect("a missing file is not an error");

    assert_eq!(report, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn load_projects_a_closed_default_and_one_rule_per_member() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("group-plugin");
    write_services_file(
        dir.path(),
        GROUPS_FILE,
        &grants_yaml("plugin", &plugin, &["europe-devs", "india-devs"]),
    );

    let report = load(&db.pool, dir.path())
        .await
        .expect("load the group gate");

    assert_eq!(report, 2);
    assert_eq!(
        group_rules(&db.pool).await,
        vec![
            (plugin.clone(), "europe-devs".to_owned()),
            (plugin.clone(), "india-devs".to_owned()),
        ]
    );

    let default_included = sqlx::query_scalar::<_, bool>(
        "SELECT default_included FROM access_control_entities
         WHERE entity_type = 'plugin' AND entity_id = $1",
    )
    .bind(&plugin)
    .fetch_one(&*db.pool)
    .await
    .expect("catalog row exists");
    assert!(
        !default_included,
        "a user outside every group falls through to the entity default, which must be closed"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_forces_an_open_default_closed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("group-plugin");
    insert_acl_entity(&db.pool, "plugin", &plugin, true).await;
    write_services_file(
        dir.path(),
        GROUPS_FILE,
        &grants_yaml("plugin", &plugin, &["europe-devs"]),
    );

    load(&db.pool, dir.path())
        .await
        .expect("load the group gate");

    let default_included = sqlx::query_scalar::<_, bool>(
        "SELECT default_included FROM access_control_entities
         WHERE entity_type = 'plugin' AND entity_id = $1",
    )
    .bind(&plugin)
    .fetch_one(&*db.pool)
    .await
    .expect("catalog row exists");
    assert!(
        !default_included,
        "the gate owns the final word on defaults"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_retracts_rules_the_file_no_longer_carries() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("group-plugin");
    write_services_file(
        dir.path(),
        GROUPS_FILE,
        &grants_yaml("plugin", &plugin, &["europe-devs", "india-devs"]),
    );
    load(&db.pool, dir.path()).await.expect("first load");

    write_services_file(
        dir.path(),
        GROUPS_FILE,
        &grants_yaml("plugin", &plugin, &["india-devs"]),
    );
    load(&db.pool, dir.path()).await.expect("second load");

    assert_eq!(
        group_rules(&db.pool).await,
        vec![(plugin, "india-devs".to_owned())]
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_rejects_an_unknown_entity_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        GROUPS_FILE,
        &grants_yaml("teapot", "x", &["europe-devs"]),
    );

    let rejected = load(&db.pool, dir.path()).await.is_err();

    assert!(rejected, "an unknown entity kind must not be stored");

    db.cleanup().await;
}

#[tokio::test]
async fn load_rejects_a_grant_with_no_members() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        GROUPS_FILE,
        "grants:\n  - entity_type: plugin\n    entity_id: p\n    members: []\n",
    );

    let rejected = load(&db.pool, dir.path()).await.is_err();

    assert!(
        rejected,
        "a grant naming no group would close the entity to everyone silently"
    );

    db.cleanup().await;
}
