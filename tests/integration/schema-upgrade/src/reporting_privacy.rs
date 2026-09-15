//! Astound060 coordinates reporting copies with raw expiry on fresh and
//! release-upgraded databases; replay and rebuild cannot restore expired data.

use astound_test_common::TempDb;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::analytics::projection::{ReportingProjector, ReportingRow};
use systemprompt::database::install_extension_schemas;
use systemprompt::extension::ExtensionRegistry;

async fn execute(pool: &PgPool, sql: &str) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(pool)
        .await
        .expect("fixture SQL");
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await
        .expect("fixture count")
}

async fn rebuild(db: &TempDb) {
    systemprompt::system::reporting::rebuild(&db.db_pool())
        .await
        .expect("production reporting rebuild");
}

async fn drain_reporting(db: &TempDb) {
    for _ in 0..10 {
        if systemprompt::system::reporting::process_pending(&db.db_pool(), 256)
            .await
            .expect("production reporting delivery")
            == 0
        {
            return;
        }
    }
    panic!("bounded fixture reporting queue did not drain");
}

async fn seed(pool: &PgPool) {
    execute(pool, "INSERT INTO users(id,name,email,roles) VALUES('privacy-owner','Privacy owner','privacy-owner@example.test',ARRAY['admin']);
        INSERT INTO user_sessions(session_id,user_id,started_at,last_activity_at,ended_at,expires_at,ip_address,fingerprint_hash)
          VALUES('privacy-old-session','privacy-owner','2020-01-01','2020-01-01','2020-01-02','2020-01-03','192.0.2.61','old-sensitive-fingerprint'),
          ('privacy-active-session','privacy-owner','2020-01-01',clock_timestamp(),NULL,clock_timestamp()+interval '7 days','192.0.2.63','active-fingerprint');
        INSERT INTO ai_requests(id,request_id,user_id,context_id,provider,model,actor_kind,actor_id,status,cost_microdollars,created_at,updated_at)
          VALUES('privacy-old-request','privacy-old-provider','privacy-owner','privacy-old-context','fixture','fixture','user','privacy-owner','failed',125,'2020-01-01','2020-01-01'),
          ('privacy-recent-request','privacy-recent-provider','privacy-owner','privacy-recent-context','fixture','fixture','user','privacy-owner','failed',250,clock_timestamp(),clock_timestamp());
        INSERT INTO plugin_usage_events(id,user_id,session_id,event_type,created_at)
          VALUES('privacy-old-invocation','privacy-owner','privacy-old-session','tool_use','2020-01-01');").await;
}

async fn expired_reporting_is_absent(pool: &PgPool) {
    for sql in [
        "SELECT count(*) FROM user_sessions WHERE session_id='privacy-old-session'",
        "SELECT count(*) FROM analytics_report_ai_requests WHERE id='privacy-old-request'",
        "SELECT count(*) FROM analytics_report_user_sessions WHERE session_id='privacy-old-session'",
        "SELECT count(*) FROM analytics_projection_revisions WHERE entity_key IN ('privacy-old-request','privacy-old-session')",
        "SELECT count(*) FROM event_outbox WHERE consumer='analytics_reporting' AND (fact->'data'->>'key') IN ('privacy-old-request','privacy-old-session')",
    ] {
        assert_eq!(count(pool, sql).await, 0, "expired identity remains: {sql}");
    }
    assert_eq!(count(pool, "SELECT count(*) FROM analytics_report_ai_requests WHERE id='privacy-recent-request' AND cost_microdollars=250 AND status='failed'").await, 1);
    assert_eq!(count(pool, "SELECT count(*) FROM user_sessions WHERE session_id='privacy-active-session' AND fingerprint_hash='active-fingerprint'").await, 1);
}

async fn exercise_expiry(db: &TempDb) {
    rebuild(db).await;
    seed(&db.pool).await;
    let stale: Json<ReportingRow> = sqlx::query_scalar(
        "SELECT fact->'data' FROM event_outbox WHERE consumer='analytics_reporting' AND fact->'data'->>'key'='privacy-old-request' ORDER BY created_at LIMIT 1"
    ).fetch_one(&*db.pool).await.unwrap();
    let error = sqlx::query("SELECT expire_feedback_raw('2021-01-01T00:00:00Z')")
        .execute(&*db.pool)
        .await
        .expect_err("pending reporting evidence must fence expiry");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("55000")
    );
    assert_eq!(
        count(
            &db.pool,
            "SELECT count(*) FROM ai_requests WHERE id='privacy-old-request'"
        )
        .await,
        1
    );
    assert!(count(&db.pool, "SELECT count(*) FROM event_outbox WHERE consumer='analytics_reporting' AND processed_at IS NULL").await > 0);
    drain_reporting(db).await;
    assert_eq!(count(&db.pool, "SELECT cost_microdollars FROM analytics_report_ai_requests WHERE id='privacy-old-request'").await, 125);
    // This fixture has no feedback workers or normalized contributions. Mark
    // only its source admission/backfill barriers drained to isolate reporting.
    execute(&db.pool, "UPDATE feedback_ingestion_outbox SET pending=false; UPDATE feedback_ingestion_backfill SET done=true; UPDATE analytics_ingestion_producers SET pending_count=0,oldest_pending_at=NULL;").await;
    assert_eq!(
        count(
            &db.pool,
            "SELECT expire_feedback_raw('2021-01-01T00:00:00Z')"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &db.pool,
            "SELECT count(*) FROM plugin_usage_events WHERE id='privacy-old-invocation'"
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &db.pool,
            "SELECT count(*) FROM ai_requests WHERE id='privacy-old-request'"
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &db.pool,
            "SELECT count(*) FROM feedback_ingestion_outbox WHERE occurred_at<'2021-01-01'"
        )
        .await,
        0
    );
    expired_reporting_is_absent(&db.pool).await;
    let mut tx = db.pool.begin().await.unwrap();
    assert!(
        !ReportingProjector::apply_fact(&mut tx, &stale.0)
            .await
            .unwrap()
    );
    tx.commit().await.unwrap();
    rebuild(db).await;
    expired_reporting_is_absent(&db.pool).await;
    let cleared: String = sqlx::query_scalar(
        "SELECT COALESCE(current_setting('astound.feedback_retention',true),'')",
    )
    .fetch_one(&*db.pool)
    .await
    .unwrap();
    assert!(
        cleared.is_empty(),
        "expiry capture suppression must not escape its transaction"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reporting_copies_and_outbox_expire_without_rebuild_resurrection_after_upgrade() {
    let upgraded = TempDb::create_empty().await.expect("upgrade database");
    execute(&upgraded.pool, &super::upgrade::fixture_sql()).await;
    let registry = ExtensionRegistry::discover().unwrap();
    install_extension_schemas(&registry, upgraded.db_pool().write())
        .await
        .unwrap();
    let fresh = TempDb::create().await.expect("fresh database");
    for db in [&upgraded, &fresh] {
        exercise_expiry(db).await;
    }
    upgraded.cleanup().await;
    fresh.cleanup().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deleting_a_principal_erases_session_reporting_and_queued_identity_before_rebuild() {
    let db = TempDb::create().await.expect("privacy database");
    rebuild(&db).await;
    execute(&db.pool, "INSERT INTO users(id,name,email,roles) VALUES('deleted-principal','Delete me','delete@example.test',ARRAY['user']);
        INSERT INTO user_sessions(session_id,user_id,ip_address,fingerprint_hash) VALUES('deleted-session','deleted-principal','192.0.2.62','deleted-fingerprint');").await;
    drain_reporting(&db).await;
    assert_eq!(
        count(
            &db.pool,
            "SELECT count(*) FROM analytics_report_user_sessions WHERE session_id='deleted-session'"
        )
        .await,
        1
    );
    systemprompt::users::UserRepository::new(&db.db_pool())
        .unwrap()
        .delete(&systemprompt::identifiers::UserId::new("deleted-principal"))
        .await
        .expect("production hard deletion");
    for sql in [
        "SELECT count(*) FROM user_sessions WHERE session_id='deleted-session'",
        "SELECT count(*) FROM analytics_report_users WHERE id='deleted-principal'",
        "SELECT count(*) FROM analytics_report_user_sessions WHERE session_id='deleted-session'",
        "SELECT count(*) FROM event_outbox WHERE consumer='analytics_reporting' AND fact::text LIKE '%deleted-principal%'",
    ] {
        assert_eq!(
            count(&db.pool, sql).await,
            0,
            "deleted principal reference: {sql}"
        );
    }
    rebuild(&db).await;
    assert_eq!(count(&db.pool, "SELECT count(*) FROM analytics_report_user_sessions WHERE session_id='deleted-session' OR fingerprint_hash='deleted-fingerprint'").await, 0);
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expiry_before_reporting_initialization_fences_the_first_later_rebuild() {
    let db = TempDb::create()
        .await
        .expect("uninitialized reporting database");
    assert_eq!(
        count(
            &db.pool,
            "SELECT count(*) FROM analytics_projection_state WHERE initialized"
        )
        .await,
        0
    );
    seed(&db.pool).await;
    // No normalized feedback was admitted in this isolated fixture.
    execute(&db.pool, "UPDATE feedback_ingestion_outbox SET pending=false; UPDATE feedback_ingestion_backfill SET done=true; UPDATE analytics_ingestion_producers SET pending_count=0,oldest_pending_at=NULL;").await;
    assert_eq!(
        count(
            &db.pool,
            "SELECT expire_feedback_raw('2021-01-01T00:00:00Z')"
        )
        .await,
        1
    );
    rebuild(&db).await;
    expired_reporting_is_absent(&db.pool).await;
    db.cleanup().await;
}
