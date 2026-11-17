//! A task's link to a template (contract §2): `PUT /routines/{id}` derives the
//! link from `definition.template`, allows one live task per person and
//! template, and refuses a locked field that differs from the revision the
//! write names; permissions are a ceiling. Unlinking is an ordinary write.

use axum::http::{Method, StatusCode};
use dynapps_test_common::db_or_skip;
use serde_json::{Value, json};

use crate::support::{Call, router, seed_user};
use crate::template_support::{
    activate, body, draft, get, linked, publish, publish_draft, rule, seed_admin, template_columns,
};

async fn edit(router: &axum::Router, revision: i64, definition: Value) -> (StatusCode, Value) {
    Call::new(Method::PUT, "/routines/task-1", "alice")
        .if_match(revision)
        .body(json!({ "definition": definition }))
        .send(router)
        .await
}

#[tokio::test]
async fn an_activation_links_the_task_and_a_second_one_is_already_activated() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let router = router(&db.pool);

    let first = activate(
        &router,
        "alice",
        "task-1",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK, "{}", first.1);
    assert_eq!(
        template_columns(&db.pool, "alice", "task-1").await,
        (Some("sales-review".to_owned()), Some(1))
    );

    let (status, refused) = activate(
        &router,
        "alice",
        "task-2",
        linked("Again", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert_eq!(refused["error"], "already_activated");
    assert_eq!(
        refused["routine"]["id"], "task-1",
        "the live task is returned"
    );
    assert_eq!(refused["routine"]["revision"], 1);
    let links = get(&router, "/templates", "alice", None).await;
    assert_eq!(
        links.body["links"],
        json!([{ "id": "sales-review", "state": "published", "retire": null }])
    );
}

#[tokio::test]
async fn a_locked_field_that_differs_from_the_named_revision_is_refused() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let router = router(&db.pool);
    let task = linked("Sales review", "sales-review", 1);
    assert_eq!(
        activate(&router, "alice", "task-1", task.clone()).await.0,
        StatusCode::OK
    );

    let changes: [(&str, Value, Value); 6] = [
        (
            "instructions",
            json!(["instructions"]),
            json!({ "kind": "prompt", "text": "Something else" }),
        ),
        ("agent", json!(["agent"]), json!("plan")),
        (
            "connectors",
            json!(["options", "connectors"]),
            json!(["dynapps-odoo", "dynbrain"]),
        ),
        (
            "weekday of a time lock",
            json!(["schedule", "cron"]),
            json!("0 8 * * *"),
        ),
        (
            "time limit over its ceiling",
            json!(["options", "timeLimit"]),
            json!(90),
        ),
        (
            "permissions above the ceiling",
            json!(["options", "permissions"]),
            json!("bypass"),
        ),
    ];
    for (what, path, value) in changes {
        let mut edited = task.clone();
        set(&mut edited, &path, value);
        let (status, refused) = edit(&router, 1, edited).await;
        assert_eq!(status, StatusCode::CONFLICT, "{what}: {refused}");
        assert_eq!(refused["error"], "template_locked", "{what}");
        assert_eq!(refused["current"]["revision"], 1, "{what}: the current row");
        assert_eq!(refused["current"]["definition"], task, "{what}");
    }

    let mut free = task.clone();
    free["name"] = json!("My sales review");
    free["enabled"] = json!(false);
    free["schedule"] = json!({ "cron": "30 7 * * 1-5", "timezone": "Atlantic/Canary" });
    free["options"]["timeLimit"] = json!(20);
    free["options"]["notify"] = json!("always");
    free["options"]["recover"] = json!("none");
    free["options"]["catchUp"] = json!(2);
    free["options"]["folder"] = json!("/Users/alice/sales");
    let (status, saved) = edit(&router, 1, free.clone()).await;
    assert_eq!(status, StatusCode::OK, "free fields: {saved}");
    assert_eq!(saved["definition"], free);
}

#[tokio::test]
async fn permissions_are_a_ceiling() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    let mut unattended = body("Unattended review");
    unattended["definition"]["options"]["permissions"] = json!("bypass");
    unattended["personal"]["folder"] = json!("required");
    publish_draft(
        &db.pool,
        &admin,
        draft("unattended", &["sales"], false, unattended),
    )
    .await;
    let router = router(&db.pool);
    let mut task = linked("Unattended review", "unattended", 1);
    task["options"]["permissions"] = json!("bypass");
    task["options"]["folder"] = json!("/Users/alice/sales");
    let (status, created) = activate(&router, "alice", "task-1", task.clone()).await;
    assert_eq!(status, StatusCode::OK, "{created}");

    task["options"]["permissions"] = json!("ask");
    let (status, asked) = edit(&router, 1, task.clone()).await;
    assert_eq!(status, StatusCode::OK, "bypass → ask: {asked}");
    task["options"]["permissions"] = json!("bypass");
    let (status, back) = edit(&router, 2, task).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "back up to the revision's own bypass: {back}"
    );

    publish(&db.pool, &admin, "careful", &["sales"], "Careful review").await;
    let mut careful = linked("Careful review", "careful", 1);
    assert_eq!(
        activate(&router, "alice", "task-2", careful.clone())
            .await
            .0,
        StatusCode::OK
    );
    careful["options"]["permissions"] = json!("bypass");
    let (status, refused) = Call::new(Method::PUT, "/routines/task-2", "alice")
        .if_match(1)
        .body(json!({ "definition": careful }))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "ask → bypass: {refused}");
    assert_eq!(refused["error"], "template_locked");
}

#[tokio::test]
async fn the_lock_check_reads_the_revision_the_write_names() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let router = router(&db.pool);
    assert_eq!(
        activate(
            &router,
            "alice",
            "task-1",
            linked("Sales review", "sales-review", 1)
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut second = body("Sales review");
    second["definition"]["instructions"]["text"] = json!("Review this week's Odoo activity");
    publish_draft(
        &db.pool,
        &admin,
        draft("sales-review", &["sales"], false, second),
    )
    .await;

    let mut stale = linked("Sales review", "sales-review", 2);
    let (status, refused) = edit(&router, 1, stale.clone()).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "revision 2's instructions differ: {refused}"
    );
    assert_eq!(refused["error"], "template_locked");

    stale["instructions"]["text"] = json!("Review this week's Odoo activity");
    let (status, merged) = edit(&router, 1, stale.clone()).await;
    assert_eq!(status, StatusCode::OK, "the merge to revision 2: {merged}");
    assert_eq!(
        template_columns(&db.pool, "alice", "task-1").await,
        (Some("sales-review".to_owned()), Some(2))
    );

    let (status, back) = edit(&router, 2, linked("Sales review", "sales-review", 1)).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a link never goes back a revision: {back}"
    );
    assert_eq!(back["error"], "template_locked");
    let (status, missing) = edit(&router, 2, linked("Sales review", "sales-review", 9)).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "no such revision: {missing}"
    );
}

#[tokio::test]
async fn unlinking_is_a_write_without_the_template() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let router = router(&db.pool);
    let mut task = linked("Sales review", "sales-review", 1);
    assert_eq!(
        activate(&router, "alice", "task-1", task.clone()).await.0,
        StatusCode::OK
    );

    task.as_object_mut().expect("an object").remove("template");
    task["instructions"]["text"] = json!("Anything, now that it is mine");
    let (status, unlinked) = edit(&router, 1, task).await;
    assert_eq!(status, StatusCode::OK, "{unlinked}");
    assert_eq!(
        template_columns(&db.pool, "alice", "task-1").await,
        (None, None)
    );
    let after = get(&router, "/templates", "alice", None).await;
    assert_eq!(
        after.body["links"],
        json!([]),
        "the link is no longer listed"
    );
    let (status, again) = activate(
        &router,
        "alice",
        "task-2",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "it can be activated again: {again}");
}

#[tokio::test]
async fn only_a_published_template_the_caller_sees_takes_a_new_activation() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    rule(&db.pool, "finance", "user", "nobody", "allow").await;
    publish(
        &db.pool,
        &admin,
        "finance-close",
        &["finance"],
        "Finance close",
    )
    .await;
    publish(&db.pool, &admin, "closed-one", &["sales"], "Closed").await;
    sqlx::query("UPDATE dynwork_routine_templates SET state = 'closed' WHERE id = 'closed-one'")
        .execute(&*db.pool)
        .await
        .expect("close a template");
    let router = router(&db.pool);

    for (template, name) in [
        ("finance-close", "Finance close"),
        ("closed-one", "Closed"),
        ("nowhere", "Nowhere"),
    ] {
        let (status, refused) =
            activate(&router, "alice", "task-1", linked(name, template, 1)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{template}: {refused}");
    }
    let revision = get(
        &router,
        "/templates/finance-close/revisions/1",
        "alice",
        None,
    )
    .await;
    assert_eq!(
        revision.status,
        StatusCode::NOT_FOUND,
        "a refused activation leaves no link to read revisions through"
    );
}

#[tokio::test]
async fn a_delete_keeps_the_link_on_the_tombstone() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let router = router(&db.pool);
    assert_eq!(
        activate(
            &router,
            "alice",
            "task-1",
            linked("Sales review", "sales-review", 1)
        )
        .await
        .0,
        StatusCode::OK
    );

    let (status, deleted) = Call::new(Method::DELETE, "/routines/task-1", "alice")
        .if_match(1)
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::OK, "{deleted}");
    assert_eq!(deleted["deleted"], true);
    assert_eq!(
        template_columns(&db.pool, "alice", "task-1").await,
        (Some("sales-review".to_owned()), Some(1))
    );
}

fn set(definition: &mut Value, path: &Value, value: Value) {
    let keys: Vec<&str> = path
        .as_array()
        .expect("a path")
        .iter()
        .map(|key| key.as_str().expect("a key"))
        .collect();
    let (last, parents) = keys.split_last().expect("a key");
    let parent = parents.iter().fold(definition, |node, key| &mut node[*key]);
    parent[*last] = value;
}
