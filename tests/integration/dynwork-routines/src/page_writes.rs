//! The writes the "My routines" page makes: pause and resume set
//! `definition.enabled` as a new revision, delete leaves a tombstone without
//! `If-Match`, and all of them reach the feed. Also what the page lists.

use chrono::Utc;
use dynapps_test_common::db_or_skip;
use dynwork_routines::ids::{RoutineId, RunId, RunState};
use dynwork_routines::store::{self, DeleteRoutine, PutRun, RunOutcome, WriteOutcome};

use crate::support::{create, drain_feed, router, seed_user};

fn id(text: &str) -> RoutineId {
    RoutineId::parse(text).expect("a routine id")
}

#[tokio::test]
async fn pause_resume_and_delete_write_new_revisions_into_the_feed() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    let (_, cursor) = drain_feed(&router, "alice", "0", 50).await;

    let WriteOutcome::Written(paused) =
        store::set_enabled(&db.pool, &alice, &id("daily"), false, Utc::now())
            .await
            .expect("pause")
    else {
        panic!("pause did not write");
    };
    assert_eq!(paused.revision, 2);
    assert_eq!(paused.definition["enabled"], false);
    assert_eq!(
        paused.definition["name"], "daily",
        "the rest of the definition stays"
    );

    let resumed = store::set_enabled(&db.pool, &alice, &id("daily"), true, Utc::now())
        .await
        .expect("resume");
    assert!(matches!(resumed, WriteOutcome::Written(ref r) if r.definition["enabled"] == true));

    let deleted = store::delete_routine(
        &db.pool,
        &DeleteRoutine {
            user: &alice,
            id: &id("daily"),
            expected: None,
            mandatory: &[],
            now: Utc::now(),
        },
    )
    .await
    .expect("delete");
    assert!(matches!(deleted, WriteOutcome::Written(ref r) if r.deleted_at.is_some()));
    let again = store::set_enabled(&db.pool, &alice, &id("daily"), true, Utc::now())
        .await
        .expect("resume a tombstone");
    assert_eq!(again, WriteOutcome::NotFound);

    let (changes, _) = drain_feed(&router, "alice", &cursor, 50).await;
    assert_eq!(
        changes.len(),
        1,
        "the feed carries the latest revision only"
    );
    assert_eq!(changes[0]["revision"], 4);
    assert_eq!(changes[0]["deleted"], true);
    assert!(
        store::list_live_routines(&db.pool, &alice)
            .await
            .expect("list")
            .is_empty()
    );
}

#[tokio::test]
async fn the_page_lists_live_routines_and_at_most_the_recent_runs() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let router = router(&db.pool);
    create(&router, "alice", "daily", "mac-1").await;
    for n in 0..7 {
        let run_id = RunId::parse(&format!("run-{n}")).expect("a run id");
        let outcome = store::put_run(
            &db.pool,
            &PutRun {
                user: &alice,
                id: &run_id,
                routine_id: &id("daily"),
                device: "mac-1",
                state: RunState::Ok,
                reason: None,
                started_at: Some(Utc::now() + chrono::Duration::minutes(n)),
                ended_at: None,
                now: Utc::now(),
            },
        )
        .await
        .expect("a run");
        assert!(matches!(outcome, RunOutcome::Written(_)));
    }
    let live = store::list_live_routines(&db.pool, &alice)
        .await
        .expect("list");
    assert_eq!(live.len(), 1);
    let runs = store::list_recent_runs(&db.pool, &alice, 5)
        .await
        .expect("runs");
    let ids: Vec<&str> = runs.iter().map(|run| run.id.as_str()).collect();
    assert_eq!(ids, vec!["run-6", "run-5", "run-4", "run-3", "run-2"]);
}
