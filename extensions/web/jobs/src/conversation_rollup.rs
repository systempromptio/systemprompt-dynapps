//! `conversation_rollup` job: keeps `conversation_facts` current.
//!
//! Every minute it re-derives the fact row for every conversation touched on
//! any plane since the last tick — a request settled, a hook event, a
//! governance decision, a tool execution, an artifact — through
//! `refresh_conversation_facts_since` (schema 45_). The upsert is idempotent
//! and the window overlaps the schedule, so a missed tick loses nothing and a
//! double tick costs nothing. `-p all=true` rebuilds every row; `-p
//! lookback_minutes=N` widens one run after a backfill.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

// Why: three ticks of overlap — settlement and hook delivery both lag the
// request by seconds, occasionally by a minute.
const DEFAULT_LOOKBACK_MINUTES: i64 = 3;

#[derive(Debug, Clone, Copy, Default)]
pub struct ConversationRollupJob;

impl ConversationRollupJob {
    pub async fn execute_with_pool(
        pool: &PgPool,
        lookback_minutes: i64,
        all: bool,
    ) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let written = if all {
            sqlx::query_scalar!(r#"SELECT refresh_conversation_facts(NULL) AS "written!""#)
                .fetch_one(pool)
                .await?
        } else {
            sqlx::query_scalar!(
                r#"SELECT refresh_conversation_facts_since(now() - make_interval(mins => $1::int)) AS "written!""#,
                i32::try_from(lookback_minutes.clamp(1, 527_040)).unwrap_or(3)
            )
            .fetch_one(pool)
            .await?
        };
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let written = u64::try_from(written).unwrap_or(0);
        tracing::debug!(
            rows = written,
            duration_ms,
            all,
            "conversation rollup completed"
        );
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for ConversationRollupJob {
    fn name(&self) -> &'static str {
        "conversation_rollup"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Re-derives the per-conversation fact row for every conversation touched since the last tick"
    }

    fn schedule(&self) -> &'static str {
        "30 * * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let lookback = ctx
            .get_parameter_parsed::<i64>("lookback_minutes")?
            .unwrap_or(DEFAULT_LOOKBACK_MINUTES);
        let all = ctx.get_parameter_parsed::<bool>("all")?.unwrap_or(false);
        Ok(Self::execute_with_pool(&db.write_pool(), lookback, all).await?)
    }
}

systemprompt::traits::submit_job!(&ConversationRollupJob);
