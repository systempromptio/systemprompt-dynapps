//! `repositories::config::adfs_yaml_loader` — adfs.yaml -> the
//! `rule_type = 'adfs_group'` gate rows and closed entity defaults.

use systemprompt_web_admin::authz::adfs_group::{adfs_group_dimension, adfs_group_rule_type};
use systemprompt_web_admin::repositories::config::adfs_yaml_loader::load_adfs_groups_from_yaml;

use crate::fixtures::{insert_acl_entity, unique, write_services_file};
use crate::tempdb::TempDb;

fn adfs_yaml(entity_type: &str, entity_id: &str, groups: &[&str]) -> String {
    let groups = groups
        .iter()
        .map(|g| format!("      - {g}\n"))
        .collect::<String>();
    format!(
        "grants:\n  - entity_type: {entity_type}\n    entity_id: {entity_id}\n    groups:\n{groups}"
    )
}

async fn group_rules(pool: &sqlx::PgPool) -> Vec<(String, String)> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT entity_id, rule_value FROM access_control_rules
         WHERE rule_type = $1 ORDER BY entity_id, rule_value",
    )
    .bind(adfs_group_rule_type().as_str())
    .fetch_all(pool)
    .await
    .expect("read projected rules")
}

#[test]
fn the_adfs_group_dimension_sits_below_role() {
    let dim = adfs_group_dimension();
    assert_eq!(dim.rule_type.as_str(), "adfs_group");
    assert_eq!(dim.label, "AD group");
    assert!(
        dim.precedence > 100 && dim.precedence < 200,
        "must out-rank the role band and defer to the user band, got {}",
        dim.precedence
    );
}

#[tokio::test]
async fn load_adfs_groups_from_yaml_is_a_no_op_when_the_file_is_absent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");

    let report = load_adfs_groups_from_yaml(&db.pool, dir.path())
        .await
        .expect("missing adfs.yaml is not an error");

    assert_eq!(report.grants_projected, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn load_adfs_groups_from_yaml_projects_a_closed_default_and_one_rule_per_group() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("group-plugin");
    write_services_file(
        dir.path(),
        "access-control/adfs.yaml",
        &adfs_yaml("plugin", &plugin, &["Systemprompt-A", "Systemprompt-B"]),
    );

    let report = load_adfs_groups_from_yaml(&db.pool, dir.path())
        .await
        .expect("load adfs gate");

    assert_eq!(report.grants_projected, 2);
    assert_eq!(
        group_rules(&db.pool).await,
        vec![
            (plugin.clone(), "Systemprompt-A".to_owned()),
            (plugin.clone(), "Systemprompt-B".to_owned()),
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
async fn load_adfs_groups_from_yaml_forces_an_open_default_closed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("group-plugin");
    insert_acl_entity(&db.pool, "plugin", &plugin, true).await;
    write_services_file(
        dir.path(),
        "access-control/adfs.yaml",
        &adfs_yaml("plugin", &plugin, &["Systemprompt-A"]),
    );

    load_adfs_groups_from_yaml(&db.pool, dir.path())
        .await
        .expect("load adfs gate");

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
async fn load_adfs_groups_from_yaml_retracts_rules_the_file_no_longer_carries() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    let plugin = unique("group-plugin");
    write_services_file(
        dir.path(),
        "access-control/adfs.yaml",
        &adfs_yaml("plugin", &plugin, &["Systemprompt-A", "Systemprompt-B"]),
    );
    load_adfs_groups_from_yaml(&db.pool, dir.path())
        .await
        .expect("first load");

    write_services_file(
        dir.path(),
        "access-control/adfs.yaml",
        &adfs_yaml("plugin", &plugin, &["Systemprompt-B"]),
    );
    load_adfs_groups_from_yaml(&db.pool, dir.path())
        .await
        .expect("second load");

    assert_eq!(
        group_rules(&db.pool).await,
        vec![(plugin, "Systemprompt-B".to_owned())]
    );

    db.cleanup().await;
}

#[tokio::test]
async fn load_adfs_groups_from_yaml_rejects_an_unknown_entity_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        "access-control/adfs.yaml",
        &adfs_yaml("teapot", "x", &["Systemprompt-A"]),
    );

    let result = load_adfs_groups_from_yaml(&db.pool, dir.path()).await;

    assert!(result.is_err(), "an unknown entity kind must not be stored");

    db.cleanup().await;
}

#[tokio::test]
async fn load_adfs_groups_from_yaml_rejects_a_grant_with_no_groups() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        "access-control/adfs.yaml",
        "grants:\n  - entity_type: plugin\n    entity_id: p\n    groups: []\n",
    );

    let result = load_adfs_groups_from_yaml(&db.pool, dir.path()).await;

    assert!(
        result.is_err(),
        "a grant naming no group would close the entity to everyone silently"
    );

    db.cleanup().await;
}
