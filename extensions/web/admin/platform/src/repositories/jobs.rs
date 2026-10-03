//! Scheduled job records surfaced on the governance pages.

use sqlx::PgPool;
use systemprompt::identifiers::{JobName, ScheduledJobId};

use systemprompt_web_admin_core::types::JobSummary;

pub async fn list_jobs(pool: &PgPool) -> Result<Vec<JobSummary>, sqlx::Error> {
    sqlx::query_as!(
        JobSummary,
        r#"
        SELECT
            id AS "id: ScheduledJobId",
            job_name AS "job_name: JobName",
            schedule,
            enabled,
            last_run,
            next_run,
            last_status,
            last_error,
            run_count,
            created_at,
            updated_at
        FROM scheduled_jobs
        ORDER BY job_name
        "#,
    )
    .fetch_all(pool)
    .await
}
