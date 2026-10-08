//! A scheduled job's last run as `scheduled_jobs` records it — read by every
//! page that reports what a background job last did.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// One scheduled job's last run as `scheduled_jobs` records it, with the
/// message a successful run left behind (the retention job reports
/// `table=deleted` pairs there).
#[derive(Debug, Clone)]
pub struct JobRunSummary {
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub last_message: Option<String>,
}

pub async fn find_job_run(
    pool: &PgPool,
    job_name: &str,
) -> Result<Option<JobRunSummary>, sqlx::Error> {
    sqlx::query_as!(
        JobRunSummary,
        r"
        SELECT last_run, next_run, last_status, last_error, last_message
        FROM scheduled_jobs
        WHERE job_name = $1
        ",
        job_name
    )
    .fetch_optional(pool)
    .await
}
