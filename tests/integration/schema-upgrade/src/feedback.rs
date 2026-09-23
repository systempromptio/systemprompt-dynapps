//! Stateful acceptance for marketplace004–011, analytics005–007, AI024 and
//! Astound059–070 on fresh and real release-upgrade paths. The release fixture
//! still carries the evaluation engine's tables; marketplace011 drops them on
//! the upgrade path and they never exist on the fresh one.

use astound_test_common::TempDb;
use sqlx::PgPool;
use systemprompt::database::install_extension_schemas;
use systemprompt::extension::ExtensionRegistry;

const LEGACY: &str = include_str!("feedback_legacy.sql");
const CURRENT: &str = include_str!("feedback_current.sql");

async fn execute(pool: &PgPool, sql: &str) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(pool)
        .await
        .expect("feedback fixture SQL");
}

async fn install(db: &TempDb) {
    let registry = ExtensionRegistry::discover().expect("linked extensions");
    install_extension_schemas(&registry, db.db_pool().write())
        .await
        .expect("feedback upgrade/install");
}

async fn scalar(pool: &PgPool, sql: &str) -> String {
    sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await
        .expect("fixture projection")
}

// These exact columns existed in the release fixture. New nullable receipt
// identity columns intentionally are not invented for historical evidence.
async fn legacy_evidence(pool: &PgPool) -> String {
    scalar(pool, "SELECT jsonb_build_array(
      (SELECT jsonb_build_array(digest,manifest,rationale) FROM managed_revisions WHERE id='feedback-revision'),
      (SELECT jsonb_build_array(review_id,generation,revision_id,bundle_digest) FROM managed_publications WHERE id='feedback-publication'),
      (SELECT jsonb_build_array(installed_manifest,client_evidence,bundle_digest) FROM managed_installation_receipts WHERE id='feedback-legacy-receipt'),
      (SELECT jsonb_build_array(input_tokens,output_tokens,tokens_used,cost_microdollars,completed_at) FROM ai_requests WHERE id='feedback-ai-completed')
    )::text").await
}

// Snapshot complete rows, including lease fences, timestamps, revisions and
// diagnostics, before an installer restart. Table names are fixture constants.
async fn retained_rows(pool: &PgPool) -> Vec<String> {
    let mut rows = Vec::new();
    for table in [
        "ai_requests",
        "managed_consumer_credentials",
        "managed_consumer_grants",
        "managed_installation_receipts",
        "managed_consumer_session_bindings",
        "managed_inventory_state",
        "managed_inventory_entries",
        "managed_inventory_membership",
        "managed_inventory_bindings",
        "managed_installation_coverage_state",
        "managed_installation_coverage",
    ] {
        let query = format!(
            "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb)::text FROM {table} t"
        );
        rows.push(format!("{table}:{}", scalar(pool, &query).await));
    }
    rows
}

async fn rejects(pool: &PgPool, sql: &str, code: &str) {
    let error = sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(pool)
        .await
        .expect_err("database must reject invalid evidence");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some(code),
        "{sql}: {error}"
    );
}

async fn contracts(pool: &PgPool) {
    assert_eq!(scalar(pool,"SELECT (status='failed' AND accounting_failed_at='2020-01-02T00:00:00Z'::timestamptz AND accounting_error='fixture accounting mismatch' AND input_tokens=11 AND output_tokens=7 AND tokens_used=18 AND cost_microdollars=500 AND completed_at='2020-01-01T00:00:00Z'::timestamptz)::text FROM ai_requests WHERE id='feedback-ai-completed'").await,"true");
    assert_eq!(scalar(pool, "SELECT (consumer_id IS NULL AND device_id IS NULL AND host IS NULL AND fully_verified IS NULL)::text FROM managed_installation_receipts WHERE id='feedback-legacy-receipt'").await, "true");
    for statement in [
        "UPDATE managed_revisions SET rationale='rewritten' WHERE id='feedback-revision'",
        "UPDATE managed_consumer_session_bindings SET native_session_id='forged' WHERE id='feedback-binding'",
        "UPDATE managed_inventory_bindings SET bound_by='feedback-consumer' WHERE entry_id='feedback-entry'",
    ] {
        rejects(pool, statement, "23514").await;
    }
    rejects(pool, "INSERT INTO managed_consumer_grants(owner_id,resource_id,consumer_id) VALUES('feedback-consumer','feedback-resource','feedback-owner')", "23503").await;
    rejects(pool, "INSERT INTO managed_inventory_membership(owner_id,entry_id,effective_from,record) VALUES('feedback-owner','feedback-entry',now(),'{}')", "23505").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn seeded_feedback_survives_release_upgrade_and_installer_restart() {
    let upgraded = TempDb::create_empty()
        .await
        .expect("database required for stateful release upgrade");
    execute(&upgraded.pool, &super::upgrade::fixture_sql()).await;
    execute(&upgraded.pool, LEGACY).await;
    let before = legacy_evidence(&upgraded.pool).await;
    install(&upgraded).await;
    assert_eq!(
        scalar(
            &upgraded.pool,
            "SELECT (count(*)=0)::text FROM pg_tables WHERE tablename LIKE 'eval\\_%'"
        )
        .await,
        "true",
        "marketplace011 drops every evaluation table on the upgrade path"
    );
    assert_eq!(
        scalar(
            &upgraded.pool,
            "SELECT (count(*)=0)::text FROM extension_migrations WHERE extension_id='evaluation'"
        )
        .await,
        "true",
        "the evaluation extension leaves no migration rows behind"
    );
    assert_eq!(
        before,
        legacy_evidence(&upgraded.pool).await,
        "release evidence must survive migrations"
    );
    let fresh = TempDb::create()
        .await
        .expect("database required for fresh stateful schema");
    execute(&fresh.pool, LEGACY).await;
    assert_eq!(
        before,
        legacy_evidence(&fresh.pool).await,
        "fresh and upgraded retained evidence parity"
    );
    for db in [&upgraded, &fresh] {
        assert_eq!(scalar(&db.pool,"SELECT (accounting_failed_at IS NULL AND accounting_error IS NULL AND status='completed')::text FROM ai_requests WHERE id='feedback-ai-completed'").await,"true","upgrade must not invent historical accounting failure");
        execute(&db.pool, CURRENT).await;
        let retained = retained_rows(&db.pool).await;
        install(db).await;
        assert_eq!(
            retained,
            retained_rows(&db.pool).await,
            "installer must not reset grants, inventory, leases or diagnostics"
        );
        contracts(&db.pool).await;
        super::active_devices::contracts(&db.pool).await;
        retired(&db.pool).await;
        retention(&db.pool).await;
    }
    upgraded.cleanup().await;
    fresh.cleanup().await;
}

// Why: the feedback ingestion pipeline (capture triggers, outbox, evidence
// copy) is retired by migration 105 on the upgrade path and never declared on
// the fresh one; neither may carry any of it.
async fn retired(pool: &PgPool) {
    assert_eq!(
        scalar(
            pool,
            "SELECT (count(*)=0)::text FROM pg_trigger WHERE NOT tgisinternal AND tgname LIKE 'feedback\\_capture%'"
        )
        .await,
        "true",
        "no feedback capture trigger survives"
    );
    assert_eq!(
        scalar(
            pool,
            "SELECT (count(*)=0)::text FROM pg_tables WHERE tablename LIKE 'feedback\\_ingestion%'"
        )
        .await,
        "true",
        "no feedback ingestion table survives"
    );
}

async fn retention(pool: &PgPool) {
    rejects(
        pool,
        "SELECT expire_raw_evidence(clock_timestamp()-interval '89 days')",
        "23514",
    )
    .await;
    assert_eq!(
        scalar(
            pool,
            "SELECT expire_raw_evidence('2021-01-01T00:00:00Z')::text"
        )
        .await,
        "1"
    );
    for table in ["plugin_usage_events", "ai_requests"] {
        assert_eq!(
            scalar(pool, &format!("SELECT count(*)::text FROM {table}")).await,
            "0",
            "expired raw evidence: {table}"
        );
    }
    assert_eq!(
        scalar(
            pool,
            "SELECT count(*)::text FROM managed_installation_receipts"
        )
        .await,
        "2",
        "distribution evidence is separate from expired usage"
    );
    assert_eq!(
        scalar(pool, "SELECT count(*)::text FROM managed_consumer_grants").await,
        "1"
    );
}
