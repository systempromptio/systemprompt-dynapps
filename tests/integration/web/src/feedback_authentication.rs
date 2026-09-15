//! Actual retained publication plus authenticated consumer context drives late
//! attribution.
use super::feedback_attribution::{attribute, authenticate_event, enrol_device, publish_skill};
use super::feedback_ingestion::{drain, seed};
use crate::tempdb::TempDb;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::managed::ManagedRepository;

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
    let repo = ManagedRepository::new(&db.db_pool()).expect("build the managed repository");
    let device = enrol_device(pool, &repo, &consumer).await;
    let published = publish_skill(&repo, &owner, "skill", "my_skill", b"# Skill").await;
    repo.set_consumer_grant(&owner, &published.resource, &consumer, true)
        .await
        .unwrap();
    authenticate_event(pool, "event1", &consumer, &device, "native-session").await;
    drain(pool).await;
    let (identified,attributed):(i64,i64)=sqlx::query_as("SELECT count(consumer_id),count(resource_id) FROM analytics_normalized_facts WHERE fact_kind='invocation'").fetch_one(pool).await.unwrap();
    assert_eq!((identified, attributed), (1, 0));
    attribute(&repo, &published, &consumer, &device, "native-session").await;
    drain(pool).await;
    let (invocations,attributed):(i64,i64)=sqlx::query_as("SELECT count(*),count(resource_id) FROM analytics_normalized_facts WHERE fact_kind='invocation' AND NOT deleted").fetch_one(pool).await.unwrap();
    assert_eq!((invocations, attributed), (2, 1));
    let second_owner = UserId::new("publisher-two");
    sqlx::query("INSERT INTO users(id,name,email) VALUES('publisher-two','publisher-two','publisher-two@example.invalid')").execute(pool).await.unwrap();
    let conflicting = publish_skill(
        &repo,
        &second_owner,
        "different",
        "my-skill",
        b"# Other skill",
    )
    .await;
    attribute(&repo, &conflicting, &consumer, &device, "native-session").await;
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
    verify_native_evidence_expiry_preserves_installation_receipts(pool, device.cert.as_str()).await;
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
    crate::fixtures::acknowledge_reporting_outbox(pool).await;
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
        let query = format!("SELECT count(*) FROM {table} WHERE {column}='old-native'");
        let remaining = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(query.as_str()))
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
