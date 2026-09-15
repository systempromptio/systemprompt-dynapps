//! Shared row builders. Kept deliberately minimal: every test owns its own
//! database, so fixtures only need to satisfy NOT NULL columns and leave the
//! interesting fields to the caller.

use chrono::{TimeZone, Utc};
use systemprompt::identifiers::SourceId;
use systemprompt_web_shared::models::{ContentSeed, CreateContentParams, CreateLinkParams};

pub fn source_id() -> SourceId {
    SourceId::new("test-source".to_string())
}

pub fn content_params(slug: &str, source: &SourceId) -> CreateContentParams {
    CreateContentParams::new(ContentSeed {
        slug: slug.to_string(),
        title: format!("Title for {slug}"),
        description: format!("Description for {slug}"),
        body: format!("Body for {slug}"),
        author: "Test Author".to_string(),
        published_at: Utc
            .with_ymd_and_hms(2026, 1, 1, 12, 0, 0)
            .single()
            .expect("fixed timestamp is unambiguous"),
        source_id: source.clone(),
    })
    .with_keywords("alpha,beta".to_string())
    .with_version_hash("hash-v1".to_string())
}

pub fn link_params(short_code: &str) -> CreateLinkParams {
    CreateLinkParams::new(
        short_code.to_string(),
        format!("https://example.com/{short_code}"),
        "cta",
    )
}

// Why: every raw-retention path opens with core's reporting-privacy preamble,
// which refuses while committed reporting evidence sits undelivered in the
// event outbox. These databases run no relay, so a test that seeds traffic
// and then expires it acknowledges the delivery itself — the retention rules
// are what it exercises, not the delivery gate in front of them.
pub async fn acknowledge_reporting_outbox(pool: &sqlx::PgPool) {
    sqlx::query(
        "UPDATE event_outbox SET processed_at=clock_timestamp() \
         WHERE consumer='analytics_reporting' AND processed_at IS NULL",
    )
    .execute(pool)
    .await
    .expect("acknowledge the reporting outbox");
}

// Why: raw retention also waits for the reporting consumers — facts consumed,
// the snapshot worker caught up, an evidence cutoff established. None of those
// workers run in a throwaway database, so a test that only wants the sweep to
// run over its traffic establishes the completed barrier explicitly.
pub async fn establish_retention_barrier(pool: &sqlx::PgPool) {
    for sql in [
        "UPDATE analytics_fact_deltas SET consumed_at=clock_timestamp()",
        "INSERT INTO analytics_fact_consumers(owner_id,consumer,generation) SELECT owner_id,'snapshots-v1',generation FROM analytics_fact_checkpoints ON CONFLICT(owner_id,consumer) DO UPDATE SET generation=EXCLUDED.generation",
        "INSERT INTO analytics_snapshot_state(owner_id,fact_generation,evidence_cutoff) SELECT owner_id,generation,clock_timestamp()-interval '90 days' FROM analytics_fact_checkpoints ON CONFLICT(owner_id) DO UPDATE SET fact_generation=EXCLUDED.fact_generation,evidence_cutoff=EXCLUDED.evidence_cutoff",
    ] {
        sqlx::query(sql)
            .execute(pool)
            .await
            .expect("establish the retention barrier");
    }
    acknowledge_reporting_outbox(pool).await;
}
