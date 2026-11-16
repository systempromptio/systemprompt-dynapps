//! The change feed: complete across pages, even while writers race, and
//! carrying tombstones.

use std::collections::HashMap;

use axum::http::{Method, StatusCode};
use dynapps_test_common::db_or_skip;
use serde_json::json;

use crate::support::{Call, create, definition, drain_feed, router, seed_user};

#[tokio::test]
async fn paging_from_a_cursor_misses_nothing_written_between_pages() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    for n in 0..5 {
        create(&router, "alice", &format!("r{n}"), "mac-1").await;
    }
    let (first, cursor) = drain_feed(&router, "alice", "0", 2).await;
    assert_eq!(first.len(), 5);

    create(&router, "alice", "r5", "mac-1").await;
    let (status, _) = Call::new(Method::PUT, "/routines/r0", "alice")
        .if_match(1)
        .body(json!({ "definition": definition("r0 edited") }))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::OK);

    let (second, _) = drain_feed(&router, "alice", &cursor, 1).await;
    let ids: Vec<&str> = second
        .iter()
        .map(|r| r["id"].as_str().expect("an id"))
        .collect();
    assert_eq!(ids, vec!["r5", "r0"]);
    assert_eq!(second[1]["revision"], 2);
}

#[tokio::test]
async fn a_reader_paging_while_writers_race_ends_with_every_latest_revision() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    let writers: Vec<_> = (0..12)
        .map(|n| {
            let router = router.clone();
            tokio::spawn(async move {
                let id = format!("w{n}");
                create(&router, "alice", &id, "mac-1").await;
                for revision in 1..3 {
                    let (status, _) = Call::new(Method::PUT, format!("/routines/{id}"), "alice")
                        .if_match(revision)
                        .body(json!({ "definition": definition(&format!("{id} v{revision}")) }))
                        .send(&router)
                        .await;
                    assert_eq!(status, StatusCode::OK);
                }
            })
        })
        .collect();

    let mut latest: HashMap<String, i64> = HashMap::new();
    let mut cursor = "0".to_owned();
    let mut finished = false;
    while !finished {
        finished = writers.iter().all(tokio::task::JoinHandle::is_finished);
        let (changes, next) = drain_feed(&router, "alice", &cursor, 3).await;
        for change in changes {
            let id = change["id"].as_str().expect("an id").to_owned();
            let revision = change["revision"].as_i64().expect("a revision");
            let previous = latest.insert(id, revision).unwrap_or(0);
            assert!(revision > previous, "the feed went backwards");
        }
        cursor = next;
        tokio::task::yield_now().await;
    }
    for writer in writers {
        writer.await.expect("a writer");
    }
    let (tail, _) = drain_feed(&router, "alice", &cursor, 3).await;
    for change in tail {
        latest.insert(
            change["id"].as_str().expect("an id").to_owned(),
            change["revision"].as_i64().expect("a revision"),
        );
    }
    assert_eq!(latest.len(), 12);
    assert!(latest.values().all(|revision| *revision == 3), "{latest:?}");
}

#[tokio::test]
async fn a_delete_leaves_a_tombstone_in_the_feed() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    let (_, cursor) = drain_feed(&router, "alice", "0", 50).await;

    let (status, tombstone) = Call::new(Method::DELETE, "/routines/daily", "alice")
        .if_match(1)
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tombstone["deleted"], true);

    let (changes, _) = drain_feed(&router, "alice", &cursor, 50).await;
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["id"], "daily");
    assert_eq!(changes[0]["deleted"], true);
    assert_eq!(changes[0]["revision"], 2);
    assert!(
        changes[0]["definition"].is_null(),
        "a tombstone carries no definition"
    );

    let (recreate, _) = Call::new(Method::PUT, "/routines/daily", "alice")
        .if_match(2)
        .body(json!({ "definition": definition("back") }))
        .send(&router)
        .await;
    assert_eq!(
        recreate,
        StatusCode::CONFLICT,
        "a tombstone is not updated back to life"
    );
}

#[tokio::test]
async fn a_bad_cursor_is_a_bad_request() {
    let db = db_or_skip!();
    seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    let (status, _) = Call::new(Method::GET, "/changes?since=yesterday", "alice")
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
