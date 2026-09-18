//! Restart-safe bounded source scans; concurrent inserts are captured by
//! triggers.
use super::FeedbackIngestionRepository;
use crate::JobError;

impl FeedbackIngestionRepository {
    pub async fn backfill_page(&self, limit: u32) -> Result<u64, JobError> {
        if !(1..=256).contains(&limit) {
            return Err(JobError::other("Invalid backfill page size"));
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query!("SELECT producer FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE").fetch_one(&mut *tx).await?;
        let progress = sqlx::query!("SELECT source,after_id FROM feedback_ingestion_backfill WHERE NOT done ORDER BY source LIMIT 1 FOR UPDATE SKIP LOCKED").fetch_optional(&mut *tx).await?;
        let Some(progress) = progress else {
            return Ok(0);
        };
        let limit = i64::from(limit);
        let rows = sqlx::query!("SELECT id,user_id,session_id FROM (SELECT id,user_id,session_id FROM plugin_usage_events WHERE $1='plugin_usage_events' UNION ALL SELECT id,user_id,client_session_id AS session_id FROM ai_requests WHERE $1='ai_requests' UNION ALL SELECT id,user_id,session_id FROM evaluation_runs WHERE $1='evaluation_runs') rows WHERE id>$2 ORDER BY id LIMIT $3",progress.source,progress.after_id,limit).fetch_all(&mut *tx).await?;
        for row in &rows {
            if progress.source == "plugin_usage_events" {
                sqlx::query!("INSERT INTO feedback_ingestion_evidence(event_id,evidence) SELECT id,to_jsonb(e) FROM plugin_usage_events e WHERE id=$1 ON CONFLICT DO NOTHING",row.id).execute(&mut *tx).await?;
            }
            if progress.source == "ai_requests" {
                sqlx::query!(
                    "SELECT enqueue_feedback_fact('request',NULL,$1,$2,$3)",
                    row.id,
                    row.user_id,
                    row.session_id
                )
                .fetch_one(&mut *tx)
                .await?;
            }
            sqlx::query!(
                "SELECT enqueue_feedback_session($1,$2)",
                row.user_id,
                row.session_id
            )
            .fetch_one(&mut *tx)
            .await?;
        }
        if let Some(last) = rows.last() {
            sqlx::query!(
                "UPDATE feedback_ingestion_backfill SET after_id=$2 WHERE source=$1",
                progress.source,
                last.id
            )
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE feedback_ingestion_backfill SET done=true WHERE source=$1",
                progress.source
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!("SELECT feedback_producer_recount(false)")
                .fetch_one(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(rows.len() as u64)
    }
}
