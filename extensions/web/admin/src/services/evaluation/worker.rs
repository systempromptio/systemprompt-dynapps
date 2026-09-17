//! The drain loop shared by the scheduler tick and the console buttons.
//!
//! Given leased queue rows, judge each one and settle its lease. Whoever
//! asked is recorded on the run: the scheduler as `automatic`, a click as
//! `manual` with the admin's id — but the judge call itself is always made
//! as the job actor, so manual scoring never counts against the person who
//! clicked.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::Judge;
use crate::repositories::dashboard::usage_aggregations::update_session_title;
use crate::repositories::evaluation::queue::{self, EvaluationLease};
use crate::repositories::evaluation::runs::{self, RunTrigger};
use crate::repositories::evaluation::runs_summary;

#[derive(Debug, Clone)]
pub enum Trigger {
    Automatic,
    Manual { requested_by: UserId },
}

impl Trigger {
    #[must_use]
    pub const fn kind(&self) -> RunTrigger {
        match self {
            Self::Automatic => RunTrigger::Automatic,
            Self::Manual { .. } => RunTrigger::Manual,
        }
    }

    #[must_use]
    pub fn requested_by(&self) -> Option<UserId> {
        match self {
            Self::Automatic => None,
            Self::Manual { requested_by } => Some(requested_by.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DrainReport {
    pub leased: usize,
    pub succeeded: u64,
    pub failed: u64,
}

pub async fn drain(
    pool: &PgPool,
    judge: &Judge,
    leases: &[EvaluationLease],
    trigger: &Trigger,
) -> Result<DrainReport, sqlx::Error> {
    let mut report = DrainReport {
        leased: leases.len(),
        ..DrainReport::default()
    };
    for lease in leases {
        match judge.evaluate(pool, lease).await {
            Ok(mut verdict) => {
                verdict.run.trigger = trigger.kind();
                verdict.run.requested_by = trigger.requested_by();
                runs::insert_run(pool, &verdict.run).await?;
                if verdict.run.succeeded {
                    report.succeeded += 1;
                    if let Some(title) = verdict.title {
                        update_session_title(pool, &lease.session_id, &title.title, &title.summary)
                            .await?;
                    }
                    queue::complete_lease(pool, lease).await?;
                } else {
                    report.failed += 1;
                    let error = verdict.run.error.as_deref().unwrap_or("judge failed");
                    tracing::warn!(session = %lease.session_id, criterion = %lease.criterion, error, "Session evaluation failed");
                    queue::retry_lease(pool, lease, error, judge.config.worker.max_attempts)
                        .await?;
                }
            },
            Err(error) => {
                report.failed += 1;
                tracing::error!(%error, session = %lease.session_id, criterion = %lease.criterion, "Session evaluation could not run");
                queue::retry_lease(
                    pool,
                    lease,
                    &error.to_string(),
                    judge.config.worker.max_attempts,
                )
                .await?;
            },
        }
    }
    // Why: the gateway prices the call after the audit row lands, so a run
    // recorded a moment ago may still be uncosted; the back-fill is cheap
    // and idempotent, so it runs on every drain rather than a separate tick.
    runs_summary::update_run_costs(pool).await?;
    Ok(report)
}
