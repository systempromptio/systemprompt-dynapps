//! Fenced transactional admission. No sequence allocation is a commit
//! watermark.
use crate::JobError;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::analytics::feedback::FeedbackFactsRepository;
use systemprompt::identifiers::{AnalyticsChangeId, UserId};
use systemprompt::models::feedback::analytics::{
    AnalyticsChange, AnalyticsChangeOperation, AnalyticsFactKey, NormalizedAnalyticsFact,
};

/// PostgreSQL adapter queue with transactional producer barriers and fenced
/// retries.
#[derive(Debug, Clone)]
pub struct FeedbackIngestionRepository {
    pub(super) pool: PgPool,
}

/// Claim of one exact source revision; a newer revision or expired token
/// invalidates it.
#[derive(Debug, Clone)]
pub struct IngestionLease {
    pub id: String,
    pub revision: i64,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl FeedbackIngestionRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn claim(&self, limit: u32) -> Result<Vec<IngestionLease>, JobError> {
        if !(1..=256).contains(&limit) {
            return Err(JobError::other("Invalid ingestion batch size"));
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query!("SELECT producer FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE").fetch_one(&mut *tx).await?;
        let token = uuid::Uuid::new_v4().to_string();
        let limit = i64::from(limit);
        let rows = sqlx::query!("WITH pending AS (SELECT id FROM feedback_ingestion_outbox WHERE pending AND next_attempt<=clock_timestamp() AND (lease_until IS NULL OR lease_until<=clock_timestamp()) ORDER BY next_attempt,id LIMIT $1 FOR UPDATE SKIP LOCKED) UPDATE feedback_ingestion_outbox q SET lease_token=$2,lease_until=clock_timestamp()+interval '60 seconds',attempts=q.attempts+1 FROM pending WHERE q.id=pending.id RETURNING q.id,q.revision,q.lease_until",limit,token).fetch_all(&mut *tx).await?;
        tx.commit().await?;
        rows.into_iter()
            .map(|row| {
                Ok(IngestionLease {
                    id: row.id,
                    revision: row.revision,
                    token: token.clone(),
                    expires_at: row
                        .lease_until
                        .ok_or(JobError::other("Missing ingestion lease"))?,
                })
            })
            .collect()
    }

    pub async fn process(&self, owner: &UserId, lease: &IngestionLease) -> Result<bool, JobError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("SELECT producer FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE").fetch_one(&mut *tx).await?;
        let row = sqlx::query!("SELECT feedback_fact_key(kind,event_id,request_id,user_id) AS key,feedback_normalized_fact(id) AS fact,updated_at,occurred_at FROM feedback_ingestion_outbox WHERE id=$1 AND pending AND revision=$2 AND lease_token=$3 AND lease_until>clock_timestamp() FOR UPDATE",lease.id,lease.revision,lease.token).fetch_optional(&mut *tx).await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let key: AnalyticsFactKey =
            serde_json::from_value(row.key.ok_or(JobError::other("Missing fact key"))?)?;
        let fact: Option<NormalizedAnalyticsFact> =
            row.fact.map(serde_json::from_value).transpose()?;
        let occurred_at = fact.as_ref().map_or(row.occurred_at, |fact| match fact {
            NormalizedAnalyticsFact::Invocation(value) => value.occurred_at,
            NormalizedAnalyticsFact::Request(value) => value.occurred_at,
            NormalizedAnalyticsFact::Assessment(value) => value.occurred_at,
            NormalizedAnalyticsFact::ResourceAssociation(value) => value.occurred_at,
        });
        let change = AnalyticsChange {
            change_id: AnalyticsChangeId::new(format!("astound:{}:{}", lease.id, lease.revision)),
            key,
            revision: u64::try_from(lease.revision)
                .map_err(|_| JobError::other("Invalid ingestion revision"))?,
            occurred_at,
            recorded_at: row.updated_at,
            operation: fact.map_or(AnalyticsChangeOperation::Tombstone, |fact| {
                AnalyticsChangeOperation::Replace { fact }
            }),
        };
        FeedbackFactsRepository::submit_in(&mut tx, owner, &change)
            .await
            .map_err(|error| JobError::other(error.to_string()))?;
        sqlx::query!("UPDATE feedback_ingestion_outbox SET pending=false,lease_token=NULL,lease_until=NULL,last_error=NULL WHERE id=$1",lease.id).execute(&mut *tx).await?;
        sqlx::query!("UPDATE analytics_ingestion_producers SET pending_count=pending_count-1,oldest_pending_at=CASE WHEN pending_count=1 THEN NULL ELSE oldest_pending_at END,checkpoint=checkpoint+1,last_drained_at=clock_timestamp(),updated_at=clock_timestamp() WHERE producer='astound.feedback'").execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(true)
    }

    pub async fn retry(&self, lease: &IngestionLease) -> Result<(), JobError> {
        sqlx::query!("UPDATE feedback_ingestion_outbox SET lease_token=NULL,lease_until=NULL,next_attempt=clock_timestamp()+least(attempts,60)*interval '5 seconds',last_error='normalization_failed' WHERE id=$1 AND revision=$2 AND lease_token=$3",lease.id,lease.revision,lease.token).execute(&self.pool).await?;
        Ok(())
    }
}
