//! Who has a task of a template, and how its runs went: one activation per
//! person (the live task, or the latest tombstone), its recent runs newest
//! first, and each run's recorded conversation, found from the person and
//! `uuidv5(NS, run id)` in core's session contexts (contract §7). A run core
//! has no conversation for, or one recorded under another person, has none.

use axum::http::{Method, StatusCode};
use chrono::{Duration, Utc};
use dynapps_test_common::db_or_skip;
use dynwork_routines::ids::{RoutineId, RunId, RunState, TemplateId};
use dynwork_routines::store::{self, PutRun, RunOutcome, WriteOutcome};

use crate::support::{Call, router, seed_user};
use crate::template_support::{activate, linked, publish, rule, seed_admin};

async fn run(pool: &sqlx::PgPool, who: &str, id: &str, state: RunState, hours_ago: i64) {
    let written = store::put_run(
        pool,
        &PutRun {
            user: &crate::support::user(who),
            id: &RunId::parse(id).expect("a run id"),
            routine_id: &RoutineId::parse("task-1").expect("a routine id"),
            device: "mac-1",
            state,
            reason: None,
            started_at: Some(Utc::now() - Duration::hours(hours_ago)),
            ended_at: None,
            now: Utc::now(),
        },
    )
    .await
    .expect("a run");
    assert!(matches!(written, RunOutcome::Written(_)), "{who} {id}");
}

async fn conversation(pool: &sqlx::PgPool, who: &str, client_session: &str, context: &str) {
    sqlx::query(
        "INSERT INTO conversation_session_contexts (user_id, client_session_id, context_id)
         VALUES ($1, $2, $3)",
    )
    .bind(who)
    .bind(client_session)
    .bind(context)
    .execute(pool)
    .await
    .expect("record a conversation");
}

#[tokio::test]
async fn activations_name_each_persons_task_its_runs_and_their_conversations() {
    let db = db_or_skip!();
    let desktop = router(&db.pool);
    let admin = seed_admin(&db.pool, "admin").await;
    for who in ["alice", "bob", "carol", "dave"] {
        seed_user(&db.pool, who).await;
    }
    rule(&db.pool, "sales", "role", "user", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    for who in ["alice", "bob", "carol"] {
        let (status, task) = activate(
            &desktop,
            who,
            "task-1",
            linked("Sales review", "sales-review", 1),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{who}: {task}");
    }
    let (status, _) = Call::new(Method::DELETE, "/routines/task-1", "bob")
        .if_match(1)
        .send(&desktop)
        .await;
    assert_eq!(status, StatusCode::OK, "bob deactivates his task");
    let paused = store::set_enabled(
        &db.pool,
        &crate::support::user("carol"),
        &RoutineId::parse("task-1").expect("a routine id"),
        false,
        Utc::now(),
    )
    .await
    .expect("pause");
    assert!(matches!(paused, WriteOutcome::Written(_)));
    run(&db.pool, "alice", "ses_alice_ok", RunState::Ok, 2).await;
    run(&db.pool, "alice", "ses_alice_failed", RunState::Failed, 1).await;
    run(&db.pool, "carol", "ses_carol_ok", RunState::Ok, 1).await;
    // uuidv5(7c1f5b6e-3a2d-4e8f-9b0c-2d6a1e4f8c73, "ses_alice_ok") and
    // (…, "ses_carol_ok"), as DynWork stamps them. Carol's is recorded under
    // dave, so it is not hers.
    let alice_ok = "7d0d2f8e-2a52-4c79-8a43-6b1f0f5e9c01";
    conversation(
        &db.pool,
        "alice",
        "46a82b9d-9d94-5358-8f48-48d894039cd7",
        alice_ok,
    )
    .await;
    conversation(
        &db.pool,
        "dave",
        "52f7dd96-64e2-5185-841b-df5fbc1a0f37",
        "a4c1e9b2-6f0d-4e35-b8a7-1c2d3e4f5a6b",
    )
    .await;

    let activations = store::list_activations(
        &db.pool,
        &TemplateId::parse("sales-review").expect("a template id"),
        5,
    )
    .await
    .expect("activations");

    let people: Vec<&str> = activations.iter().map(|a| a.user.as_str()).collect();
    assert_eq!(
        people,
        vec!["alice", "bob", "carol"],
        "dave has no task of it"
    );

    let alice = &activations[0];
    assert_eq!(alice.routine.id.as_str(), "task-1");
    assert_eq!(alice.routine.deleted_at, None);
    assert_eq!(alice.routine.template_revision, Some(1));
    assert_eq!(alice.routine.owner_device, "mac-1");
    let runs: Vec<(&str, Option<&str>)> = alice
        .runs
        .iter()
        .map(|run| (run.run.id.as_str(), run.conversation.as_deref()))
        .collect();
    assert_eq!(
        runs,
        vec![("ses_alice_failed", None), ("ses_alice_ok", Some(alice_ok))],
        "newest first; only the recorded run has a conversation"
    );

    let bob = &activations[1];
    assert!(
        bob.routine.deleted_at.is_some(),
        "deactivated, with its date"
    );
    assert!(bob.runs.is_empty());

    let carol = &activations[2];
    assert_eq!(carol.routine.definition["enabled"], false);
    assert_eq!(carol.runs.len(), 1);
    assert_eq!(carol.runs[0].conversation, None);
}

#[tokio::test]
async fn a_live_task_wins_over_an_earlier_tombstone_and_runs_are_capped() {
    let db = db_or_skip!();
    let desktop = router(&db.pool);
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "role", "user", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let (status, _) = activate(
        &desktop,
        "alice",
        "old-task",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = Call::new(Method::DELETE, "/routines/old-task", "alice")
        .if_match(1)
        .send(&desktop)
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, task) = activate(
        &desktop,
        "alice",
        "task-1",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{task}");
    for n in 0..4 {
        run(&db.pool, "alice", &format!("ses_{n}"), RunState::Ok, 10 - n).await;
    }

    let activations = store::list_activations(
        &db.pool,
        &TemplateId::parse("sales-review").expect("a template id"),
        2,
    )
    .await
    .expect("activations");
    assert_eq!(activations.len(), 1);
    assert_eq!(activations[0].routine.id.as_str(), "task-1");
    let runs: Vec<&str> = activations[0]
        .runs
        .iter()
        .map(|run| run.run.id.as_str())
        .collect();
    assert_eq!(runs, vec!["ses_3", "ses_2"]);
}
