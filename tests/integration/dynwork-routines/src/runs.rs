//! Run summaries: only the routine's owner device writes them.

use axum::http::{Method, StatusCode};
use dynapps_test_common::db_or_skip;
use serde_json::json;

use crate::support::{Call, create, router, seed_user};

#[tokio::test]
async fn only_the_owner_device_reports_a_run() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    let run = |device: &str, state: &str| {
        json!({
            "routine_id": "daily",
            "device": device,
            "state": state,
            "started_at": "2026-11-16T09:00:00Z",
        })
    };

    let (foreign, body) = Call::new(Method::PUT, "/runs/run-1", "alice")
        .body(run("pc-2", "running"))
        .send(&router)
        .await;
    assert_eq!(foreign, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "not_owner_device");

    let (started, body) = Call::new(Method::PUT, "/runs/run-1", "alice")
        .body(run("mac-1", "running"))
        .send(&router)
        .await;
    assert_eq!(started, StatusCode::OK, "{body}");
    let (finished, body) = Call::new(Method::PUT, "/runs/run-1", "alice")
        .body(json!({
            "routine_id": "daily",
            "device": "mac-1",
            "state": "failed",
            "reason": "time limit",
            "started_at": "2026-11-16T09:00:00Z",
            "ended_at": "2026-11-16T09:30:00Z",
        }))
        .send(&router)
        .await;
    assert_eq!(finished, StatusCode::OK, "{body}");
    assert_eq!(body["state"], "failed");
    assert_eq!(body["reason"], "time limit");
}

#[tokio::test]
async fn a_run_id_stays_with_its_routine_and_states_are_checked() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    create(&router, "alice", "weekly", "mac-1").await;
    let run = |routine: &str, state: &str| json!({ "routine_id": routine, "device": "mac-1", "state": state });

    let (first, _) = Call::new(Method::PUT, "/runs/run-1", "alice")
        .body(run("daily", "ok"))
        .send(&router)
        .await;
    assert_eq!(first, StatusCode::OK);
    let (moved, body) = Call::new(Method::PUT, "/runs/run-1", "alice")
        .body(run("weekly", "ok"))
        .send(&router)
        .await;
    assert_eq!(moved, StatusCode::CONFLICT);
    assert_eq!(body["error"], "run_conflict");
    let (unknown, _) = Call::new(Method::PUT, "/runs/run-2", "alice")
        .body(run("daily", "exploded"))
        .send(&router)
        .await;
    assert_eq!(unknown, StatusCode::BAD_REQUEST);
}
