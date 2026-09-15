//! One real signed/device-authenticated hook through durable ingestion to a
//! snapshot. This measures local functional completion, never throughput/p95.

use axum::http::StatusCode;
use chrono::Utc;
use std::time::Instant;
use systemprompt::analytics::feedback::{FactsProcessingService, FeedbackFactsRepository};
use systemprompt::analytics::snapshots::FeedbackSnapshotsRepository;
use systemprompt::identifiers::{DeviceCertId, TaskId};
use systemprompt::marketplace::managed::ManagedRepository;
use systemprompt_web_jobs::feedback_ingestion::FeedbackIngestionRepository;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::seed::TokenSpec;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authenticated_hook_commits_once_and_reaches_owner_snapshot_with_fresh_watermarks() {
    assert!(globals::init(), "profile required for real hook acceptance");
    let db = TempDb::create()
        .await
        .expect("database required for functional freshness");
    let credentials = principal::provision(&db.pool).await;
    let owner = credentials.admin_user_id.clone();
    let consumer = credentials.non_admin_user_id.clone();
    assert_ne!(owner, consumer);
    let app = App::new(&db.pool, credentials);
    let managed = ManagedRepository::new(db.pool.as_ref().clone());
    let device = DeviceCertId::generate();
    sqlx::query(
        "INSERT INTO user_device_certs(id,user_id,fingerprint,label) VALUES($1,$2,$1,'freshness')",
    )
    .bind(device.as_str())
    .bind(consumer.as_str())
    .execute(db.pool.as_ref())
    .await
    .unwrap();
    let credential = managed.issue_consumer_credential(&device).await.unwrap();
    let token = seed::mint(&TokenSpec::hook(consumer.as_str()));
    let authorization = format!("Bearer {token}");
    let session = seed::unique("feedback-session");
    let body = serde_json::json!({
        "session_id": session,
        "event_id": seed::unique("feedback-delivery"),
        "cwd":"/tmp/feedback-contract", "permission_mode":"default",
        "transcript_path":"/tmp/feedback-contract/session.jsonl",
        "hook_event_name":"UserPromptSubmit", "prompt":"/contract-plugin:freshness"
    })
    .to_string();
    let call = || Call {
        method: "post",
        path: "/hooks/track",
        principal: Principal::Anonymous,
        content_type: Some("application/json"),
        body: Some(&body),
    };
    let (rejected, _) = app
        .response_headers_with(
            call(),
            &[
                ("authorization", authorization.as_str()),
                ("x-systemprompt-device-credential", "forged-credential"),
                ("x-systemprompt-host", "codex"),
            ],
        )
        .await;
    assert_eq!(rejected, StatusCode::UNAUTHORIZED);
    let started = Instant::now();
    let headers = [
        ("authorization", authorization.as_str()),
        (
            "x-systemprompt-device-credential",
            credential.credential.as_str(),
        ),
        ("x-systemprompt-host", "codex"),
    ];
    for _ in 0..2 {
        let (status, _) = app.response_headers_with(call(), &headers).await;
        assert_eq!(status, StatusCode::OK);
    }
    let committed = Instant::now();
    let evidence: (i64, i64) = sqlx::query_as("SELECT count(*),count(a.event_id) FROM plugin_usage_events e LEFT JOIN feedback_authenticated_events a ON a.event_id=e.id AND a.consumer_id=$1 AND a.device_id=$2 AND a.host='codex' AND a.native_session_id=e.session_id WHERE e.session_id=$3")
        .bind(consumer.as_str()).bind(device.as_str()).bind(&session).fetch_one(db.pool.as_ref()).await.unwrap();
    assert_eq!(
        evidence,
        (1, 1),
        "identical retry must preserve one authenticated event"
    );
    let ingestion = FeedbackIngestionRepository::new(db.pool.as_ref().clone());
    let facts = FactsProcessingService::new(FeedbackFactsRepository::new(db.pool.as_ref().clone()));
    let snapshots = FeedbackSnapshotsRepository::new(
        db.pool.as_ref().clone(),
        systemprompt::analytics::feedback::FeedbackFactsRepository::new(db.pool.as_ref().clone()),
    );
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        for _ in 0..8 {
            ingestion.backfill_page(128).await.unwrap();
        }
        for _ in 0..16 {
            let batch = ingestion.claim(256).await.unwrap();
            if batch.is_empty() {
                break;
            }
            for lease in batch {
                assert!(ingestion.process(&owner, &lease).await.unwrap());
            }
        }
        for _ in 0..16 {
            if facts
                .drain(&owner, &TaskId::new("freshness-facts"), 256)
                .await
                .unwrap()
                == 0
            {
                break;
            }
        }
        for _ in 0..16 {
            if snapshots
                .process(&owner, &TaskId::new("freshness-snapshots"), Utc::now())
                .await
                .unwrap()
                == 0
            {
                break;
            }
        }
        snapshots.refresh(&owner, &[], Utc::now()).await.unwrap();
    })
    .await
    .expect("bounded functional worker completion");
    let retained = snapshots.snapshot(&owner, None, 1).await.unwrap().unwrap();
    let health = snapshots.health(&owner).await.unwrap();
    assert_eq!(retained.metrics.invocations, 1);
    assert_eq!(retained.distinct_users, Some(1));
    assert_eq!(retained.distinct_sessions, Some(1));
    assert_eq!(health.pending_changes, 0);
    assert_eq!(health.pending_producer_changes, 0);
    assert_eq!(health.fact_generation, health.facts_generation);
    assert_eq!(retained.fact_generation, health.facts_generation);
    assert!(health.generation > 0);
    let normalized: (String, Option<String>) = sqlx::query_as("SELECT consumer_id,resource_id FROM analytics_normalized_facts WHERE owner_id=$1 AND fact_kind='invocation' AND session_id=$2")
        .bind(owner.as_str()).bind(&session).fetch_one(db.pool.as_ref()).await.unwrap();
    assert_eq!(
        normalized,
        (consumer.to_string(), None),
        "device identity is established; missing installation attribution stays unknown"
    );
    eprintln!(
        "feedback functional freshness: one local authenticated hook + identical retry, one consumer, one configured owner, serial worker drains; commit-to-snapshot={}ms total={}ms fact_generation={} snapshot_generation={}; no native CLI, paid inference, load or p95 measurement",
        committed.elapsed().as_millis(),
        started.elapsed().as_millis(),
        health.fact_generation,
        health.generation
    );
    db.cleanup().await;
}
