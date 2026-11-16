//! Optimistic concurrency: `If-Match` names the revision a write replaces.

use axum::http::{Method, StatusCode};
use dynapps_test_common::db_or_skip;
use serde_json::json;

use crate::support::{Call, create, create_body, definition, router, seed_user};

#[tokio::test]
async fn a_create_starts_at_revision_one_and_an_update_moves_it_on() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    let created = create(&router, "alice", "daily", "mac-1").await;
    assert_eq!(created["revision"], 1);
    assert_eq!(created["owner_device"], "mac-1");

    let (status, updated) = Call::new(Method::PUT, "/routines/daily", "alice")
        .if_match(1)
        .body(json!({ "definition": definition("Daily digest") }))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["revision"], 2);
    assert_eq!(updated["definition"]["name"], "Daily digest");
    assert_eq!(
        updated["owner_device"], "mac-1",
        "an update without an owner keeps it"
    );
}

#[tokio::test]
async fn a_stale_if_match_answers_409_with_the_current_row() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    let edit = |name: &str| json!({ "definition": definition(name) });
    let (first, _) = Call::new(Method::PUT, "/routines/daily", "alice")
        .if_match(1)
        .body(edit("From the laptop"))
        .send(&router)
        .await;
    assert_eq!(first, StatusCode::OK);

    let (status, conflict) = Call::new(Method::PUT, "/routines/daily", "alice")
        .if_match(1)
        .body(edit("From the desktop"))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(conflict["error"], "revision_conflict");
    assert_eq!(conflict["current"]["revision"], 2);
    assert_eq!(conflict["current"]["definition"]["name"], "From the laptop");
}

#[tokio::test]
async fn creating_an_existing_routine_or_a_stale_delete_is_a_conflict() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;

    let (again, body) = Call::new(Method::PUT, "/routines/daily", "alice")
        .body(create_body("daily", "mac-2"))
        .send(&router)
        .await;
    assert_eq!(again, StatusCode::CONFLICT);
    assert_eq!(body["current"]["owner_device"], "mac-1");

    let (stale, _) = Call::new(Method::DELETE, "/routines/daily", "alice")
        .if_match(7)
        .send(&router)
        .await;
    assert_eq!(stale, StatusCode::CONFLICT);
    let (no_precondition, _) = Call::new(Method::DELETE, "/routines/daily", "alice")
        .send(&router)
        .await;
    assert_eq!(no_precondition, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn racing_writers_on_one_revision_leave_exactly_one_winner() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    let writers = (0..8).map(|n| {
        let router = router.clone();
        tokio::spawn(async move {
            Call::new(Method::PUT, "/routines/daily", "alice")
                .if_match(1)
                .body(json!({ "definition": definition(&format!("writer {n}")) }))
                .send(&router)
                .await
                .0
        })
    });
    let mut statuses = Vec::new();
    for writer in writers {
        statuses.push(writer.await.expect("a writer"));
    }
    assert_eq!(statuses.iter().filter(|s| **s == StatusCode::OK).count(), 1);
    assert_eq!(
        statuses
            .iter()
            .filter(|s| **s == StatusCode::CONFLICT)
            .count(),
        7
    );
}

#[tokio::test]
async fn an_update_of_a_routine_that_never_existed_is_not_found() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    let (status, _) = Call::new(Method::PUT, "/routines/ghost", "alice")
        .if_match(1)
        .body(json!({ "definition": definition("ghost") }))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (bad, _) = Call::new(Method::PUT, "/routines/new", "alice")
        .body(json!({ "definition": { "enabled": true } }))
        .send(&router)
        .await;
    assert_eq!(bad, StatusCode::BAD_REQUEST, "a definition needs a name");
}
