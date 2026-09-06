//! This suite's throwaway database, plus the queries its assertions read back.
//!
//! Unlike the other suites this one does not want the full extension schema:
//! the audit writers touch two tables, and a hand-written pair is both faster
//! and a tighter statement of what the code under test depends on. The
//! database itself, and the `CI` guard that stops a missing Postgres from
//! reading as a pass, come from `astound-test-common`.

use std::sync::Arc;

use sqlx::{AssertSqlSafe, PgPool};
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

    // The knowledge schema is the crate's own file, applied through the same
    // function the binary uses at boot, so a drift between the tests and the
    // shipped schema is impossible by construction.
    pub async fn create_knowledge_schema(&self) {
        systemprompt_mcp_knowledge_bank::schema::apply(&*self.inner.pool)
            .await
            .expect("pgvector must be installed on the test server");
    }

    // Every (subject_kind, subject_value) grant on a document, sorted.
    pub async fn grants_of(&self, doc_id: &str) -> Vec<(String, String)> {
        let mut rows = sqlx::query_as::<_, (String, String)>(
            "SELECT subject_kind, subject_value FROM knowledge_document_grants WHERE doc_id = $1",
        )
        .bind(doc_id)
        .fetch_all(&*self.inner.pool)
        .await
        .expect("read grants");
        rows.sort();
        rows
    }

    pub async fn count(&self, table: &str) -> i64 {
        sqlx::query_scalar::<_, i64>(AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
            .fetch_one(&*self.inner.pool)
            .await
            .expect("count rows")
    }

    // Backdate a user's sync state so a prune sees them as stale.
    pub async fn age_sync_state(&self, user_id: &str, days: i32) {
        sqlx::query(
            "UPDATE knowledge_user_sync_state
             SET last_run_at = NOW() - make_interval(days => $2)
             WHERE user_id = $1",
        )
        .bind(user_id)
        .bind(days)
        .execute(&*self.inner.pool)
        .await
        .expect("age sync state");
    }

    pub async fn insert_document(&self, doc_id: &str, source_type: &str) {
        sqlx::query(
            r"INSERT INTO knowledge_documents
                (doc_id, source_type, source, source_ref, title, uri, content_hash)
              VALUES ($1, $2, 'test', $1, $1, 'https://example.invalid/' || $1, 'hash')",
        )
        .bind(doc_id)
        .bind(source_type)
        .execute(&*self.inner.pool)
        .await
        .expect("seed document");
    }

    pub async fn grant(&self, doc_id: &str, kind: &str, value: &str) {
        sqlx::query(
            "INSERT INTO knowledge_document_grants (doc_id, subject_kind, subject_value) \
             VALUES ($1, $2, $3)",
        )
        .bind(doc_id)
        .bind(kind)
        .bind(value)
        .execute(&*self.inner.pool)
        .await
        .expect("seed grant");
    }

    pub async fn revoke(&self, doc_id: &str, kind: &str, value: &str) {
        sqlx::query(
            "DELETE FROM knowledge_document_grants \
             WHERE doc_id = $1 AND subject_kind = $2 AND subject_value = $3",
        )
        .bind(doc_id)
        .bind(kind)
        .bind(value)
        .execute(&*self.inner.pool)
        .await
        .expect("revoke grant");
    }

    pub fn raw_pool_arc(&self) -> Arc<PgPool> {
        Arc::clone(&self.inner.pool)
    }
}
