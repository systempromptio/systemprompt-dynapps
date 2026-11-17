//! `GET /templates` and `GET /templates/{id}/revisions/{revision}`: who sees
//! a template is decided by core's access resolution over its anchor plugins,
//! `links` names every template the caller has a live task of, the answer
//! carries an `ETag`, and a revision is served only to someone linked to it.

use axum::http::StatusCode;
use chrono::Utc;
use dynapps_test_common::db_or_skip;
use dynwork_routines::library;
use serde_json::json;

use crate::support::{create, router, seed_user};
use crate::template_support::{body, draft, drop_rules, get, ids, link, publish, rule, seed_admin};

#[tokio::test]
async fn a_template_is_seen_when_core_admits_one_of_its_anchor_plugins() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    seed_user(&db.pool, "bob").await;
    seed_user(&db.pool, "carol").await;
    // `sales` admits alice by name; `ops` admits every `user` but denies bob,
    // and the person band is narrower than the role band, so bob's deny wins.
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    rule(&db.pool, "ops", "role", "user", "allow").await;
    rule(&db.pool, "ops", "user", "bob", "deny").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    publish(&db.pool, &admin, "ops-review", &["ops"], "Ops review").await;
    publish(&db.pool, &admin, "either", &["sales", "ops"], "Either").await;
    library::save_draft(
        &db.pool,
        &draft("unpublished", &["ops"], false, body("Draft")),
        &admin,
        Utc::now(),
    )
    .await
    .expect("save a draft");
    let router = router(&db.pool);

    let alice = get(&router, "/templates", "alice", None).await;
    assert_eq!(alice.status, StatusCode::OK, "{}", alice.body);
    assert_eq!(ids(&alice), ["either", "ops-review", "sales-review"]);
    let bob = get(&router, "/templates", "bob", None).await;
    assert!(
        ids(&bob).is_empty(),
        "a deny on the only admitting anchor hides it"
    );
    let carol = get(&router, "/templates", "carol", None).await;
    assert_eq!(
        ids(&carol),
        ["either", "ops-review"],
        "one admitting anchor of two is enough"
    );

    let served = alice.body["templates"]
        .as_array()
        .expect("templates")
        .iter()
        .find(|template| template["id"] == "sales-review")
        .expect("sales-review")
        .clone();
    let mut expected = body("Sales review");
    expected["id"] = json!("sales-review");
    expected["revision"] = json!(1);
    expected["mandatory"] = json!(false);
    assert_eq!(served, expected, "the contract's Template, as published");
    assert_eq!(alice.body["links"], json!([]));
}

#[tokio::test]
async fn links_name_every_live_task_and_say_when_access_is_gone() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    publish(&db.pool, &admin, "retired-one", &["sales"], "Old").await;
    sqlx::query(
        "UPDATE dynwork_routine_templates SET state = 'retired', retire_mode = 'pause'
         WHERE id = 'retired-one'",
    )
    .execute(&*db.pool)
    .await
    .expect("retire a template");
    let router = router(&db.pool);
    create(&router, "alice", "task-1", "mac-1").await;
    create(&router, "alice", "task-2", "mac-1").await;
    link(&db.pool, "alice", "task-1", "sales-review", 1).await;
    link(&db.pool, "alice", "task-2", "retired-one", 1).await;

    let before = get(&router, "/templates", "alice", None).await;
    assert_eq!(
        ids(&before),
        ["sales-review"],
        "a retired template is not served"
    );
    assert_eq!(
        before.body["links"],
        json!([
            { "id": "retired-one", "state": "retired", "retire": "pause" },
            { "id": "sales-review", "state": "published", "retire": null },
        ])
    );

    drop_rules(&db.pool, "sales").await;
    let after = get(&router, "/templates", "alice", None).await;
    assert!(ids(&after).is_empty());
    assert_eq!(
        after.body["links"],
        json!([
            { "id": "retired-one", "state": "retired", "retire": "pause" },
            { "id": "sales-review", "state": "no_access", "retire": null },
        ])
    );
}

#[tokio::test]
async fn an_unchanged_answer_is_a_304_and_a_publish_changes_the_etag() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let router = router(&db.pool);

    let first = get(&router, "/templates", "alice", None).await;
    let etag = first.etag.expect("an ETag");
    let again = get(&router, "/templates", "alice", Some(&etag)).await;
    assert_eq!(again.status, StatusCode::NOT_MODIFIED);
    assert_eq!(again.etag.as_deref(), Some(etag.as_str()));
    assert_eq!(again.body, serde_json::Value::Null);

    publish(
        &db.pool,
        &admin,
        "sales-review",
        &["sales"],
        "Sales review, v2",
    )
    .await;
    let changed = get(&router, "/templates", "alice", Some(&etag)).await;
    assert_eq!(changed.status, StatusCode::OK);
    assert_ne!(changed.etag.as_deref(), Some(etag.as_str()));
    assert_eq!(changed.body["templates"][0]["revision"], 2);
}

#[tokio::test]
async fn a_revision_is_served_only_to_someone_linked_to_the_template() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    seed_user(&db.pool, "bob").await;
    rule(&db.pool, "sales", "role", "user", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    publish(
        &db.pool,
        &admin,
        "sales-review",
        &["sales"],
        "Sales review, v2",
    )
    .await;
    let router = router(&db.pool);
    create(&router, "alice", "task-1", "mac-1").await;
    link(&db.pool, "alice", "task-1", "sales-review", 1).await;
    sqlx::query("UPDATE dynwork_routines SET deleted_at = now() WHERE user_id = 'alice'")
        .execute(&*db.pool)
        .await
        .expect("delete alice's task");

    let base = get(
        &router,
        "/templates/sales-review/revisions/1",
        "alice",
        None,
    )
    .await;
    assert_eq!(base.status, StatusCode::OK, "a deleted link still counts");
    assert_eq!(base.body["revision"], 1);
    assert_eq!(base.body["name"], "Sales review");
    let bob = get(&router, "/templates/sales-review/revisions/1", "bob", None).await;
    assert_eq!(
        bob.status,
        StatusCode::NOT_FOUND,
        "bob sees it but never had it"
    );
    let missing = get(
        &router,
        "/templates/sales-review/revisions/9",
        "alice",
        None,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let malformed = get(
        &router,
        "/templates/sales-review/revisions/x",
        "alice",
        None,
    )
    .await;
    assert_eq!(malformed.status, StatusCode::NOT_FOUND);
}
