//! Throwaway-database harness. Each test provisions its own database on the
//! server named by `DATABASE_URL`, applies the minimal schema the audit writers
//! touch (`users` + `user_activity`), and drops it on teardown — the shared
//! application tables are never involved.

use std::sync::Arc;

use sqlx::{AssertSqlSafe, PgPool};
use systemprompt::database::{Database, DbPool};
use url::Url;

pub struct TempDb {
    pub pool: DbPool,
    pg: PgPool,
    admin_url: String,
    db_name: String,
}

// Resolve the maintenance-server URL for tests, or `None` (test self-skips).
//
// Refuses to run against a development database: only database names `test`,
// `postgres`, or `*_test` are accepted, so a stray `DATABASE_URL` pointing at
// a dev server's live database panics instead of mutating it. Tests only ever
// CREATE/DROP throwaway `mcp_ext_test_<uuid>` databases on the server.
fn database_url() -> Option<String> {
    let raw = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()?;
    let parsed = Url::parse(&raw).expect("test database URL must be a valid URL");
    let db_name = parsed.path().trim_start_matches('/');
    let allowed = db_name == "test" || db_name == "postgres" || db_name.ends_with("_test");
    assert!(
        allowed,
        "Refusing to run integration tests against database '{db_name}'. Set \
         SYSTEMPROMPT_TEST_DATABASE_URL to a database reserved for tests: the database name \
         must be 'test', 'postgres', or end in '_test'."
    );
    Some(raw)
}

// Rebuild a connection URL against the same server but a different database.
fn with_database(base: &str, db_name: &str) -> String {
    let mut url = Url::parse(base).expect("DATABASE_URL is a valid URL");
    url.set_path(&format!("/{db_name}"));
    url.into()
}

impl TempDb {
    pub async fn create() -> Option<Self> {
        let base = database_url()?;
        // Maintenance connection lives on the `postgres` database; CREATE
        // DATABASE cannot run inside a transaction, so use a plain autocommit
        // execute here.
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("mcp_ext_test_{}", uuid::Uuid::new_v4().simple());

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        // Name is a UUID-derived literal, not user input — safe to interpolate.
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{db_name}\"")))
            .execute(&admin)
            .await
            .expect("create throwaway database");
        admin.close().await;

        let child_url = with_database(&base, &db_name);
        let pg = PgPool::connect(&child_url)
            .await
            .expect("connect to throwaway database");

        // Minimal schema: only the columns the audit writers read/write.
        sqlx::query(
            r"CREATE TABLE users (
                id         TEXT PRIMARY KEY,
                email      TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(&pg)
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
        .execute(&pg)
        .await
        .expect("create user_activity table");

        let pool: DbPool = Arc::new(Database::from_pools(Arc::new(pg.clone()), None));

        Some(Self {
            pool,
            pg,
            admin_url,
            db_name,
        })
    }

    // Insert a user row with the given id/email. Used to seed (or omit) the
    // reserved anonymous principal.
    pub async fn insert_user(&self, id: &str, email: &str) {
        sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2)")
            .bind(id)
            .bind(email)
            .execute(&self.pg)
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
        .fetch_all(&self.pg)
        .await
        .expect("query recorded activity")
    }

    // The pgvector-backed knowledge schema, plus seed/grant helpers. Kept
    // here rather than reading extensions/web/schema/19_knowledge_bank.sql so
    // the throwaway database carries only what these tests touch.
    pub async fn create_knowledge_schema(&self) {
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&self.pg)
            .await
            .expect("pgvector must be installed on the test server");

        sqlx::query(
            r"CREATE TABLE knowledge_documents (
                doc_id      TEXT PRIMARY KEY,
                source_type TEXT NOT NULL,
                source      TEXT NOT NULL,
                title       TEXT NOT NULL,
                uri         TEXT NOT NULL,
                doc_date    TIMESTAMPTZ,
                date_source TEXT,
                content_hash TEXT NOT NULL,
                mtime       TIMESTAMPTZ,
                indexed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                archived    BOOLEAN NOT NULL DEFAULT FALSE
            )",
        )
        .execute(&self.pg)
        .await
        .expect("create knowledge_documents");

        sqlx::query(
            r"CREATE TABLE knowledge_chunks (
                chunk_id    TEXT PRIMARY KEY,
                doc_id      TEXT NOT NULL REFERENCES knowledge_documents(doc_id) ON DELETE CASCADE,
                text        TEXT NOT NULL,
                embedding   vector(1024) NOT NULL,
                lines       TEXT,
                chunk_index INTEGER NOT NULL,
                chunk_total INTEGER NOT NULL,
                tsv         tsvector GENERATED ALWAYS AS (to_tsvector('english', text)) STORED
            )",
        )
        .execute(&self.pg)
        .await
        .expect("create knowledge_chunks");

        sqlx::query(
            r"CREATE TABLE knowledge_document_grants (
                doc_id        TEXT NOT NULL REFERENCES knowledge_documents(doc_id) ON DELETE CASCADE,
                subject_kind  TEXT NOT NULL,
                subject_value TEXT NOT NULL,
                PRIMARY KEY (doc_id, subject_kind, subject_value)
            )",
        )
        .execute(&self.pg)
        .await
        .expect("create knowledge_document_grants");

        sqlx::query(
            r"CREATE TABLE knowledge_index_state (
                key        TEXT PRIMARY KEY,
                value      TEXT NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(&self.pg)
        .await
        .expect("create knowledge_index_state");
    }

    pub async fn insert_document(&self, doc_id: &str, source_type: &str) {
        sqlx::query(
            r"INSERT INTO knowledge_documents
                (doc_id, source_type, source, title, uri, content_hash)
              VALUES ($1, $2, 'test', $1, 'https://example.invalid/' || $1, 'hash')",
        )
        .bind(doc_id)
        .bind(source_type)
        .execute(&self.pg)
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
        .execute(&self.pg)
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
        .execute(&self.pg)
        .await
        .expect("revoke grant");
    }

    pub fn raw_pool_arc(&self) -> Arc<PgPool> {
        Arc::new(self.pg.clone())
    }

    pub async fn cleanup(self) {
        self.pg.close().await;
        let admin = PgPool::connect(&self.admin_url)
            .await
            .expect("reconnect maintenance database for drop");
        // Force-drop even if a stray connection lingers.
        sqlx::query(AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.db_name
        )))
        .execute(&admin)
        .await
        .expect("drop throwaway database");
        admin.close().await;
    }
}
