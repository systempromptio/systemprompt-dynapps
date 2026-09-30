//! Scheduled job listing for the admin handlers.

use sqlx::PgPool;

use crate::repositories;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::types::JobSummary;

pub(crate) async fn list_jobs(pool: &PgPool) -> AdminResult<Vec<JobSummary>> {
    let jobs = repositories::jobs::list_jobs(pool).await?;
    Ok(jobs)
}
