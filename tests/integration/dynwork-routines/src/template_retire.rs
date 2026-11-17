//! Closing and retiring a template (contract §8), on two people's tasks:
//! close and retire·unlink change the template only; retire·pause writes
//! `enabled = false` and retire·remove tombstones every live linked task, each
//! a change in its person's feed. Re-publishing a closed template reopens it.

use axum::Router;
use axum::http::StatusCode;
use chrono::Utc;
use dynapps_test_common::db_or_skip;
use dynwork_routines::ids::TemplateId;
use dynwork_routines::library::{self, LibraryError};
use dynwork_routines::store::TemplateState;
use dynwork_routines::template::RetireMode;
use serde_json::{Value, json};
use systemprompt::identifiers::UserId;

use crate::support::{drain_feed, router, seed_user};
use crate::template_support::{activate, get, linked, publish, rule, seed_admin, template_columns};

fn template_id(text: &str) -> TemplateId {
    TemplateId::parse(text).expect("a template id")
}

// Alice and bob each with a task of `sales-review`, and each one's feed cursor
// after it.
async fn two_holders(pool: &sqlx::PgPool, router: &Router) -> (UserId, [String; 2]) {
    let admin = seed_admin(pool, "admin").await;
    seed_user(pool, "alice").await;
    seed_user(pool, "bob").await;
    rule(pool, "sales", "role", "user", "allow").await;
    publish(pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let mut cursors = [String::new(), String::new()];
    for (who, cursor) in ["alice", "bob"].into_iter().zip(cursors.iter_mut()) {
        let (status, created) = activate(
            router,
            who,
            "task-1",
            linked("Sales review", "sales-review", 1),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{who}: {created}");
        *cursor = drain_feed(router, who, "0", 50).await.1;
    }
    (admin, cursors)
}

async fn links(router: &Router, who: &str) -> Value {
    get(router, "/templates", who, None).await.body["links"].clone()
}

#[tokio::test]
async fn closing_takes_no_new_activations_and_republishing_reopens() {
    let db = db_or_skip!();
    let router = router(&db.pool);
    let (admin, cursors) = two_holders(&db.pool, &router).await;
    seed_user(&db.pool, "carol").await;

    let closed = library::close(&db.pool, &template_id("sales-review"), &admin, Utc::now())
        .await
        .expect("close");
    assert_eq!(closed.state, TemplateState::Closed);
    for (who, cursor) in ["alice", "bob"].into_iter().zip(&cursors) {
        assert!(
            drain_feed(&router, who, cursor, 50).await.0.is_empty(),
            "{who}'s task is unchanged"
        );
        assert_eq!(
            links(&router, who).await,
            json!([{ "id": "sales-review", "state": "closed", "retire": null }])
        );
    }
    let (status, refused) = activate(
        &router,
        "carol",
        "task-1",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "closed: {refused}");

    let reopened = publish(
        &db.pool,
        &admin,
        "sales-review",
        &["sales"],
        "Sales review, v2",
    )
    .await;
    assert_eq!(
        (reopened.state, reopened.revision),
        (TemplateState::Published, 2)
    );
    let (status, created) = activate(
        &router,
        "carol",
        "task-1",
        linked("Sales review, v2", "sales-review", 2),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reopened: {created}");
}

#[tokio::test]
async fn retire_unlink_leaves_the_tasks_alone() {
    let db = db_or_skip!();
    let router = router(&db.pool);
    let (admin, cursors) = two_holders(&db.pool, &router).await;

    let retired = library::retire(
        &db.pool,
        &template_id("sales-review"),
        RetireMode::Unlink,
        &admin,
        Utc::now(),
    )
    .await
    .expect("retire");
    assert_eq!(
        (retired.state, retired.retire_mode),
        (TemplateState::Retired, Some(RetireMode::Unlink))
    );
    for (who, cursor) in ["alice", "bob"].into_iter().zip(&cursors) {
        assert!(
            drain_feed(&router, who, cursor, 50).await.0.is_empty(),
            "{who}"
        );
        assert_eq!(
            links(&router, who).await,
            json!([{ "id": "sales-review", "state": "retired", "retire": "unlink" }])
        );
    }
}

#[tokio::test]
async fn retire_pause_pauses_every_linked_task_in_its_feed() {
    let db = db_or_skip!();
    let router = router(&db.pool);
    let (admin, cursors) = two_holders(&db.pool, &router).await;

    library::retire(
        &db.pool,
        &template_id("sales-review"),
        RetireMode::Pause,
        &admin,
        Utc::now(),
    )
    .await
    .expect("retire");
    for (who, cursor) in ["alice", "bob"].into_iter().zip(&cursors) {
        let (changes, _) = drain_feed(&router, who, cursor, 50).await;
        assert_eq!(changes.len(), 1, "{who}: {changes:?}");
        assert_eq!(changes[0]["id"], "task-1");
        assert_eq!(changes[0]["revision"], 2, "{who}");
        assert_eq!(changes[0]["definition"]["enabled"], false, "{who}");
        assert_eq!(
            changes[0]["definition"]["name"], "Sales review",
            "{who}: the rest stays"
        );
        assert_eq!(
            links(&router, who).await,
            json!([{ "id": "sales-review", "state": "retired", "retire": "pause" }])
        );
    }
}

#[tokio::test]
async fn retire_remove_tombstones_every_linked_task_and_keeps_the_link() {
    let db = db_or_skip!();
    let router = router(&db.pool);
    let (admin, cursors) = two_holders(&db.pool, &router).await;

    library::retire(
        &db.pool,
        &template_id("sales-review"),
        RetireMode::Remove,
        &admin,
        Utc::now(),
    )
    .await
    .expect("retire");
    for (who, cursor) in ["alice", "bob"].into_iter().zip(&cursors) {
        let (changes, _) = drain_feed(&router, who, cursor, 50).await;
        assert_eq!(changes.len(), 1, "{who}: {changes:?}");
        assert_eq!(changes[0]["deleted"], true, "{who}");
        assert_eq!(
            template_columns(&db.pool, who, "task-1").await,
            (Some("sales-review".to_owned()), Some(1)),
            "{who}: the tombstone keeps its template"
        );
        assert_eq!(
            links(&router, who).await,
            json!([]),
            "{who}: no live task, no link"
        );
    }
}

#[tokio::test]
async fn only_an_admin_closes_or_retires_and_a_retired_template_stays_retired() {
    let db = db_or_skip!();
    let router = router(&db.pool);
    let (admin, _) = two_holders(&db.pool, &router).await;
    let alice = crate::support::user("alice");
    let id = template_id("sales-review");

    let closed = library::close(&db.pool, &id, &alice, Utc::now()).await;
    assert!(matches!(closed, Err(LibraryError::NotAdmin)), "{closed:?}");
    let retired = library::retire(&db.pool, &id, RetireMode::Remove, &alice, Utc::now()).await;
    assert!(
        matches!(retired, Err(LibraryError::NotAdmin)),
        "{retired:?}"
    );

    library::retire(&db.pool, &id, RetireMode::Unlink, &admin, Utc::now())
        .await
        .expect("retire");
    let again = library::retire(&db.pool, &id, RetireMode::Remove, &admin, Utc::now()).await;
    assert!(matches!(again, Err(LibraryError::Missing)), "{again:?}");
    let closed = library::close(&db.pool, &id, &admin, Utc::now()).await;
    assert!(matches!(closed, Err(LibraryError::Missing)), "{closed:?}");
    let unknown = library::close(&db.pool, &template_id("nowhere"), &admin, Utc::now()).await;
    assert!(matches!(unknown, Err(LibraryError::Missing)), "{unknown:?}");
}
