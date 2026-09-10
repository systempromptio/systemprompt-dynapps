//! This suite's throwaway database, plus the queries its assertions read back.
//!
//! Unlike the other suites this one does not want the full extension schema:
//! the audit writers touch two tables, and a hand-written pair is both faster
//! and a tighter statement of what the code under test depends on. The
//! database itself, and the `CI` guard that stops a missing Postgres from
//! reading as a pass, come from `astound-test-common`.

use systemprompt::database::DbPool;

pub struct TempDb {
    inner: astound_test_common::TempDb,
    pub pool: DbPool,
}

impl TempDb {
    pub async fn create() -> Option<Self> {
        let inner = astound_test_common::TempDb::create_empty().await?;
        let pg = &*inner.pool;

        // Minimal schema: only the columns the audit writers read/write.
        sqlx::query(
            r"CREATE TABLE users (
                id         TEXT PRIMARY KEY,
                email      TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(pg)
        .await
        .expect("create users table");

        sqlx::query(
            r"CREATE TABLE user_activity (
                id          TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
                user_id     TEXT NOT NULL,
                category    TEXT NOT NULL,
                action      TEXT NOT NULL,
                entity_type TEXT,
                entity_id   TEXT,
                entity_name TEXT,
                description TEXT NOT NULL,
                metadata    JSONB DEFAULT '{}'::jsonb,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(pg)
        .await
        .expect("create user_activity table");

        let pool = inner.db_pool();
        Some(Self { inner, pool })
    }

    pub async fn cleanup(self) {
        self.inner.cleanup().await;
    }

    // Insert a user row with the given id/email. Used to seed (or omit) the
    // reserved anonymous principal.
    pub async fn insert_user(&self, id: &str, email: &str) {
        sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2)")
            .bind(id)
            .bind(email)
            .execute(&*self.inner.pool)
            .await
            .expect("seed user");
    }

    // All `mcp_access` rows recorded for a given `entity_name` (server or
    // tool), returned as (user_id, action, entity_type, description).
    pub async fn mcp_rows(
        &self,
        entity_name: &str,
    ) -> Vec<(String, String, Option<String>, String)> {
        sqlx::query_as::<_, (String, String, Option<String>, String)>(
            r"SELECT user_id, action, entity_type, description
              FROM user_activity
              WHERE category = 'mcp_access' AND entity_name = $1",
        )
        .bind(entity_name)
        .fetch_all(&*self.inner.pool)
        .await
        .expect("query recorded activity")
    }
}
