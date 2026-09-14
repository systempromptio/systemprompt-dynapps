//! Stateful acceptance for marketplace004–008, analytics005–007,
//! evaluation015, AI024 and Astound059 on fresh and real release-upgrade paths.

use astound_test_common::{TempDb, db_or_skip, empty_db_or_skip};
use sqlx::PgPool;
use systemprompt::ExtensionRegistry;
use systemprompt::database::install_extension_schemas;

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
      (SELECT jsonb_build_array(spec,spec_digest) FROM eval_experiments WHERE id='feedback-experiment'),
      (SELECT jsonb_build_array(fencing_token,result) FROM eval_executions WHERE id='feedback-execution'),
      (SELECT jsonb_build_array(fencing_token,digest,manifest) FROM eval_execution_evidence WHERE execution_id='feedback-execution'),
      (SELECT jsonb_build_array(status,attempts) FROM eval_execution_cleanup WHERE execution_id='feedback-execution'),
      (SELECT jsonb_build_array(attempted_cost_microdollars,accounting_status,verified_success) FROM eval_execution_measurements WHERE execution_id='feedback-execution'),
      (SELECT jsonb_build_array(cap,reserved,settled) FROM eval_budget_accounts WHERE id='feedback-budget'),
      (SELECT jsonb_build_array(input_tokens,output_tokens,tokens_used,cost_microdollars,completed_at) FROM ai_requests WHERE id='feedback-ai-completed')
    )::text").await
}

// Snapshot complete rows, including lease fences, timestamps, revisions and
// diagnostics, before an installer restart. Table names are fixture constants.
async fn retained_rows(pool: &PgPool) -> Vec<String> {
    let mut rows = Vec::new();
    for table in [
        "managed_api_operations",
        "ai_requests",
        "managed_consumer_credentials",
        "managed_consumer_grants",
        "managed_installation_receipts",
        "managed_consumer_session_bindings",
        "managed_consumer_invocation_evidence",
        "managed_consumer_attribution_projection",
        "managed_consumer_attribution_history",
        "managed_dependency_verifications",
        "managed_resource_git_bindings",
        "managed_inventory_state",
        "managed_inventory_entries",
        "managed_inventory_membership",
        "managed_inventory_bindings",
        "managed_inventory_captures",
        "managed_installation_coverage_state",
        "managed_installation_coverage",
        "analytics_fact_changes",
        "analytics_normalized_facts",
        "analytics_fact_deltas",
        "analytics_fact_checkpoints",
        "analytics_fact_consumers",
        "analytics_ingestion_producers",
        "analytics_snapshot_state",
        "analytics_snapshot_daily",
        "analytics_feedback_snapshots",
        "analytics_snapshot_jobs",
        "eval_campaigns",
        "eval_campaign_diagnostics",
        "eval_campaign_holdout_proposals",
        "eval_holdout_content_consumption",
        "feedback_ingestion_backfill",
        "feedback_ingestion_outbox",
        "feedback_ingestion_evidence",
        "reviewed_production_failures",
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
    assert_eq!(scalar(pool,"SELECT (fence=7 AND state='pending' AND input_checkpoint->>'retained'='capture-input')::text FROM managed_api_operations WHERE owner_id='feedback-owner' AND id='feedback-api-pending'").await,"true");
    assert_eq!(scalar(pool,"SELECT issuance_operation FROM managed_consumer_credentials WHERE device_id='feedback-device'").await,"feedback-api-completed");
    rejects(pool,"INSERT INTO managed_api_operations(owner_id,id,kind,request_digest) VALUES('feedback-owner','feedback-api-pending','source_capture','other')","23505").await;
    rejects(
        pool,
        "UPDATE managed_api_operations SET state='published' WHERE id='feedback-api-pending'",
        "23514",
    )
    .await;
    assert_eq!(scalar(pool, "SELECT (consumer_id IS NULL AND device_id IS NULL AND host IS NULL AND fully_verified IS NULL)::text FROM managed_installation_receipts WHERE id='feedback-legacy-receipt'").await, "true");
    for statement in [
        "UPDATE managed_revisions SET rationale='rewritten' WHERE id='feedback-revision'",
        "UPDATE managed_consumer_invocation_evidence SET evidence='{}' WHERE id='feedback-evidence'",
        "UPDATE managed_consumer_session_bindings SET native_session_id='forged' WHERE id='feedback-binding'",
        "UPDATE managed_dependency_verifications SET manifest='{}' WHERE id='feedback-verification'",
        "UPDATE managed_resource_git_bindings SET relative_root='other' WHERE resource_id='feedback-resource'",
        "UPDATE managed_inventory_bindings SET bound_by='feedback-consumer' WHERE entry_id='feedback-entry'",
        "UPDATE feedback_ingestion_evidence SET evidence='{}' WHERE event_id='feedback-raw'",
        "UPDATE eval_campaign_holdout_proposals SET spec='{}' WHERE id='feedback-holdout'",
    ] {
        rejects(pool, statement, "23514").await;
    }
    rejects(pool, "INSERT INTO managed_consumer_grants(owner_id,resource_id,consumer_id) VALUES('feedback-consumer','feedback-resource','feedback-owner')", "23503").await;
    rejects(pool, "INSERT INTO managed_inventory_membership(owner_id,entry_id,effective_from,record) VALUES('feedback-owner','feedback-entry',now(),'{}')", "23505").await;
    rejects(pool, "INSERT INTO eval_holdout_content_consumption(owner_id,content_digest,experiment_id) VALUES('feedback-owner',repeat('5',64),'feedback-experiment')", "23505").await;
    execute(pool, "UPDATE eval_campaign_holdout_proposals SET confirmed_by='feedback-owner',confirmed_at='2020-02-01T00:00:00Z',experiment_id='feedback-experiment' WHERE id='feedback-holdout'").await;
    rejects(pool, "UPDATE eval_campaign_holdout_proposals SET confirmed_by='feedback-consumer' WHERE id='feedback-holdout'", "23514").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn seeded_feedback_survives_release_upgrade_and_installer_restart() {
    let upgraded = empty_db_or_skip!();
    execute(&upgraded.pool, &super::upgrade::fixture_sql()).await;
    execute(&upgraded.pool, LEGACY).await;
    let before = legacy_evidence(&upgraded.pool).await;
    install(&upgraded).await;
    assert_eq!(
        before,
        legacy_evidence(&upgraded.pool).await,
        "release evidence and reserved budget must survive migrations"
    );
    let fresh = db_or_skip!();
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
            "installer must not reset grants, checkpoints, seeds, leases, snapshots or diagnostics"
        );
        contracts(&db.pool).await;
        privacy(&db.pool).await;
    }
    upgraded.cleanup().await;
    fresh.cleanup().await;
}

async fn privacy(pool: &PgPool) {
    rejects(
        pool,
        "SELECT expire_feedback_raw(clock_timestamp()-interval '89 days')",
        "23514",
    )
    .await;
    rejects(
        pool,
        "SELECT expire_feedback_raw('2021-01-01T00:00:00Z')",
        "55000",
    )
    .await;
    assert_eq!(
        scalar(
            pool,
            "SELECT count(*)::text FROM reviewed_production_failures WHERE id='feedback-reviewed'"
        )
        .await,
        "1"
    );
    // This isolated database has no workers. Explicitly acknowledge the seeded
    // source/backfill work to exercise later barriers, not bypass a live queue.
    execute(pool, "UPDATE feedback_ingestion_outbox SET pending=false; UPDATE feedback_ingestion_backfill SET done=true; UPDATE analytics_ingestion_producers SET pending_count=0,oldest_pending_at=NULL").await;
    rejects(
        pool,
        "SELECT expire_feedback_raw('2021-01-01T00:00:00Z')",
        "55000",
    )
    .await;
    execute(pool, "UPDATE analytics_snapshot_state SET evidence_cutoff='2021-01-01T00:00:00Z' WHERE owner_id='feedback-owner'").await;
    assert_eq!(
        scalar(
            pool,
            "SELECT expire_feedback_raw('2021-01-01T00:00:00Z')::text"
        )
        .await,
        "1"
    );
    for table in [
        "reviewed_production_failures",
        "plugin_usage_events",
        "feedback_ingestion_evidence",
        "managed_consumer_invocation_evidence",
        "managed_consumer_attribution_projection",
        "managed_consumer_attribution_history",
        "feedback_ingestion_outbox",
    ] {
        assert_eq!(
            scalar(pool, &format!("SELECT count(*)::text FROM {table}")).await,
            "0",
            "expired evidence and indirect references: {table}"
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
    assert_eq!(
        scalar(
            pool,
            "SELECT COALESCE(current_setting('astound.feedback_retention',true),'')"
        )
        .await,
        ""
    );
}
