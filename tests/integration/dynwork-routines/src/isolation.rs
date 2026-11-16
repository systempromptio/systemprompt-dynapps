//! One person's routines and runs are invisible to, and untouchable by,
//! another; a token that is not a desktop's is refused.

use axum::http::{Method, StatusCode};
use dynapps_test_common::db_or_skip;
use serde_json::json;

use crate::support::{Call, create, drain_feed, router, seed_user};

#[tokio::test]
async fn another_user_neither_sees_nor_writes_the_routine() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    seed_user(&db.pool, "bob").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;

    let (bobs, _) = drain_feed(&router, "bob", "0", 50).await;
    assert!(bobs.is_empty());

    let (update, _) = Call::new(Method::PUT, "/routines/daily", "bob")
        .if_match(1)
        .body(json!({ "definition": { "name": "hijack", "enabled": false } }))
        .send(&router)
        .await;
    assert_eq!(update, StatusCode::NOT_FOUND);
    let (delete, _) = Call::new(Method::DELETE, "/routines/daily", "bob")
        .if_match(1)
        .send(&router)
        .await;
    assert_eq!(delete, StatusCode::NOT_FOUND);
    let (run, _) = Call::new(Method::PUT, "/runs/run-1", "bob")
        .body(json!({ "routine_id": "daily", "device": "mac-1", "state": "running" }))
        .send(&router)
        .await;
    assert_eq!(run, StatusCode::NOT_FOUND);

    let bobs_own = create(&router, "bob", "daily", "pc-9").await;
    assert_eq!(bobs_own["revision"], 1, "ids are scoped per user");
    let (alices, _) = drain_feed(&router, "alice", "0", 50).await;
    assert_eq!(alices.len(), 1);
    assert_eq!(alices[0]["owner_device"], "mac-1");
}

#[tokio::test]
async fn a_request_without_a_desktop_token_is_unauthorized() {
    let db = db_or_skip!();
    let router = router(&db.pool);
    let response = tower::ServiceExt::oneshot(
        router,
        axum::http::Request::builder()
            .uri("/changes")
            .body(axum::body::Body::empty())
            .expect("a request"),
    )
    .await
    .expect("the router answers");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
