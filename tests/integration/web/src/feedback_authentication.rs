//! Actual retained publication plus authenticated consumer context drives late
//! attribution.
use super::feedback_ingestion::{drain, seed};
use crate::tempdb::TempDb;
use chrono::Utc;
use std::collections::BTreeMap;
use systemprompt::identifiers::{ConsumerInstallationId, DeviceCertId, NativeSessionId, UserId};
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, ManagedRepository, NewResource, NewRevision, PublicationAction,
    PublicationRequest, ResourceKind, RevisionFiles, SnapshotProvenance, SourceSpec,
};
use systemprompt::models::feedback::receipts::{
    ConsumerReceiptRequest, ReadbackStatus, RuntimeFileReadback, SessionBindingRequest,
};
use systemprompt::models::feedback::{ContentDigest, EvaluatorClient};

#[tokio::test]
async fn publisher_differs_from_consumer_and_late_session_binding_corrects_without_duplicate_invocation()
 {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    seed(pool).await;
    sqlx::query(
        "UPDATE plugin_usage_events SET prompt_preview='/plugin:my-skill' WHERE id='event1'",
    )
    .execute(pool)
    .await
    .unwrap();
    let owner = UserId::new("publisher");
    let consumer = UserId::new("consumer");
    let repo = ManagedRepository::new(pool.clone());
    let cert = DeviceCertId::generate();
    sqlx::query("INSERT INTO user_device_certs(id,user_id,fingerprint,label) VALUES($1,'consumer',$1,'feedback')").bind(cert.as_str()).execute(pool).await.unwrap();
    let credential = repo.issue_consumer_credential(&cert).await.unwrap();
    let source = repo
        .register_source(&owner, "authoring", &SourceSpec::Managed)
        .await
        .unwrap();
    let snapshot = repo
        .capture_snapshot(
            &owner,
            &source,
            &SnapshotProvenance {
                source_kind: "managed".to_owned(),
                commit: None,
                tree_digest: AssetDigest::of(b"tree"),
                importer_version: "test".to_owned(),
            },
        )
        .await
        .unwrap();
    let resource = repo
        .bind_resource(
            &owner,
            &NewResource {
                source_id: source,
                upstream_key: "skill".to_owned(),
                kind: ResourceKind::Skill,
                resource_key: "my_skill".to_owned(),
            },
        )
        .await
        .unwrap();
    let revision = repo
        .create_revision(
            &owner,
            &NewRevision {
                resource_id: resource.clone(),
                snapshot_id: snapshot,
                parent_id: None,
                files: RevisionFiles(BTreeMap::from([(
                    "SKILL.md".to_owned(),
                    AssetFile {
                        bytes: b"# Skill".to_vec(),
                        media_type: "text/markdown".to_owned(),
                        executable: false,
                    },
                )])),
                dependencies: BTreeMap::new(),
                rationale: "fixture".to_owned(),
            },
        )
        .await
        .unwrap();
    let publication = repo
        .review_and_publish(
            &owner,
            &owner,
            &PublicationRequest {
                resource_id: resource.clone(),
                revision_id: Some(revision.clone()),
                action: PublicationAction::InitialAdoption,
                expected_generation: 0,
                operation_key: "initial".to_owned(),
                comparison_evidence: serde_json::json!({}),
                limitations: String::new(),
            },
        )
        .await
        .unwrap();
    repo.set_consumer_grant(&owner, &resource, &consumer, true)
        .await
        .unwrap();
    sqlx::query("INSERT INTO feedback_authenticated_events(event_id,consumer_id,device_id,host,native_session_id) VALUES('event1','consumer',$1,'codex','native-session')").bind(cert.as_str()).execute(pool).await.unwrap();
    drain(pool).await;
    let (identified,attributed):(i64,i64)=sqlx::query_as("SELECT count(consumer_id),count(resource_id) FROM analytics_normalized_facts WHERE fact_kind='invocation'").fetch_one(pool).await.unwrap();
    assert_eq!((identified, attributed), (1, 0));
    let plan = repo
        .consumer_installation_plan(
            &credential.credential,
            &resource,
            &publication.publication_id,
            EvaluatorClient::Codex,
        )
        .await
        .unwrap();
    let files = plan
        .canonical_files
        .into_iter()
        .map(|mut file| {
            file.content_check = ReadbackStatus::Verified;
            file.mode_check = ReadbackStatus::Verified;
            file
        })
        .collect();
    let runtime_files = plan
        .runtime_files
        .iter()
        .map(|file| RuntimeFileReadback {
            path: file.path.clone(),
            digest: ContentDigest::of(&file.bytes),
            bytes: file.bytes.len() as u64,
            executable: file.executable,
            content_check: ReadbackStatus::Verified,
            mode_check: ReadbackStatus::Verified,
        })
        .collect();
    let receipt = repo
        .record_consumer_receipt(
            &credential.credential,
            &ConsumerReceiptRequest {
                installation_id: ConsumerInstallationId::generate(),
                publication_id: publication.publication_id,
                resource_id: resource,
                revision_id: revision,
                generation: 1,
                bundle_digest: plan.bundle_digest,
                host: EvaluatorClient::Codex,
                observed_at: Utc::now(),
                files,
                runtime_files,
            },
        )
        .await
        .unwrap();
    repo.bind_consumer_session(
        &credential.credential,
        &SessionBindingRequest {
            receipt_id: receipt.receipt_id,
            host: EvaluatorClient::Codex,
            session_id: NativeSessionId::new("native-session"),
        },
    )
    .await
    .unwrap();
    drain(pool).await;
    let (invocations,attributed):(i64,i64)=sqlx::query_as("SELECT count(*),count(resource_id) FROM analytics_normalized_facts WHERE fact_kind='invocation' AND NOT deleted").fetch_one(pool).await.unwrap();
    assert_eq!((invocations, attributed), (2, 1));
    let second_owner = UserId::new("publisher-two");
    sqlx::query("INSERT INTO users(id,name,email) VALUES('publisher-two','publisher-two','publisher-two@example.invalid')").execute(pool).await.unwrap();
    let source = repo
        .register_source(&second_owner, "authoring", &SourceSpec::Managed)
        .await
        .unwrap();
    let snapshot = repo
        .capture_snapshot(
            &second_owner,
            &source,
            &SnapshotProvenance {
                source_kind: "managed".to_owned(),
                commit: None,
                tree_digest: AssetDigest::of(b"other-tree"),
                importer_version: "test".to_owned(),
            },
        )
        .await
        .unwrap();
    let resource = repo
        .bind_resource(
            &second_owner,
            &NewResource {
                source_id: source,
                upstream_key: "different".to_owned(),
                kind: ResourceKind::Skill,
                resource_key: "my-skill".to_owned(),
            },
        )
        .await
        .unwrap();
    let revision = repo
        .create_revision(
            &second_owner,
            &NewRevision {
                resource_id: resource.clone(),
                snapshot_id: snapshot,
                parent_id: None,
                files: RevisionFiles(BTreeMap::from([(
                    "SKILL.md".to_owned(),
                    AssetFile {
                        bytes: b"# Other skill".to_vec(),
                        media_type: "text/markdown".to_owned(),
                        executable: false,
                    },
                )])),
                dependencies: BTreeMap::new(),
                rationale: "conflicting publisher".to_owned(),
            },
        )
        .await
        .unwrap();
    let publication = repo
        .review_and_publish(
            &second_owner,
            &second_owner,
            &PublicationRequest {
                resource_id: resource.clone(),
                revision_id: Some(revision.clone()),
                action: PublicationAction::InitialAdoption,
                expected_generation: 0,
                operation_key: "initial".to_owned(),
                comparison_evidence: serde_json::json!({}),
                limitations: String::new(),
            },
        )
        .await
        .unwrap();
    repo.set_consumer_grant(&second_owner, &resource, &consumer, true)
        .await
        .unwrap();
    let plan = repo
        .consumer_installation_plan(
            &credential.credential,
            &resource,
            &publication.publication_id,
            EvaluatorClient::Codex,
        )
        .await
        .unwrap();
    let files = plan
        .canonical_files
        .into_iter()
        .map(|mut file| {
            file.content_check = ReadbackStatus::Verified;
            file.mode_check = ReadbackStatus::Verified;
            file
        })
        .collect();
    let runtime_files = plan
        .runtime_files
        .iter()
        .map(|file| RuntimeFileReadback {
            path: file.path.clone(),
            digest: ContentDigest::of(&file.bytes),
            bytes: file.bytes.len() as u64,
            executable: file.executable,
            content_check: ReadbackStatus::Verified,
            mode_check: ReadbackStatus::Verified,
        })
        .collect();
    let receipt = repo
        .record_consumer_receipt(
            &credential.credential,
            &ConsumerReceiptRequest {
                installation_id: ConsumerInstallationId::generate(),
                publication_id: publication.publication_id,
                resource_id: resource,
                revision_id: revision,
                generation: 1,
                bundle_digest: plan.bundle_digest,
                host: EvaluatorClient::Codex,
                observed_at: Utc::now(),
                files,
                runtime_files,
            },
        )
        .await
        .unwrap();
    repo.bind_consumer_session(
        &credential.credential,
        &SessionBindingRequest {
            receipt_id: receipt.receipt_id,
            host: EvaluatorClient::Codex,
            session_id: NativeSessionId::new("native-session"),
        },
    )
    .await
    .unwrap();
    drain(pool).await;
    let attribution: serde_json::Value =
        sqlx::query_scalar("SELECT feedback_event_attribution('event1')")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        attribution["status"], "unknown",
        "hyphen/underscore collisions across retained publishers must not select one receipt"
    );
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM analytics_normalized_facts WHERE fact_kind='invocation' AND NOT deleted").fetch_one(pool).await.unwrap(),2);
    sqlx::query("DELETE FROM feedback_authenticated_events WHERE event_id='event1'")
        .execute(pool)
        .await
        .unwrap();
    drain(pool).await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(resource_id) FROM analytics_normalized_facts WHERE fact_kind='invocation'"
        )
        .fetch_one(pool)
        .await
        .unwrap(),
        0
    );
    verify_native_evidence_expiry_preserves_installation_receipts(pool, cert.as_str()).await;
    db.cleanup().await;
}

async fn verify_native_evidence_expiry_preserves_installation_receipts(
    pool: &sqlx::PgPool,
    device: &str,
) {
    sqlx::query("INSERT INTO managed_consumer_invocation_evidence(id,consumer_id,device_id,host,native_session_id,invocation_id,resource_id,evidence,occurred_at) VALUES('old-native','consumer',$1,'codex','old-native-session','old-call','retained-resource','{}',clock_timestamp()-interval '100 days')").bind(device).execute(pool).await.unwrap();
    sqlx::query(
        "INSERT INTO managed_consumer_attribution_projection(evidence_id) VALUES('old-native')",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO managed_consumer_attribution_history(evidence_id,version,reason) VALUES('old-native',1,'unknown')").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO managed_consumer_session_bindings(id,receipt_id,consumer_id,device_id,host,native_session_id,bound_at) SELECT 'old-binding',id,'consumer',$1,'codex','old-native-session',clock_timestamp()-interval '100 days' FROM managed_installation_receipts WHERE consumer_id='consumer' LIMIT 1").bind(device).execute(pool).await.unwrap();
    drain(pool).await;
    let receipts =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM managed_installation_receipts")
            .fetch_one(pool)
            .await
            .unwrap();
    // The snapshot worker has separate parity tests; this fixture establishes its
    // completed barrier explicitly to isolate raw-source erasure and FK cascades.
    sqlx::query("UPDATE analytics_fact_deltas SET consumed_at=clock_timestamp()")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO analytics_fact_consumers(owner_id,consumer,generation) SELECT owner_id,'snapshots-v1',generation FROM analytics_fact_checkpoints ON CONFLICT(owner_id,consumer) DO UPDATE SET generation=EXCLUDED.generation").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO analytics_snapshot_state(owner_id,evidence_cutoff) SELECT owner_id,clock_timestamp()-interval '90 days' FROM analytics_fact_checkpoints ON CONFLICT(owner_id) DO UPDATE SET evidence_cutoff=EXCLUDED.evidence_cutoff").execute(pool).await.unwrap();
    sqlx::query("SELECT expire_feedback_raw(clock_timestamp()-interval '90 days 1 second')")
        .execute(pool)
        .await
        .unwrap();
    for table in [
        "managed_consumer_invocation_evidence",
        "managed_consumer_attribution_projection",
        "managed_consumer_attribution_history",
    ] {
        let column = if table == "managed_consumer_invocation_evidence" {
            "id"
        } else {
            "evidence_id"
        };
        let remaining = sqlx::query_scalar::<_, i64>(&format!(
            "SELECT count(*) FROM {table} WHERE {column}='old-native'"
        ))
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(
            remaining, 0,
            "{table} retained expired native usage identity"
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM managed_consumer_session_bindings WHERE id='old-binding'"
        )
        .fetch_one(pool)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM managed_installation_receipts")
            .fetch_one(pool)
            .await
            .unwrap(),
        receipts
    );
}
