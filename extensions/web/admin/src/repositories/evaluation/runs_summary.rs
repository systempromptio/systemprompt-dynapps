//! Rollups over judge runs for the Evaluation page's tiles, and the cost
//! back-fill that runs once the gateway has priced the calls.

use serde::Serialize;
use sqlx::PgPool;

use super::CriterionName;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CriterionAverage {
    pub score: Option<f64>,
    pub sessions: i64,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct RunsSummary {
    pub runs: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub queued: i64,
    pub cost_microdollars: i64,
    pub intent: CriterionAverage,
    pub quality: CriterionAverage,
    pub speed: CriterionAverage,
    pub cost: CriterionAverage,
}

pub async fn get_runs_summary(pool: &PgPool, since_days: i32) -> Result<RunsSummary, sqlx::Error> {
    let totals = sqlx::query!(
        r#"SELECT count(*) AS "runs!",
                  count(*) FILTER (WHERE status = 'succeeded') AS "succeeded!",
                  count(*) FILTER (WHERE status = 'failed') AS "failed!",
                  COALESCE(sum(cost_microdollars), 0)::BIGINT AS "cost!"
             FROM evaluation_runs
            WHERE created_at > clock_timestamp() - make_interval(days => $1)"#,
        since_days
    )
    .fetch_one(pool)
    .await?;
    let queued =
        sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM evaluation_queue WHERE pending"#)
            .fetch_one(pool)
            .await?;
    let averages = sqlx::query!(
        r#"SELECT criterion, avg(score)::FLOAT8 AS score, count(DISTINCT session_id) AS "sessions!"
             FROM evaluation_runs
            WHERE status = 'succeeded' AND created_at > clock_timestamp() - make_interval(days => $1)
            GROUP BY criterion"#,
        since_days
    )
    .fetch_all(pool)
    .await?;
    let mut summary = RunsSummary {
        runs: totals.runs,
        succeeded: totals.succeeded,
        failed: totals.failed,
        queued,
        cost_microdollars: totals.cost,
        ..RunsSummary::default()
    };
    for row in averages {
        let average = CriterionAverage {
            score: row.score,
            sessions: row.sessions,
        };
        match CriterionName::parse(&row.criterion) {
            Some(CriterionName::Intent) => summary.intent = average,
            Some(CriterionName::Quality) => summary.quality = average,
            Some(CriterionName::Speed) => summary.speed = average,
            Some(CriterionName::Cost) => summary.cost = average,
            None => {},
        }
    }
    Ok(summary)
}

// Why: copies the gateway's priced cost onto runs recorded before the audit
// row landed; called once per job tick after the audit tasks drain.
pub async fn update_run_costs(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE evaluation_runs r SET cost_microdollars = a.cost_microdollars
           FROM ai_requests a
          WHERE a.request_id = r.ai_request_id AND r.cost_microdollars IS NULL"
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
