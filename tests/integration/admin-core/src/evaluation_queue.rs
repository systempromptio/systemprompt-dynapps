//! `repositories::evaluation::queue` — the console's way in: queue one
//! session on the spot and lease only its rows.
//!
//! The scheduler's `insert_closed_sessions` only ever adds rows, so a session
//! the queue already considers finished (scored, or dead-lettered after its
//! retries) would be silently ignored by a click. `insert_session_now` must
//! revive such a row, and `claim_session_leases` must never pick up a
//! neighbour's work while it does.

use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::config::evaluation::CriterionName;
use systemprompt_web_admin::repositories::evaluation::queue::{
    claim_session_leases, insert_session_now,
};

use crate::fixtures::{SummarySpec, insert_summary, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

struct QueueRow {
    pending: bool,
    attempts: i32,
    closed_reason: String,
    last_error: Option<String>,
}

async fn read_row(pool: &sqlx::PgPool, session_id: &str, criterion: &str) -> QueueRow {
    let row: (bool, i32, String, Option<String>) = sqlx::query_as(
        "SELECT pending, attempts, closed_reason, last_error
           FROM evaluation_queue WHERE session_id = $1 AND criterion = $2",
    )
    .bind(session_id)
    .bind(criterion)
    .fetch_one(pool)
    .await
    .expect("read queue row");
    QueueRow {
        pending: row.0,
        attempts: row.1,
        closed_reason: row.2,
        last_error: row.3,
    }
}

async fn seeded_session(pool: &sqlx::PgPool, label: &str) -> String {
    let user = insert_user(pool, &unique("user"), &unclaimed_email(label)).await;
    let session = unique("session");
    insert_summary(pool, &SummarySpec::open(&session, &user)).await;
    session
}

#[tokio::test]
async fn insert_session_now_queues_every_criterion_as_manual() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "manual-all").await;

    let queued = insert_session_now(&db.pool, &SessionId::new(&session), &CriterionName::ALL)
        .await
        .expect("queue the session");

    assert_eq!(queued, 4);
    for criterion in CriterionName::ALL {
        let row = read_row(&db.pool, &session, criterion.as_str()).await;
        assert!(row.pending);
        assert_eq!(row.closed_reason, "manual");
    }
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_now_revives_a_finished_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = seeded_session(&db.pool, "revive").await;
    insert_session_now(
        &db.pool,
        &SessionId::new(&session),
        &[CriterionName::Quality],
    )
    .await
    .expect("queue the session");
    sqlx::query(
        "UPDATE evaluation_queue
            SET pending = false, attempts = 5, last_error = 'gave up', closed_reason = 'session_end'
          WHERE session_id = $1",
    )
    .bind(&session)
    .execute(&*db.pool)
    .await
    .expect("dead-letter the row");

    insert_session_now(
        &db.pool,
        &SessionId::new(&session),
        &[CriterionName::Quality],
    )
    .await
    .expect("revive the row");

    let row = read_row(&db.pool, &session, "quality").await;
    assert!(row.pending);
    assert_eq!(row.attempts, 0);
    assert_eq!(row.closed_reason, "manual");
    assert!(row.last_error.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn insert_session_now_refuses_a_session_without_a_summary() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let missing = SessionId::new(unique("never-seen"));

    let err = insert_session_now(&db.pool, &missing, &CriterionName::ALL)
        .await
        .expect_err("no summary row");

    assert!(matches!(err, sqlx::Error::RowNotFound), "{err}");
    db.cleanup().await;
}

#[tokio::test]
async fn claim_session_leases_takes_only_that_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = seeded_session(&db.pool, "mine").await;
    let other = seeded_session(&db.pool, "other").await;
    for session in [&mine, &other] {
        insert_session_now(&db.pool, &SessionId::new(session), &CriterionName::ALL)
            .await
            .expect("queue");
    }

    let leases = claim_session_leases(&db.pool, &SessionId::new(&mine), 60)
        .await
        .expect("lease the session");

    assert_eq!(leases.len(), 4);
    assert!(leases.iter().all(|l| l.session_id.as_str() == mine));
    assert!(leases.iter().all(|l| l.attempts == 1));
    let untouched = read_row(&db.pool, &other, "intent").await;
    assert_eq!(untouched.attempts, 0);

    let again = claim_session_leases(&db.pool, &SessionId::new(&mine), 60)
        .await
        .expect("second claim");
    assert!(again.is_empty(), "a live lease is not re-claimed");
    db.cleanup().await;
}
