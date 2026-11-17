//! The two stores and the users they are tested with.

use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use dynapps_test_common::TempDb;
use dynwork_chats::store::ChatStore;
use dynwork_chats::store::memory::MemoryChatStore;
use dynwork_chats::store::postgres::PgChatStore;
use systemprompt::identifiers::UserId;

use dynwork_chats as _;

pub(crate) struct Fixture {
    pub(crate) store: Arc<dyn ChatStore>,
    pub(crate) alice: UserId,
    pub(crate) bob: UserId,
    // Keeps the throwaway database alive for the test; `finish` drops it.
    db: Option<TempDb>,
}

impl Fixture {
    // The throwaway databases share one Postgres with little disk, so each
    // test drops its own instead of leaving it for the next run.
    pub(crate) async fn finish(self) {
        if let Some(db) = self.db {
            db.cleanup().await;
        }
    }
}

pub(crate) fn user(name: &str) -> UserId {
    UserId::new(name.to_owned())
}

// A fixed point in time, `seconds` after the instant the suites share.
pub(crate) fn at(seconds: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 17, 9, 0, 0)
        .single()
        .expect("a valid instant")
        + Duration::seconds(seconds)
}

pub(crate) fn memory() -> Fixture {
    Fixture {
        store: Arc::new(MemoryChatStore::new()),
        alice: user("alice"),
        bob: user("bob"),
        db: None,
    }
}

pub(crate) async fn seed_users(db: &TempDb) {
    for name in ["alice", "bob"] {
        sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $2)")
            .bind(name)
            .bind(format!("{name}@example.test"))
            .execute(&*db.pool)
            .await
            .expect("seed a user");
    }
}

pub(crate) async fn postgres(db: TempDb) -> Fixture {
    seed_users(&db).await;
    Fixture {
        store: Arc::new(PgChatStore::new(Arc::clone(&db.pool))),
        alice: user("alice"),
        bob: user("bob"),
        db: Some(db),
    }
}
