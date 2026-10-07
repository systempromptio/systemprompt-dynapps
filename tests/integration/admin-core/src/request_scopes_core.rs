//! `ai_request_scopes` under core's gateway attribution: the project and group
//! core recorded in `ai_request_attributions` replace the person's defaults,
//! and a dimension core did not attribute keeps the default.

use sqlx::PgPool;
use systemprompt_web_admin_core::repositories::scope::defaults::set_scope_defaults;

use crate::fixtures::{
    RequestSpec, insert_group, insert_project, insert_request, insert_user, unclaimed_email,
    unique, unique_group, unique_project,
};
use crate::tempdb::TempDb;

async fn attribute(pool: &PgPool, request_id: &str, rows: &[(&str, &str, &str)]) {
    let dimensions: Vec<&str> = rows.iter().map(|r| r.0).collect();
    let values: Vec<&str> = rows.iter().map(|r| r.1).collect();
    let sources: Vec<&str> = rows.iter().map(|r| r.2).collect();
    sqlx::query(
        "INSERT INTO ai_request_attributions (request_id, dimension, value, source)
         SELECT $1, t.d, t.v, t.s FROM UNNEST($2::text[], $3::text[], $4::text[]) AS t(d, v, s)",
    )
    .bind(request_id)
    .bind(&dimensions)
    .bind(&values)
    .bind(&sources)
    .execute(pool)
    .await
    .expect("insert core attribution");
}

async fn stamp(pool: &PgPool, request_id: &str) -> (Option<String>, Option<String>, String) {
    sqlx::query_as::<_, (Option<String>, Option<String>, String)>(
        "SELECT group_id, project_id, source FROM ai_request_scopes WHERE request_id = $1",
    )
    .bind(request_id)
    .fetch_one(pool)
    .await
    .expect("read stamp")
}

#[tokio::test]
async fn core_attribution_replaces_the_defaults_it_names_and_only_those() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    let primary = unique_project("prj");
    let chosen = unique_project("prj");
    insert_group(&db.pool, &group, group.as_str()).await;
    insert_project(&db.pool, &primary, primary.as_str()).await;
    insert_project(&db.pool, &chosen, chosen.as_str()).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("attr")).await;
    set_scope_defaults(&db.pool, &person, Some(&group), Some(&primary))
        .await
        .expect("set defaults");

    let request = unique("req");
    insert_request(&db.pool, &RequestSpec::completed(&request, &person)).await;
    attribute(
        &db.pool,
        &request,
        &[("project", chosen.as_str(), "header")],
    )
    .await;

    assert_eq!(
        stamp(&db.pool, &request).await,
        (
            Some(group.as_str().to_owned()),
            Some(chosen.as_str().to_owned()),
            "header".to_owned()
        ),
        "the header's project wins; the unattributed group keeps the primary"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn core_sources_map_onto_the_stamp_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let project = unique_project("prj");
    insert_project(&db.pool, &project, project.as_str()).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("src")).await;

    for (core_source, expected) in [("api_key", "api_key"), ("default", "primary")] {
        let request = unique("req");
        insert_request(&db.pool, &RequestSpec::completed(&request, &person)).await;
        attribute(
            &db.pool,
            &request,
            &[("project", project.as_str(), core_source)],
        )
        .await;
        let (_, project_id, source) = stamp(&db.pool, &request).await;
        assert_eq!(project_id.as_deref(), Some(project.as_str()));
        assert_eq!(source, expected, "core source {core_source}");
    }
    db.cleanup().await;
}

#[tokio::test]
async fn a_value_that_is_not_a_project_leaves_the_default_in_place() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let primary = unique_project("prj");
    insert_project(&db.pool, &primary, primary.as_str()).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("gone")).await;
    set_scope_defaults(&db.pool, &person, None, Some(&primary))
        .await
        .expect("set defaults");

    let request = unique("req");
    insert_request(&db.pool, &RequestSpec::completed(&request, &person)).await;
    attribute(
        &db.pool,
        &request,
        &[("project", "no-such-project", "header")],
    )
    .await;

    assert_eq!(
        stamp(&db.pool, &request).await,
        (
            None,
            Some(primary.as_str().to_owned()),
            "primary".to_owned()
        )
    );
    db.cleanup().await;
}
