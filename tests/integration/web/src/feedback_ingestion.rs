//! Committed raw-reference parity, fenced recovery, corrections and
//! authenticated attribution.
use crate::tempdb::TempDb;
use sqlx::PgPool;
use systemprompt::analytics::feedback::{FactsProcessingService, FeedbackFactsRepository};
use systemprompt::identifiers::{AnalyticsWorkerId, UserId};
use systemprompt_web_jobs::feedback_ingestion::FeedbackIngestionRepository;

pub(super) async fn seed(pool: &PgPool) {
    sqlx::query("INSERT INTO users(id,name,email) VALUES('publisher','publisher','publisher@example.invalid'),('consumer','consumer','consumer@example.invalid')").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO plugin_usage_events(id,user_id,session_id,event_type,prompt_preview,dedup_key) VALUES('event1','consumer','native-session','UserPromptSubmit','/plugin:skill','dedup1'),('event2','consumer','native-session','UserPromptSubmit','/plugin:other','dedup2')").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,context_id,client_session_id,provider,model,status,cost_microdollars,input_tokens,output_tokens,latency_ms,actor_kind,actor_id) VALUES('request1','request1','consumer','00000000-0000-4000-8000-000000000001','native-session','anthropic','model','failed',125,12,3,9,'user','consumer'),('request2','request2','consumer','00000000-0000-4000-8000-000000000001','native-session','anthropic','model','completed',0,NULL,NULL,5,'user','consumer')").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO evaluation_runs(session_id,user_id,host,criterion,attempt,status,score_source,score,provider,model,system_prompt,user_prompt,config_hash) VALUES('native-session','consumer','claude-code','quality',1,'succeeded','judge',85,'e2e-mock','e2e-mock-sonnet','system','user','seed')").execute(pool).await.unwrap();
}

pub(super) async fn drain(pool: &PgPool) {
    drain_as(pool, &UserId::new("publisher")).await;
}

// Why: the same ingestion → facts pass the scheduler runs, owned by whichever
// user the calling fixture seeded as the projection owner.
pub(crate) async fn drain_as(pool: &PgPool, owner: &UserId) {
    let repository = FeedbackIngestionRepository::new(pool.clone());
    for _ in 0..8 {
        repository.backfill_page(128).await.unwrap();
    }
    for _ in 0..8 {
        let leases = repository.claim(256).await.unwrap();
        if leases.is_empty() {
            break;
        }
        for lease in leases {
            assert!(repository.process(owner, &lease).await.unwrap());
        }
    }
    let service = FactsProcessingService::new(FeedbackFactsRepository::new(pool.clone()));
    for _ in 0..8 {
        if service
            .drain(owner, &AnalyticsWorkerId::new("ingestion-test"), 256)
            .await
            .unwrap()
            == 0
        {
            break;
        }
    }
}

#[tokio::test]
async fn normalized_source_parity_preserves_failed_spend_shared_cost_and_assessment_conversation() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    seed(pool).await;
    drain(pool).await;
    let counts: (i64,i64,i64,i64,i64) = sqlx::query_as("SELECT count(*) FILTER(WHERE fact_kind='invocation'),count(*) FILTER(WHERE fact_kind='request'),count(*) FILTER(WHERE fact_kind='resource_association'),count(*) FILTER(WHERE fact_kind='assessment'),coalesce(sum(amount_micros) FILTER(WHERE fact_kind='request'),0)::bigint FROM analytics_normalized_facts WHERE NOT deleted").fetch_one(pool).await.unwrap();
    assert_eq!(counts, (2, 2, 4, 2, 125));
    let raw: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM analysis_skill_events),(SELECT count(*) FROM ai_requests),(SELECT sum(cost_microdollars)::bigint FROM ai_requests)").fetch_one(pool).await.unwrap();
    assert_eq!((counts.0, counts.1, counts.4), raw);
    let (identities, conversations): (i64, i64) = sqlx::query_as(
        "SELECT count(consumer_id),count(DISTINCT conversation_id) FROM analytics_normalized_facts",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!((identities, conversations), (0, 1));
    let pricing: serde_json::Value = sqlx::query_scalar("SELECT fact->'value'->'spend' FROM analytics_normalized_facts WHERE fact_kind='request' AND amount_micros IS NULL").fetch_one(pool).await.unwrap();
    assert_eq!(pricing["status"], "unknown_pricing");
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM analytics_normalized_facts")
        .fetch_one(pool)
        .await
        .unwrap();
    drain(pool).await;
    assert_eq!(
        before,
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM analytics_normalized_facts")
            .fetch_one(pool)
            .await
            .unwrap()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn corrections_delete_contributions_and_stale_leases_cannot_commit() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    seed(pool).await;
    let repository = FeedbackIngestionRepository::new(pool.clone());
    let leases = repository.claim(256).await.unwrap();
    sqlx::query("UPDATE ai_requests SET cost_microdollars=225 WHERE id='request1'")
        .execute(pool)
        .await
        .unwrap();
    let mut stale = 0;
    for lease in leases {
        if !repository
            .process(&UserId::new("publisher"), &lease)
            .await
            .unwrap()
        {
            stale += 1;
        }
    }
    assert!(stale > 0);
    drain(pool).await;
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT sum(amount_micros)::bigint FROM analytics_normalized_facts WHERE fact_kind='request'").fetch_one(pool).await.unwrap(),225);
    sqlx::query("DELETE FROM plugin_usage_events WHERE id='event2'")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM ai_requests WHERE id='request2'")
        .execute(pool)
        .await
        .unwrap();
    drain(pool).await;
    assert!(
        sqlx::query(
            "UPDATE feedback_ingestion_evidence SET evidence='{}'::jsonb WHERE event_id='event1'"
        )
        .execute(pool)
        .await
        .is_err()
    );
    let active:(i64,i64,i64)=sqlx::query_as("SELECT count(*) FILTER(WHERE fact_kind='invocation'),count(*) FILTER(WHERE fact_kind='request'),count(*) FILTER(WHERE fact_kind='resource_association') FROM analytics_normalized_facts WHERE NOT deleted").fetch_one(pool).await.unwrap();
    assert_eq!(active, (1, 1, 1));
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT pending_count FROM analytics_ingestion_producers WHERE producer='astound.feedback'").fetch_one(pool).await.unwrap(),0);
    db.cleanup().await;
}

#[tokio::test]
async fn reversed_commit_order_and_restart_do_not_skip_committed_sources() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    seed(pool).await;
    drain(pool).await;
    let mut early = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO users(id,name,email) VALUES('early','early','early@example.invalid')")
        .execute(&mut *early)
        .await
        .unwrap();
    sqlx::query("UPDATE ai_requests SET cost_microdollars=333 WHERE id='request1'")
        .execute(pool)
        .await
        .unwrap();
    drain(pool).await;
    sqlx::query("INSERT INTO plugin_usage_events(id,user_id,session_id,event_type,prompt_preview,dedup_key) VALUES('aaa-late-commit','early','early-session','UserPromptSubmit','/plugin:skill','early-dedup')").execute(&mut *early).await.unwrap();
    early.commit().await.unwrap();
    let repository = FeedbackIngestionRepository::new(pool.clone());
    let leases = repository.claim(256).await.unwrap();
    sqlx::query("UPDATE feedback_ingestion_outbox SET lease_until=clock_timestamp()-interval '1 second' WHERE pending").execute(pool).await.unwrap();
    let recovered = FeedbackIngestionRepository::new(pool.clone())
        .claim(256)
        .await
        .unwrap();
    assert!(!recovered.is_empty());
    for lease in leases {
        assert!(
            !repository
                .process(&UserId::new("publisher"), &lease)
                .await
                .unwrap()
        );
    }
    for lease in recovered {
        assert!(
            repository
                .process(&UserId::new("publisher"), &lease)
                .await
                .unwrap()
        );
    }
    drain(pool).await;
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM analytics_normalized_facts WHERE fact_kind='invocation' AND NOT deleted").fetch_one(pool).await.unwrap(),3);
    db.cleanup().await;
}

#[tokio::test]
async fn concurrent_claims_are_disjoint_and_retention_waits_for_snapshot_consumer() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    seed(pool).await;
    let repository = FeedbackIngestionRepository::new(pool.clone());
    let (first, second) = tokio::join!(repository.claim(2), repository.claim(2));
    let first = first.unwrap();
    let second = second.unwrap();
    assert!(!first.is_empty() && !second.is_empty());
    for a in &first {
        assert!(second.iter().all(|b| b.id != a.id));
    }
    for lease in first.into_iter().chain(second) {
        assert!(
            repository
                .process(&UserId::new("publisher"), &lease)
                .await
                .unwrap()
        );
    }
    drain(pool).await;
    assert!(
        sqlx::query_scalar::<_, i64>(
            "SELECT expire_feedback_raw(clock_timestamp()-interval '90 days')"
        )
        .fetch_one(pool)
        .await
        .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM plugin_usage_events")
            .fetch_one(pool)
            .await
            .unwrap(),
        2
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_opencode_skill_tool_call_counts_as_an_invocation_under_its_plugin_identity() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    sqlx::query(
        "INSERT INTO users(id,name,email) VALUES('consumer','consumer','consumer@example.invalid')",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO plugin_usage_events(id,user_id,session_id,event_type,tool_name,metadata,dedup_key) VALUES \
         ('oc1','consumer','ses_abc','PostToolUse','skill','{\"tool_input\":{\"name\":\"code-review\"},\"tool_use_id\":\"call_1\",\"skill_ref\":\"astound-dev:code-review\",\"native_host\":\"opencode\"}','oc-dedup1'), \
         ('oc2','consumer','ses_abc','PostToolUse','skill','{\"tool_input\":{\"name\":\"local_only\"},\"tool_use_id\":\"call_2\"}','oc-dedup2'), \
         ('oc3','consumer','ses_abc','PostToolUse','read','{\"tool_input\":{\"name\":\"ignored\"},\"tool_use_id\":\"call_3\"}','oc-dedup3')",
    )
    .execute(pool)
    .await
    .unwrap();
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT skill, source, tool_use_id FROM analysis_skill_events WHERE session_id='ses_abc' ORDER BY skill",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            (
                "astound-dev:code-review".to_owned(),
                "tool".to_owned(),
                Some("call_1".to_owned())
            ),
            (
                "opencode:local-only".to_owned(),
                "tool".to_owned(),
                Some("call_2".to_owned())
            ),
        ]
    );
}
